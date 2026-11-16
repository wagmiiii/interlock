use interlock_core::{
    calculate_cost, compute_backtest_summary, estimate_liquidation_price, AssetId, CostError,
    ImpactModel, MarketId, PositionInput, Side, VenueParams,
};
use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

fn default_venue() -> VenueParams {
    VenueParams {
        open_fee_bps: dec!(6),
        close_fee_bps: dec!(6),
        borrow_rate_per_hour: dec!(0.00001), // 0.001%/hr
        max_leverage: dec!(20),
        maintenance_margin: dec!(0.02),
        impact: None,
    }
}

#[test]
fn test_cost_calculation_basic() {
    let input = PositionInput {
        market: MarketId::from("SOL-PERP"),
        side: Side::Long,
        collateral_usd: dec!(100),
        leverage: dec!(5),
        horizon_hours: dec!(72),
        collateral_asset: AssetId::from("USDC"),
    };

    let venue = default_venue();
    let breakdown = calculate_cost(&input, &venue, dec!(150), 300_000_000).unwrap();

    // Size = 100 * 5 = 500 USD
    assert_eq!(breakdown.size_usd, dec!(500));
    // Open fee = 500 * (6 / 10000) = 0.30 USD
    assert_eq!(breakdown.open_fee_usd, dec!(0.30));
    // Close fee = 500 * (6 / 10000) = 0.30 USD
    assert_eq!(breakdown.close_fee_usd, dec!(0.30));
    // Borrow fee = 500 * 0.00001 * 72 = 0.36 USD
    assert_eq!(breakdown.borrow_usd, dec!(0.36));
    // Total = 0.30 + 0.30 + 0.36 = 0.96 USD
    assert_eq!(breakdown.total_usd, dec!(0.96));
    // Pct of collateral = 0.96 / 100 * 100 = 0.96%
    assert_eq!(breakdown.total_pct_of_collateral, dec!(0.96));
}

#[test]
fn test_carry_arithmetic_from_prd() {
    // PRD Section 2: 0.001% per hour on a $1,000 position is $1.68 per week ($1,000 x 0.00001 x 168 hours)
    let input = PositionInput {
        market: MarketId::from("SOL-PERP"),
        side: Side::Long,
        collateral_usd: dec!(1000),
        leverage: dec!(1),
        horizon_hours: dec!(168),
        collateral_asset: AssetId::from("USDC"),
    };

    let venue = VenueParams {
        open_fee_bps: dec!(0),
        close_fee_bps: dec!(0),
        borrow_rate_per_hour: dec!(0.00001),
        max_leverage: dec!(10),
        maintenance_margin: dec!(0.01),
        impact: None,
    };

    let breakdown = calculate_cost(&input, &venue, dec!(200), 1).unwrap();
    assert_eq!(breakdown.borrow_usd, dec!(1.68));
}

#[test]
fn test_zero_hours_holding() {
    let input = PositionInput {
        market: MarketId::from("SOL-PERP"),
        side: Side::Long,
        collateral_usd: dec!(500),
        leverage: dec!(4),
        horizon_hours: dec!(0),
        collateral_asset: AssetId::from("USDC"),
    };
    let venue = default_venue();
    let breakdown = calculate_cost(&input, &venue, dec!(100), 10).unwrap();

    // Size = 2000 USD
    assert_eq!(breakdown.size_usd, dec!(2000));
    // Borrow fee for 0 hours must be exactly zero
    assert_eq!(breakdown.borrow_usd, dec!(0));
    // Total cost is just open fee + close fee (1.20 + 1.20 = 2.40)
    assert_eq!(breakdown.total_usd, dec!(2.40));
}

#[test]
fn test_fractional_hours_holding() {
    let input = PositionInput {
        market: MarketId::from("SOL-PERP"),
        side: Side::Short,
        collateral_usd: dec!(1000),
        leverage: dec!(10),
        horizon_hours: dec!(0.5), // 30 minutes
        collateral_asset: AssetId::from("USDC"),
    };
    let venue = default_venue();
    let breakdown = calculate_cost(&input, &venue, dec!(100), 10).unwrap();

    // Size = 10,000 USD
    // Borrow = 10,000 * 0.00001 * 0.5 = 0.05 USD
    assert_eq!(breakdown.borrow_usd, dec!(0.05));
}

#[test]
fn test_leverage_bounds() {
    let venue = default_venue(); // max_leverage = 20

    // Leverage < 1.0 must fail
    let input_sub_one = PositionInput {
        market: MarketId::from("SOL-PERP"),
        side: Side::Long,
        collateral_usd: dec!(100),
        leverage: dec!(0.9),
        horizon_hours: dec!(10),
        collateral_asset: AssetId::from("USDC"),
    };
    assert!(matches!(
        calculate_cost(&input_sub_one, &venue, dec!(100), 1),
        Err(CostError::InvalidLeverage(_))
    ));

    // Leverage == 1.0 must pass
    let input_one = PositionInput {
        market: MarketId::from("SOL-PERP"),
        side: Side::Long,
        collateral_usd: dec!(100),
        leverage: dec!(1.0),
        horizon_hours: dec!(10),
        collateral_asset: AssetId::from("USDC"),
    };
    assert!(calculate_cost(&input_one, &venue, dec!(100), 1).is_ok());

    // Leverage == max_leverage must pass
    let input_max = PositionInput {
        market: MarketId::from("SOL-PERP"),
        side: Side::Long,
        collateral_usd: dec!(100),
        leverage: dec!(20),
        horizon_hours: dec!(10),
        collateral_asset: AssetId::from("USDC"),
    };
    assert!(calculate_cost(&input_max, &venue, dec!(100), 1).is_ok());

    // Leverage > max_leverage must fail
    let input_above_max = PositionInput {
        market: MarketId::from("SOL-PERP"),
        side: Side::Long,
        collateral_usd: dec!(100),
        leverage: dec!(20.01),
        horizon_hours: dec!(10),
        collateral_asset: AssetId::from("USDC"),
    };
    assert!(matches!(
        calculate_cost(&input_above_max, &venue, dec!(100), 1),
        Err(CostError::LeverageExceedsMax { .. })
    ));
}

#[test]
fn test_collateral_and_horizon_negative_checks() {
    let venue = default_venue();

    // Zero collateral fails
    let input_zero_col = PositionInput {
        market: MarketId::from("SOL-PERP"),
        side: Side::Long,
        collateral_usd: dec!(0),
        leverage: dec!(2),
        horizon_hours: dec!(10),
        collateral_asset: AssetId::from("USDC"),
    };
    assert!(matches!(
        calculate_cost(&input_zero_col, &venue, dec!(100), 1),
        Err(CostError::ZeroCollateral(_))
    ));

    // Negative hours fails
    let input_neg_hours = PositionInput {
        market: MarketId::from("SOL-PERP"),
        side: Side::Long,
        collateral_usd: dec!(100),
        leverage: dec!(2),
        horizon_hours: dec!(-1),
        collateral_asset: AssetId::from("USDC"),
    };
    assert!(matches!(
        calculate_cost(&input_neg_hours, &venue, dec!(100), 1),
        Err(CostError::NegativeHorizon(_))
    ));
}

#[test]
fn test_impact_model_incorporation() {
    let mut venue = default_venue();
    venue.impact = Some(ImpactModel {
        base_impact_bps: dec!(5), // 0.05%
        slope_bps_per_million: dec!(1),
    });

    let input = PositionInput {
        market: MarketId::from("SOL-PERP"),
        side: Side::Long,
        collateral_usd: dec!(1000),
        leverage: dec!(2),
        horizon_hours: dec!(0),
        collateral_asset: AssetId::from("USDC"),
    };

    let breakdown = calculate_cost(&input, &venue, dec!(100), 1).unwrap();
    // Size = 2000
    // Open = 2000 * 0.0006 = 1.20
    // Close = 2000 * 0.0006 = 1.20
    // Impact = 2000 * (5.002 / 10000) = 1.0004
    // Borrow = 0
    // Total = 3.4004
    assert_eq!(breakdown.impact_usd, Some(dec!(1.0004)));
    assert_eq!(breakdown.total_usd, dec!(3.4004));

    // Institutional size ($1M):
    // base = 5 bps, slope = 1 bps/M -> total 6 bps
    // 1,000,000 * 0.0006 = 600.00 USD
    let input_large = PositionInput {
        market: MarketId::from("SOL-PERP"),
        side: Side::Long,
        collateral_usd: dec!(250_000),
        leverage: dec!(4),
        horizon_hours: dec!(0),
        collateral_asset: AssetId::from("USDC"),
    };
    let breakdown_large = calculate_cost(&input_large, &venue, dec!(100), 1).unwrap();
    assert_eq!(breakdown_large.impact_usd, Some(dec!(600.00)));
}

#[test]
fn test_liquidation_price_estimation() {
    let (liq_long, dist_long) =
        estimate_liquidation_price(Side::Long, dec!(100), dec!(5), dec!(0.02));
    // Long: 100 * (1 - 0.20 + 0.02) = 100 * 0.82 = 82
    assert_eq!(liq_long, dec!(82.00));
    assert_eq!(dist_long, dec!(18.00));

    let (liq_short, dist_short) =
        estimate_liquidation_price(Side::Short, dec!(100), dec!(5), dec!(0.02));
    // Short: 100 * (1 + 0.20 - 0.02) = 100 * 1.18 = 118
    assert_eq!(liq_short, dec!(118.00));
    assert_eq!(dist_short, dec!(18.00));
}

#[test]
fn test_backtest_coverage_refusal() {
    let res = compute_backtest_summary(
        MarketId::from("SOL-PERP"),
        dec!(24),
        vec![dec!(1.0), dec!(2.0)],
        dec!(1.5),
        dec!(89.9), // below 90%
    );
    assert!(res.is_err());
}

// Property tests using proptest
proptest! {
    #[test]
    fn prop_cost_scales_linearly_with_horizon(
        collateral in 1u32..10_000,
        leverage in 1u32..20,
        hours1 in 0u32..200,
        hours2 in 0u32..200,
    ) {
        let venue = default_venue();
        let input1 = PositionInput {
            market: MarketId::from("SOL-PERP"),
            side: Side::Long,
            collateral_usd: Decimal::from(collateral),
            leverage: Decimal::from(leverage),
            horizon_hours: Decimal::from(hours1),
            collateral_asset: AssetId::from("USDC"),
        };
        let input2 = PositionInput {
            market: MarketId::from("SOL-PERP"),
            side: Side::Long,
            collateral_usd: Decimal::from(collateral),
            leverage: Decimal::from(leverage),
            horizon_hours: Decimal::from(hours2),
            collateral_asset: AssetId::from("USDC"),
        };

        let b1 = calculate_cost(&input1, &venue, dec!(100), 1).unwrap();
        let b2 = calculate_cost(&input2, &venue, dec!(100), 1).unwrap();

        if hours1 <= hours2 {
            prop_assert!(b1.borrow_usd <= b2.borrow_usd);
            prop_assert!(b1.total_usd <= b2.total_usd);
        } else {
            prop_assert!(b1.borrow_usd >= b2.borrow_usd);
            prop_assert!(b1.total_usd >= b2.total_usd);
        }
    }
}
