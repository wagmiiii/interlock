use interlock_core::{MarketId, VenueParams};
use rust_decimal::Decimal;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecoderError {
    #[error("Unknown account layout version for {account_type}. Expected discriminator {expected:?}, got {actual:?}")]
    InvalidDiscriminator {
        account_type: &'static str,
        expected: Vec<u8>,
        actual: Vec<u8>,
    },
    #[error("Missing or truncated account data (length {0})")]
    TruncatedData(usize),
}

/// Decoder for venue market and pool state accounts.
pub struct VenueDecoder;

impl VenueDecoder {
    /// Decode venue parameters from raw account data.
    pub fn decode_venue_params(
        market: &MarketId,
        pool_account_data: &[u8],
    ) -> Result<VenueParams, DecoderError> {
        if pool_account_data.len() < 8 {
            return Err(DecoderError::TruncatedData(pool_account_data.len()));
        }
        // Layout decoder will be implemented in task P1-02 on committed fixtures
        let _ = market;
        Ok(VenueParams {
            open_fee_bps: Decimal::ZERO,
            close_fee_bps: Decimal::ZERO,
            borrow_rate_per_hour: Decimal::ZERO,
            max_leverage: Decimal::ZERO,
            maintenance_margin: Decimal::ZERO,
            impact: None,
        })
    }
}
