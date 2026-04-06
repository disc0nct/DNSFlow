-- DNS Servers table
CREATE TABLE IF NOT EXISTS dns_servers (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    address TEXT NOT NULL,
    secondary_address TEXT,
    protocol TEXT NOT NULL DEFAULT 'udp',
    is_default INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- App Rules table  
CREATE TABLE IF NOT EXISTS app_rules (
    id INTEGER PRIMARY KEY,
    app_name TEXT NOT NULL,
    app_path TEXT,
    dns_server_id INTEGER NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    use_ld_preload INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (dns_server_id) REFERENCES dns_servers(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_app_rules_app_name ON app_rules(app_name);

-- DNS Query Log table
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

CREATE INDEX IF NOT EXISTS idx_dns_query_log_domain ON dns_query_log(domain);
CREATE INDEX IF NOT EXISTS idx_dns_query_log_timestamp ON dns_query_log(timestamp);
CREATE INDEX IF NOT EXISTS idx_dns_query_log_pid ON dns_query_log(pid);

-- Config table
CREATE TABLE IF NOT EXISTS config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
