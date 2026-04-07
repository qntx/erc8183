//! Core lifecycle operations: create, assign, fund, submit, complete, reject, refund.

use alloy::{
    primitives::{Bytes, FixedBytes, U256},
    providers::Provider,
};

use super::JobHandle;
use crate::{
    contracts::IERC8183,
    error::{Result, SdkError},
    types::CreateJobParams,
};

impl<P: Provider> JobHandle<P> {
    /// Create a new job in `Open` state.
    ///
    /// The caller (`msg.sender`) becomes the **client**. Provider may be
    /// [`Address::ZERO`] for deferred assignment via [`set_provider`](Self::set_provider).
    ///
    /// Returns the newly created `jobId` (`U256`).
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction fails (e.g. evaluator is zero,
    /// `expiredAt` is not in the future).
    pub async fn create_job(&self, params: CreateJobParams) -> Result<U256> {
        let receipt = self
            .standard()
            .createJob(
                params.provider,
                params.evaluator,
                params.expired_at,
                params.description,
                params.hook,
            )
            .send()
            .await?
            .get_receipt()
            .await?;
        Self::parse_job_id(&receipt)
    }

    /// Assign a provider to an Open job.
    ///
    /// Must be called by the **client**. Reverts if the job already has a
    /// provider or is not in `Open` state.
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction fails.
    pub async fn set_provider(
        &self,
        job_id: U256,
        provider: alloy::primitives::Address,
        opt_params: Option<Bytes>,
    ) -> Result<()> {
        self.standard()
            .setProvider(job_id, provider, opt_params.unwrap_or_default())
            .send()
            .await?
            .get_receipt()
            .await?;
        Ok(())
    }

    /// Set or update the budget for an Open job.
    ///
    /// May be called by the **client** or **provider** to propose or
    /// negotiate a price.
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction fails.
    pub async fn set_budget(
        &self,
        job_id: U256,
        amount: U256,
        opt_params: Option<Bytes>,
    ) -> Result<()> {
        self.standard()
            .setBudget(job_id, amount, opt_params.unwrap_or_default())
            .send()
            .await?
            .get_receipt()
            .await?;
        Ok(())
    }

    /// Fund the job escrow, transitioning from `Open` to `Funded`.
    ///
    /// Must be called by the **client**. The caller must have approved the
    /// contract to spend `expected_budget` of the payment token.
    ///
    /// `expected_budget` acts as front-running protection — the transaction
    /// reverts if `job.budget != expected_budget`.
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction fails (e.g. provider not set,
    /// budget mismatch, insufficient allowance).
    pub async fn fund(
        &self,
        job_id: U256,
        expected_budget: U256,
        opt_params: Option<Bytes>,
    ) -> Result<()> {
        self.standard()
            .fund(job_id, expected_budget, opt_params.unwrap_or_default())
            .send()
            .await?
            .get_receipt()
            .await?;
        Ok(())
    }

    /// Submit work deliverable, transitioning from `Funded` to `Submitted`.
    ///
    /// Must be called by the **provider**.
    ///
    /// # Parameters
    ///
    /// - `job_id`: The target job.
    /// - `deliverable`: A `bytes32` reference to submitted work (e.g. IPFS CID
    ///   hash, attestation commitment).
    /// - `opt_params`: Optional parameters forwarded to the hook contract.
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction fails.
    pub async fn submit(
        &self,
        job_id: U256,
        deliverable: FixedBytes<32>,
        opt_params: Option<Bytes>,
    ) -> Result<()> {
        self.standard()
            .submit(job_id, deliverable, opt_params.unwrap_or_default())
            .send()
            .await?
            .get_receipt()
            .await?;
        Ok(())
    }

    /// Mark the job as completed, releasing escrow to the provider.
    ///
    /// Must be called by the **evaluator** when the job is in `Submitted` state.
    /// On completion, escrowed funds are transferred to the provider (minus
    /// optional platform and evaluator fees).
    ///
    /// # Parameters
    ///
    /// - `job_id`: The target job.
    /// - `reason`: Optional attestation reason (e.g. hash of off-chain evidence).
    ///   Use `FixedBytes::ZERO` for no reason.
    /// - `opt_params`: Optional parameters forwarded to the hook contract.
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction fails.
    pub async fn complete(
        &self,
        job_id: U256,
        reason: FixedBytes<32>,
        opt_params: Option<Bytes>,
    ) -> Result<()> {
        self.standard()
            .complete(job_id, reason, opt_params.unwrap_or_default())
            .send()
            .await?
            .get_receipt()
            .await?;
        Ok(())
    }

    /// Reject the job, refunding escrow to the client.
    ///
    /// - **Open**: only the **client** may reject.
    /// - **Funded** or **Submitted**: only the **evaluator** may reject.
    ///
    /// # Parameters
    ///
    /// - `job_id`: The target job.
    /// - `reason`: Optional attestation reason. Use `FixedBytes::ZERO` for no reason.
    /// - `opt_params`: Optional parameters forwarded to the hook contract.
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction fails.
    pub async fn reject(
        &self,
        job_id: U256,
        reason: FixedBytes<32>,
        opt_params: Option<Bytes>,
    ) -> Result<()> {
        self.standard()
            .reject(job_id, reason, opt_params.unwrap_or_default())
            .send()
            .await?
            .get_receipt()
            .await?;
        Ok(())
    }

    /// Claim a refund after job expiry.
    ///
    /// Anyone may call this when `block.timestamp >= job.expiredAt` and the
    /// job is in `Funded` or `Submitted` state. Transitions the job to `Expired`
    /// and refunds the client.
    ///
    /// This function is deliberately **not hookable** so that refunds after
    /// expiry cannot be blocked by a malicious hook.
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction fails.
    pub async fn claim_refund(&self, job_id: U256) -> Result<()> {
        self.standard()
            .claimRefund(job_id)
            .send()
            .await?
            .get_receipt()
            .await?;
        Ok(())
    }

    /// Parse `jobId` from a transaction receipt's `JobCreated` event.
    ///
    /// Uses the standard [`IERC8183::JobCreated`] event for portability.
    fn parse_job_id(receipt: &alloy::rpc::types::TransactionReceipt) -> Result<U256> {
        receipt
            .inner
            .logs()
            .iter()
            .find_map(|log| {
                log.log_decode::<IERC8183::JobCreated>()
                    .ok()
                    .map(|e| e.inner.data.jobId)
            })
            .ok_or(SdkError::EventNotFound {
                context: "JobCreated event not found in createJob receipt",
            })
    }
}
