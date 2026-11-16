use anyhow::Result;
use clap::{Parser, Subcommand};

mod cmd;

#[derive(Debug, Parser)]
#[command(name = "interlock")]
#[command(about = "Non-custodial Solana perps cost engine and atomic oracle guard")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Calculate position cost breakdown and carry projections
    Cost(cmd::cost::CostArgs),
    /// Market state snapshot recorder and coverage reporter
    Snapshot(cmd::snapshot::SnapshotArgs),
    /// Historical carry backtest across logged windows
    Backtest(cmd::backtest::BacktestArgs),
    /// Simulate atomic guard execution against oracle conditions
    Guard(cmd::guard::GuardSimulateArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Cost(args) => cmd::cost::run(args).await,
        Commands::Snapshot(args) => cmd::snapshot::run(args).await,
        Commands::Backtest(args) => cmd::backtest::run(args).await,
        Commands::Guard(args) => cmd::guard::run(args).await,
    }
}
