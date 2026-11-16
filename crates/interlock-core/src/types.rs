use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Market identifier, e.g. "SOL-PERP".
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MarketId(pub String);

impl From<&str> for MarketId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Asset identifier, e.g. "USDC" or "SOL".
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetId(pub String);

impl From<&str> for AssetId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Trade position side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Long,
    Short,
}

/// Input parameters for cost calculation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PositionInput {
    pub market: MarketId,
    pub side: Side,
    /// Collateral amount in USD.
    pub collateral_usd: Decimal,
    /// Target leverage factor.
    pub leverage: Decimal,
    /// Expected holding period in hours (may be fractional).
    pub horizon_hours: Decimal,
    /// Collateral asset mint/symbol.
    pub collateral_asset: AssetId,
}

/// Estimated price impact model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImpactModel {
    pub base_impact_bps: Decimal,
    pub slope_bps_per_million: Decimal,
}

/// Parameters decoded from venue on-chain state at a specific slot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VenueParams {
    /// Open fee in basis points (e.g. 6 = 0.06%).
    pub open_fee_bps: Decimal,
    /// Close fee in basis points (e.g. 6 = 0.06%).
    pub close_fee_bps: Decimal,
    /// Hourly borrow rate fraction (e.g. 0.00001 = 0.001%/hour).
    pub borrow_rate_per_hour: Decimal,
    /// Maximum allowable leverage.
    pub max_leverage: Decimal,
    /// Maintenance margin requirement fraction.
    pub maintenance_margin: Decimal,
    /// Price impact parameters if derivable from liquidity state.
    pub impact: Option<ImpactModel>,
}

/// Market snapshot recorded by the logger.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarketSnapshot {
    /// Slot at which state was fetched.
    pub slot: u64,
    /// Unix timestamp in UTC seconds.
    pub unix_ts: i64,
    /// Target market identifier.
    pub market: MarketId,
    /// Venue mark price.
    pub mark_price: Decimal,
    /// Primary oracle price (e.g. Pyth).
    pub oracle_price: Decimal,
    /// Primary oracle confidence interval in USD.
    pub oracle_conf: Decimal,
    /// Pool utilization in basis points (0 - 10000).
    pub utilization_bps: u32,
    /// Decoded venue parameters.
    pub venue: VenueParams,
    /// RPC provider identifier that served the snapshot.
    pub rpc_id: String,
}

/// Projected cost at a specific holding horizon.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HorizonPoint {
    pub hours: Decimal,
    pub borrow_usd: Decimal,
    pub total_usd: Decimal,
    pub total_pct_of_collateral: Decimal,
}

/// Sensitivity row comparing borrow cost under rate variations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RateSensitivity {
    pub multiplier: Decimal,
    pub borrow_rate_per_hour: Decimal,
    pub borrow_usd: Decimal,
    pub total_usd: Decimal,
}

/// Complete itemised cost breakdown for a position.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CostBreakdown {
    /// Slot number at which state was observed.
    pub as_of_slot: u64,
    /// Position size in USD (collateral * leverage).
    pub size_usd: Decimal,
    /// Open fee in USD.
    pub open_fee_usd: Decimal,
    /// Close fee in USD.
    pub close_fee_usd: Decimal,
    /// Estimated price impact in USD (None if unavailable).
    pub impact_usd: Option<Decimal>,
    /// Projected borrow cost in USD over requested horizon.
    pub borrow_usd: Decimal,
    /// Total cost in USD (open + close + impact + borrow).
    pub total_usd: Decimal,
    /// Total cost expressed as percentage of initial collateral.
    pub total_pct_of_collateral: Decimal,
    /// Required price move percentage to break even on costs.
    pub breakeven_move_pct: Decimal,
    /// Estimated liquidation price in USD.
    pub liq_price: Decimal,
    /// Distance to liquidation as percentage of current mark price.
    pub liq_distance_pct: Decimal,
    /// Cost sweep across standardized horizons (1h, 24h, 72h, 168h).
    pub sweep: Vec<HorizonPoint>,
    /// Rate sensitivity scenarios (e.g. 0.5x, 1.0x, 2.0x).
    pub sensitivity: Vec<RateSensitivity>,
    /// Explicit list of assumptions made during computation.
    pub assumptions: Vec<String>,
    /// Explicit list of unverified inputs or parameters used.
    pub unverified_inputs: Vec<String>,
}

/// Historical backtest summary across rolling holding windows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BacktestSummary {
    pub market: MarketId,
    pub horizon_hours: Decimal,
    pub sample_count: usize,
    pub coverage_pct: Decimal,
    pub min_realized_borrow_usd: Decimal,
    pub median_realized_borrow_usd: Decimal,
    pub p95_realized_borrow_usd: Decimal,
    pub max_realized_borrow_usd: Decimal,
    pub entry_implied_borrow_usd: Decimal,
}

/// Snapshot logger coverage report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoverageReport {
    pub market: MarketId,
    pub total_slots: usize,
    pub start_ts: i64,
    pub end_ts: i64,
    pub coverage_pct: Decimal,
    pub gaps_over_five_minutes: Vec<(i64, i64)>,
}
