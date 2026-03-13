# erc8183

[![CI][ci-badge]][ci-url]
[![crates.io][crate-badge]][crate-url]
[![docs.rs][doc-badge]][doc-url]
[![License][license-badge]][license-url]
[![Rust][rust-badge]][rust-url]

[ci-badge]: https://github.com/qntx/erc8183/actions/workflows/rust.yml/badge.svg
[ci-url]: https://github.com/qntx/erc8183/actions/workflows/rust.yml
[crate-badge]: https://img.shields.io/crates/v/erc8183.svg
[crate-url]: https://crates.io/crates/erc8183
[doc-badge]: https://img.shields.io/docsrs/erc8183.svg
[doc-url]: https://docs.rs/erc8183
[license-badge]: https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg
[license-url]: LICENSE-MIT
[rust-badge]: https://img.shields.io/badge/rust-edition%202024-orange.svg
[rust-url]: https://doc.rust-lang.org/edition-guide/

**Type-safe Rust SDK for the [ERC-8183](https://eips.ethereum.org/EIPS/eip-8183) Agentic Commerce Protocol — on-chain job escrow with evaluator attestation for AI agent commerce.**

ERC-8183 enables **trustless commerce between AI agents**: a client locks funds in escrow, a provider submits work, and an evaluator attests completion or rejection. This SDK provides ergonomic, alloy-native bindings for the full job lifecycle with strict type safety and comprehensive documentation.

> **Note**: ERC-8183 is currently a **Draft** EIP with no official contract deployments yet. Once deployed, update the contract address and the SDK is ready to use.

See [SECURITY.md](SECURITY.md) before using in production.

## Quick Start

### Create a Job (Write)

```rust
use alloy::{
    network::EthereumWallet,
    primitives::{Address, U256},
    providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
};
use erc8183::{Erc8183, types::CreateJobParams};

let signer: PrivateKeySigner = std::env::var("PRIVATE_KEY")?.parse()?;
let wallet = EthereumWallet::from(signer);

let provider = ProviderBuilder::new()
    .wallet(wallet)
    .connect_http("https://eth.llamarpc.com".parse()?);

// Replace with actual deployed contract address
let client = Erc8183::new(provider)
    .with_address("0x1234...".parse()?);

let job_handle = client.job()?;

// Create a job with deferred provider assignment
let params = CreateJobParams::new(
    Address::ZERO,                  // provider (deferred)
    "0xEvaluator...".parse()?,      // evaluator
    U256::from(1_900_000_000u64),   // expiredAt (Unix timestamp)
    "Build a REST API for payments",
);

let job_id = job_handle.create_job(&params).await?;
println!("Created job: {job_id}");
```

### Full Job Lifecycle

```rust
// 1. Client creates job (see above)
let job_id = job_handle.create_job(&params).await?;

// 2. Client assigns provider
job_handle.set_provider(job_id, provider_address, None).await?;

// 3. Client or provider sets budget
job_handle.set_budget(job_id, U256::from(1000), None).await?;

// 4. Client funds escrow (requires ERC-20 approval)
job_handle.fund(job_id, U256::from(1000), None).await?;

// 5. Provider submits work
let deliverable = FixedBytes::from_slice(&ipfs_cid_hash);
job_handle.submit(job_id, deliverable, None).await?;

// 6. Evaluator completes (releases escrow to provider)
job_handle.complete(job_id, FixedBytes::ZERO, None).await?;

// Or: Evaluator rejects (refunds client)
// job_handle.reject(job_id, reason_hash, None).await?;
```

### Query Job Data (Read-Only)

```rust
use alloy::providers::ProviderBuilder;
use erc8183::Erc8183;

let provider = ProviderBuilder::new()
    .connect_http("https://eth.llamarpc.com".parse()?);

let client = Erc8183::new(provider)
    .with_address("0x1234...".parse()?);

let job = client.job()?.get_job(U256::from(1)).await?;
println!("Status: {}", job.status);
println!("Client: {}", job.client);
println!("Provider: {}", job.provider);
println!("Budget: {}", job.budget);
```

## Architecture

| Module | Description |
| --- | --- |
| **[`Erc8183`](erc8183/src/client.rs)** | Top-level client — generic over `P: Provider`, builder pattern for address configuration |
| **[`JobHandle`](erc8183/src/job.rs)** | Job operations — `create_job`, `set_provider`, `set_budget`, `fund`, `submit`, `complete`, `reject`, `claim_refund` |
| **[`types`](erc8183/src/types.rs)** | Domain types — `JobStatus`, `Job`, `CreateJobParams`, `SubmitParams`, `AttestParams` |
| **[`contracts`](erc8183/src/contracts.rs)** | Inline Solidity bindings (`sol!` macro) — `AgenticCommerce` contract and `IACPHook` interface |
| **[`error`](erc8183/src/error.rs)** | Error types — `Error` enum with `thiserror`, covers contract/transport/status errors |
| **[`networks`](erc8183/src/networks.rs)** | Network configuration — placeholder addresses for future deployments |

## ERC-8183 Protocol

### State Machine

```text
Open ──────► Funded ──────► Submitted ──────► Completed
  │            │               │
  ▼            ▼               ▼
Rejected    Rejected        Rejected
            Expired         Expired
```

| State | Description |
| --- | --- |
| **Open** | Created; budget not yet set or not yet funded |
| **Funded** | Budget escrowed; provider may submit work |
| **Submitted** | Work submitted; evaluator may complete or reject |
| **Completed** | Terminal; escrow released to provider |
| **Rejected** | Terminal; escrow refunded to client |
| **Expired** | Terminal; refund after `expiredAt` timestamp |

### Roles

| Role | Capabilities |
| --- | --- |
| **Client** | Creates job, sets provider/budget, funds escrow, rejects (Open only) |
| **Provider** | Proposes budget, submits work deliverable |
| **Evaluator** | Completes or rejects (Funded/Submitted states) |

### Core Functions (EIP-8183 Spec)

| Function | Caller | Description |
| --- | --- | --- |
| `createJob` | Client | Create job in Open state |
| `setProvider` | Client | Assign provider to Open job |
| `setBudget` | Client/Provider | Set or negotiate budget |
| `fund` | Client | Fund escrow, transition to Funded |
| `submit` | Provider | Submit work, transition to Submitted |
| `complete` | Evaluator | Release escrow to provider |
| `reject` | Client/Evaluator | Refund escrow to client |
| `claimRefund` | Anyone | Refund after expiry (not hookable) |

### Events (EIP-8183 Recommended Minimum)

| Event | Parameters |
| --- | --- |
| `JobCreated` | jobId, client, provider, evaluator, expiredAt |
| `ProviderSet` | jobId, provider |
| `BudgetSet` | jobId, amount |
| `JobFunded` | jobId, client, amount |
| `JobSubmitted` | jobId, provider, deliverable |
| `JobCompleted` | jobId, evaluator, reason |
| `JobRejected` | jobId, rejector, reason |
| `JobExpired` | jobId |
| `PaymentReleased` | jobId, provider, amount |
| `Refunded` | jobId, client, amount |

### Hooks (Optional)

The `IACPHook` interface — the only normative Solidity interface in EIP-8183 — allows extending the protocol with custom logic:

```solidity
interface IACPHook {
    function beforeAction(uint256 jobId, bytes4 selector, bytes calldata data) external;
    function afterAction(uint256 jobId, bytes4 selector, bytes calldata data) external;
}
```

Hooks are called before and after core functions (except `claimRefund`). Use cases include:

- Bidding/auction mechanisms
- Two-phase token transfers
- Reputation integration ([ERC-8004](https://eips.ethereum.org/EIPS/eip-8004))
- Custom validation logic

## Design

- **Zero `async_trait`** — pure RPITIT, no trait-object overhead
- **Inline Solidity bindings** — `sol!` macro preserves struct names, enums, and visibility; no JSON ABI files
- **Provider-generic** — works with any alloy transport (HTTP, WebSocket, IPC) and any signer configuration
- **Strict linting** — `pedantic` + `nursery` + `correctness` (deny), see [`clippy.toml`](clippy.toml)
- **Spec-compliant** — all 8 core functions and 10 recommended events strictly match EIP-8183 specification

## Workspace

| Crate | Description |
| --- | --- |
| `erc8183` | Core SDK library |
| `erc8183-cli` | CLI tool (planned) |

## Examples

| Example | Description |
| --- | --- |
| [`basic`](erc8183/examples/basic.rs) | Basic client setup and job parameter preparation |

```bash
cargo run --example basic
```

## Related Standards

| Standard | Relationship |
| --- | --- |
| [ERC-8004](https://eips.ethereum.org/EIPS/eip-8004) | Trustless Agents — reputation/identity layer, composable with ERC-8183 |
| [ERC-20](https://eips.ethereum.org/EIPS/eip-20) | Payment token standard used for escrow |
| [ERC-2771](https://eips.ethereum.org/EIPS/eip-2771) | Meta-transactions — optional gasless execution support |

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project shall be dual-licensed as above, without any additional terms or conditions.

---

<div align="center">

A **[QNTX](https://qntx.fun)** open-source project.

<a href="https://qntx.fun"><img alt="QNTX" width="369" src="https://raw.githubusercontent.com/qntx/.github/main/profile/qntx-banner.svg" /></a>

<!--prettier-ignore-->
Code is law. We write both.

</div>
