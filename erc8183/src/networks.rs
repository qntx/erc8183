//! Pre-configured network definitions with known contract addresses.
//!
//! Each [`Network`] variant represents a chain where the ERC-8183
//! `AgenticCommerce` contract has been officially deployed.
//! Only networks with **live deployments** are listed.
//!
//! For custom or private deployments, use
//! [`Erc8183::with_address`](crate::Erc8183::with_address) directly.

use alloy::primitives::{Address, address};

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

    /// All known ERC-8183 network variants.
    pub const ALL: &[Self] = &[Self::MonadMainnet];

    /// Look up a [`Network`] by its EIP-155 chain ID.
    ///
    /// Returns [`None`] if the chain ID is not a known ERC-8183 deployment.
    #[must_use]
    pub fn from_chain_id(chain_id: u64) -> Option<Self> {
        Self::ALL.iter().find(|n| n.chain_id() == chain_id).copied()
    }
}
