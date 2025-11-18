CREATE TABLE IF NOT EXISTS token_events (
  id TEXT PRIMARY KEY,
  generated_at TIMESTAMP WITH TIME ZONE DEFAULT now(),
  token_type TEXT,
  market_cap_usd DOUBLE PRECISION,
  dev_hold_pct DOUBLE PRECISION,
  liquidity_usd DOUBLE PRECISION,
  holders INTEGER,
  upgradeable BOOLEAN,
  freeze_authority BOOLEAN,
  momentum BOOLEAN,
  graduation BOOLEAN,
  base_price DOUBLE PRECISION,
  score DOUBLE PRECISION
);

CREATE TABLE IF NOT EXISTS trades (
  id SERIAL PRIMARY KEY,
  token_id TEXT,
  action TEXT,
  entry_price DOUBLE PRECISION,
  exit_price DOUBLE PRECISION,
  qty DOUBLE PRECISION,
  usd_in DOUBLE PRECISION,
  pnl DOUBLE PRECISION,
  opened_at TIMESTAMP WITH TIME ZONE,
  closed_at TIMESTAMP WITH TIME ZONE,
  score DOUBLE PRECISION
);

CREATE TABLE IF NOT EXISTS run_metadata (
  id SERIAL PRIMARY KEY,
  started_at TIMESTAMP WITH TIME ZONE DEFAULT now(),
  finished_at TIMESTAMP WITH TIME ZONE
);