use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenEvent {
    pub id: String,
    pub token_type: String,
    pub market_cap_usd: f64,
    pub dev_hold_pct: f64,
    pub liquidity_usd: f64,
    pub holders: i32,
    pub upgradeable: bool,
    pub freeze_authority: bool,
    pub momentum: bool,
    pub graduation: bool,
    pub base_price: f64,
    // New fields for enhanced strategy
    pub dev_wallet_address: Option<String>,
    pub is_dev_known_rugger: bool,
    pub entry_market_cap: f64,
    pub raydium_lp_detected: bool,
}

impl TokenEvent {
    pub fn compute_score(&self) -> f64 {
        let mut score = 50.0;

        // Known rugger = instant fail
        if self.is_dev_known_rugger {
            return 0.0;
        }

        // Holder count: 200+ holders is critical (up to +30 points)
        if self.holders >= 200 {
            score += ((self.holders as f64 - 200.0) / 50.0).min(30.0);
        } else {
            // Penalty for low holders
            score -= (200.0 - self.holders as f64) / 10.0;
        }

        // Dev hold percentage: stricter penalties
        if self.dev_hold_pct > 15.0 {
            score -= 100.0; // Auto-fail
        } else if self.dev_hold_pct > 10.0 {
            score -= (self.dev_hold_pct - 10.0) * 4.0; // -20 points at 15%
        } else if self.dev_hold_pct < 5.0 {
            score += 10.0; // Bonus for low dev hold
        }

        // Liquidity: strong buy pressure indicator (up to +25 points)
        score += (self.liquidity_usd / 1000.0).min(25.0);

        // Market cap sweet spot: $50k-$250k
        if self.market_cap_usd >= 50_000.0 && self.market_cap_usd <= 250_000.0 {
            score += 15.0;
        } else if self.market_cap_usd > 250_000.0 && self.market_cap_usd <= 300_000.0 {
            score += 5.0; // Small bonus for near sweet spot
        }

        // Safety flags
        if self.upgradeable {
            score -= 20.0;
        }
        if self.freeze_authority {
            score -= 15.0;
        }

        // Momentum and graduation signals
        if self.momentum {
            score += 20.0;
        }
        if self.graduation {
            score += 25.0;
        }

        score.max(0.0).min(100.0)
    }

    pub fn passes_basic_filters(&self) -> bool {
        // Known rugger = instant reject
        if self.is_dev_known_rugger {
            return false;
        }
        // Market cap: $50k - $250k sweet spot (allow up to $300k)
        if self.market_cap_usd < 50_000.0 || self.market_cap_usd > 300_000.0 {
            return false;
        }
        // Holders: minimum 200
        if self.holders < 200 {
            return false;
        }
        // Dev hold: strict < 15%
        if self.dev_hold_pct >= 15.0 {
            return false;
        }
        // Safety: no upgradeable or freeze authority
        if self.upgradeable || self.freeze_authority {
            return false;
        }
        true
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TradeDecision {
    pub should_buy: bool,
    pub score: f64,
}

pub fn decide(event: &TokenEvent) -> TradeDecision {
    let score = event.compute_score();
    let basic = event.passes_basic_filters();
    // README specifies score >= 75 threshold
    let should_buy = basic && score >= 75.0 && (event.momentum || event.graduation);
    TradeDecision { should_buy, score }
}

#[derive(Debug, Clone)]
pub struct ExitDecision {
    pub should_exit: bool,
    pub reason: String,
}

/// Determine if a position should be exited based on current token state
///
/// Exit conditions per README:
/// - Option A: +50% to +100% profit from entry MC
/// - Option B: Raydium LP detected (liquidity spike >2x)
/// - Stop loss: -20% from entry MC
pub fn should_exit(event: &TokenEvent, entry_liquidity: f64) -> ExitDecision {
    // Stop loss: -20% from entry MC
    if event.market_cap_usd < event.entry_market_cap * 0.8 {
        return ExitDecision {
            should_exit: true,
            reason: "stop_loss".to_string(),
        };
    }

    // Option A: Profit target (50-100% gain)
    let profit_pct = (event.market_cap_usd - event.entry_market_cap) / event.entry_market_cap;
    if profit_pct >= 0.5 && profit_pct <= 1.0 {
        return ExitDecision {
            should_exit: true,
            reason: "profit_target".to_string(),
        };
    }

    // Option B: Raydium LP detected (liquidity spike >2x)
    // Also check for raydium_lp_detected flag
    if event.raydium_lp_detected
        || (entry_liquidity > 0.0 && event.liquidity_usd > entry_liquidity * 2.0)
    {
        return ExitDecision {
            should_exit: true,
            reason: "lp_spike".to_string(),
        };
    }

    // Graduation flag (legacy support)
    if event.graduation {
        return ExitDecision {
            should_exit: true,
            reason: "graduation".to_string(),
        };
    }

    ExitDecision {
        should_exit: false,
        reason: String::new(),
    }
}
