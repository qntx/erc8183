//! Typed error definitions for the ERC-8183 SDK.

use alloy::sol_types::SolError;

use crate::contracts::AgenticCommerce;

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

    /// Expected event was not found in a transaction receipt.
    #[error("event not found in receipt: {context}")]
    EventNotFound {
        /// Description of which event was expected.
        context: &'static str,
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

/// Decode a 4-byte revert selector into a human-readable error name.
///
/// Returns the error name if it matches a known `AgenticCommerce` custom
/// error, or `None` for unrecognized selectors.
///
/// # Examples
///
/// ```
/// assert_eq!(
///     erc8183::error::decode_revert_reason(&[0xd3, 0x6c, 0x85, 0x00]),
///     Some("InvalidExpiry"),
/// );
/// ```
#[must_use]
pub fn decode_revert_reason(data: &[u8]) -> Option<&'static str> {
    let sel: [u8; 4] = data.get(..4)?.try_into().ok()?;
    match sel {
        AgenticCommerce::ZeroAddress::SELECTOR => Some("ZeroAddress"),
        AgenticCommerce::InvalidExpiry::SELECTOR => Some("InvalidExpiry"),
        AgenticCommerce::InvalidStatus::SELECTOR => Some("InvalidStatus"),
        AgenticCommerce::Unauthorized::SELECTOR => Some("Unauthorized"),
        AgenticCommerce::ProviderAlreadySet::SELECTOR => Some("ProviderAlreadySet"),
        AgenticCommerce::ProviderNotSet::SELECTOR => Some("ProviderNotSet"),
        AgenticCommerce::BudgetMismatch::SELECTOR => Some("BudgetMismatch"),
        AgenticCommerce::ZeroBudget::SELECTOR => Some("ZeroBudget"),
        AgenticCommerce::JobAlreadyExpired::SELECTOR => Some("JobAlreadyExpired"),
        AgenticCommerce::JobNotExpired::SELECTOR => Some("JobNotExpired"),
        AgenticCommerce::FeeTooHigh::SELECTOR => Some("FeeTooHigh"),
        AgenticCommerce::JobDoesNotExist::SELECTOR => Some("JobDoesNotExist"),
        AgenticCommerce::HookNotWhitelisted::SELECTOR => Some("HookNotWhitelisted"),
        AgenticCommerce::HookInterfaceNotSupported::SELECTOR => Some("HookInterfaceNotSupported"),
        AgenticCommerce::DescriptionTooLong::SELECTOR => Some("DescriptionTooLong"),
        AgenticCommerce::OwnableUnauthorizedAccount::SELECTOR => Some("OwnableUnauthorizedAccount"),
        AgenticCommerce::OwnableInvalidOwner::SELECTOR => Some("OwnableInvalidOwner"),
        _ => None,
    }
}
