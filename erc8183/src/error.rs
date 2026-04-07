//! Typed error definitions for the ERC-8183 SDK.

use alloy::sol_types::SolError;

use crate::contracts::AgenticCommerce;

/// The primary error type for all ERC-8183 SDK operations.
#[derive(Debug, thiserror::Error)]
pub enum SdkError {
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

    /// The chain ID does not correspond to a known ERC-8183 deployment.
    #[error("unknown chain ID: {chain_id}")]
    UnknownChainId {
        /// The unrecognised chain ID.
        chain_id: u64,
    },
}

/// A convenience type alias used throughout the SDK.
pub type Result<T> = core::result::Result<T, SdkError>;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_revert_returns_none_for_unknown() {
        assert_eq!(decode_revert_reason(&[0xFF, 0xFF, 0xFF, 0xFF]), None);
    }

    #[test]
    fn decode_revert_returns_none_for_short_input() {
        assert_eq!(decode_revert_reason(&[0x01, 0x02, 0x03]), None);
        assert_eq!(decode_revert_reason(&[]), None);
    }

    #[test]
    fn decode_revert_all_custom_errors() {
        let selectors: &[([u8; 4], &str)] = &[
            (AgenticCommerce::ZeroAddress::SELECTOR, "ZeroAddress"),
            (AgenticCommerce::InvalidExpiry::SELECTOR, "InvalidExpiry"),
            (AgenticCommerce::InvalidStatus::SELECTOR, "InvalidStatus"),
            (AgenticCommerce::Unauthorized::SELECTOR, "Unauthorized"),
            (
                AgenticCommerce::ProviderAlreadySet::SELECTOR,
                "ProviderAlreadySet",
            ),
            (AgenticCommerce::ProviderNotSet::SELECTOR, "ProviderNotSet"),
            (AgenticCommerce::BudgetMismatch::SELECTOR, "BudgetMismatch"),
            (AgenticCommerce::ZeroBudget::SELECTOR, "ZeroBudget"),
            (
                AgenticCommerce::JobAlreadyExpired::SELECTOR,
                "JobAlreadyExpired",
            ),
            (AgenticCommerce::JobNotExpired::SELECTOR, "JobNotExpired"),
            (AgenticCommerce::FeeTooHigh::SELECTOR, "FeeTooHigh"),
            (
                AgenticCommerce::JobDoesNotExist::SELECTOR,
                "JobDoesNotExist",
            ),
            (
                AgenticCommerce::HookNotWhitelisted::SELECTOR,
                "HookNotWhitelisted",
            ),
            (
                AgenticCommerce::HookInterfaceNotSupported::SELECTOR,
                "HookInterfaceNotSupported",
            ),
            (
                AgenticCommerce::DescriptionTooLong::SELECTOR,
                "DescriptionTooLong",
            ),
            (
                AgenticCommerce::OwnableUnauthorizedAccount::SELECTOR,
                "OwnableUnauthorizedAccount",
            ),
            (
                AgenticCommerce::OwnableInvalidOwner::SELECTOR,
                "OwnableInvalidOwner",
            ),
        ];
        for (sel, name) in selectors {
            assert_eq!(
                decode_revert_reason(sel),
                Some(*name),
                "selector {sel:02x?} should decode to {name}"
            );
        }
    }

    #[test]
    fn error_display_messages() {
        assert_eq!(
            SdkError::ContractNotConfigured.to_string(),
            "contract not configured"
        );
        assert_eq!(
            SdkError::InvalidJobStatus { status: 99 }.to_string(),
            "invalid job status: 99"
        );
        assert_eq!(
            SdkError::UnknownChainId { chain_id: 42 }.to_string(),
            "unknown chain ID: 42"
        );
    }
}
