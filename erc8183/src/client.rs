//! The top-level [`Erc8183`] client for interacting with the Agentic Commerce Protocol.
//!
//! # Usage
//!
//! ```rust,no_run
//! use alloy::primitives::U256;
//! use alloy::providers::ProviderBuilder;
//! use erc8183::{Erc8183, Network};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let network = Network::MonadMainnet;
//! let provider = ProviderBuilder::new()
//!     .connect_http(network.rpc_url().parse()?);
//!
//! let job = Erc8183::new(provider)
//!     .with_network(network)
//!     .job()?
//!     .get_job(U256::from(1)).await?;
//! # Ok(())
//! # }
//! ```

use alloy::{primitives::Address, providers::Provider};

use crate::{
    error::{Error, Result},
    job::JobHandle,
    networks::Network,
};

/// The main client for interacting with the ERC-8183 Agentic Commerce Protocol.
///
/// `Erc8183` is generic over the alloy [`Provider`], which means it
/// works seamlessly with any transport (HTTP, `WebSocket`, IPC) and any
/// signer configuration the user has already set up via
/// [`ProviderBuilder`](alloy::providers::ProviderBuilder).
///
/// # Examples
///
/// Built-in network:
///
/// ```rust,no_run
/// use alloy::providers::ProviderBuilder;
/// use erc8183::{Erc8183, Network};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let network = Network::MonadMainnet;
/// let client = Erc8183::new(
///     ProviderBuilder::new()
///         .connect_http(network.rpc_url().parse()?),
/// )
/// .with_network(network);
/// # Ok(())
/// # }
/// ```
///
/// Custom deployment:
///
/// ```rust,no_run
/// use alloy::providers::ProviderBuilder;
/// use erc8183::Erc8183;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Erc8183::new(
///     ProviderBuilder::new()
///         .connect_http("https://rpc.example.com".parse()?),
/// )
/// .with_address("0x1234...".parse()?);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Erc8183<P> {
    provider: P,
    contract_address: Option<Address>,
}

impl<P: Provider> Erc8183<P> {
    /// Create a new `Erc8183` client wrapping the given alloy provider.
    ///
    /// No contract address is configured yet. Use
    /// [`with_network`](Self::with_network) to set the address from a known
    /// network, or [`with_address`](Self::with_address) for custom deployments.
    #[must_use]
    pub const fn new(provider: P) -> Self {
        Self {
            provider,
            contract_address: None,
        }
    }

    /// Configure the contract address from a pre-defined [`Network`].
    #[must_use]
    pub const fn with_network(mut self, network: Network) -> Self {
        self.contract_address = Some(network.address());
        self
    }

    /// Set a custom `AgenticCommerce` contract address.
    ///
    /// Use this for deployments not yet listed in [`Network`].
    #[must_use]
    pub const fn with_address(mut self, address: Address) -> Self {
        self.contract_address = Some(address);
        self
    }

    /// Get a handle to the Agentic Commerce contract for job operations.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ContractNotConfigured`] if the contract address
    /// has not been set.
    pub fn job(&self) -> Result<JobHandle<&P>> {
        let address = self.contract_address.ok_or(Error::ContractNotConfigured)?;
        Ok(JobHandle::new(&self.provider, address))
    }

    /// Get a reference to the underlying alloy provider.
    #[must_use]
    pub const fn provider(&self) -> &P {
        &self.provider
    }

    /// Consume this client and return the underlying alloy provider.
    #[must_use]
    pub fn into_provider(self) -> P {
        self.provider
    }

    /// Get the configured contract address, if any.
    #[must_use]
    pub const fn contract_address(&self) -> Option<Address> {
        self.contract_address
    }
}
