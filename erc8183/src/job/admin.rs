//! Owner-restricted admin operations on the `AgenticCommerce` contract.

use alloy::{
    primitives::{Address, U256},
    providers::Provider,
};

use super::JobHandle;
use crate::error::Result;

impl<P: Provider> JobHandle<P> {
    /// Set the platform fee in basis points. Restricted to the contract owner.
    ///
    /// The combined platform + evaluator fee must not exceed `MAX_FEE_BP` (5000).
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction fails.
    pub async fn set_platform_fee(&self, new_fee_bp: U256) -> Result<()> {
        self.contract()
            .setPlatformFee(new_fee_bp)
            .send()
            .await?
            .get_receipt()
            .await?;
        Ok(())
    }

    /// Set the evaluator fee in basis points. Restricted to the contract owner.
    ///
    /// The combined platform + evaluator fee must not exceed `MAX_FEE_BP` (5000).
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction fails.
    pub async fn set_evaluator_fee(&self, new_fee_bp: U256) -> Result<()> {
        self.contract()
            .setEvaluatorFee(new_fee_bp)
            .send()
            .await?
            .get_receipt()
            .await?;
        Ok(())
    }

    /// Set the treasury address. Restricted to the contract owner.
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction fails.
    pub async fn set_treasury(&self, new_treasury: Address) -> Result<()> {
        self.contract()
            .setTreasury(new_treasury)
            .send()
            .await?
            .get_receipt()
            .await?;
        Ok(())
    }

    /// Whitelist or de-whitelist a hook contract. Restricted to the contract owner.
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction fails.
    pub async fn set_hook_whitelist(&self, hook: Address, status: bool) -> Result<()> {
        self.contract()
            .setHookWhitelist(hook, status)
            .send()
            .await?
            .get_receipt()
            .await?;
        Ok(())
    }

    /// Start two-step ownership transfer. Restricted to the current owner.
    ///
    /// The `new_owner` must call [`accept_ownership`](Self::accept_ownership)
    /// to complete the transfer.
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction fails.
    pub async fn transfer_ownership(&self, new_owner: Address) -> Result<()> {
        self.contract()
            .transferOwnership(new_owner)
            .send()
            .await?
            .get_receipt()
            .await?;
        Ok(())
    }

    /// Accept a pending ownership transfer. Must be called by the pending owner.
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction fails.
    pub async fn accept_ownership(&self) -> Result<()> {
        self.contract()
            .acceptOwnership()
            .send()
            .await?
            .get_receipt()
            .await?;
        Ok(())
    }
}
