use anyhow::Result;
use clap::Args;
use interlock_core::{
    calculate_cost, AssetId, CostBreakdown, MarketId, PositionInput, Side, VenueParams,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Args)]
pub struct CostArgs {
    /// Market symbol (e.g. SOL)
    #[arg(long, default_value = "SOL")]
    pub market: String,

    /// Position side: long or short
    #[arg(long, value_parser = parse_side)]
    pub side: Side,

    /// Collateral amount in USD
    #[arg(long)]
    pub collateral: Decimal,

    /// Target leverage factor
    #[arg(long)]
    pub leverage: Decimal,

    /// Holding horizon in hours
    #[arg(long)]
    pub hours: Decimal,

    /// Collateral asset
    #[arg(long, default_value = "USDC")]
    pub collateral_asset: String,
}

fn parse_side(s: &str) -> Result<Side, String> {
    match s.to_lowercase().as_str() {
        "long" => Ok(Side::Long),
        "short" => Ok(Side::Short),
        other => Err(format!(
            "Invalid side '{other}'. Expected 'long' or 'short'"
        )),
    }
}

pub async fn run(args: CostArgs) -> Result<()> {
    let input = PositionInput {
        market: MarketId(format!("{}-PERP", args.market)),
        side: args.side,
        collateral_usd: args.collateral,
        leverage: args.leverage,
        horizon_hours: args.hours,
        collateral_asset: AssetId(args.collateral_asset),
    };

    // Placeholder venue parameters until RPC decoders connect in P1-02
    let venue = VenueParams {
        open_fee_bps: dec!(6),
        close_fee_bps: dec!(6),
        borrow_rate_per_hour: dec!(0.00001),
        max_leverage: dec!(20),
        maintenance_margin: dec!(0.02),
        impact: None,
    };

    let breakdown = calculate_cost(&input, &venue, dec!(150), 300_000_000)?;
    print_breakdown(&breakdown);
    Ok(())
}

fn print_breakdown(b: &CostBreakdown) {
    println!("=== Interlock Cost Breakdown ===");
    println!("As of slot:       {}", b.as_of_slot);
    println!("Position size:    ${}", b.size_usd);
    println!("Open fee:         ${}", b.open_fee_usd);
    println!("Close fee:        ${}", b.close_fee_usd);
    println!("Borrow cost:      ${}", b.borrow_usd);
    println!(
        "Total cost:       ${} ({}% of collateral)",
        b.total_usd, b.total_pct_of_collateral
    );
    println!("Breakeven move:   {}%", b.breakeven_move_pct);
    println!(
        "Estimated liq:    ${} (distance {}%)",
        b.liq_price, b.liq_distance_pct
    );
    println!("\nHorizon Sweep:");
    for p in &b.sweep {
        println!(
            "  {:>3}h: ${:>6.2} (borrow ${:>6.2})",
            p.hours, p.total_usd, p.borrow_usd
        );
    }
    println!("\nAssumptions:");
    for a in &b.assumptions {
        println!("  - {a}");
    }
    println!("\nUnverified inputs:");
    for u in &b.unverified_inputs {
        println!("  - {u}");
    }
}
