//! Core domain types for the ERC-8183 SDK.
//!
//! These types model the on-chain data structures defined by the ERC-8183
//! specification (Agentic Commerce Protocol) and provide ergonomic Rust
//! wrappers around the raw contract return values.

use alloy::primitives::{Address, B256, Bytes, FixedBytes, U256};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// The six possible states of a job in the Agentic Commerce Protocol.
///
/// State transitions follow:
/// `Open → Funded → Submitted → Completed | Rejected | Expired`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum JobStatus {
    /// Created; budget not yet set or not yet funded.
    Open = 0,
    /// Budget escrowed. Provider may submit work.
    Funded = 1,
    /// Provider has submitted work. Evaluator may complete or reject.
    Submitted = 2,
    /// Terminal. Escrow released to provider (minus optional platform fee).
    Completed = 3,
    /// Terminal. Escrow refunded to client.
    Rejected = 4,
    /// Terminal. Same as Rejected; escrow refunded after expiry.
    Expired = 5,
}

impl JobStatus {
    /// Parse a raw `u8` status value from the contract into a [`JobStatus`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidJobStatus`] if the value is out of range.
    pub const fn from_u8(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::Open),
            1 => Ok(Self::Funded),
            2 => Ok(Self::Submitted),
            3 => Ok(Self::Completed),
            4 => Ok(Self::Rejected),
            5 => Ok(Self::Expired),
            _ => Err(Error::InvalidJobStatus { status: value }),
        }
    }

    /// Returns `true` if the job has reached a terminal state.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Rejected | Self::Expired)
    }
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "Open"),
            Self::Funded => write!(f, "Funded"),
            Self::Submitted => write!(f, "Submitted"),
            Self::Completed => write!(f, "Completed"),
            Self::Rejected => write!(f, "Rejected"),
            Self::Expired => write!(f, "Expired"),
        }
    }
}

/// A fully resolved job as returned by `getJob`.
#[derive(Debug, Clone)]
pub struct Job {
    /// The address that created and funded the job.
    pub client: Address,
    /// The address that performs the work. May be [`Address::ZERO`] if not yet assigned.
    pub provider: Address,
    /// The address that attests completion or rejection.
    pub evaluator: Address,
    /// Human-readable job description (e.g. brief, scope reference).
    pub description: String,
    /// The escrowed budget amount in payment token units.
    pub budget: U256,
    /// Unix timestamp after which anyone may trigger a refund.
    pub expired_at: U256,
    /// Current job status.
    pub status: JobStatus,
    /// Optional hook contract address. [`Address::ZERO`] means no hook.
    pub hook: Address,
    /// Deliverable reference submitted by the provider (e.g. IPFS CID hash).
    /// [`B256::ZERO`] if not yet submitted.
    pub deliverable: B256,
}

/// Parameters for creating a new job via [`crate::client::Erc8183::job`].
///
/// Use the builder methods to configure optional fields before passing
/// to [`Job::create_job`](crate::job::JobHandle::create_job).
#[derive(Debug, Clone)]
pub struct CreateJobParams {
    /// Provider address. Use [`Address::ZERO`] for deferred provider assignment.
    pub provider: Address,
    /// Evaluator address. Must be non-zero.
    pub evaluator: Address,
    /// Unix timestamp for job expiry.
    pub expired_at: U256,
    /// Job description string.
    pub description: String,
    /// Optional hook contract address. Defaults to [`Address::ZERO`] (no hook).
    pub hook: Address,
}

impl CreateJobParams {
    /// Create new job parameters with the required fields.
    ///
    /// `provider` may be [`Address::ZERO`] for deferred assignment.
    /// `hook` defaults to [`Address::ZERO`] (no hook).
    #[must_use]
    pub fn new(
        provider: Address,
        evaluator: Address,
        expired_at: U256,
        description: impl Into<String>,
    ) -> Self {
        Self {
            provider,
            evaluator,
            expired_at,
            description: description.into(),
            hook: Address::ZERO,
        }
    }

    /// Set the hook contract address.
    #[must_use]
    pub const fn with_hook(mut self, hook: Address) -> Self {
        self.hook = hook;
        self
    }
}

/// Parameters for submitting work on a job.
#[derive(Debug, Clone)]
pub struct SubmitParams {
    /// The job ID.
    pub job_id: U256,
    /// Reference to the deliverable (e.g. IPFS CID hash, attestation commitment).
    pub deliverable: FixedBytes<32>,
    /// Optional parameters forwarded to the hook contract.
    pub opt_params: Option<Bytes>,
}

/// Parameters for completing or rejecting a job.
#[derive(Debug, Clone)]
pub struct AttestParams {
    /// The job ID.
    pub job_id: U256,
    /// Optional attestation reason (e.g. hash of off-chain evidence).
    /// Defaults to `bytes32(0)` if not specified.
    pub reason: FixedBytes<32>,
    /// Optional parameters forwarded to the hook contract.
    pub opt_params: Option<Bytes>,
}
