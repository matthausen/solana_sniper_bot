
# Strategy

Buy low-market-cap Pump.fun tokens before Raydium listing, and sell right before or right after the first liquidity event, while filtering out rugs using on-chain data.

1. ENTRY STRATEGY (When to Buy)

Target: Early Pump.fun tokens with low market cap
	•	Market Cap at Entry:
$50k → $250k
(Avoid >$300k — rug probability increases sharply)

Only buy tokens that meet ALL conditions below:

✔ A. Reputation Filters
	•	Dev wallet not linked to known rug pulls (track dev addresses from prior tokens)
	•	Dev holds <15% of supply
(Big dev bags = high rug probability)

✔ B. Organic Traction Filters
	•	Token is new and listed on:
	•	Moralis Pump.fun feed
	•	Trending on Solscan, DEX Screener, Birdeye
	•	At least 200+ holders pre-Raydium
	•	Strong buy pressure (liquidity flowing in)

✔ C. On-Chain Safety Checks
	•	No suspicious liquidity provider patterns
	•	No stealth mints
	•	No sudden supply increases
	•	Verified metadata present


✅ 2. EXIT STRATEGY (When to Sell)

Two exit conditions — whichever comes first:

Option A — BEFORE Raydium Listing

Sell when MC reaches:
+50% to +100% profit
(High win-rate because most pumps fade before LP)

Option B — RIGHT AFTER Raydium LP Creation

Sell immediately after first Raydium liquidity add
	•	This is usually a fast 1.3x–3x spike
	•	Take profits early — don’t hold longs after initial LP

DO NOT hold long-term unless it becomes a real community coin (rare).

⸻

📊 3. Position Sizing & Portfolio Rules
	•	Total simulated bankroll: 3 SOL
	•	Max per trade: 0.5 SOL
	•	Never overlap more than 5 active positions
	•	Use fixed risk:
	•	Cut loss if MC drops −20% from entry
	•	Or if dev behavior turns suspicious

⸻

🧠 4. Bot Logic Summary (What the scanner does)

A. Data Sources (all free or low-cost, now standardized to Moralis)
	•	Moralis Pump.fun New Listings API → discover tokens
	•	Moralis Token Metadata → supply, decimals, creation
	•	Moralis Holder API → holder count
	•	Moralis Token Distribution → dev bag %

B. Scoring System

Bot assigns a “trade score” based on:
	•	Holder count
	•	Dev bag %
	•	Market cap
	•	Traction speed
	•	Rug signals
	•	Liquidity inflow

Only buys tokens above a threshold (e.g., Score ≥ 75).

⸻

💰 5. Expected Outcomes (Statistically)

With proper filtering:
	•	Win rate: 55–70%
	•	Typical profit per win: +40–150%
	•	Typical loss per rug: −20–100%
	•	Overall expectancy: positive if rug filters are strong

Snipers who follow this approach often grow small portfolios 3×–10× over weeks, but it requires strict filtering and fast exits.


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
