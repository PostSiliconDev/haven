use anyhow::Result;
use haven_dns::server::DNSServer;
use log::info;

mod config;
use config::Config;

async fn run_server(config: Config) -> Result<()> {
    // 创建并运行服务器
    let server = DNSServer::new(config.dns).await?;
    info!("Starting Haven DNS server...");
    server.run().await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    env_logger::init();

    // 加载配置
    let config = Config::load("config.toml")?;
    info!("Loaded configuration from config.toml");

    // 运行服务器
    run_server(config).await?;

    Ok(())
}
