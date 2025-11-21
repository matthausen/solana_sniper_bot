use crate::scanner::Scanner;
use crate::strategy::{TokenEvent, decide};
use anyhow::Result;
use chrono::Utc;
use rand::Rng;
use sqlx::PgPool;

pub struct Portfolio {
    pub sol_balance: f64,
    pub positions: Vec<Position>,
}

#[allow(dead_code)]
pub struct Position {
    pub token_id: String,
    pub entry_price: f64,
    pub qty: f64,
    pub usd_in: f64,
    pub opened_at: chrono::DateTime<Utc>,
    pub score: f64,
}

impl Portfolio {
    pub fn new(sol_balance: f64) -> Self {
        Self {
            sol_balance,
            positions: vec![],
        }
    }
}

pub async fn run_simulation(pool: &PgPool, minutes: u64, scanner: &Scanner) -> Result<()> {
    let mut collected = Vec::new();

    // Set deadline based on minutes parameter
    let start_time = std::time::Instant::now();
    let duration = std::time::Duration::from_secs(minutes * 60);
    let deadline = start_time + duration;

    println!("Simulation will run for {} minutes", minutes);

    while std::time::Instant::now() < deadline {
        let listings = scanner.fetch_pumpfun_listings().await.unwrap_or_default();
        println!("Fetched {} listings from Pump.fun", listings.len());
        for l in listings.into_iter() {
            // Check if we've exceeded the time limit
            if std::time::Instant::now() >= deadline {
                println!("Time limit reached, stopping collection...");
                break;
            }

            // Enrich with Moralis holder data and dexscreener data
            let mut ev: TokenEvent = l.clone().into();

            // Get holder count from Moralis
            if let Ok(Some(holder_stats)) = scanner.query_token_holder_stats(&l.token_address).await
            {
                ev.holders = holder_stats.total.unwrap_or(0) as i32;
            }

            // Get top holders to calculate dev hold percentage
            if let Ok(Some(top_holders)) = scanner.query_token_top_holders(&l.token_address).await {
                if let Some(holders_list) = top_holders.result {
                    if let Some(first_holder) = holders_list.first() {
                        // Assume first holder is the dev/creator
                        ev.dev_hold_pct = first_holder
                            .percentage_relative_to_total_supply
                            .unwrap_or(0.0);
                        ev.dev_wallet_address = first_holder.owner_address.clone();
                    }
                }
            }
            if let Ok(Some(d)) = scanner.query_dexscreener_pair(&l.token_address).await {
                if let Some(pairs) = d.pairs {
                    if let Some(first) = pairs.get(0) {
                        ev.liquidity_usd = first.liquidity_usd.unwrap_or(0.0);
                        if ev.base_price <= 0.0 {
                            ev.base_price = first.price_usd.unwrap_or(0.0);
                        }
                    }
                }
            }
            // heuristics for momentum/graduation: Pump.fun may include flags; here we set based on market cap or liquidity
            ev.momentum = ev.liquidity_usd > 1000.0;
            ev.graduation = ev.market_cap_usd >= 50000.0
                && ev.market_cap_usd <= 300000.0
                && ev.liquidity_usd > 1000.0;

            collected.push(ev);
        }
        // small delay to avoid hammering (and to wait for new listings on next poll)
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }

    // portfolio setup
    let mut portfolio = Portfolio::new(3.0);
    let sol_usd_price = 30.0;
    let max_per_trade_sol = 0.5;

    for ev in collected.into_iter() {
        // persist token event
        let score = ev.compute_score();
        sqlx::query("INSERT INTO token_events (id, token_type, market_cap_usd, dev_hold_pct, liquidity_usd, holders, upgradeable, freeze_authority, momentum, graduation, base_price, score) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12) ON CONFLICT (id) DO NOTHING")
            .bind(&ev.id)
            .bind(&ev.token_type)
            .bind(ev.market_cap_usd)
            .bind(ev.dev_hold_pct)
            .bind(ev.liquidity_usd)
            .bind(ev.holders)
            .bind(ev.upgradeable)
            .bind(ev.freeze_authority)
            .bind(ev.momentum)
            .bind(ev.graduation)
            .bind(ev.base_price)
            .bind(score)
            .execute(pool)
            .await?;

        let decision = decide(&ev);
        // Enforce max 5 active positions (per README)
        if decision.should_buy && portfolio.sol_balance > 0.01 && portfolio.positions.len() < 5 {
            let to_spend_sol = f64::min(max_per_trade_sol, portfolio.sol_balance);
            let mut rng = rand::thread_rng();
            let impact = 1.0 + rng.gen_range(0.0..0.05);
            let entry_price = ev.base_price * impact;
            let usd_in = to_spend_sol * sol_usd_price;
            let qty = if entry_price > 0.0 {
                usd_in / entry_price
            } else {
                0.0
            };

            portfolio.sol_balance -= to_spend_sol;
            let pos = Position {
                token_id: ev.id.clone(),
                entry_price,
                qty,
                usd_in,
                opened_at: Utc::now(),
                score,
            };
            portfolio.positions.push(pos);

            sqlx::query("INSERT INTO trades (token_id, action, entry_price, qty, usd_in, opened_at, score) VALUES ($1,$2,$3,$4,$5,NOW(),$6)")
                .bind(&ev.id)
                .bind("BUY")
                .bind(entry_price)
                .bind(qty)
                .bind(usd_in)
                .bind(score)
                .execute(pool)
                .await?;
        }

        // Simulate exits using strategy-based exit logic
        let mut closed_idxs = vec![];
        for (idx, pos) in portfolio.positions.iter().enumerate() {
            // Re-query current state for this token
            let mut current_ev = ev.clone();
            current_ev.id = pos.token_id.clone();

            // Get current liquidity for LP spike detection
            let entry_liquidity = ev.liquidity_usd;

            if let Ok(Some(d)) = scanner.query_dexscreener_pair(&pos.token_id).await {
                if let Some(p) = d.pairs.and_then(|v| v.get(0).cloned()) {
                    let current_liquidity = p.liquidity_usd.unwrap_or(0.0);
                    current_ev.liquidity_usd = current_liquidity;

                    // Detect Raydium LP spike (>2x liquidity increase)
                    if current_liquidity > entry_liquidity * 2.0 {
                        current_ev.raydium_lp_detected = true;
                    }
                }
            }

            // Use strategy exit logic
            use crate::strategy::should_exit;
            let exit_decision = should_exit(&current_ev, entry_liquidity);

            if exit_decision.should_exit {
                let mut rng = rand::thread_rng();
                // Different multipliers based on exit reason
                let mult = match exit_decision.reason.as_str() {
                    "profit_target" => rng.gen_range(1.5..2.5),
                    "lp_spike" => rng.gen_range(1.3..3.0),
                    "stop_loss" => rng.gen_range(0.6..0.8),
                    "graduation" => rng.gen_range(1.5..3.0),
                    _ => rng.gen_range(1.2..2.0),
                };

                let exit_price = pos.entry_price * mult;
                let proceeds_usd = pos.qty * exit_price;
                let proceeds_sol = proceeds_usd / sol_usd_price;
                portfolio.sol_balance += proceeds_sol;

                sqlx::query("UPDATE trades SET action=$1, exit_price=$2, pnl=$3, closed_at=NOW() WHERE token_id=$4 AND action='BUY' AND exit_price IS NULL")
                    .bind("SELL")
                    .bind(exit_price)
                    .bind(proceeds_usd - pos.usd_in)
                    .bind(&pos.token_id)
                    .execute(pool)
                    .await?;

                closed_idxs.push(idx);

                println!(
                    "Exit: {} reason={} mult={:.2}x pnl=${:.2}",
                    pos.token_id,
                    exit_decision.reason,
                    mult,
                    proceeds_usd - pos.usd_in
                );
            }
        }
        for j in closed_idxs.iter().rev() {
            portfolio.positions.remove(*j);
        }
    }

    sqlx::query("INSERT INTO run_metadata (finished_at) VALUES (NOW())")
        .execute(pool)
        .await?;

    println!(
        "Simulation finished. Remaining SOL balance: {} SOL",
        portfolio.sol_balance
    );
    Ok(())
}
