
# Solana Memecoin Strategy Simulator — Rust

This repository is a **local paper-trading simulator** and strategy validation tool implemented in Rust. It implements the strategy you specified:

- **Entry**: buy tokens with market cap between 50k–250k (simulated Pump.fun universe)
- **Exit**: exit 50–100% before "graduation" or immediately after first Raydium-style liquidity add
- Extra filters: dev wallet concentration, holders > 200, avoid devs >15% supply

**Important:** This is a *simulator / paper-trader* only. It does not place real on-chain transactions.

## What is included

- `docker-compose.yml` — runs a local Postgres instance for recording simulated trades and history
- `Cargo.toml` — Rust dependencies
- `src/` — Rust source implementing:
  - a lightweight event scanner (mock + pluggable real-API stubs)
  - strategy scoring & filter logic
  - execution simulator (buy/sell mechanics, portfolio of 3 SOL)
  - persistence to Postgres (trade records, token events)
  - CLI flags for `--mock` (fast local sim) or `--realtime` (placeholder for plugging real APIs)
- `migrations/` — SQL to create tables


## Quick start (local)

1. Start Postgres with docker-compose:

```bash
docker-compose up -d
```

2. Build & run the simulator in mock mode (fast):

```bash
export DATABASE_URL=postgres://postgres:postgres@localhost:5432/memebot
cargo run --release -- --mock --hours 24
```

- `--mock` runs the internal event generator (no external API needed)
- `--hours` controls the simulated timeframe compression (defaults to 24)

3. After run completes, inspect `trades` and `token_events` tables in Postgres, or enable CSV export (see config).


## Files

```
.
├─ docker-compose.yml
├─ Cargo.toml
├─ migrations/
│  └─ 001_create_tables.sql
└─ src/
   ├─ main.rs
   ├─ config.rs
   ├─ db.rs
   ├─ scanner.rs
   ├─ strategy.rs
   └─ simulator.rs



### Api Keys
Birdeye       | yes/no Required for reliable volume + MC + holders
Solscan       | yes/no Required
DEX Screener  | not needed Public API but rate limits apply
Pump.fun      | no key needed But we must use correct endpoints
