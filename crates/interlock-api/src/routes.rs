use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use interlock_core::{
    calculate_cost, BacktestSummary, CostBreakdown, CostError, MarketId, PositionInput, VenueParams,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Deserialize)]
pub struct BacktestQuery {
    #[serde(default = "default_market")]
    pub market: String,
    #[serde(default = "default_hours")]
    pub hours: Decimal,
}

fn default_market() -> String {
    "SOL-PERP".to_string()
}

fn default_hours() -> Decimal {
    dec!(72)
}

pub fn create_router() -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/cost", post(cost_handler))
        .route("/backtest", get(backtest_handler))
        .route("/scenario/{id}", get(scenario_handler))
}

async fn health_handler() -> impl IntoResponse {
    Json(handle_health())
}

async fn cost_handler(
    Json(input): Json<PositionInput>,
) -> Result<Json<ApiResponse<CostBreakdown>>, (StatusCode, String)> {
    handle_cost(input)
        .map(Json)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

async fn backtest_handler(Query(query): Query<BacktestQuery>) -> impl IntoResponse {
    Json(handle_backtest(query.market, query.hours))
}

async fn scenario_handler(
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<ScenarioResponse>>, (StatusCode, String)> {
    handle_scenario(&id)
        .map(Json)
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))
}

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Cost calculation error: {0}")]
    CostCalculation(#[from] CostError),
    #[error("Scenario not found: {0}")]
    ScenarioNotFound(String),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
    pub version: &'static str,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ApiResponse<T> {
    pub as_of_slot: u64,
    pub unverified_inputs: Vec<String>,
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ScenarioResponse {
    pub scenario_id: String,
    pub name: String,
    pub synthetic: bool,
    pub market: String,
    pub oracle_price: String,
    pub reference_price: String,
    pub max_deviation_bps: u16,
    pub expected_guard_result: String,
}

pub fn handle_health() -> HealthResponse {
    HealthResponse {
        status: "ok",
        service: "interlock-api",
        version: "0.1.0",
    }
}

pub fn handle_cost(input: PositionInput) -> Result<ApiResponse<CostBreakdown>, ApiError> {
    let venue = VenueParams {
        open_fee_bps: dec!(6),
        close_fee_bps: dec!(6),
        borrow_rate_per_hour: dec!(0.00001),
        max_leverage: dec!(20),
        maintenance_margin: dec!(0.02),
        impact: None,
    };
    let as_of_slot = 300_000_000;
    let mark_price = dec!(150.00);

    let breakdown = calculate_cost(&input, &venue, mark_price, as_of_slot)?;
    let unverified_inputs = breakdown.unverified_inputs.clone();

    Ok(ApiResponse {
        as_of_slot,
        unverified_inputs,
        data: breakdown,
    })
}

pub fn handle_backtest(market: String, horizon_hours: Decimal) -> ApiResponse<BacktestSummary> {
    let as_of_slot = 300_000_000;
    let summary = BacktestSummary {
        market: MarketId(market),
        horizon_hours,
        sample_count: 720,
        coverage_pct: dec!(96.5),
        min_realized_borrow_usd: dec!(0.85),
        median_realized_borrow_usd: dec!(1.18),
        p95_realized_borrow_usd: dec!(1.55),
        max_realized_borrow_usd: dec!(2.40),
        entry_implied_borrow_usd: dec!(1.20),
    };

    ApiResponse {
        as_of_slot,
        unverified_inputs: vec![
            "Backtest coverage calculated from mock snapshot window [UNVERIFIED: gate G3]"
                .to_string(),
        ],
        data: summary,
    }
}

pub fn handle_scenario(id: &str) -> Result<ApiResponse<ScenarioResponse>, ApiError> {
    let as_of_slot = 300_000_000;
    let scenario = match id {
        "a_normal" | "normal" => ScenarioResponse {
            scenario_id: "a_normal".to_string(),
            name: "Normal Market Conditions (Guarded Pass)".to_string(),
            synthetic: false,
            market: "SOL-PERP".to_string(),
            oracle_price: "150.05".to_string(),
            reference_price: "150.00".to_string(),
            max_deviation_bps: 50,
            expected_guard_result: "PASS".to_string(),
        },
        "b_spike_synthetic" | "spike" => ScenarioResponse {
            scenario_id: "b_spike_synthetic".to_string(),
            name: "Synthetic Oracle Price Spike (Guarded Revert)".to_string(),
            synthetic: true,
            market: "SOL-PERP".to_string(),
            oracle_price: "172.50".to_string(),
            reference_price: "150.00".to_string(),
            max_deviation_bps: 50,
            expected_guard_result: "REVERT".to_string(),
        },
        other => return Err(ApiError::ScenarioNotFound(other.to_string())),
    };

    Ok(ApiResponse {
        as_of_slot,
        unverified_inputs: vec![],
        data: scenario,
    })
}
