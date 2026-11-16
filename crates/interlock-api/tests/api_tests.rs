use interlock_api::routes::{
    handle_backtest, handle_cost, handle_health, handle_scenario, ApiError,
};
use interlock_core::{AssetId, MarketId, PositionInput, Side};
use rust_decimal_macros::dec;

#[test]
fn test_health_endpoint() {
    let health = handle_health();
    assert_eq!(health.status, "ok");
    assert_eq!(health.service, "interlock-api");
}

#[test]
fn test_cost_endpoint_includes_slot_and_unverified() {
    let input = PositionInput {
        market: MarketId("SOL-PERP".to_string()),
        side: Side::Long,
        collateral_usd: dec!(1000),
        leverage: dec!(5),
        horizon_hours: dec!(24),
        collateral_asset: AssetId("USDC".to_string()),
    };

    let resp = handle_cost(input).expect("Cost calculation should succeed");
    assert_eq!(resp.as_of_slot, 300_000_000);
    assert!(!resp.unverified_inputs.is_empty());
    assert_eq!(resp.data.size_usd, dec!(5000));
    assert_eq!(resp.data.total_usd, dec!(7.20));
}

#[test]
fn test_backtest_endpoint_includes_slot_and_coverage() {
    let resp = handle_backtest("SOL-PERP".to_string(), dec!(24));
    assert_eq!(resp.as_of_slot, 300_000_000);
    assert_eq!(resp.data.coverage_pct, dec!(96.5));
    assert!(resp.data.coverage_pct >= dec!(90.0));
    assert!(!resp.unverified_inputs.is_empty());
}

#[test]
fn test_scenario_endpoint() {
    let sc_a = handle_scenario("a_normal").expect("Scenario A should exist");
    assert_eq!(sc_a.as_of_slot, 300_000_000);
    assert!(!sc_a.data.synthetic);
    assert_eq!(sc_a.data.expected_guard_result, "PASS");

    let sc_b = handle_scenario("b_spike_synthetic").expect("Scenario B should exist");
    assert_eq!(sc_b.as_of_slot, 300_000_000);
    assert!(sc_b.data.synthetic);
    assert_eq!(sc_b.data.expected_guard_result, "REVERT");

    let sc_unknown = handle_scenario("unknown_xyz");
    assert!(matches!(sc_unknown, Err(ApiError::ScenarioNotFound(_))));
}
