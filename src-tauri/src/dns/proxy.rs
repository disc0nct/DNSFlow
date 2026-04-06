use crate::db::database::Database;
use crate::db::query_logger::QueryLogger;
use crate::debug;
use crate::dns::rules::RulesEngine;
use crate::process::lookup::lookup_pid_by_socket;
use crate::state::DnsQueryLog;
use hickory_proto::op::Message;
use hickory_proto::rr::{rdata::A, RData, Record};
use hickory_proto::xfer::Protocol;
use hickory_resolver::config::{NameServerConfig, ResolverConfig};
use hickory_resolver::lookup_ip::LookupIp;
use hickory_resolver::name_server::TokioConnectionProvider;
use hickory_resolver::TokioResolver;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Notify;

pub const PROXY_LISTEN_IP: &str = "127.0.0.53";
pub const PROXY_LISTEN_PORT: u16 = 53;
pub const PROXY_RESOLV_CONF: &str = "/tmp/dnsflow_resolv.conf";
pub const PROXY_FALLBACK_IPS: &[&str] = &["127.0.0.53", "127.0.0.54", "127.0.0.1"];

pub fn proxy_addr() -> String {
    format!("{}:{}", PROXY_LISTEN_IP, PROXY_LISTEN_PORT)
}

pub fn write_proxy_resolv_conf(ip: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = format!("nameserver {}\n", ip);
    let _ = std::fs::remove_file(PROXY_RESOLV_CONF);
    std::fs::write(PROXY_RESOLV_CONF, &content)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(PROXY_RESOLV_CONF, std::fs::Permissions::from_mode(0o644));
    }
    debug!(
        "Wrote proxy resolv.conf to {} with nameserver {}",
        PROXY_RESOLV_CONF, ip
    );
    Ok(())
}

pub fn cleanup_proxy_resolv_conf() {
    let _ = std::fs::remove_file(PROXY_RESOLV_CONF);
}

#[derive(Debug)]
pub struct DnsProxyServer {
    addr: String,
    shutdown: Arc<Notify>,
}

impl DnsProxyServer {
    pub fn bound_addr(&self) -> &str {
        &self.addr
    }

    pub async fn new(addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            addr: addr.to_string(),
            shutdown: Arc::new(Notify::new()),
        })
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind(&self.addr).await?;
        debug!("DNS proxy listening on {}", self.addr);

        let shutdown = self.shutdown.clone();
        tokio::spawn(async move {
            let mut buf = [0u8; 512];
            loop {
                tokio::select! {
                    _ = shutdown.notified() => break,
                    result = socket.recv_from(&mut buf) => {
                        let (len, src) = match result {
                            Ok(v) => v,
                            Err(_) => continue,
                        };
                        let resolver = TokioResolver::builder_tokio().unwrap().build();
                        match Message::from_vec(&buf[..len]) {
                            Ok(query) => {
                                if let Some(question) = query.queries().first() {
                                    let name = question.name().to_string();
                                    if let Ok(response) = resolver.lookup_ip(&name).await {
                                        let mut resp_msg = Message::new();
                                        resp_msg.set_id(query.id());
                                        resp_msg.set_message_type(hickory_proto::op::MessageType::Response);
                                        resp_msg.add_query(question.clone());
                                        for ip in response.iter().filter(|ip| ip.is_ipv4()) {
                                            if let std::net::IpAddr::V4(v4) = ip {
                                                resp_msg.add_answer(Record::from_rdata(question.name().clone(), 300, RData::A(A(v4))));
                                            }
                                        }
                                        if let Ok(bytes) = resp_msg.to_vec() {
                                            let _ = socket.send_to(&bytes, src).await;
                                        }
                                    }
                                }
                            }
                            Err(_) => {}
                        }
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.shutdown.notify_one();
        Ok(())
    }
}

impl Drop for DnsProxyServer {
    fn drop(&mut self) {
        self.shutdown.notify_one();
    }
}

impl DnsProxyServer {
    pub async fn query(
        &self,
        domain: &str,
    ) -> Result<Vec<std::net::IpAddr>, Box<dyn std::error::Error>> {
        let response: LookupIp = TokioResolver::builder_tokio()?
            .build()
            .lookup_ip(domain)
            .await?;
        Ok(response.iter().collect())
    }

    pub async fn start_with_rules(
        &self,
        rules_engine: Arc<RulesEngine>,
        query_logger: Arc<QueryLogger>,
        db: Arc<Database>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        debug!("DNS proxy binding to {}", self.addr);
        let socket = UdpSocket::bind(&self.addr).await?;
        debug!("DNS proxy listening on {}", self.addr);

        let shutdown = self.shutdown.clone();
        tokio::spawn(async move {
            let mut buf = [0u8; 512];
            loop {
                tokio::select! {
                    _ = shutdown.notified() => break,
                    result = socket.recv_from(&mut buf) => {
                        let (len, src) = match result {
                            Ok(v) => v,
                            Err(_) => continue,
                        };
                        let src_port = src.port();
                        debug!("Received {} bytes from {}", len, src);
                        let pid = lookup_pid_by_socket(src_port).ok().flatten();
                        debug!("PID lookup for port {}: {:?}", src_port, pid);

                        let dns_server = if let Some(pid) = pid {
                            rules_engine.lookup_by_pid(pid).await
                        } else {
                            None
                        };

                        debug!(
                            "Rule match for PID {:?}: {:?}",
                            pid,
                            dns_server.as_ref().map(|s| &s.name)
                        );

                        let server_addr = dns_server.as_ref().map(|s| s.address.clone());
                        let server_id = dns_server.as_ref().and_then(|s| s.id);

                        let query_resolver = if let Some(server) = dns_server {
                            let addr: std::net::SocketAddr = format!("{}:53", server.address)
                                .parse()
                                .unwrap_or_else(|_| "8.8.8.8:53".parse().unwrap());
                            let mut config = ResolverConfig::new();
                            config.add_name_server(NameServerConfig::new(addr, Protocol::Udp));
                            Arc::new(
                                TokioResolver::builder_with_config(
                                    config,
                                    TokioConnectionProvider::default(),
                                )
                                .build(),
                            )
                        } else {
                            Arc::new(TokioResolver::builder_tokio().unwrap().build())
                        };

                        let start_time = std::time::Instant::now();

                        match Message::from_vec(&buf[..len]) {
                            Ok(query) => {
                                if let Some(question) = query.queries().first() {
                                    let name = question.name().to_string();
                                    debug!(
                                        "Query: {} from port {} (PID {:?})",
                                        name, src_port, pid
                                    );
                                    match query_resolver.lookup_ip(&name).await {
                                        Ok(response) => {
                                            let latency = start_time.elapsed().as_millis() as i64;
                                            let resolved_ips: Vec<String> = response
                                                .iter()
                                                .map(|ip| ip.to_string())
                                                .collect();

                                            debug!(
                                                "Resolved {} -> {:?} ({}ms, server: {:?})",
                                                name, resolved_ips, latency, server_addr
                                            );

                                            let app_name = if let Some(p) = pid {
                                                crate::process::monitor::get_process_by_pid(p)
                                                    .ok()
                                                    .flatten()
                                                    .map(|info| info.name)
                                            } else {
                                                None
                                            };

                                            let log_entry = DnsQueryLog {
                                                id: None,
                                                domain: name.clone(),
                                                pid: pid.map(|p| p as i64),
                                                app_name,
                                                dns_server_id: server_id,
                                                resolved_ip: resolved_ips.first().cloned(),
                                                latency_ms: Some(latency),
                                                timestamp: chrono::Local::now()
                                                    .format("%Y-%m-%d %H:%M:%S")
                                                    .to_string(),
                                            };

                                            if let Err(e) = query_logger.log_query(log_entry.clone()).await {
                                                debug!("Failed to log query: {}", e);
                                            }
                                            let _ = db.insert_query_log(&log_entry);

                                            let mut resp_msg = Message::new();
                                            resp_msg.set_id(query.id());
                                            resp_msg.set_message_type(
                                                hickory_proto::op::MessageType::Response,
                                            );
                                            resp_msg.add_query(question.clone());

                                            for ip in response.iter() {
                                                let rdata = if ip.is_ipv4() {
                                                    if let std::net::IpAddr::V4(v4) = ip {
                                                        RData::A(A(v4))
                                                    } else {
                                                        continue;
                                                    }
                                                } else {
                                                    continue;
                                                };
                                                let record = Record::from_rdata(
                                                    question.name().clone(),
                                                    300,
                                                    rdata,
                                                );
                                                resp_msg.add_answer(record);
                                            }

                                            match resp_msg.to_vec() {
                                                Ok(bytes) => {
                                                    if let Err(e) =
                                                        socket.send_to(&bytes, src).await
                                                    {
                                                        debug!(
                                                            "Failed to send DNS response to {}: {}",
                                                            src, e
                                                        );
                                                    }
                                                }
                                                Err(e) => {
                                                    debug!(
                                                        "Failed to serialize DNS response: {}",
                                                        e
                                                    );
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            debug!("DNS lookup failed for {}: {}", name, e);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                debug!("Failed to parse DNS query from {}: {}", src, e);
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }
}
