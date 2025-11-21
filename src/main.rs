mod config;
mod db;
mod scanner;
mod simulator;
mod strategy;

use crate::config::Config;
use crate::db::{connect, ensure_migrations};
use anyhow::Result;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "sol-memebot")]
struct Opt {
    /// simulated minutes to run
    #[structopt(long, default_value = "60")]
    minutes: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let opt = Opt::from_args();
    let cfg = Config::from_env();

    let pool = connect(&cfg.database_url).await?;
    ensure_migrations(&pool).await.expect("migrations failed");

    let scanner = scanner::Scanner::new(
        cfg.moralis_key.clone(),
        cfg.solscan_key.clone(),
        cfg.dexscreener_key.clone(),
    );

    println!(
        "Running simulation for {} minutes (using real APIs)...",
        opt.minutes
    );
    simulator::run_simulation(&pool, opt.minutes, &scanner).await?;

    Ok(())
}
