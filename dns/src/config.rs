use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamConfig {
    pub listen: SocketAddr,
    pub upstream_nameservers: Vec<NameServer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type", content = "value")]
pub enum NameServer {
    Udp(SocketAddr),
    Doh(String),
}
