/// Centralized configuration for all trading strategy parameters
/// All filter thresholds and trading rules are defined here for easy tuning

#[derive(Debug, Clone)]
pub struct StrategyConfig {
    // === ENTRY FILTERS ===
    /// Minimum market cap in USD to consider buying
    pub min_market_cap_usd: f64,

    /// Maximum market cap in USD to consider buying
    pub max_market_cap_usd: f64,

    /// Minimum number of holders required
    pub min_holders: i32,

    /// Maximum dev/creator hold percentage allowed (e.g., 15.0 = 15%)
    pub max_dev_hold_pct: f64,

    /// Minimum liquidity in USD required
    pub min_liquidity_usd: f64,

    /// Reject if token is upgradeable
    pub reject_upgradeable: bool,

    /// Reject if token has freeze authority
    pub reject_freeze_authority: bool,

    /// Minimum score required to buy (0-100)
    pub min_score_to_buy: f64,

    /// Require momentum flag (liquidity > threshold)
    pub require_momentum_or_graduation: bool,

    // === SCORING WEIGHTS ===
    /// Bonus points for low dev hold (< 5%)
    pub low_dev_hold_bonus: f64,

    /// Penalty multiplier for high dev hold (10-15%)
    pub high_dev_hold_penalty_multiplier: f64,

    /// Liquidity bonus divisor (liquidity_usd / this = bonus points, max 25)
    pub liquidity_bonus_divisor: f64,

    /// Market cap sweet spot bonus (within min/max range)
    pub market_cap_sweet_spot_bonus: f64,

    /// Momentum bonus points
    pub momentum_bonus: f64,

    /// Graduation bonus points
    pub graduation_bonus: f64,

    /// Penalty for upgradeable token
    pub upgradeable_penalty: f64,

    /// Penalty for freeze authority
    pub freeze_authority_penalty: f64,

    // === EXIT RULES ===
    /// Stop loss percentage (e.g., 0.2 = -20%)
    pub stop_loss_pct: f64,

    /// Minimum profit target percentage (e.g., 0.5 = +50%)
    pub min_profit_target_pct: f64,

    /// Maximum profit target percentage (e.g., 1.0 = +100%)
    pub max_profit_target_pct: f64,

    /// Liquidity spike multiplier for exit (e.g., 2.0 = 2x increase)
    pub lp_spike_exit_multiplier: f64,

    // === PORTFOLIO RULES ===
    /// Maximum number of concurrent positions
    pub max_positions: usize,

    /// Maximum SOL to spend per trade
    pub max_sol_per_trade: f64,

    /// Starting SOL balance for simulation
    pub starting_sol_balance: f64,

    /// Assumed SOL/USD price for calculations
    pub sol_usd_price: f64,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            // === ENTRY FILTERS ===
            min_market_cap_usd: 5_000.0, // $5k minimum (was $50k - too high for new tokens)
            max_market_cap_usd: 300_000.0, // $300k maximum
            min_holders: 10,             // 10 holders minimum (was 200 - too high for new tokens)
            max_dev_hold_pct: 15.0,      // 15% max dev hold
            min_liquidity_usd: 1_000.0,  // $1k minimum liquidity
            reject_upgradeable: true,    // Reject upgradeable tokens
            reject_freeze_authority: true, // Reject tokens with freeze authority
            min_score_to_buy: 75.0,      // 75/100 minimum score
            require_momentum_or_graduation: true, // Require momentum OR graduation

            // === SCORING WEIGHTS ===
            low_dev_hold_bonus: 10.0, // +10 points for dev hold < 5%
            high_dev_hold_penalty_multiplier: 4.0, // -4 points per % above 10%
            liquidity_bonus_divisor: 1_000.0, // liquidity_usd / 1000 = bonus (max 25)
            market_cap_sweet_spot_bonus: 15.0, // +15 points for $50k-$250k range
            momentum_bonus: 20.0,     // +20 points for momentum
            graduation_bonus: 25.0,   // +25 points for graduation
            upgradeable_penalty: 20.0, // -20 points if upgradeable
            freeze_authority_penalty: 15.0, // -15 points if freeze authority

            // === EXIT RULES ===
            stop_loss_pct: 0.2,            // -20% stop loss
            min_profit_target_pct: 0.5,    // +50% minimum profit target
            max_profit_target_pct: 1.0,    // +100% maximum profit target
            lp_spike_exit_multiplier: 2.0, // Exit if liquidity 2x

            // === PORTFOLIO RULES ===
            max_positions: 5,          // Max 5 concurrent positions
            max_sol_per_trade: 0.5,    // 0.5 SOL per trade
            starting_sol_balance: 3.0, // Start with 3 SOL
            sol_usd_price: 30.0,       // Assume $30/SOL
        }
    }
}

impl StrategyConfig {
    /// Create config optimized for early sniping (catching tokens right at launch)
    pub fn early_snipe() -> Self {
        Self {
            min_market_cap_usd: 1_000.0, // $1k minimum - catch very early
            min_holders: 5,              // Only 5 holders needed
            min_liquidity_usd: 500.0,    // $500 minimum liquidity
            min_score_to_buy: 65.0,      // Lower score threshold
            ..Default::default()
        }
    }

    /// Create config optimized for safer, established tokens
    pub fn conservative() -> Self {
        Self {
            min_market_cap_usd: 50_000.0, // $50k minimum
            min_holders: 200,             // 200 holders minimum
            max_dev_hold_pct: 10.0,       // Stricter 10% max
            min_liquidity_usd: 5_000.0,   // $5k minimum liquidity
            min_score_to_buy: 80.0,       // Higher score threshold
            ..Default::default()
        }
    }

    /// Create config optimized for aggressive trading
    pub fn aggressive() -> Self {
        Self {
            min_market_cap_usd: 2_000.0, // $2k minimum
            min_holders: 3,              // Only 3 holders
            max_dev_hold_pct: 20.0,      // Allow higher dev hold
            min_liquidity_usd: 300.0,    // $300 minimum
            min_score_to_buy: 60.0,      // Lower threshold
            max_positions: 10,           // More positions
            ..Default::default()
        }
    }
}
