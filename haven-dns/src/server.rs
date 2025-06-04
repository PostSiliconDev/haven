use crate::config::{NameServer, UpstreamConfig};
use anyhow::Result;
use futures::future::select_all;
use hickory_proto::op::Message;
use hickory_proto::serialize::binary::{BinDecodable, BinEncodable};
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
    pub async fn new(config: UpstreamConfig) -> Result<Self> {
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

    async fn query_upstreams(&self, request: Message) -> Result<Vec<u8>> {
        // Create futures for UDP requests
        let mut udp_futures = Vec::new();
        for upstream in &self.udp_upstreams {
            let socket = UdpSocket::bind("0.0.0.0:0").await?;
            let request = request.clone();
            let upstream = *upstream;
            udp_futures.push(tokio::spawn(async move {
                let mut response_buf = [0u8; 512];
                socket.send_to(&request.to_bytes()?, upstream).await?;
                let (size, _) = socket.recv_from(&mut response_buf).await?;
                Ok::<_, anyhow::Error>(response_buf[..size].to_vec())
            }));
        }

        // Create futures for DoH requests
        let mut doh_futures = Vec::new();
        for upstream in &self.doh_upstreams {
            let client = self.client.clone();
            let request = request.clone();
            let upstream = upstream.clone();
            doh_futures.push(tokio::spawn(async move {
                let response = client
                    .post(upstream)
                    .body(request.to_bytes()?)
                    .send()
                    .await?
                    .bytes()
                    .await?;
                Ok::<_, anyhow::Error>(response.to_vec())
            }));
        }

        // Combine all futures
        let mut all_futures = Vec::new();
        all_futures.extend(udp_futures);
        all_futures.extend(doh_futures);

        let (result, _, _) = select_all(all_futures).await;

        let res = result??;

        Ok(res)
    }

    async fn handle_query(&self, buf: &mut [u8]) -> Result<()> {
        let (size, src) = self.listener.recv_from(buf).await?;
        log::debug!("Received {} bytes from {}", size, src);

        let request = Message::from_bytes(&buf[..size])?;
        let response = self.query_upstreams(request).await?;
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
