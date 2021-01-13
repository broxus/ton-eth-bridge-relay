use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub address: String,
    pub next_block_timeout_sec: u32,
    pub parallel_connections: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            address: "https://main.ton.dev/graphql".to_owned(),
            next_block_timeout_sec: 60,
            parallel_connections: 100,
        }
    }
}
