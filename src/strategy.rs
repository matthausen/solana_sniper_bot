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
}

impl TokenEvent {
    pub fn compute_score(&self) -> f64 {
        let mut score = 50.0;
        score -= (self.dev_hold_pct - 5.0) * 0.6;
        score += (self.liquidity_usd / 2000.0).min(20.0);
        score += (self.holders as f64 / 200.0).min(20.0);
        if self.upgradeable { score -= 15.0; }
        if self.freeze_authority { score -= 10.0; }
        if self.momentum { score += 20.0; }
        if self.graduation { score += 25.0; }
        score.max(0.0).min(100.0)
    }

    pub fn passes_basic_filters(&self) -> bool {
        if self.market_cap_usd < 50_000.0 || self.market_cap_usd > 300_000.0 { return false; }
        if self.holders < 200 { return false; }
        if self.dev_hold_pct > 15.0 { return false; }
        if self.upgradeable || self.freeze_authority { return false; }
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
    let should_buy = basic && score >= 50.0 && (event.momentum || event.graduation);
    TradeDecision { should_buy, score }
}