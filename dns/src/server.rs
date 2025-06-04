use anyhow::Result;
use haven_db::Database;
use hickory_proto::op::Message;
use hickory_proto::serialize::binary::{BinDecodable, BinEncodable};
use tokio::net::UdpSocket;

use crate::config::{NameServer, UpstreamConfig};
use crate::query::QueryHandler;

pub struct DNSServer<'a> {
    listener: UdpSocket,
    query_handler: QueryHandler,
    database: &'a Database,
}

impl<'a> DNSServer<'a> {
    pub async fn new(config: UpstreamConfig, database: &'a Database) -> Result<Self> {
        let listener = UdpSocket::bind(config.listen).await?;

        let mut udp_upstreams = Vec::new();
        let mut doh_upstreams = Vec::new();

        for nameserver in config.upstream_nameservers {
            match nameserver {
                NameServer::Udp(addr) => udp_upstreams.push(addr),
                NameServer::Doh(url) => doh_upstreams.push(url),
            }
        }

        let query_handler = QueryHandler::new(udp_upstreams, doh_upstreams);

        Ok(Self {
            listener,
            query_handler,
            database,
        })
    }

    async fn handle_query(&self, buf: &mut [u8]) -> Result<()> {
        let (size, src) = self.listener.recv_from(buf).await?;
        log::debug!("Received {} bytes from {}", size, src);

        let request = Message::from_bytes(&buf[..size])?;

        // Try to resolve from local database first
        if let Some(response) = self
            .query_handler
            .query_local(request.clone(), self.database)
            .await?
        {
            self.listener.send_to(&response.to_bytes()?, src).await?;
            return Ok(());
        }

        log::debug!("No local record found, querying upstream servers");

        // Fall back to upstream servers if local resolution fails
        let response = self.query_handler.query_upstreams(request).await?;
        self.listener.send_to(&response, src).await?;

        Ok(())
    }

    pub async fn run(&self) -> Result<()> {
        log::info!("DNS Server listening on {}", self.listener.local_addr()?);

        let mut buf = [0u8; 512];

        loop {
            if let Err(e) = self.handle_query(&mut buf).await {
                log::error!("Failed to handle query: {}", e);
            }
        }
    }
}
