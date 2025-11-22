use crate::strategy_config::StrategyConfig;
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
    pub fn compute_score(&self, config: &StrategyConfig) -> f64 {
        let mut score = 50.0;

        // Known rugger = instant fail
        if self.is_dev_known_rugger {
            return 0.0;
        }

        // Holder count: bonus for holders above minimum
        if self.holders >= config.min_holders {
            score += ((self.holders as f64 - config.min_holders as f64) / 50.0).min(30.0);
        } else {
            // Penalty for low holders
            score -= (config.min_holders as f64 - self.holders as f64) / 10.0;
        }

        // Dev hold percentage: stricter penalties
        if self.dev_hold_pct > config.max_dev_hold_pct {
            score -= 100.0; // Auto-fail
        } else if self.dev_hold_pct > 10.0 {
            score -= (self.dev_hold_pct - 10.0) * config.high_dev_hold_penalty_multiplier;
        } else if self.dev_hold_pct < 5.0 {
            score += config.low_dev_hold_bonus;
        }

        // Liquidity: strong buy pressure indicator
        score += (self.liquidity_usd / config.liquidity_bonus_divisor).min(25.0);

        // Market cap sweet spot
        if self.market_cap_usd >= 50_000.0 && self.market_cap_usd <= 250_000.0 {
            score += config.market_cap_sweet_spot_bonus;
        } else if self.market_cap_usd > 250_000.0
            && self.market_cap_usd <= config.max_market_cap_usd
        {
            score += 5.0; // Small bonus for near sweet spot
        }

        // Safety flags
        if self.upgradeable {
            score -= config.upgradeable_penalty;
        }
        if self.freeze_authority {
            score -= config.freeze_authority_penalty;
        }

        // Momentum and graduation signals
        if self.momentum {
            score += config.momentum_bonus;
        }
        if self.graduation {
            score += config.graduation_bonus;
        }

        score.max(0.0).min(100.0)
    }

    pub fn passes_basic_filters(&self, config: &StrategyConfig) -> bool {
        // Known rugger = instant reject
        if self.is_dev_known_rugger {
            return false;
        }
        // Market cap range
        if self.market_cap_usd < config.min_market_cap_usd
            || self.market_cap_usd > config.max_market_cap_usd
        {
            return false;
        }
        // Holders minimum
        if self.holders < config.min_holders {
            return false;
        }
        // Dev hold maximum
        if self.dev_hold_pct >= config.max_dev_hold_pct {
            return false;
        }
        // Safety: reject based on config
        if config.reject_upgradeable && self.upgradeable {
            return false;
        }
        if config.reject_freeze_authority && self.freeze_authority {
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

pub fn decide(event: &TokenEvent, config: &StrategyConfig) -> TradeDecision {
    let score = event.compute_score(config);
    let basic = event.passes_basic_filters(config);

    let should_buy = basic
        && score >= config.min_score_to_buy
        && (!config.require_momentum_or_graduation || event.momentum || event.graduation);

    TradeDecision { should_buy, score }
}

#[derive(Debug, Clone)]
pub struct ExitDecision {
    pub should_exit: bool,
    pub reason: String,
}

/// Determine if a position should be exited based on current token state
pub fn should_exit(
    event: &TokenEvent,
    entry_liquidity: f64,
    config: &StrategyConfig,
) -> ExitDecision {
    // Stop loss
    if event.market_cap_usd < event.entry_market_cap * (1.0 - config.stop_loss_pct) {
        return ExitDecision {
            should_exit: true,
            reason: "stop_loss".to_string(),
        };
    }

    // Profit target
    let profit_pct = (event.market_cap_usd - event.entry_market_cap) / event.entry_market_cap;
    if profit_pct >= config.min_profit_target_pct && profit_pct <= config.max_profit_target_pct {
        return ExitDecision {
            should_exit: true,
            reason: "profit_target".to_string(),
        };
    }

    // Liquidity spike (Raydium LP detected)
    if event.raydium_lp_detected
        || (entry_liquidity > 0.0
            && event.liquidity_usd > entry_liquidity * config.lp_spike_exit_multiplier)
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
