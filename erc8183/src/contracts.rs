//! Contract bindings generated via inline Solidity interfaces.
//!
//! These interfaces are derived from the [ERC-8183](https://eips.ethereum.org/EIPS/eip-8183)
//! specification (Agentic Commerce Protocol).
//!
//! **Important**: ERC-8183 is currently a Draft EIP. The specification defines
//! the core functions and events but does NOT mandate specific view functions
//! or struct layouts. Implementations MAY vary in their auxiliary APIs.
//!
//! Using inline Solidity is the alloy-recommended best practice as it
//! preserves full type information (visibility, struct names, etc.) that
//! JSON ABI files omit.

use alloy::sol;

sol! {
    /// ERC-8183 Agentic Commerce Protocol — job escrow with evaluator attestation.
    ///
    /// The core contract manages job lifecycle (Open → Funded → Submitted → Terminal),
    /// ERC-20 escrow, and optional hook callbacks.
    ///
    /// **Note**: The `Job` struct and view functions below are common implementation
    /// patterns but are NOT mandated by the EIP-8183 specification. The specification
    /// only requires the core mutating functions and events.
    #[allow(missing_docs, clippy::too_many_arguments)]
    #[sol(rpc)]
    contract AgenticCommerce {
        /// Job data structure. Field order and presence may vary by implementation.
        /// The EIP-8183 spec requires at minimum: client, provider, evaluator,
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

        /// Emitted when a new job is created via `createJob`.
        event JobCreated(
            uint256 indexed jobId,
            address indexed client,
            address provider,
            address evaluator,
            uint256 expiredAt,
            string description,
            address hook
        );

        /// Emitted when a provider is assigned via `setProvider`.
        event ProviderSet(uint256 indexed jobId, address indexed provider);

        /// Emitted when the budget is set via `setBudget`.
        event BudgetSet(uint256 indexed jobId, uint256 amount);

        /// Emitted when the job is funded via `fund`.
        event JobFunded(uint256 indexed jobId, uint256 amount);

        /// Emitted when work is submitted via `submit`. Includes the deliverable hash.
        event JobSubmitted(uint256 indexed jobId, bytes32 deliverable);

        /// Emitted when the job is completed via `complete`.
        event JobCompleted(uint256 indexed jobId, bytes32 reason);

        /// Emitted when the job is rejected via `reject`.
        event JobRejected(uint256 indexed jobId, bytes32 reason);

        /// Emitted when the job expires and refund is claimed via `claimRefund`.
        event JobExpired(uint256 indexed jobId);

        /// Create a new job in `Open` state.
        ///
        /// Called by client. `provider` MAY be `address(0)` for deferred assignment.
        /// SHALL revert if `evaluator` is zero or `expiredAt` is not in the future.
        /// `hook` MAY be `address(0)` (no hook).
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
        /// Called by evaluator only when job is Submitted. `reason` is an
        /// optional attestation hash for audit/reputation composition.
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

        /// Read job data by ID.
        ///
        /// **Note**: This view function is NOT mandated by EIP-8183 but is a
        /// common implementation pattern. Implementations MAY expose job data
        /// via different APIs.
        function getJob(uint256 jobId) external view returns (Job memory);
    }
}

sol! {
    /// `IACPHook` — Optional hook interface for extending the Agentic Commerce Protocol.
    ///
    /// Hooks are called before and after core functions (except `claimRefund`).
    /// The `selector` identifies which core function triggered the hook,
    /// and `data` carries ABI-encoded arguments specific to each action.
    ///
    /// Data encoding per action:
    /// - `setProvider`: `abi.encode(address provider, bytes optParams)`
    /// - `setBudget`: `abi.encode(uint256 amount, bytes optParams)`
    /// - `fund`: `optParams` only
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
