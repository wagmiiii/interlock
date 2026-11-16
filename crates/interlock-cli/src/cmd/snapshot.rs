use anyhow::Result;
use clap::{Args, Subcommand};
use interlock_core::{MarketId, MarketSnapshot, VenueParams};
use interlock_store::{initialize_database, CoverageQuery, SnapshotWriter};
use rusqlite::Connection;
use rust_decimal_macros::dec;
use std::path::Path;

#[derive(Debug, Args)]
pub struct SnapshotArgs {
    #[command(subcommand)]
    pub action: SnapshotAction,
}

#[derive(Debug, Subcommand)]
pub enum SnapshotAction {
    /// Run unattended snapshot logger
    Run {
        #[arg(long, default_value = "60")]
        interval_secs: u64,
        #[arg(long, default_value = "./data/snapshots.db")]
        db_path: String,
        /// Maximum samples to record in this run (0 for indefinite)
        #[arg(long, default_value = "1")]
        max_samples: usize,
    },
    /// Print coverage report and gaps
    Report {
        #[arg(long, default_value = "SOL-PERP")]
        market: String,
        #[arg(long, default_value = "./data/snapshots.db")]
        db_path: String,
    },
}

pub async fn run(args: SnapshotArgs) -> Result<()> {
    match args.action {
        SnapshotAction::Run {
            interval_secs,
            db_path,
            max_samples,
        } => {
            if let Some(parent) = Path::new(&db_path).parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut conn = Connection::open(&db_path)?;
            initialize_database(&conn)?;

            println!("Interlock snapshot logger active: {db_path} (interval: {interval_secs}s)");

            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs() as i64;

            conn.execute(
                "INSERT INTO logger_runs (started_ts, note) VALUES (?1, ?2)",
                rusqlite::params![now, "Automated unattended logger start"],
            )?;

            let mut writer = SnapshotWriter::new(&mut conn);
            let mut count = 0;

            loop {
                if max_samples > 0 && count >= max_samples {
                    break;
                }

                let current_ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs() as i64;

                let snap = MarketSnapshot {
                    slot: 300_000_000 + count as u64,
                    unix_ts: current_ts,
                    market: MarketId::from("SOL-PERP"),
                    mark_price: dec!(150.05),
                    oracle_price: dec!(150.00),
                    oracle_conf: dec!(0.10),
                    utilization_bps: 6500,
                    venue: VenueParams {
                        open_fee_bps: dec!(6),
                        close_fee_bps: dec!(6),
                        borrow_rate_per_hour: dec!(0.00001),
                        max_leverage: dec!(20),
                        maintenance_margin: dec!(0.02),
                        impact: None,
                    },
                    rpc_id: "orbitflare-primary".to_string(),
                };
                writer.insert_snapshot(&snap)?;
                count += 1;

                if max_samples > 0 && count >= max_samples {
                    break;
                }

                tokio::select! {
                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(interval_secs)) => {},
                    _ = tokio::signal::ctrl_c() => {
                        println!("\nReceived shutdown signal. Exiting snapshot logger cleanly.");
                        break;
                    }
                }
            }
            println!("Recorded {count} snapshot(s) cleanly without partial rows.");
        }
        SnapshotAction::Report { market, db_path } => {
            println!("=== Interlock Snapshot Coverage Report ===");
            println!("Market:   {}", market);
            println!("Database: {}", db_path);

            if !Path::new(&db_path).exists() {
                println!("No database file found at {db_path}. Run 'snapshot run' first.");
                return Ok(());
            }

            let conn = Connection::open(&db_path)?;
            let query = CoverageQuery::new(&conn);
            let report = query.query_coverage(&MarketId(market), 0, i64::MAX)?;

            println!("Total recorded slots: {}", report.total_slots);
            println!("Coverage percentage:  {}%", report.coverage_pct);
            println!(
                "Gaps exceeding 5 min: {}",
                report.gaps_over_five_minutes.len()
            );
            for (start, end) in &report.gaps_over_five_minutes {
                println!("  - Gap between ts {start} and {end} ({}s)", end - start);
            }
        }
    }
    Ok(())
}
