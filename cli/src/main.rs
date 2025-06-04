use anyhow::Result;
use haven_db::Database;
use haven_dns::server::DNSServer;
use log::info;

mod config;
use config::Config;

async fn run_server(config: Config) -> Result<()> {
    let database = Database::new(&config.database).await?;
    database.migrate().await?;

    let server = DNSServer::new(config.dns, &database).await?;
    info!("Starting Haven DNS server...");
    server.run().await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let config = Config::load("config.toml")?;
    info!("Loaded configuration from config.toml");

    run_server(config).await?;

    Ok(())
}
