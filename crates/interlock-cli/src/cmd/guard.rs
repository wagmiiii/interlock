use anyhow::{Context, Result};
use clap::Args;
use interlock_chain::TransactionComposer;
use interlock_ix::{
    execute_sanity_checks, OracleCheckInputs, OracleGuardParams, ReferenceKind, VerificationLevel,
};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Args)]
pub struct GuardSimulateArgs {
    /// Scenario name (e.g. "normal", "spike") or path to scenario JSON
    #[arg(long, default_value = "normal")]
    pub scenario: String,
}

#[derive(Debug, Deserialize)]
struct ScenarioData {
    scenario_id: String,
    name: String,
    synthetic: bool,
    market: String,
    oracle_price: String,
    reference_price: String,
    #[serde(default)]
    _deviation_bps: Option<u16>,
    max_deviation_bps: u16,
    #[serde(default)]
    _expected_guard_result: Option<String>,
    #[serde(default)]
    _expected_error: Option<String>,
}

pub async fn run(args: GuardSimulateArgs) -> Result<()> {
    let scenario_path = match args.scenario.as_str() {
        "normal" | "a" => "scenarios/a_normal.json".to_string(),
        "spike" | "b" => "scenarios/b_spike_synthetic.json".to_string(),
        custom => {
            if Path::new(custom).exists() {
                custom.to_string()
            } else {
                format!("scenarios/{custom}.json")
            }
        }
    };

    let content = fs::read_to_string(&scenario_path)
        .with_context(|| format!("Failed to read scenario file at '{scenario_path}'"))?;
    let data: ScenarioData = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse scenario json at '{scenario_path}'"))?;

    println!("========================================================");
    println!(" Scenario: {} ({})", data.name, data.scenario_id);
    if data.synthetic {
        println!(" [SYNTHETIC: artificial test vector for simulation]");
    }
    println!(" Market:          {}", data.market);
    println!(" Oracle Price:    ${}", data.oracle_price);
    println!(" Reference Price: ${}", data.reference_price);
    println!(" Max Dev Allowed: {} bps", data.max_deviation_bps);
    println!("========================================================");

    // Parse prices into 8-decimal fixed-point integers
    let p_oracle_f: f64 = data.oracle_price.parse()?;
    let p_ref_f: f64 = data.reference_price.parse()?;
    let p_oracle = (p_oracle_f * 100_000_000.0).round() as u64;
    let p_ref = (p_ref_f * 100_000_000.0).round() as u64;

    let feed_id = [1u8; 32];
    let params = OracleGuardParams {
        feed_id,
        max_age_secs: 30,
        max_conf_bps: 100,
        max_deviation_bps: data.max_deviation_bps,
        reference_kind: ReferenceKind::PythSecondFeed,
        reference_invert: false,
    };

    println!("\nExecuting Solana Transaction Simulation...");
    println!("Transaction Instructions Order:");
    println!("  [0] ComputeBudget::set_compute_unit_limit(200_000)");
    println!("  [1] InterlockIx::assert_oracle_sane");
    println!("  [2] Venue::open_position (Market: {})", data.market);

    let inputs = OracleCheckInputs {
        now: 1_000_000,
        publish_time: 999_995,
        price: p_oracle,
        conf: 10_000,
        feed_id,
        verification_level: VerificationLevel::Full,
        account_owner_ok: true,
        ref_price: p_ref,
    };

    let check_res = execute_sanity_checks(&params, &inputs);

    match check_res {
        Ok(measured_bps) => {
            println!("\n[Instruction 1] Guard Execution:");
            println!("  Status: SUCCESS");
            println!(
                "  Measured Deviation: {measured_bps} bps (<= {max_bps} bps)",
                max_bps = data.max_deviation_bps
            );
            println!("\n[Instruction 2] Venue Trade Execution:");
            println!("  Status: EXECUTED");
            println!("  Outcome: Order successfully filled under sane oracle parameters.");
            println!("\nFinal Transaction Status: [PASS] Confirmed in simulated slot.");
        }
        Err(err) => {
            let diagnostic = TransactionComposer::map_error_to_diagnostic(err);
            println!("\n[Instruction 1] Guard Execution:");
            println!("  Status: FAILED / REVERTED");
            println!("  Error Code: {:?}", err);
            println!("  Diagnostic: {diagnostic}");
            println!("\n[Instruction 2] Venue Trade Execution:");
            println!("  Status: NOT EXECUTED (Atomic Reversion)");
            println!("  Outcome: User order blocked before execution. No funds lost to bad oracle price.");
            println!("\nFinal Transaction Status: [REVERT] Transaction failed atomically at instruction 1.");
            println!("\nWithout Interlock:");
            println!("  Trade would have filled at bad oracle price (${}), resulting in instant capital loss.", data.oracle_price);
            println!("With Interlock:");
            println!("  Atomic rollback protected 100% of trade collateral; only minimal transaction fee spent.");
        }
    }

    Ok(())
}
