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

    const fn contract(&self) -> AgenticCommerce::AgenticCommerceInstance<&P> {
        AgenticCommerce::new(self.address, &self.provider)
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
        let receipt = self
            .contract()
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
        self.contract()
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
        self.contract()
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
        self.contract()
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
        self.contract()
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
        self.contract()
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
        self.contract()
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
        self.contract()
            .claimRefund(job_id)
            .send()
            .await?
            .get_receipt()
            .await?;
        Ok(())
    }

    /// Get the full job data by ID.
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
