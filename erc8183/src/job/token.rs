//! ERC-20 payment token helper operations.

use alloy::{
    primitives::{Address, Bytes, U256},
    providers::Provider,
};

use super::JobHandle;
use crate::{contracts::IERC20, error::Result};

impl<P: Provider> JobHandle<P> {
    /// Approve the payment token for the contract to spend.
    ///
    /// This is a prerequisite for [`fund`](Self::fund). The contract must be
    /// approved to transfer at least `amount` tokens from the caller.
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction fails.
    pub async fn approve_payment_token(&self, amount: U256) -> Result<()> {
        let token = self.contract().PAYMENT_TOKEN().call().await?;
        IERC20::new(token, &self.provider)
            .approve(self.address, amount)
            .send()
            .await?
            .get_receipt()
            .await?;
        Ok(())
    }

    /// Approve and fund in one call: approve the payment token, then fund
    /// the escrow.
    ///
    /// Convenience method that combines
    /// [`approve_payment_token`](Self::approve_payment_token) +
    /// [`fund`](Self::fund).
    ///
    /// # Errors
    ///
    /// Returns an error if either transaction fails.
    pub async fn approve_and_fund(
        &self,
        job_id: U256,
        expected_budget: U256,
        opt_params: Option<Bytes>,
    ) -> Result<()> {
        self.approve_payment_token(expected_budget).await?;
        self.fund(job_id, expected_budget, opt_params).await
    }

    /// Get the caller's current allowance for the payment token.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn payment_token_allowance(&self, owner: Address) -> Result<U256> {
        let token = self.contract().PAYMENT_TOKEN().call().await?;
        Ok(IERC20::new(token, &self.provider)
            .allowance(owner, self.address)
            .call()
            .await?)
    }

    /// Get the caller's payment token balance.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn payment_token_balance(&self, account: Address) -> Result<U256> {
        let token = self.contract().PAYMENT_TOKEN().call().await?;
        Ok(IERC20::new(token, &self.provider)
            .balanceOf(account)
            .call()
            .await?)
    }
}
