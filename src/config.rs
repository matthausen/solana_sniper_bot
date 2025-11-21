#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub dexscreener_key: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        Config {
            database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://postgres:postgres@localhost:5432/memebot".to_string()
            }),
            dexscreener_key: std::env::var("DEXSCREENER_KEY").ok(),
        }
    }
}
