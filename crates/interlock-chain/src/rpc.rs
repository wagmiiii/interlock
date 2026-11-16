use thiserror::Error;

#[derive(Debug, Error)]
pub enum RpcError {
    #[error("HTTP / RPC transport error: {0}")]
    Transport(String),
    #[error("Failed to decode account data for {pubkey}: {reason}")]
    DecodeError { pubkey: String, reason: String },
    #[error(
        "Slot mismatch during consistent batch read: requested {expected_slot}, got {actual_slot}"
    )]
    SlotMismatch {
        expected_slot: u64,
        actual_slot: u64,
    },
}

/// Slot-consistent raw account data fetched via getMultipleAccounts.
#[derive(Debug, Clone)]
pub struct AccountResponse {
    pub pubkey: String,
    pub data: Vec<u8>,
    pub owner: String,
    pub lamports: u64,
    pub slot: u64,
}

/// Slot-consistent RPC client stub.
pub struct InterlockRpcClient {
    pub endpoint: String,
    pub secondary_endpoint: Option<String>,
}

impl InterlockRpcClient {
    pub fn new(endpoint: String, secondary_endpoint: Option<String>) -> Self {
        Self {
            endpoint,
            secondary_endpoint,
        }
    }

    /// Read multiple accounts at a single slot boundary.
    pub async fn get_multiple_accounts_consistent(
        &self,
        pubkeys: &[String],
    ) -> Result<(u64, Vec<AccountResponse>), RpcError> {
        // Scaffold stub: will call Solana getMultipleAccounts with commitment: confirmed
        Ok((0, Vec::with_capacity(pubkeys.len())))
    }
}
