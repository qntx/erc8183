//! Contract bindings generated via inline Solidity interfaces.
//!
//! These interfaces are derived from the [ERC-8183](https://eips.ethereum.org/EIPS/eip-8183)
//! specification (Agentic Commerce Protocol).
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
    #[allow(missing_docs, clippy::too_many_arguments)]
    #[sol(rpc)]
    contract AgenticCommerce {
        // ── Structs ──────────────────────────────────────────────────────

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

        // ── Events ───────────────────────────────────────────────────────

        event JobCreated(
            uint256 indexed jobId,
            address indexed client,
            address provider,
            address evaluator,
            uint256 expiredAt,
            string description,
            address hook
        );
        event ProviderSet(uint256 indexed jobId, address indexed provider);
        event BudgetSet(uint256 indexed jobId, uint256 amount);
        event JobFunded(uint256 indexed jobId, uint256 amount);
        event JobSubmitted(uint256 indexed jobId, bytes32 deliverable);
        event JobCompleted(uint256 indexed jobId, bytes32 reason);
        event JobRejected(uint256 indexed jobId, bytes32 reason);
        event JobExpired(uint256 indexed jobId);

        // ── Core Functions ───────────────────────────────────────────────

        /// Create a new job in `Open` state. `provider` MAY be `address(0)`.
        function createJob(
            address provider,
            address evaluator,
            uint256 expiredAt,
            string calldata description,
            address hook
        ) external returns (uint256 jobId);

        /// Assign a provider to an Open job (client only).
        function setProvider(
            uint256 jobId,
            address provider,
            bytes calldata optParams
        ) external;

        /// Set or update the budget for an Open job (client or provider).
        function setBudget(
            uint256 jobId,
            uint256 amount,
            bytes calldata optParams
        ) external;

        /// Fund the job escrow, transitioning Open → Funded (client only).
        function fund(
            uint256 jobId,
            uint256 expectedBudget,
            bytes calldata optParams
        ) external;

        /// Submit work deliverable, transitioning Funded → Submitted (provider only).
        function submit(
            uint256 jobId,
            bytes32 deliverable,
            bytes calldata optParams
        ) external;

        /// Mark the job as completed, releasing escrow to provider (evaluator only).
        function complete(
            uint256 jobId,
            bytes32 reason,
            bytes calldata optParams
        ) external;

        /// Reject the job, refunding escrow to client.
        /// Open: client only. Funded/Submitted: evaluator only.
        function reject(
            uint256 jobId,
            bytes32 reason,
            bytes calldata optParams
        ) external;

        /// Claim refund after expiry (anyone may call).
        /// NOT hookable — ensures refunds cannot be blocked.
        function claimRefund(uint256 jobId) external;

        // ── View Functions ───────────────────────────────────────────────

        /// Read job data by ID.
        function getJob(uint256 jobId) external view returns (Job memory);

        /// The ERC-20 payment token address.
        function paymentToken() external view returns (address);

        /// Platform fee in basis points (e.g. 100 = 1%).
        function feeBps() external view returns (uint256);

        /// Treasury address that receives platform fees.
        function treasury() external view returns (address);

        /// Total number of jobs created.
        function jobCount() external view returns (uint256);

        /// Contract version string.
        function getVersion() external pure returns (string);
    }
}

sol! {
    /// Optional hook interface for extending the Agentic Commerce Protocol.
    ///
    /// Hooks are called before and after core functions (except `claimRefund`).
    /// The `selector` identifies which core function triggered the hook,
    /// and `data` carries ABI-encoded arguments specific to each action.
    #[allow(missing_docs)]
    #[sol(rpc)]
    contract IACPHook {
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
