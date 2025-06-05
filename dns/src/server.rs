use anyhow::Result;
use haven_db::Database;
use hickory_proto::op::{Header, Message};
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
        log::trace!("Request: {:#?}", request);

        let mut response = Message::new();
        response.set_header(Header::response_from_request(request.header()));

        for query in request.queries() {
            let records = self
                .query_handler
                .do_query(request.header(), &query, self.database)
                .await?;

            // response.add_query(query.clone());
            response.add_answers(records);
        }

        log::trace!("Response: {:#?}", response);
        let response_bytes = response.to_bytes()?;
        self.listener.send_to(&response_bytes, src).await?;

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
