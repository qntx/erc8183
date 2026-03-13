//! Contract bindings generated via inline Solidity interfaces.
//!
//! These interfaces are derived from the [ERC-8183](https://eips.ethereum.org/EIPS/eip-8183)
//! specification (Agentic Commerce Protocol).
//!
//! ERC-8183 is currently a **Draft** EIP. The specification defines core
//! mutating functions and recommends a minimum set of events. It does NOT
//! mandate specific view functions, struct layouts, or indexed modifiers.
//!
//! The event signatures below match the spec's recommended minimum parameters.
//! Since `topic0` is derived from the full event signature, the SDK's events
//! **must** match the target contract exactly for log decoding to succeed.
//! If a deployed contract uses different event signatures, the user should
//! provide custom bindings.
//!
//! Only [`IACPHook`] has a normative Solidity interface definition in the spec.
//! The `AgenticCommerce` contract binding is our best-effort interpretation of
//! the spec's prose requirements.

use alloy::sol;

sol! {
    /// ERC-8183 Agentic Commerce Protocol — job escrow with evaluator attestation.
    ///
    /// Lifecycle: Open → Funded → Submitted → Completed | Rejected | Expired.
    /// ERC-20 escrow with optional hook callbacks and platform fee.
    #[allow(missing_docs, clippy::too_many_arguments)]
    #[sol(rpc)]
    contract AgenticCommerce {
        /// Job data structure (implementation-specific, not mandated by spec).
        ///
        /// The spec requires at minimum: client, provider, evaluator,
        /// description, budget, expiredAt, status, and optionally hook.
        struct Job {
            address client;
            address provider;
            address evaluator;
            string description;
            uint256 budget;
            uint256 expiredAt;
            uint8 status;
            address hook;
        }

        /// Spec: "JobCreated(jobId, client, provider, evaluator, expiredAt)"
        event JobCreated(
            uint256 indexed jobId,
            address indexed client,
            address provider,
            address evaluator,
            uint256 expiredAt
        );

        /// Spec: "ProviderSet(jobId, provider)"
        event ProviderSet(uint256 indexed jobId, address indexed provider);

        /// Spec: "BudgetSet(jobId, amount)"
        event BudgetSet(uint256 indexed jobId, uint256 amount);

        /// Spec: "JobFunded(jobId, client, amount)"
        event JobFunded(uint256 indexed jobId, address indexed client, uint256 amount);

        /// Spec: "JobSubmitted(jobId, provider, deliverable)"
        event JobSubmitted(uint256 indexed jobId, address indexed provider, bytes32 deliverable);

        /// Spec: "JobCompleted(jobId, evaluator, reason)"
        event JobCompleted(uint256 indexed jobId, address indexed evaluator, bytes32 reason);

        /// Spec: "JobRejected(jobId, rejector, reason)"
        event JobRejected(uint256 indexed jobId, address indexed rejector, bytes32 reason);

        /// Spec: "JobExpired(jobId)"
        event JobExpired(uint256 indexed jobId);

        /// Spec: "PaymentReleased(jobId, provider, amount)"
        event PaymentReleased(uint256 indexed jobId, address indexed provider, uint256 amount);

        /// Spec: "Refunded(jobId, client, amount)"
        event Refunded(uint256 indexed jobId, address indexed client, uint256 amount);

        /// Create a new job in `Open` state.
        ///
        /// Called by client. `provider` MAY be `address(0)` for deferred assignment.
        /// SHALL revert if `evaluator` is zero or `expiredAt` is not in the future.
        /// `hook` MAY be `address(0)` (no hook). Returns `jobId`.
        function createJob(
            address provider,
            address evaluator,
            uint256 expiredAt,
            string calldata description,
            address hook
        ) external returns (uint256 jobId);

        /// Assign a provider to an Open job.
        ///
        /// Called by client only. SHALL revert if job is not Open, current
        /// `job.provider != address(0)`, or `provider == address(0)`.
        function setProvider(
            uint256 jobId,
            address provider,
            bytes calldata optParams
        ) external;

        /// Set or update the budget for an Open job.
        ///
        /// Called by client or provider. SHALL revert if job is not Open.
        function setBudget(
            uint256 jobId,
            uint256 amount,
            bytes calldata optParams
        ) external;

        /// Fund the job escrow, transitioning Open → Funded.
        ///
        /// Called by client. SHALL revert if job is not Open, budget is zero,
        /// provider is not set, or `job.budget != expectedBudget`.
        function fund(
            uint256 jobId,
            uint256 expectedBudget,
            bytes calldata optParams
        ) external;

        /// Submit work deliverable, transitioning Funded → Submitted.
        ///
        /// Called by provider only. `deliverable` is a `bytes32` reference
        /// (e.g. IPFS CID hash, attestation commitment).
        function submit(
            uint256 jobId,
            bytes32 deliverable,
            bytes calldata optParams
        ) external;

        /// Mark the job as completed, releasing escrow to provider.
        ///
        /// Called by evaluator only when job is Submitted. `reason` MAY be
        /// `bytes32(0)` or an attestation hash for audit/reputation composition.
        function complete(
            uint256 jobId,
            bytes32 reason,
            bytes calldata optParams
        ) external;

        /// Reject the job, refunding escrow to client.
        ///
        /// Open: client only. Funded/Submitted: evaluator only.
        function reject(
            uint256 jobId,
            bytes32 reason,
            bytes calldata optParams
        ) external;

        /// Claim refund after expiry (anyone may call).
        ///
        /// NOT hookable — ensures refunds cannot be blocked by malicious hooks.
        /// Transitions job to Expired and refunds client.
        function claimRefund(uint256 jobId) external;

        /// Read job data by ID (implementation-specific, not mandated by spec).
        function getJob(uint256 jobId) external view returns (Job memory);
    }
}

sol! {
    /// `IACPHook` — the only normative Solidity interface in EIP-8183.
    ///
    /// Hooks are called before and after core functions (except `claimRefund`).
    /// The `selector` identifies which core function triggered the hook,
    /// and `data` carries ABI-encoded arguments specific to each action.
    ///
    /// Data encoding per action:
    /// - `setProvider`: `abi.encode(address provider, bytes optParams)`
    /// - `setBudget`: `abi.encode(uint256 amount, bytes optParams)`
    /// - `fund`: `optParams` (raw bytes)
    /// - `submit`: `abi.encode(bytes32 deliverable, bytes optParams)`
    /// - `complete`: `abi.encode(bytes32 reason, bytes optParams)`
    /// - `reject`: `abi.encode(bytes32 reason, bytes optParams)`
    #[allow(missing_docs)]
    #[sol(rpc)]
    interface IACPHook {
        function beforeAction(
            uint256 jobId,
            bytes4 selector,
            bytes calldata data
        ) external;

        function afterAction(
            uint256 jobId,
            bytes4 selector,
            bytes calldata data
        ) external;
    }
}
