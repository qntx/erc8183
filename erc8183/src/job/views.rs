//! Read-only view queries against the `AgenticCommerce` contract.

use alloy::{
    primitives::{Address, U256},
    providers::Provider,
};

use super::JobHandle;
use crate::{
    error::Result,
    types::{Job, JobStatus},
};

impl<P: Provider> JobHandle<P> {
    /// Get the full job data by ID.
    ///
    /// **Note**: Uses the QNTX `AgenticCommerce.getJob()` return struct.
    /// Other ERC-8183 implementations may return a different struct layout.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails or the status is invalid.
    pub async fn get_job(&self, job_id: U256) -> Result<Job> {
        let raw = self.contract().getJob(job_id).call().await?;
        Ok(Job {
            id: raw.id,
            client: raw.client,
            provider: raw.provider,
            evaluator: raw.evaluator,
            description: raw.description.clone(),
            budget: raw.budget,
            expired_at: raw.expiredAt,
            status: JobStatus::from_u8(raw.status)?,
            hook: raw.hook,
            deliverable: raw.deliverable,
        })
    }

    /// Get the total number of jobs created.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn total_jobs(&self) -> Result<U256> {
        Ok(self.contract().totalJobs().call().await?)
    }

    /// Get the ERC-20 payment token address.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn payment_token(&self) -> Result<Address> {
        Ok(self.contract().PAYMENT_TOKEN().call().await?)
    }

    /// Get the current platform fee in basis points.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn platform_fee_bp(&self) -> Result<U256> {
        Ok(self.contract().platformFeeBp().call().await?)
    }

    /// Get the current evaluator fee in basis points.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn evaluator_fee_bp(&self) -> Result<U256> {
        Ok(self.contract().evaluatorFeeBp().call().await?)
    }

    /// Get the platform treasury address.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn treasury(&self) -> Result<Address> {
        Ok(self.contract().treasury().call().await?)
    }

    /// Check if a hook contract is whitelisted.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn is_hook_whitelisted(&self, hook: Address) -> Result<bool> {
        Ok(self.contract().whitelistedHooks(hook).call().await?)
    }

    /// Get the contract owner address.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn owner(&self) -> Result<Address> {
        Ok(self.contract().owner().call().await?)
    }

    /// Get the pending owner address (for two-step ownership transfer).
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn pending_owner(&self) -> Result<Address> {
        Ok(self.contract().pendingOwner().call().await?)
    }
}
