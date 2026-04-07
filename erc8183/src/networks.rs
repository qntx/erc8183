//! Pre-configured network definitions with known contract addresses.
//!
//! Each [`Network`] variant represents a chain where the ERC-8183
//! `AgenticCommerce` contract has been officially deployed.
//! Only networks with **live deployments** are listed.
//!
//! For custom or private deployments, use
//! [`Erc8183::with_address`](crate::Erc8183::with_address) directly.

use alloy::primitives::{Address, address};

use crate::error::{Result, SdkError};

const MONAD_MAINNET_RPC: &str = "https://rpc.monad.xyz";
const MONAD_MAINNET_EXPLORER: &str = "https://monad.socialscan.io";

/// The Monad Mainnet `AgenticCommerce` deployment.
const MONAD_MAINNET: Address = address!("E8c4FFb4A6F7B8040a7AE39F6651290E06A40725");

/// Pre-defined network configurations for ERC-8183 deployments.
///
/// Only networks with live contract deployments are included.
/// For unlisted chains, use [`Erc8183::with_address`](crate::Erc8183::with_address).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Network {
    /// Monad Mainnet (chain ID 143).
    MonadMainnet,
}

impl Network {
    /// Returns the deployed `AgenticCommerce` contract address for this network.
    #[must_use]
    pub const fn address(self) -> Address {
        match self {
            Self::MonadMainnet => MONAD_MAINNET,
        }
    }

    /// Returns the EIP-155 chain ID for this network.
    #[must_use]
    pub const fn chain_id(self) -> u64 {
        match self {
            Self::MonadMainnet => 143,
        }
    }

    /// Returns the public RPC endpoint URL for this network.
    #[must_use]
    pub const fn rpc_url(self) -> &'static str {
        match self {
            Self::MonadMainnet => MONAD_MAINNET_RPC,
        }
    }

    /// Returns the block explorer base URL for this network.
    #[must_use]
    pub const fn explorer_base_url(self) -> &'static str {
        match self {
            Self::MonadMainnet => MONAD_MAINNET_EXPLORER,
        }
    }

    /// Returns a block explorer URL for a given contract address.
    #[must_use]
    pub fn explorer_url(self, address: Address) -> String {
        format!("{}/address/{address}", self.explorer_base_url())
    }

    /// All known ERC-8183 network variants.
    pub const ALL: &[Self] = &[Self::MonadMainnet];
}

impl TryFrom<u64> for Network {
    type Error = SdkError;

    /// Look up a [`Network`] by its EIP-155 chain ID.
    ///
    /// # Errors
    ///
    /// Returns [`SdkError::UnknownChainId`] if the chain ID is not a known
    /// ERC-8183 deployment.
    fn try_from(chain_id: u64) -> Result<Self> {
        Self::ALL
            .iter()
            .find(|n| n.chain_id() == chain_id)
            .copied()
            .ok_or(SdkError::UnknownChainId { chain_id })
    }
}

impl std::fmt::Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MonadMainnet => write!(f, "Monad Mainnet"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_valid_chain_id() {
        let net = Network::try_from(143).unwrap();
        assert_eq!(net, Network::MonadMainnet);
    }

    #[test]
    fn try_from_invalid_chain_id() {
        assert!(Network::try_from(999).is_err());
    }

    #[test]
    fn all_roundtrips_via_try_from() {
        for net in Network::ALL {
            let resolved = Network::try_from(net.chain_id()).unwrap();
            assert_eq!(*net, resolved);
        }
    }

    #[test]
    fn monad_mainnet_properties() {
        let net = Network::MonadMainnet;
        assert_eq!(net.chain_id(), 143);
        assert!(!net.address().is_zero());
        assert!(net.rpc_url().starts_with("https://"));
        assert!(net.explorer_base_url().starts_with("https://"));
    }

    #[test]
    fn explorer_url_contains_address() {
        let net = Network::MonadMainnet;
        let addr = net.address();
        let url = net.explorer_url(addr);
        assert!(url.starts_with(net.explorer_base_url()));
        assert!(
            url.contains(&format!("{addr}")),
            "URL should embed the address: {url}"
        );
    }

    #[test]
    fn display() {
        assert_eq!(Network::MonadMainnet.to_string(), "Monad Mainnet");
    }
}
