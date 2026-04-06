use crate::state::{AppConfig, AppRule, DnsQueryLog, DnsServer};
use rusqlite::{params, Connection};
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(path: &str) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn initialize(&self) -> Result<(), String> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("Database lock poisoned: {}", e))?;

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS dns_servers (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                address TEXT NOT NULL,
                secondary_address TEXT,
                protocol TEXT NOT NULL DEFAULT 'udp',
                is_default INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS app_rules (
                id INTEGER PRIMARY KEY,
                app_name TEXT NOT NULL,
                app_path TEXT,
                dns_server_id INTEGER NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                use_ld_preload INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (dns_server_id) REFERENCES dns_servers(id)
            );

            CREATE TABLE IF NOT EXISTS dns_query_log (
                id INTEGER PRIMARY KEY,
                domain TEXT NOT NULL,
                pid INTEGER,
                app_name TEXT,
                dns_server_id INTEGER,
                resolved_ip TEXT,
                latency_ms INTEGER,
                timestamp TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            ",
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn seed_if_empty(&self) -> Result<(), String> {
        let count = {
            let conn = self
                .conn
                .lock()
                .map_err(|e| format!("Database lock poisoned: {}", e))?;
            conn.query_row("SELECT COUNT(*) FROM dns_servers", [], |row| {
                row.get::<_, i64>(0)
            })
            .map_err(|e| e.to_string())?
        };

        if count == 0 {
            self.seed()?;
        }

        Ok(())
    }

    fn seed(&self) -> Result<(), String> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("Database lock poisoned: {}", e))?;

        conn.execute_batch(
            "
            INSERT INTO dns_servers (name, address, secondary_address, protocol, is_default) VALUES
                ('Google DNS', '8.8.8.8', '8.8.4.4', 'udp', 0),
                ('Cloudflare', '1.1.1.1', '1.0.0.1', 'udp', 1),
                ('Quad9', '9.9.9.9', '149.112.112.112', 'udp', 0);

            INSERT INTO config (key, value) VALUES
                ('proxy_port', '5353'),
                ('log_enabled', 'true'),
                ('auto_start', 'false'),
                ('debug', 'false')
            ON CONFLICT(key) DO UPDATE SET value = excluded.value;
            ",
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn clear_all_data(&self) -> Result<(), String> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("Database lock poisoned: {}", e))?;

        conn.execute_batch(
            "
            DELETE FROM app_rules;
            DELETE FROM dns_query_log;
            DELETE FROM dns_servers;
            DELETE FROM config;
            ",
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn reset_to_defaults(&self) -> Result<(), String> {
        self.clear_all_data()?;
        self.seed()?;
        Ok(())
    }

    pub fn get_dns_servers(&self) -> Result<Vec<DnsServer>, String> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("Database lock poisoned: {}", e))?;
        let mut stmt = conn
            .prepare("SELECT id, name, address, secondary_address, protocol, is_default FROM dns_servers ORDER BY id")
            .map_err(|e| e.to_string())?;

        let servers = stmt
            .query_map([], |row| {
                Ok(DnsServer {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                    address: row.get(2)?,
                    secondary_address: row.get(3)?,
                    protocol: row.get(4)?,
                    is_default: row.get::<_, i64>(5)? != 0,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(servers)
    }

    pub fn add_dns_server(
        &self,
        name: &str,
        address: &str,
        secondary_address: Option<&str>,
        protocol: &str,
    ) -> Result<DnsServer, String> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("Database lock poisoned: {}", e))?;
        conn.execute(
            "INSERT INTO dns_servers (name, address, secondary_address, protocol) VALUES (?1, ?2, ?3, ?4)",
            params![name, address, secondary_address, protocol],
        )
        .map_err(|e| e.to_string())?;

        let id = conn.last_insert_rowid();

        Ok(DnsServer {
            id: Some(id),
            name: name.to_string(),
            address: address.to_string(),
            secondary_address: secondary_address.map(|s| s.to_string()),
            protocol: protocol.to_string(),
            is_default: false,
        })
    }

    pub fn update_dns_server(
        &self,
        id: i64,
        name: &str,
        address: &str,
        secondary_address: Option<&str>,
        protocol: &str,
    ) -> Result<DnsServer, String> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("Database lock poisoned: {}", e))?;

        conn.execute(
            "UPDATE dns_servers SET name = ?1, address = ?2, secondary_address = ?3, protocol = ?4 WHERE id = ?5",
            params![name, address, secondary_address, protocol, id],
        )
        .map_err(|e| e.to_string())?;

        let is_default: bool = conn
            .query_row(
                "SELECT is_default FROM dns_servers WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .unwrap_or(false);

        Ok(DnsServer {
            id: Some(id),
            name: name.to_string(),
            address: address.to_string(),
            secondary_address: secondary_address.map(|s| s.to_string()),
            protocol: protocol.to_string(),
            is_default,
        })
    }

    pub fn set_default_dns_server(&self, id: i64) -> Result<bool, String> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("Database lock poisoned: {}", e))?;

        conn.execute_batch(&format!(
            "BEGIN;
                 UPDATE dns_servers SET is_default = 0;
                 UPDATE dns_servers SET is_default = 1 WHERE id = {};
                 COMMIT;",
            id
        ))
        .map_err(|e| e.to_string())?;

        Ok(true)
    }

    pub fn delete_dns_server(&self, id: i64) -> Result<bool, String> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("Database lock poisoned: {}", e))?;
        let rows_affected = conn
            .execute("DELETE FROM dns_servers WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;

        Ok(rows_affected > 0)
    }

    pub fn get_rules(&self) -> Result<Vec<AppRule>, String> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("Database lock poisoned: {}", e))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, app_name, app_path, dns_server_id, enabled, use_ld_preload FROM app_rules ORDER BY id",
            )
            .map_err(|e| e.to_string())?;

        let rules = stmt
            .query_map([], |row| {
                Ok(AppRule {
                    id: Some(row.get(0)?),
                    app_name: row.get(1)?,
                    app_path: row.get(2)?,
                    dns_server_id: row.get(3)?,
                    enabled: row.get::<_, i64>(4)? != 0,
                    use_ld_preload: row.get::<_, i64>(5)? != 0,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(rules)
    }

    pub fn add_rule(
        &self,
        app_name: &str,
        app_path: Option<&str>,
        dns_server_id: i64,
        use_ld_preload: bool,
    ) -> Result<AppRule, String> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("Database lock poisoned: {}", e))?;
        conn.execute(
            "INSERT INTO app_rules (app_name, app_path, dns_server_id, use_ld_preload) VALUES (?1, ?2, ?3, ?4)",
            params![app_name, app_path, dns_server_id, if use_ld_preload { 1 } else { 0 }],
        )
        .map_err(|e| e.to_string())?;

        let id = conn.last_insert_rowid();

        Ok(AppRule {
            id: Some(id),
            app_name: app_name.to_string(),
            app_path: app_path.map(|s| s.to_string()),
            dns_server_id,
            enabled: true,
            use_ld_preload,
        })
    }

    pub fn update_rule(
        &self,
        id: i64,
        app_name: &str,
        app_path: Option<&str>,
        dns_server_id: i64,
        use_ld_preload: bool,
    ) -> Result<AppRule, String> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("Database lock poisoned: {}", e))?;

        conn.execute(
            "UPDATE app_rules SET app_name = ?1, app_path = ?2, dns_server_id = ?3, use_ld_preload = ?4 WHERE id = ?5",
            params![app_name, app_path, dns_server_id, if use_ld_preload { 1 } else { 0 }, id],
        )
        .map_err(|e| e.to_string())?;

        let enabled: bool = conn
            .query_row(
                "SELECT enabled FROM app_rules WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .unwrap_or(true);

        Ok(AppRule {
            id: Some(id),
            app_name: app_name.to_string(),
            app_path: app_path.map(|s| s.to_string()),
            dns_server_id,
            enabled,
            use_ld_preload,
        })
    }

    pub fn toggle_rule(&self, id: i64, enabled: bool) -> Result<bool, String> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("Database lock poisoned: {}", e))?;
        let rows_affected = conn
            .execute(
                "UPDATE app_rules SET enabled = ?1 WHERE id = ?2",
                params![if enabled { 1 } else { 0 }, id],
            )
            .map_err(|e| e.to_string())?;

        Ok(rows_affected > 0)
    }

    pub fn delete_rule(&self, id: i64) -> Result<bool, String> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("Database lock poisoned: {}", e))?;
        let rows_affected = conn
            .execute("DELETE FROM app_rules WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;

        Ok(rows_affected > 0)
    }

    pub fn get_config(&self) -> Result<AppConfig, String> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("Database lock poisoned: {}", e))?;
        let mut stmt = conn
            .prepare("SELECT key, value FROM config")
            .map_err(|e| e.to_string())?;

        let mut proxy_port: u16 = 5353;
        let mut log_enabled: bool = true;
        let mut auto_start: bool = false;
        let mut debug: bool = false;

        let rows = stmt
            .query_map([], |row| {
                let key: String = row.get(0)?;
                let value: String = row.get(1)?;
                Ok((key, value))
            })
            .map_err(|e| e.to_string())?;

        for row in rows {
            let (key, value) = row.map_err(|e| e.to_string())?;
            match key.as_str() {
                "proxy_port" => {
                    proxy_port = value.parse().unwrap_or(5353);
                }
                "log_enabled" => {
                    log_enabled = value == "true";
                }
                "auto_start" => {
                    auto_start = value == "true";
                }
                "debug" => {
                    debug = value == "true";
                }
                _ => {}
            }
        }

        Ok(AppConfig {
            proxy_port,
            log_enabled,
            auto_start,
            debug,
        })
    }

    pub fn update_config(&self, config: &AppConfig) -> Result<bool, String> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("Database lock poisoned: {}", e))?;

        conn.execute(
            "INSERT INTO config (key, value, updated_at) VALUES ('proxy_port', ?1, datetime('now'))
             ON CONFLICT(key) DO UPDATE SET value = ?1, updated_at = datetime('now')",
            params![config.proxy_port.to_string()],
        )
        .map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO config (key, value, updated_at) VALUES ('log_enabled', ?1, datetime('now'))
             ON CONFLICT(key) DO UPDATE SET value = ?1, updated_at = datetime('now')",
            params![if config.log_enabled { "true" } else { "false" }],
        )
        .map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO config (key, value, updated_at) VALUES ('auto_start', ?1, datetime('now'))
             ON CONFLICT(key) DO UPDATE SET value = ?1, updated_at = datetime('now')",
            params![if config.auto_start { "true" } else { "false" }],
        )
        .map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO config (key, value, updated_at) VALUES ('debug', ?1, datetime('now'))
             ON CONFLICT(key) DO UPDATE SET value = ?1, updated_at = datetime('now')",
            params![if config.debug { "true" } else { "false" }],
        )
        .map_err(|e| e.to_string())?;

        Ok(true)
    }

    pub fn insert_query_log(&self, entry: &DnsQueryLog) -> Result<(), String> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("Database lock poisoned: {}", e))?;
        conn.execute(
            "INSERT INTO dns_query_log (domain, pid, app_name, dns_server_id, resolved_ip, latency_ms, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                entry.domain,
                entry.pid,
                entry.app_name,
                entry.dns_server_id,
                entry.resolved_ip,
                entry.latency_ms,
                entry.timestamp,
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_query_logs(&self, limit: i64) -> Result<Vec<DnsQueryLog>, String> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("Database lock poisoned: {}", e))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, domain, pid, app_name, dns_server_id, resolved_ip, latency_ms, timestamp
                 FROM dns_query_log ORDER BY timestamp DESC LIMIT ?1",
            )
            .map_err(|e| e.to_string())?;

        let logs = stmt
            .query_map(params![limit], |row| {
                Ok(DnsQueryLog {
                    id: Some(row.get(0)?),
                    domain: row.get(1)?,
                    pid: row.get(2)?,
                    app_name: row.get(3)?,
                    dns_server_id: row.get(4)?,
                    resolved_ip: row.get(5)?,
                    latency_ms: row.get(6)?,
                    timestamp: row.get(7)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(logs)
    }

    pub fn add_query_log(&self, log: &DnsQueryLog) -> Result<(), String> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("Database lock poisoned: {}", e))?;
        conn.execute(
            "INSERT INTO dns_query_log (domain, pid, app_name, dns_server_id, resolved_ip, latency_ms, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                log.domain,
                log.pid,
                log.app_name,
                log.dns_server_id,
                log.resolved_ip,
                log.latency_ms,
                log.timestamp,
            ],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn clear_query_logs(&self) -> Result<bool, String> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("Database lock poisoned: {}" , e))?;
        let rows_affected = conn
            .execute("DELETE FROM dns_query_log", [])
            .map_err(|e| e.to_string())?;

        Ok(rows_affected > 0)
    }
}
