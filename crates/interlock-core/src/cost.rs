use crate::liquidation::estimate_liquidation_price;
use crate::types::{CostBreakdown, HorizonPoint, PositionInput, RateSensitivity, VenueParams};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CostError {
    #[error("Collateral must be greater than zero, got {0}")]
    ZeroCollateral(Decimal),
    #[error("Leverage must be at least 1.0, got {0}")]
    InvalidLeverage(Decimal),
    #[error("Requested leverage {requested} exceeds venue maximum {max}")]
    LeverageExceedsMax { requested: Decimal, max: Decimal },
    #[error("Horizon hours must be non-negative, got {0}")]
    NegativeHorizon(Decimal),
}

/// Calculate itemised cost breakdown from position inputs and venue parameters.
pub fn calculate_cost(
    input: &PositionInput,
    venue: &VenueParams,
    mark_price: Decimal,
    as_of_slot: u64,
) -> Result<CostBreakdown, CostError> {
    if input.collateral_usd <= dec!(0) {
        return Err(CostError::ZeroCollateral(input.collateral_usd));
    }
    if input.leverage < dec!(1.0) {
        return Err(CostError::InvalidLeverage(input.leverage));
    }
    if input.leverage > venue.max_leverage {
        return Err(CostError::LeverageExceedsMax {
            requested: input.leverage,
            max: venue.max_leverage,
        });
    }
    if input.horizon_hours < dec!(0) {
        return Err(CostError::NegativeHorizon(input.horizon_hours));
    }

    let size_usd = input.collateral_usd * input.leverage;
    let bps_divisor = dec!(10000);

    let open_fee_usd = size_usd * (venue.open_fee_bps / bps_divisor);
    let close_fee_usd = size_usd * (venue.close_fee_bps / bps_divisor);

    let impact_usd = venue.impact.as_ref().map(|impact| {
        let size_in_millions = size_usd / dec!(1_000_000);
        let impact_bps = impact.base_impact_bps + (size_in_millions * impact.slope_bps_per_million);
        size_usd * (impact_bps / bps_divisor)
    });

    let borrow_usd = size_usd * venue.borrow_rate_per_hour * input.horizon_hours;
    let base_total_usd = open_fee_usd + close_fee_usd + impact_usd.unwrap_or(dec!(0)) + borrow_usd;

    let total_pct_of_collateral = (base_total_usd / input.collateral_usd) * dec!(100);
    let breakeven_move_pct = (base_total_usd / size_usd) * dec!(100);

    let (liq_price, liq_distance_pct) = estimate_liquidation_price(
        input.side,
        mark_price,
        input.leverage,
        venue.maintenance_margin,
    );

    let sweep_horizons = [dec!(1), dec!(24), dec!(72), dec!(168)];
    let sweep = sweep_horizons
        .iter()
        .map(|&h| {
            let h_borrow = size_usd * venue.borrow_rate_per_hour * h;
            let h_total = open_fee_usd + close_fee_usd + impact_usd.unwrap_or(dec!(0)) + h_borrow;
            let h_pct = (h_total / input.collateral_usd) * dec!(100);
            HorizonPoint {
                hours: h,
                borrow_usd: h_borrow,
                total_usd: h_total,
                total_pct_of_collateral: h_pct,
            }
        })
        .collect();

    let sensitivities = [dec!(0.5), dec!(1.0), dec!(2.0)];
    let sensitivity = sensitivities
        .iter()
        .map(|&m| {
            let adj_rate = venue.borrow_rate_per_hour * m;
            let s_borrow = size_usd * adj_rate * input.horizon_hours;
            let s_total = open_fee_usd + close_fee_usd + impact_usd.unwrap_or(dec!(0)) + s_borrow;
            RateSensitivity {
                multiplier: m,
                borrow_rate_per_hour: adj_rate,
                borrow_usd: s_borrow,
                total_usd: s_total,
            }
        })
        .collect();

    let assumptions = vec![
        "Flat borrow rate assumption over the holding period".to_string(),
        "Linear fee basis points applied at entry and exit".to_string(),
        "Zero price impact assumed unless order book depth model is provided".to_string(),
    ];

    let unverified_inputs = vec![
        "Venue borrow rate accrual periodicity [UNVERIFIED: gate G3]".to_string(),
        "Maintenance margin ratio exact venue formula [UNVERIFIED: gate G3]".to_string(),
    ];

    Ok(CostBreakdown {
        as_of_slot,
        size_usd,
        open_fee_usd,
        close_fee_usd,
        impact_usd,
        borrow_usd,
        total_usd: base_total_usd,
        total_pct_of_collateral,
        breakeven_move_pct,
        liq_price,
        liq_distance_pct,
        sweep,
        sensitivity,
        assumptions,
        unverified_inputs,
    })
}
