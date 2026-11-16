use base64::prelude::*;
use serde::{Deserialize, Serialize};
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
    #[error("RPC returned error response: {code} - {message}")]
    RpcFailure { code: i64, message: String },
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

#[derive(Serialize)]
struct RpcRequest<'a> {
    jsonrpc: &'static str,
    id: u64,
    method: &'static str,
    params: (&'a [String], RpcConfig),
}

#[derive(Serialize)]
struct RpcConfig {
    encoding: &'static str,
    commitment: &'static str,
}

#[derive(Deserialize)]
struct RpcResponseRaw {
    result: Option<RpcResultValue>,
    error: Option<RpcErrorObject>,
}

#[derive(Deserialize)]
struct RpcErrorObject {
    code: i64,
    message: String,
}

#[derive(Deserialize)]
struct RpcResultValue {
    context: RpcContext,
    value: Vec<Option<RawAccountData>>,
}

#[derive(Deserialize)]
struct RpcContext {
    slot: u64,
}

#[derive(Deserialize)]
struct RawAccountData {
    data: (String, String),
    lamports: u64,
    owner: String,
}

/// Slot-consistent RPC client.
pub struct InterlockRpcClient {
    pub endpoint: String,
    pub secondary_endpoint: Option<String>,
    client: reqwest::Client,
}

impl InterlockRpcClient {
    pub fn new(endpoint: String, secondary_endpoint: Option<String>) -> Self {
        Self {
            endpoint,
            secondary_endpoint,
            client: reqwest::Client::new(),
        }
    }

    /// Read multiple accounts at a single slot boundary using Solana JSON-RPC `getMultipleAccounts`.
    pub async fn get_multiple_accounts_consistent(
        &self,
        pubkeys: &[String],
    ) -> Result<(u64, Vec<AccountResponse>), RpcError> {
        let req_body = RpcRequest {
            jsonrpc: "2.0",
            id: 1,
            method: "getMultipleAccounts",
            params: (
                pubkeys,
                RpcConfig {
                    encoding: "base64",
                    commitment: "confirmed",
                },
            ),
        };

        let response = self
            .client
            .post(&self.endpoint)
            .json(&req_body)
            .send()
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?;

        let rpc_res: RpcResponseRaw = response
            .json()
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?;

        if let Some(err) = rpc_res.error {
            return Err(RpcError::RpcFailure {
                code: err.code,
                message: err.message,
            });
        }

        let result = rpc_res.result.ok_or_else(|| {
            RpcError::Transport("Missing result object in JSON-RPC response".to_string())
        })?;

        let slot = result.context.slot;
        let mut accounts = Vec::with_capacity(pubkeys.len());

        for (i, opt_acc) in result.value.into_iter().enumerate() {
            let pubkey = pubkeys
                .get(i)
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());
            if let Some(acc) = opt_acc {
                let decoded_bytes = BASE64_STANDARD.decode(acc.data.0.as_bytes()).map_err(|e| {
                    RpcError::DecodeError {
                        pubkey: pubkey.clone(),
                        reason: e.to_string(),
                    }
                })?;
                accounts.push(AccountResponse {
                    pubkey,
                    data: decoded_bytes,
                    owner: acc.owner,
                    lamports: acc.lamports,
                    slot,
                });
            }
        }

        Ok((slot, accounts))
    }
}
