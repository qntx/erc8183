//! Core domain types for the ERC-8183 SDK.
//!
//! These types model the on-chain data structures defined by the ERC-8183
//! specification (Agentic Commerce Protocol) and provide ergonomic Rust
//! wrappers around the raw contract return values.

use alloy::primitives::{Address, B256, U256};
use serde::{Deserialize, Serialize};

use crate::error::{Result, SdkError};

/// Maximum value for `expiredAt` — the on-chain field is `uint48`.
pub const MAX_EXPIRY: u64 = (1u64 << 48) - 1;

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
    /// Returns [`SdkError::InvalidJobStatus`] if the value is out of range.
    pub const fn from_u8(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::Open),
            1 => Ok(Self::Funded),
            2 => Ok(Self::Submitted),
            3 => Ok(Self::Completed),
            4 => Ok(Self::Rejected),
            5 => Ok(Self::Expired),
            _ => Err(SdkError::InvalidJobStatus { status: value }),
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Job {
    /// The on-chain job identifier.
    pub id: U256,
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
#[non_exhaustive]
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

impl std::fmt::Display for Job {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Job #{} [{}] budget={} client={} provider={}",
            self.id, self.status, self.budget, self.client, self.provider,
        )
    }
}

#[cfg(test)]
mod tests {
    use alloy::primitives::{Address, U256};

    use super::*;

    const STATUS_TABLE: [(u8, JobStatus, &str, bool); 6] = [
        (0, JobStatus::Open, "Open", false),
        (1, JobStatus::Funded, "Funded", false),
        (2, JobStatus::Submitted, "Submitted", false),
        (3, JobStatus::Completed, "Completed", true),
        (4, JobStatus::Rejected, "Rejected", true),
        (5, JobStatus::Expired, "Expired", true),
    ];

    #[test]
    fn job_status_from_u8_roundtrip() {
        for &(raw, expected, _, _) in &STATUS_TABLE {
            let parsed = JobStatus::from_u8(raw).unwrap();
            assert_eq!(parsed, expected, "from_u8({raw})");
            assert_eq!(parsed as u8, raw, "{expected:?} repr");
        }
    }

    #[test]
    fn job_status_from_u8_rejects_out_of_range() {
        for v in (u8::try_from(STATUS_TABLE.len()).unwrap())..=255 {
            assert!(
                JobStatus::from_u8(v).is_err(),
                "expected error for status {v}"
            );
        }
    }

    #[test]
    fn job_status_is_terminal() {
        for &(_, variant, _, terminal) in &STATUS_TABLE {
            assert_eq!(variant.is_terminal(), terminal, "{variant:?}");
        }
    }

    #[test]
    fn job_status_display() {
        for &(_, variant, label, _) in &STATUS_TABLE {
            assert_eq!(variant.to_string(), label, "{variant:?}");
        }
    }

    #[test]
    fn create_job_params_new_sets_all_fields() {
        let provider = Address::repeat_byte(0x01);
        let evaluator = Address::repeat_byte(0x02);
        let expired_at = U256::from(1_700_000_000);
        let params = CreateJobParams::new(provider, evaluator, expired_at, "my job");
        assert_eq!(params.provider, provider);
        assert_eq!(params.evaluator, evaluator);
        assert_eq!(params.expired_at, expired_at);
        assert_eq!(params.description, "my job");
        assert_eq!(params.hook, Address::ZERO, "hook should default to ZERO");
    }

    #[test]
    fn create_job_params_with_hook() {
        let hook = Address::repeat_byte(0xAB);
        let params = CreateJobParams::new(Address::ZERO, Address::ZERO, U256::from(1000), "test")
            .with_hook(hook);
        assert_eq!(params.hook, hook);
    }

    #[test]
    fn job_display_contains_all_fields() {
        let client = Address::repeat_byte(0x11);
        let provider = Address::repeat_byte(0x22);
        let job = Job {
            id: U256::from(42),
            client,
            provider,
            evaluator: Address::ZERO,
            description: String::new(),
            budget: U256::from(100),
            expired_at: U256::ZERO,
            status: JobStatus::Funded,
            hook: Address::ZERO,
            deliverable: B256::ZERO,
        };
        let s = job.to_string();
        assert!(s.contains("#42"), "missing job id: {s}");
        assert!(s.contains("[Funded]"), "missing status: {s}");
        assert!(s.contains("budget=100"), "missing budget: {s}");
        assert!(
            s.contains(&format!("client={client}")),
            "missing client: {s}"
        );
        assert!(
            s.contains(&format!("provider={provider}")),
            "missing provider: {s}"
        );
    }
}
