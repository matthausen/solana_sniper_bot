use std::env;

pub struct Config {
    pub database_url: String,
    pub moralis_key: Option<String>,
    pub solscan_key: Option<String>,
    pub dexscreener_key: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/memebot".to_string());
        let moralis_key = env::var("MORALIS_KEY").ok();
        let solscan_key = env::var("SOLSCAN_KEY").ok();
        let dexscreener_key: Option<String> = env::var("DEXSCREENER_KEY").ok();
        Config { database_url, moralis_key, solscan_key, dexscreener_key }
    }
}