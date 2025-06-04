use haven_dns::{config::UpstreamConfig, server::DNSServer};
use log::info;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();

    // 创建配置
    let config = UpstreamConfig {
        listen: SocketAddr::from(([127, 0, 0, 1], 5353)),
        upstream_nameservers: vec![
            haven_dns::config::NameServer::Udp(SocketAddr::from(([8, 8, 8, 8], 53))),
            haven_dns::config::NameServer::Doh("https://dns.google/dns-query".to_string()),
        ],
    };

    // 创建并运行服务器
    let server = DNSServer::new(config).await?;
    info!("Starting Haven DNS server...");
    server.run().await?;

    Ok(())
}
