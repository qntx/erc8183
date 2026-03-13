# ERC-8183: Agentic Commerce Protocol — Rust SDK

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

A type-safe, ergonomic Rust SDK for the [ERC-8183 Agentic Commerce Protocol](https://eips.ethereum.org/EIPS/eip-8183) — the **commerce layer for AI agents**.

ERC-8183 defines a minimal on-chain protocol: a **job** with escrowed budget, six states (`Open → Funded → Submitted → Completed | Rejected | Expired`), and an **evaluator** who attests completion or rejection.

## Features

- **Type-safe** — Solidity interfaces compiled via `alloy::sol!` with full struct/event type information
- **Provider-agnostic** — Generic over any alloy `Provider` (HTTP, WebSocket, IPC)
- **Complete coverage** — All core functions: `createJob`, `setProvider`, `setBudget`, `fund`, `submit`, `complete`, `reject`, `claimRefund`
- **Domain types** — Ergonomic Rust types for `JobStatus`, `Job`, `CreateJobParams`, etc.
- **Hook support** — `IACPHook` interface bindings for extending the protocol
- **2024 edition** — Rust 2024 edition with strict clippy lints

## Quick Start

```rust
use alloy::primitives::{Address, U256};
use alloy::providers::ProviderBuilder;
use erc8183::{Erc8183, types::CreateJobParams};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = ProviderBuilder::new()
        .connect_http("https://eth.llamarpc.com".parse()?);

    let client = Erc8183::new(provider)
        .with_address("0x1234...".parse()?);

    let job = client.job()?;

    // Query contract
    let version = job.get_version().await?;
    println!("Version: {version}");

    // Create a job
    let params = CreateJobParams::new(
        Address::ZERO,                  // deferred provider
        "0xEval...".parse()?,           // evaluator
        U256::from(1_900_000_000u64),   // expiredAt
        "Build a REST API",             // description
    );
    let job_id = job.create_job(&params).await?;

    // Fund → Submit → Complete lifecycle
    job.set_provider(job_id, "0xProv...".parse()?, None).await?;
    job.set_budget(job_id, U256::from(1000u64), None).await?;
    job.fund(job_id, U256::from(1000u64), None).await?;

    Ok(())
}
```

## Architecture

```text
┌─────────────────────────────────────────────────────┐
│                    Erc8183<P>                        │
│  Top-level client, generic over alloy Provider       │
│                                                      │
│  ┌──────────────────────────────────────────────┐   │
│  │              JobHandle<P>                     │   │
│  │  create_job · set_provider · set_budget       │   │
│  │  fund · submit · complete · reject            │   │
│  │  claim_refund · get_job · fee_bps · ...       │   │
│  └──────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────┤
│  contracts   │ alloy sol! bindings                   │
│  types       │ JobStatus, Job, CreateJobParams, ...  │
│  error       │ Error enum with thiserror             │
│  networks    │ Pre-configured deployment addresses   │
└─────────────────────────────────────────────────────┘
```

## Workspace

| Crate | Description |
|---|---|
| `erc8183` | Core SDK library |
| `erc8183-cli` | CLI tool (planned) |

## ERC-8183 State Machine

```text
Open ──→ Funded ──→ Submitted ──→ Completed
  │         │           │
  └→ Rejected  └→ Rejected  └→ Rejected
               └→ Expired   └→ Expired
```

| Role | Capabilities |
|---|---|
| **Client** | Creates job, sets provider/budget, funds escrow, rejects (Open only) |
| **Provider** | Proposes budget, submits work |
| **Evaluator** | Completes or rejects (Funded/Submitted) |

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
