use interlock_core::{
    calculate_cost, compute_backtest_summary, estimate_liquidation_price, AssetId, MarketId,
    PositionInput, Side, VenueParams,
};
use rust_decimal_macros::dec;

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

    let venue = VenueParams {
        open_fee_bps: dec!(6),
        close_fee_bps: dec!(6),
        borrow_rate_per_hour: dec!(0.00001), // 0.001%/hr
        max_leverage: dec!(20),
        maintenance_margin: dec!(0.02),
        impact: None,
    };

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
