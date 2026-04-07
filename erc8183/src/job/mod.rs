//! Job operations for the Agentic Commerce Protocol.
//!
//! This module provides the [`JobHandle`] type, which wraps all read and write
//! functions exposed by the `AgenticCommerce` contract. A `JobHandle` is
//! obtained via [`Erc8183::job()`](crate::Erc8183::job).
//!
//! ## State Machine
//!
//! Jobs follow a strict state machine:
//!
//! ```text
//! Open → Funded → Submitted → Completed
//!   ↓       ↓         ↓      → Rejected
//!   Rejected Rejected  Rejected
//!            Expired   Expired
//! ```

mod admin;
mod lifecycle;
mod token;
mod views;

use alloy::{primitives::Address, providers::Provider};

use crate::contracts::{AgenticCommerce, IERC8183};

/// A handle for interacting with an ERC-8183 contract.
///
/// Core lifecycle operations (`create_job`, `fund`, `submit`, `complete`,
/// `reject`, `claim_refund`) use the standard [`IERC8183`] interface binding
/// and are portable across **any** ERC-8183 compliant implementation.
///
/// View and admin operations (`get_job`, `set_platform_fee`, etc.) use the
/// [`AgenticCommerce`] binding and are specific to the QNTX implementation.
///
/// Created via [`Erc8183::job()`](crate::Erc8183::job).
#[derive(Debug)]
pub struct JobHandle<P> {
    address: Address,
    provider: P,
}

impl<P: Provider> JobHandle<P> {
    /// Create a new `JobHandle` from a provider and contract address.
    pub(crate) const fn new(provider: P, address: Address) -> Self {
        Self { address, provider }
    }

    /// Returns the contract address this handle points to.
    #[must_use]
    pub const fn contract_address(&self) -> Address {
        self.address
    }

    /// Standard ERC-8183 interface — portable across any compliant contract.
    const fn standard(&self) -> IERC8183::IERC8183Instance<&P> {
        IERC8183::new(self.address, &self.provider)
    }

    /// Full QNTX `AgenticCommerce` binding — implementation-specific operations.
    const fn contract(&self) -> AgenticCommerce::AgenticCommerceInstance<&P> {
        AgenticCommerce::new(self.address, &self.provider)
    }
}
