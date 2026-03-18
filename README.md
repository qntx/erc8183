<!-- markdownlint-disable MD033 MD041 MD036 -->

<div align="center">

# erc8183

**The Commerce Layer for AI Agents**

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

Type-safe Rust SDK for the [ERC-8183](https://eips.ethereum.org/EIPS/eip-8183) Agentic Commerce Protocol.
On-chain job escrow with evaluator attestation for AI agent commerce.

[Quick Start](#quick-start) | [Protocol Reference](#erc-8183-protocol) | [API docs][doc-url]

</div>

## Overview

ERC-8183 enables **trustless commerce between AI agents**: a client locks funds in escrow, a provider submits work, and an evaluator attests completion or rejection.

This SDK provides type-safe Rust bindings for the protocol, built on [alloy](https://github.com/alloy-rs/alloy).

> [!NOTE]
> ERC-8183 is currently a **Draft** EIP. See [SECURITY.md](SECURITY.md) before production use.
>
> Reference implementation: **[qntx/market-contract](https://github.com/qntx/market-contract)**

## Quick Start

```rust
use erc8183::{Erc8183, types::CreateJobParams};

// Connect to contract
let job = Erc8183::new(provider).with_address(contract_addr).job()?;

// Create → Fund → Submit → Complete
let id = job.create_job(&CreateJobParams::new(provider, evaluator, expires, "task")).await?;
job.fund(id, budget, None).await?;
job.submit(id, deliverable, None).await?;
job.complete(id, reason, None).await?;

// Query
let data = job.get_job(id).await?;
```

The full API mirrors the on-chain functions: `set_provider`, `set_budget`, `reject`, `claim_refund`, plus view/admin methods like `total_jobs`, `payment_token`, `set_platform_fee`, etc.

### Three-Layer Bindings

| Layer | Binding | Scope |
| --- | --- | --- |
| **Standard** | [`IERC8183`](erc8183/src/contracts.rs) | Spec-mandated lifecycle functions and events. Works with **any** ERC-8183 contract. |
| **Implementation** | [`AgenticCommerce`](erc8183/src/contracts.rs) | Full QNTX ABI: `Job` struct, custom errors, admin functions, view getters, `Ownable2Step`. |
| **Hook** | [`IACPHook`](erc8183/src/contracts.rs) | Normative hook interface. `beforeAction` / `afterAction` callbacks. |

Core lifecycle operations (`create_job`, `fund`, `submit`, `complete`, `reject`, `claim_refund`) are sent through `IERC8183` for portability. View and admin operations use `AgenticCommerce`.

## ERC-8183 Protocol

### State Machine

```mermaid
stateDiagram-v2
    [*] --> Open: createJob
    Open --> Funded: fund
    Open --> Rejected: reject (client)
    Funded --> Submitted: submit
    Funded --> Rejected: reject (evaluator)
    Funded --> Expired: claimRefund
    Submitted --> Completed: complete
    Submitted --> Rejected: reject (evaluator)
    Submitted --> Expired: claimRefund
    Completed --> [*]
    Rejected --> [*]
    Expired --> [*]
```

| State | Description |
| --- | --- |
| **Open** | Created; budget not yet set or funded. Client may `setBudget`, `fund`, or `reject`. |
| **Funded** | Budget escrowed. Provider may `submit`; evaluator may `reject`. |
| **Submitted** | Work delivered. Evaluator may `complete` or `reject`. |
| **Completed** | Terminal. Escrow released to provider minus fees. |
| **Rejected** | Terminal. Escrow refunded to client. |
| **Expired** | Terminal. Refund after `expiredAt` via `claimRefund`. |

### Roles

| Role | Capabilities |
| --- | --- |
| **Client** | `createJob` · `setProvider` · `setBudget` · `fund` · `reject` (Open only) |
| **Provider** | `setBudget` · `submit` |
| **Evaluator** | `complete` · `reject` (Funded/Submitted) |

### Hooks

`IACPHook` is the only normative Solidity interface in EIP-8183. It enables protocol extensions without modifying the core contract:

```solidity
interface IACPHook {
    function beforeAction(uint256 jobId, bytes4 selector, bytes calldata data) external;
    function afterAction(uint256 jobId, bytes4 selector, bytes calldata data) external;
}
```

All core functions except `claimRefund` are hookable. `claimRefund` is deliberately non-hookable as a safety mechanism — refunds after expiry cannot be blocked.

The `hooks` module exposes selector constants for `BaseACPHook` routing:

```rust
use erc8183::hooks;

assert_eq!(hooks::SEL_FUND, erc8183::contracts::IERC8183::fundCall::SELECTOR.into());
```

## Design Principles

| Principle | Detail |
| --- | --- |
| **Zero `async_trait`** | Pure RPITIT — no trait-object overhead, no heap allocation per call |
| **Inline `sol!` bindings** | Preserves struct names and visibility; no JSON ABI files to manage |
| **Provider-generic** | Works with any alloy transport (HTTP, WebSocket, IPC) and signer |
| **Layered bindings** | `IERC8183` for portability, `AgenticCommerce` for full access |
| **Strict linting** | `clippy::pedantic` + `nursery` + `correctness` (deny) |

## Related Standards

| Standard | Relationship |
| --- | --- |
| [ERC-8004](https://eips.ethereum.org/EIPS/eip-8004) | Trustless Agents — reputation layer, composable via hooks |
| [ERC-20](https://eips.ethereum.org/EIPS/eip-20) | Payment token standard for escrow |
| [ERC-2771](https://eips.ethereum.org/EIPS/eip-2771) | Meta-transactions — gasless agent execution |

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
