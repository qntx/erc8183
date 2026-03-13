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

use alloy::{
    primitives::{Address, Bytes, FixedBytes, U256},
    providers::Provider,
};

use crate::{
    contracts::AgenticCommerce,
    error::{Error, Result},
    types::{CreateJobParams, Job, JobStatus},
};

/// A handle to the Agentic Commerce contract bound to a specific provider.
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
    pub async fn create_job(&self, params: &CreateJobParams) -> Result<U256> {
        let contract = AgenticCommerce::new(self.address, &self.provider);
        let receipt = contract
            .createJob(
                params.provider,
                params.evaluator,
                params.expired_at,
                params.description.clone(),
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
        provider: Address,
        opt_params: Option<Bytes>,
    ) -> Result<()> {
        let contract = AgenticCommerce::new(self.address, &self.provider);
        contract
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
        let contract = AgenticCommerce::new(self.address, &self.provider);
        contract
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
        let contract = AgenticCommerce::new(self.address, &self.provider);
        contract
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
        let contract = AgenticCommerce::new(self.address, &self.provider);
        contract
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
    /// optional platform fee to treasury).
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
        let contract = AgenticCommerce::new(self.address, &self.provider);
        contract
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
        let contract = AgenticCommerce::new(self.address, &self.provider);
        contract
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
        let contract = AgenticCommerce::new(self.address, &self.provider);
        contract
            .claimRefund(job_id)
            .send()
            .await?
            .get_receipt()
            .await?;
        Ok(())
    }

    /// Get the full job data by ID.
    ///
    /// **Note**: This view function is NOT mandated by EIP-8183 but is a
    /// common implementation pattern. The call will fail if the target
    /// contract does not implement `getJob(uint256)`.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails or the status is invalid.
    pub async fn get_job(&self, job_id: U256) -> Result<Job> {
        let contract = AgenticCommerce::new(self.address, &self.provider);
        let raw = contract.getJob(job_id).call().await?;
        Ok(Job {
            client: raw.client,
            provider: raw.provider,
            evaluator: raw.evaluator,
            description: raw.description.clone(),
            budget: raw.budget,
            expired_at: raw.expiredAt,
            status: JobStatus::from_u8(raw.status)?,
            hook: raw.hook,
        })
    }

    /// Parse `jobId` from a transaction receipt's `JobCreated` event.
    fn parse_job_id(receipt: &alloy::rpc::types::TransactionReceipt) -> Result<U256> {
        receipt
            .inner
            .logs()
            .iter()
            .find_map(|log| {
                log.log_decode::<AgenticCommerce::JobCreated>()
                    .ok()
                    .map(|e| e.inner.data.jobId)
            })
            .ok_or(Error::Contract(alloy::contract::Error::UnknownFunction(
                "createJob: no JobCreated event found".to_owned(),
            )))
    }
}
