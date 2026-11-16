use rust_decimal::Decimal;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PythError {
    #[error("Pyth price feed account truncated")]
    TruncatedFeed,
    #[error("Confidence interval {conf} exceeds maximum threshold")]
    ExcessiveConfidence { conf: Decimal },
    #[error("Price is stale (publish time {publish_time})")]
    StalePrice { publish_time: i64 },
}

/// Decoded Pyth price data.
#[derive(Debug, Clone, PartialEq)]
pub struct PythPrice {
    pub feed_id: [u8; 32],
    pub price: Decimal,
    pub conf: Decimal,
    pub publish_time: i64,
    pub expo: i32,
}

pub struct PythReader;

impl PythReader {
    pub fn parse_price_update(data: &[u8]) -> Result<PythPrice, PythError> {
        if data.len() < 32 {
            return Err(PythError::TruncatedFeed);
        }
        // Pyth receiver SDK decoder stub
        Ok(PythPrice {
            feed_id: [0u8; 32],
            price: Decimal::ZERO,
            conf: Decimal::ZERO,
            publish_time: 0,
            expo: 0,
        })
    }
}
