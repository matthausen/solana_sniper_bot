mod config;
mod db;
mod strategy;
mod scanner;
mod simulator;

use structopt::StructOpt;
use anyhow::Result;
use crate::config::Config;
use crate::db::{connect, ensure_migrations};

#[derive(StructOpt, Debug)]
#[structopt(name = "sol-memebot")]
struct Opt {
    /// simulated hours to compress
    #[structopt(long, default_value = "24")]
    hours: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let opt = Opt::from_args();
    let cfg = Config::from_env();

    let pool = connect(&cfg.database_url).await?;
    ensure_migrations(&pool).await.expect("migrations failed");

    let scanner = scanner::Scanner::new(cfg.moralis_key.clone(), cfg.solscan_key.clone(), cfg.dexscreener_key.clone());

    println!("Running simulation for {} hours (using real APIs)...", opt.hours);
    simulator::run_simulation(&pool, opt.hours, &scanner).await?;

    Ok(())
}