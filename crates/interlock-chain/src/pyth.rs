use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PythError {
    #[error("Pyth price feed account truncated")]
    TruncatedFeed,
    #[error("Confidence interval {conf} exceeds maximum threshold")]
    ExcessiveConfidence { conf: Decimal },
    #[error("Price is stale (publish time {publish_time})")]
    StalePrice { publish_time: i64 },
    #[error("Failed to parse feed data: {0}")]
    ParseError(String),
}

/// Decoded Pyth price data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PythPrice {
    pub feed_id: String,
    pub price: Decimal,
    pub conf: Decimal,
    pub publish_time: i64,
    pub expo: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythFixtureRecord {
    pub slot: u64,
    pub feed_id: String,
    pub price: String,
    pub conf: String,
    pub expo: i32,
    pub publish_time: i64,
}

pub struct PythReader;

impl PythReader {
    pub fn parse_fixture(record: &PythFixtureRecord) -> Result<PythPrice, PythError> {
        let raw_price = record
            .price
            .parse::<i64>()
            .map_err(|e| PythError::ParseError(e.to_string()))?;
        let raw_conf = record
            .conf
            .parse::<u64>()
            .map_err(|e| PythError::ParseError(e.to_string()))?;

        let scale = 10_i64.pow((-record.expo) as u32);
        let price = Decimal::from(raw_price) / Decimal::from(scale);
        let conf = Decimal::from(raw_conf) / Decimal::from(scale);

        Ok(PythPrice {
            feed_id: record.feed_id.clone(),
            price,
            conf,
            publish_time: record.publish_time,
            expo: record.expo,
        })
    }
}
