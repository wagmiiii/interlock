use anyhow::Result;
use clap::Args;
use rust_decimal::Decimal;

#[derive(Debug, Args)]
pub struct BacktestArgs {
    #[arg(long, default_value = "SOL-PERP")]
    pub market: String,

    #[arg(long, default_value = "72")]
    pub hours: Decimal,

    #[arg(long, default_value = "./data/snapshots.db")]
    pub db_path: String,
}

pub async fn run(args: BacktestArgs) -> Result<()> {
    println!("=== Interlock Backtester ===");
    println!("Market:          {}", args.market);
    println!("Horizon:         {} hours", args.hours);
    println!("Database:        {}", args.db_path);
    println!("Refusal rule:    requires >= 90% coverage");
    Ok(())
}
