use interlock_core::{calculate_cost, AssetId, MarketId, PositionInput, Side, VenueParams};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

struct GoldenCase {
    name: &'static str,
    market: &'static str,
    side: Side,
    collateral: Decimal,
    leverage: Decimal,
    hours: Decimal,
    expected_total_usd: Decimal,
}

#[test]
fn test_golden_venue_quotes_five_positions() {
    let venue = VenueParams {
        open_fee_bps: dec!(6),
        close_fee_bps: dec!(6),
        borrow_rate_per_hour: dec!(0.00001), // 0.001%/hr
        max_leverage: dec!(20),
        maintenance_margin: dec!(0.02),
        impact: None,
    };

    let cases = vec![
        GoldenCase {
            name: "Case 1: SOL 5x Long, $100 col, 72h carry",
            market: "SOL-PERP",
            side: Side::Long,
            collateral: dec!(100),
            leverage: dec!(5),
            hours: dec!(72),
            // Size: 500. Open: 0.30, Close: 0.30, Borrow: 500 * 0.00001 * 72 = 0.36. Total = 0.96
            expected_total_usd: dec!(0.96),
        },
        GoldenCase {
            name: "Case 2: SOL 10x Short, $500 col, 24h carry",
            market: "SOL-PERP",
            side: Side::Short,
            collateral: dec!(500),
            leverage: dec!(10),
            hours: dec!(24),
            // Size: 5000. Open: 3.00, Close: 3.00, Borrow: 5000 * 0.00001 * 24 = 1.20. Total = 7.20
            expected_total_usd: dec!(7.20),
        },
        GoldenCase {
            name: "Case 3: BTC 2x Long, $1,000 col, 168h (1 week) carry",
            market: "BTC-PERP",
            side: Side::Long,
            collateral: dec!(1000),
            leverage: dec!(2),
            hours: dec!(168),
            // Size: 2000. Open: 1.20, Close: 1.20, Borrow: 2000 * 0.00001 * 168 = 3.36. Total = 5.76
            expected_total_usd: dec!(5.76),
        },
        GoldenCase {
            name: "Case 4: ETH 3x Long, $250 col, 1h short carry",
            market: "ETH-PERP",
            side: Side::Long,
            collateral: dec!(250),
            leverage: dec!(3),
            hours: dec!(1),
            // Size: 750. Open: 0.45, Close: 0.45, Borrow: 750 * 0.00001 * 1 = 0.0075. Total = 0.9075
            expected_total_usd: dec!(0.9075),
        },
        GoldenCase {
            name: "Case 5: SOL 1x Long unleveraged spot-perp, $1,000 col, 0h instant",
            market: "SOL-PERP",
            side: Side::Long,
            collateral: dec!(1000),
            leverage: dec!(1),
            hours: dec!(0),
            // Size: 1000. Open: 0.60, Close: 0.60, Borrow: 0.00. Total = 1.20
            expected_total_usd: dec!(1.20),
        },
    ];

    for case in cases {
        let input = PositionInput {
            market: MarketId::from(case.market),
            side: case.side,
            collateral_usd: case.collateral,
            leverage: case.leverage,
            horizon_hours: case.hours,
            collateral_asset: AssetId::from("USDC"),
        };

        let breakdown = calculate_cost(&input, &venue, dec!(150), 300_000_000).unwrap();
        let diff = (breakdown.total_usd - case.expected_total_usd).abs();

        // Must be within 1% relative or $0.05 absolute (PRD Goal G1 and F1)
        let tolerance = (case.expected_total_usd * dec!(0.01)).max(dec!(0.05));
        assert!(
            diff <= tolerance,
            "{}: computed total {} deviated from expected {} by {} (tolerance {})",
            case.name,
            breakdown.total_usd,
            case.expected_total_usd,
            diff,
            tolerance
        );
    }
}
