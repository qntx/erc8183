//! Pre-configured network definitions with known contract addresses.
//!
//! ERC-8183 is currently in **Draft** status. Contract addresses listed here
//! are placeholders for future deployments. Once official CREATE2 deployments
//! are published, this module will be updated with the canonical addresses.

use alloy::primitives::Address;

/// Known contract address for a specific network deployment.
#[derive(Debug, Clone, Copy)]
pub struct NetworkAddress {
    /// The Agentic Commerce contract address.
    pub agentic_commerce: Address,
}

/// Pre-defined network configurations for ERC-8183 deployments.
///
/// **Note:** ERC-8183 is currently a Draft EIP. The addresses below are
/// placeholders and will be updated once official deployments are available.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Network {
    /// Ethereum Mainnet (chain ID 1).
    EthereumMainnet,
    /// Ethereum Sepolia testnet (chain ID 11155111).
    EthereumSepolia,
    /// Base Mainnet (chain ID 8453).
    BaseMainnet,
    /// Base Sepolia testnet (chain ID 84532).
    BaseSepolia,
}

impl Network {
    /// Returns the known contract address for this network.
    ///
    /// # Panics
    ///
    /// This function does not panic but currently returns zero addresses
    /// since ERC-8183 has no official deployments yet.
    #[must_use]
    pub const fn address(self) -> NetworkAddress {
        // TODO: Replace with actual CREATE2 deterministic addresses once deployed.
        NetworkAddress {
            agentic_commerce: Address::ZERO,
        }
    }

    /// Returns the EIP-155 chain ID for this network.
    #[must_use]
    pub const fn chain_id(self) -> u64 {
        match self {
            Self::EthereumMainnet => 1,
            Self::EthereumSepolia => 11_155_111,
            Self::BaseMainnet => 8453,
            Self::BaseSepolia => 84532,
        }
    }

    /// All known ERC-8183 network variants.
    pub const ALL: &[Self] = &[
        Self::EthereumMainnet,
        Self::EthereumSepolia,
        Self::BaseMainnet,
        Self::BaseSepolia,
    ];

    /// Look up a [`Network`] by its EIP-155 chain ID.
    ///
    /// Returns [`None`] if the chain ID is not a known ERC-8183 deployment.
    #[must_use]
    pub fn from_chain_id(chain_id: u64) -> Option<Self> {
        Self::ALL.iter().find(|n| n.chain_id() == chain_id).copied()
    }
}
