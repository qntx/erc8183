//! Contract bindings generated via inline Solidity interfaces.
//!
//! These bindings match the `AgenticCommerce` contract (ERC-8183 implementation)
//! deployed from `src/AgenticCommerce.sol`. The ABI-encoded struct layouts,
//! event signatures, and error selectors **must** match the deployed contract
//! exactly for encoding/decoding to succeed.
//!
//! The `IACPHook` binding matches the normative interface from the ERC-8183 spec.

use alloy::sol;

sol! {
    /// ERC-8183 Agentic Commerce Protocol — job escrow with evaluator attestation.
    ///
    /// Single ERC-20 payment token per contract. Optional hooks for extensibility.
    /// `claimRefund` is deliberately NOT hookable per spec.
    /// WARNING: Fee-on-transfer / rebasing tokens are NOT supported.
    #[allow(missing_docs, clippy::too_many_arguments)]
    #[sol(rpc)]
    contract AgenticCommerce {
        /// Minimal job descriptor returned by `getJob()`.
        /// Field order must match `IERC8183.Job` exactly for ABI compatibility.
        struct Job {
            uint256 id;
            address client;
            address provider;
            address evaluator;
            string description;
            uint256 budget;
            uint256 expiredAt;
            uint8 status;
            address hook;
            bytes32 deliverable;
        }

        // IERC8183 events
        event JobCreated(uint256 indexed jobId, address indexed client, address indexed provider, address evaluator, uint256 expiredAt, address hook);
        event ProviderSet(uint256 indexed jobId, address indexed provider);
        event BudgetSet(uint256 indexed jobId, uint256 amount);
        event JobFunded(uint256 indexed jobId, address indexed client, uint256 amount);
        event JobSubmitted(uint256 indexed jobId, address indexed provider, bytes32 deliverable);
        event JobCompleted(uint256 indexed jobId, address indexed evaluator, bytes32 reason);
        event JobRejected(uint256 indexed jobId, address indexed rejector, bytes32 reason);
        event JobExpired(uint256 indexed jobId);
        event PaymentReleased(uint256 indexed jobId, address indexed provider, uint256 amount);
        event Refunded(uint256 indexed jobId, address indexed client, uint256 amount);

        // Contract-specific events
        event EvaluatorFeePaid(uint256 indexed jobId, address indexed evaluator, uint256 amount);
        event HookWhitelistUpdated(address indexed hook, bool status);
        event PlatformFeeUpdated(uint256 oldFeeBp, uint256 newFeeBp);
        event EvaluatorFeeUpdated(uint256 oldFeeBp, uint256 newFeeBp);
        event TreasuryUpdated(address oldTreasury, address newTreasury);
        event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
        event OwnershipTransferStarted(address indexed previousOwner, address indexed newOwner);

        // Custom errors
        error ZeroAddress();
        error InvalidExpiry();
        error InvalidStatus(uint8 current);
        error Unauthorized();
        error ProviderAlreadySet();
        error ProviderNotSet();
        error BudgetMismatch(uint256 actual, uint256 expected);
        error ZeroBudget();
        error JobAlreadyExpired();
        error JobNotExpired();
        error FeeTooHigh();
        error JobDoesNotExist();
        error HookNotWhitelisted();
        error HookInterfaceNotSupported();
        error DescriptionTooLong();
        error OwnableUnauthorizedAccount(address account);
        error OwnableInvalidOwner(address owner);

        // Core lifecycle functions (IERC8183)
        function createJob(address provider, address evaluator, uint256 expiredAt, string calldata description, address hook) external returns (uint256 jobId);
        function setProvider(uint256 jobId, address provider, bytes calldata optParams) external;
        function setBudget(uint256 jobId, uint256 amount, bytes calldata optParams) external;
        function fund(uint256 jobId, uint256 expectedBudget, bytes calldata optParams) external;
        function submit(uint256 jobId, bytes32 deliverable, bytes calldata optParams) external;
        function complete(uint256 jobId, bytes32 reason, bytes calldata optParams) external;
        function reject(uint256 jobId, bytes32 reason, bytes calldata optParams) external;
        function claimRefund(uint256 jobId) external;

        // View functions
        function getJob(uint256 jobId) external view returns (Job memory);
        function totalJobs() external view returns (uint256);
        function PAYMENT_TOKEN() external view returns (address);
        function platformFeeBp() external view returns (uint256);
        function evaluatorFeeBp() external view returns (uint256);
        function treasury() external view returns (address);
        function jobCounter() external view returns (uint256);
        function whitelistedHooks(address hook) external view returns (bool);
        function owner() external view returns (address);
        function pendingOwner() external view returns (address);
        function supportsInterface(bytes4 interfaceId) external pure returns (bool);
        function MAX_FEE_BP() external pure returns (uint256);
        function BP_DENOMINATOR() external pure returns (uint256);
        function HOOK_GAS_LIMIT() external pure returns (uint256);
        function MIN_EXPIRY_DURATION() external pure returns (uint256);
        function MAX_DESCRIPTION_LENGTH() external pure returns (uint256);

        // Admin functions (onlyOwner)
        function setPlatformFee(uint256 newFeeBp) external;
        function setEvaluatorFee(uint256 newFeeBp) external;
        function setTreasury(address newTreasury) external;
        function setHookWhitelist(address hook, bool status) external;
        function transferOwnership(address newOwner) external;
        function acceptOwnership() external;
        function renounceOwnership() external;
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
        function beforeAction(uint256 jobId, bytes4 selector, bytes calldata data) external;
        function afterAction(uint256 jobId, bytes4 selector, bytes calldata data) external;
    }
}
