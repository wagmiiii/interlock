use crate::types::{BacktestSummary, MarketId};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BacktestError {
    #[error("Coverage {coverage_pct}% is below the mandatory 90% threshold for backtest refusal")]
    InsufficientCoverage { coverage_pct: Decimal },
    #[error("No sample snapshots available for the given window")]
    EmptySamples,
}

/// Compute backtest summary across historical borrow cost samples.
///
/// Refuses to generate results if data coverage is under 90%.
pub fn compute_backtest_summary(
    market: MarketId,
    horizon_hours: Decimal,
    mut realized_costs: Vec<Decimal>,
    entry_implied_borrow_usd: Decimal,
    coverage_pct: Decimal,
) -> Result<BacktestSummary, BacktestError> {
    if coverage_pct < dec!(90.0) {
        return Err(BacktestError::InsufficientCoverage { coverage_pct });
    }

    if realized_costs.is_empty() {
        return Err(BacktestError::EmptySamples);
    }

    realized_costs.sort();

    let n = realized_costs.len();
    let min = realized_costs[0];
    let max = realized_costs[n - 1];

    let median = if n % 2 == 1 {
        realized_costs[n / 2]
    } else {
        (realized_costs[(n / 2) - 1] + realized_costs[n / 2]) / dec!(2)
    };

    let p95_idx = ((n * 95) / 100).min(n.saturating_sub(1));
    let p95 = realized_costs[p95_idx];

    Ok(BacktestSummary {
        market,
        horizon_hours,
        sample_count: n,
        coverage_pct,
        min_realized_borrow_usd: min,
        median_realized_borrow_usd: median,
        p95_realized_borrow_usd: p95,
        max_realized_borrow_usd: max,
        entry_implied_borrow_usd,
    })
}
