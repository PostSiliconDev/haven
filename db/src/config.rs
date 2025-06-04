use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub postgres_url: String,
    pub redis_url: String,
}

impl DatabaseConfig {
    pub fn new(postgres_url: String, redis_url: String) -> Self {
        Self {
            postgres_url,
            redis_url,
        }
    }
}
