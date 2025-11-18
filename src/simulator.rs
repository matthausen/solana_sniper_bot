use crate::strategy::{TokenEvent, decide};
use crate::scanner::Scanner;
use sqlx::PgPool;
use chrono::{Utc};
use rand::Rng;
use anyhow::Result;

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
        Self { sol_balance, positions: vec![] }
    }
}

pub async fn run_simulation(pool: &PgPool, _hours: u64, scanner: &Scanner) -> Result<()> {
    // We'll poll Pump.fun for recent listings in a loop until we've collected a target number.
    let mut collected = Vec::new();
    let target = 1500usize;

    while collected.len() < target {
        let listings = scanner.fetch_pumpfun_listings().await.unwrap_or_default();
        for l in listings.into_iter() {
            // Enrich with solscan and dexscreener data
            let mut ev: TokenEvent = l.clone().into();
            if let Ok(Some(s)) = scanner.query_solscan_token(&l.mint).await {
                ev.holders = s.holders.unwrap_or(0) as i32;
                // naive dev_hold estimate: check top owner if present (placeholder)
                if let Some(owner_amount) = s.owner_amount.as_ref().and_then(|v| v.get(0)) {
                    // owner_amount is (address, amount) tuple in placeholder
                    let pct = owner_amount.1 / s.total_supply.unwrap_or(1.0) * 100.0;
                    ev.dev_hold_pct = pct.min(100.0);
                }
            }
            if let Ok(Some(d)) = scanner.query_dexscreener_pair(&l.mint).await {
                if let Some(pairs) = d.pairs {
                    if let Some(first) = pairs.get(0) {
                        ev.liquidity_usd = first.liquidity_usd.unwrap_or(0.0);
                        if ev.base_price <= 0.0 { ev.base_price = first.price_usd.unwrap_or(0.0); }
                    }
                }
            }
            // heuristics for momentum/graduation: Pump.fun may include flags; here we set based on market cap or liquidity
            ev.momentum = ev.liquidity_usd > 1000.0;
            ev.graduation = ev.market_cap_usd >= 50000.0 && ev.market_cap_usd <= 300000.0 && ev.liquidity_usd > 1000.0;

            collected.push(ev);
            if collected.len() >= target { break; }
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
        if decision.should_buy && portfolio.sol_balance > 0.01 {
            let to_spend_sol = f64::min(max_per_trade_sol, portfolio.sol_balance);
            let mut rng = rand::thread_rng();
            let impact = 1.0 + rng.gen_range(0.0..0.05);
            let entry_price = ev.base_price * impact;
            let usd_in = to_spend_sol * sol_usd_price;
            let qty = if entry_price>0.0 { usd_in / entry_price } else { 0.0 };

            portfolio.sol_balance -= to_spend_sol;
            let pos = Position { token_id: ev.id.clone(), entry_price, qty, usd_in, opened_at: Utc::now(), score };
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

        // simulate exits: if event.graduation or DEX shows a liquidity spike, sell
        let mut closed_idxs = vec![];
        for (idx, pos) in portfolio.positions.iter().enumerate() {
            // check if this token has a liquidity spike by re-querying DexScreener (best-effort)
            let spike = match scanner.query_dexscreener_pair(&pos.token_id).await {
                Ok(Some(d)) => {
                    if let Some(p) = d.pairs.and_then(|v| v.get(0).cloned()) {
                        p.liquidity_usd.unwrap_or(0.0) > 1000.0
                    } else { false }
                }
                _ => false
            };

            if spike {
                // sell: simulate exit multiplier small (e.g., 1.2x)
                let mut rng = rand::thread_rng();
                let mult = rng.gen_range(1.2..2.5);
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
            }

            // also sell if event.graduation flag from original event
            if ev.graduation && ev.id == pos.token_id {
                let mut rng = rand::thread_rng();
                let mult = rng.gen_range(1.5..3.0);
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
            }
        }
        for j in closed_idxs.iter().rev() {
            portfolio.positions.remove(*j);
        }
    }

    sqlx::query("INSERT INTO run_metadata (finished_at) VALUES (NOW())")
        .execute(pool)
        .await?;

    println!("Simulation finished. Remaining SOL balance: {} SOL", portfolio.sol_balance);
    Ok(())
}