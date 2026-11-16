use interlock_core::{CostBreakdown, PositionInput};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub as_of_slot: u64,
    pub unverified_inputs: Vec<String>,
    pub data: T,
}

pub fn handle_health() -> HealthResponse {
    HealthResponse {
        status: "ok",
        service: "interlock-api",
    }
}

pub fn handle_cost_stub(input: PositionInput) -> ApiResponse<Option<CostBreakdown>> {
    let _ = input;
    ApiResponse {
        as_of_slot: 0,
        unverified_inputs: vec![],
        data: None,
    }
}
