use std::{future::Future, pin::Pin};

use anyhow::Result;
use redis::{aio::MultiplexedConnection, Client as RedisClient};
use sqlx::{
    migrate::{Migration, MigrationSource, MigrationType, Migrator},
    postgres::PgPool,
};

use crate::config::DatabaseConfig;

pub struct Database {
    pg_pool: PgPool,
    redis_client: RedisClient,
    redis_conn: MultiplexedConnection,
}

impl Database {
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        let pg_pool = PgPool::connect(&config.postgres_url).await?;

        let redis_client = RedisClient::open(config.redis_url.clone())?;
        let redis_conn = redis_client.get_multiplexed_async_connection().await?;

        Ok(Self {
            pg_pool,
            redis_client,
            redis_conn,
        })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pg_pool
    }

    pub async fn migrate(&self) -> Result<()> {
        let migrator = Migrator::new(PgMigrations).await?;
        migrator.run(&self.pg_pool).await?;
        Ok(())
    }

    pub fn redis_client(&self) -> &RedisClient {
        &self.redis_client
    }

    pub fn redis_conn(&self) -> MultiplexedConnection {
        self.redis_conn.clone()
    }
}

#[derive(Debug)]
struct PgMigrations;

impl MigrationSource<'static> for PgMigrations {
    fn resolve(
        self,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Vec<Migration>, Box<dyn std::error::Error + Sync + Send>>>
                + Send
                + 'static,
        >,
    > {
        Box::pin(async move {
            let sql_0001 = include_str!("../../migrations/0001-pg-init.sql");
            let migration_0001 = Migration::new(
                0,
                "init database".into(),
                MigrationType::ReversibleUp,
                sql_0001.into(),
                false,
            );

            Ok(vec![migration_0001])
        })
    }
}
