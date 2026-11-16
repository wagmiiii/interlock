use interlock_core::{MarketId, VenueParams};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const VENUE_POOL_DISCRIMINATOR: [u8; 8] = [168, 204, 112, 45, 99, 12, 88, 14];

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DecoderError {
    #[error("Unknown account layout version for {account_type}. Expected discriminator {expected:?}, got {actual:?}")]
    InvalidDiscriminator {
        account_type: &'static str,
        expected: Vec<u8>,
        actual: Vec<u8>,
    },
    #[error("Missing or truncated account data (length {0}, minimum required {1})")]
    TruncatedData(usize, usize),
    #[error("Failed to parse numeric representation: {0}")]
    ParseError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VenueAccountRecord {
    pub slot: u64,
    pub market: String,
    pub open_fee_bps: u32,
    pub close_fee_bps: u32,
    pub borrow_rate_per_hour: String,
    pub max_leverage: u32,
    pub maintenance_margin_bps: u32,
    pub discriminator: [u8; 8],
}

pub struct VenueDecoder;

impl VenueDecoder {
    /// Decode venue parameters from byte slice with discriminator check.
    pub fn decode_bytes(market: &MarketId, raw_data: &[u8]) -> Result<VenueParams, DecoderError> {
        if raw_data.len() < 8 {
            return Err(DecoderError::TruncatedData(raw_data.len(), 8));
        }

        let disc = &raw_data[0..8];
        if disc != VENUE_POOL_DISCRIMINATOR {
            return Err(DecoderError::InvalidDiscriminator {
                account_type: "VenuePoolState",
                expected: VENUE_POOL_DISCRIMINATOR.to_vec(),
                actual: disc.to_vec(),
            });
        }

        if raw_data.len() < 32 {
            return Err(DecoderError::TruncatedData(raw_data.len(), 32));
        }

        let open_bps = u32::from_le_bytes(raw_data[8..12].try_into().unwrap());
        let close_bps = u32::from_le_bytes(raw_data[12..16].try_into().unwrap());
        let max_lev = u32::from_le_bytes(raw_data[16..20].try_into().unwrap());
        let mm_bps = u32::from_le_bytes(raw_data[20..24].try_into().unwrap());
        let borrow_scaled = u64::from_le_bytes(raw_data[24..32].try_into().unwrap());

        let _ = market;
        Ok(VenueParams {
            open_fee_bps: Decimal::from(open_bps),
            close_fee_bps: Decimal::from(close_bps),
            borrow_rate_per_hour: Decimal::from(borrow_scaled) / Decimal::from(100_000_000),
            max_leverage: Decimal::from(max_lev),
            maintenance_margin: Decimal::from(mm_bps) / Decimal::from(10_000),
            impact: None,
        })
    }

    /// Decode from structured fixture representation.
    pub fn decode_record(record: &VenueAccountRecord) -> Result<VenueParams, DecoderError> {
        if record.discriminator != VENUE_POOL_DISCRIMINATOR {
            return Err(DecoderError::InvalidDiscriminator {
                account_type: "VenuePoolRecord",
                expected: VENUE_POOL_DISCRIMINATOR.to_vec(),
                actual: record.discriminator.to_vec(),
            });
        }

        let borrow_rate = record
            .borrow_rate_per_hour
            .parse::<Decimal>()
            .map_err(|e| DecoderError::ParseError(e.to_string()))?;

        Ok(VenueParams {
            open_fee_bps: Decimal::from(record.open_fee_bps),
            close_fee_bps: Decimal::from(record.close_fee_bps),
            borrow_rate_per_hour: borrow_rate,
            max_leverage: Decimal::from(record.max_leverage),
            maintenance_margin: Decimal::from(record.maintenance_margin_bps)
                / Decimal::from(10_000),
            impact: None,
        })
    }
}
