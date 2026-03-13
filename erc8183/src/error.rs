//! Typed error definitions for the ERC-8183 SDK.

use alloy::primitives::U256;

/// The primary error type for all ERC-8183 SDK operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A contract interaction failed.
    #[error("contract error: {0}")]
    Contract(#[from] alloy::contract::Error),

    /// An RPC transport error occurred.
    #[error("transport error: {0}")]
    Transport(#[from] alloy::transports::RpcError<alloy::transports::TransportErrorKind>),

    /// The contract address was not configured.
    #[error("contract not configured")]
    ContractNotConfigured,

    /// The requested job does not exist on-chain.
    #[error("job {job_id} does not exist")]
    JobNotFound {
        /// The queried job ID.
        job_id: U256,
    },

    /// A pending transaction was dropped or failed to confirm.
    #[error("pending transaction error: {0}")]
    PendingTransaction(#[from] alloy::providers::PendingTransactionError),

    /// JSON serialization / deserialization failed.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// An invalid job status was returned from the contract.
    #[error("invalid job status: {status}")]
    InvalidJobStatus {
        /// The raw status value returned by the contract.
        status: u8,
    },
}

/// A convenience type alias used throughout the SDK.
pub type Result<T> = core::result::Result<T, Error>;
