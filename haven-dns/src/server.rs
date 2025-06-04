use crate::config::{NameServer, UpstreamConfig};
use hickory_proto::op::{Message, MessageType};
use hickory_proto::serialize::binary::BinDecodable;
use reqwest::Client;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

pub struct DNSServer {
    listener: UdpSocket,
    udp_upstreams: Vec<SocketAddr>,
    doh_upstreams: Vec<String>,
    client: Client,
}

impl DNSServer {
    pub async fn new(config: UpstreamConfig) -> Result<Self, std::io::Error> {
        let listener = UdpSocket::bind(config.listen).await?;

        let mut udp_upstreams = Vec::new();
        let mut doh_upstreams = Vec::new();

        for nameserver in config.upstream_nameservers {
            match nameserver {
                NameServer::Udp(addr) => udp_upstreams.push(addr),
                NameServer::Doh(url) => doh_upstreams.push(url),
            }
        }

        let client = Client::new();

        Ok(Self {
            listener,
            udp_upstreams,
            doh_upstreams,
            client,
        })
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("DNS Server listening on {}", self.listener.local_addr()?);

        let mut buf = [0u8; 512];

        loop {
            let (size, src) = self.listener.recv_from(&mut buf).await?;
            log::debug!("Received {} bytes from {}", size, src);

            match Message::from_bytes(&buf[..size]) {
                Ok(message) => {
                    log::debug!("DNS Message: {:?}", message);
                }
                Err(e) => {
                    log::error!("Failed to parse DNS message: {}", e);
                }
            }
        }
    }
}
