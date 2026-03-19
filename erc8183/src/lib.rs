//! # ERC-8183: Agentic Commerce Protocol Rust SDK
//!
//! A type-safe, ergonomic Rust SDK for interacting with
//! [ERC-8183](https://eips.ethereum.org/EIPS/eip-8183) on-chain contracts.
//!
//! ERC-8183 defines the **Agentic Commerce Protocol**: a **job** with escrowed
//! budget, six states (`Open → Funded → Submitted → Completed | Rejected | Expired`),
//! and an **evaluator** who alone may mark the job completed. The client funds
//! the job; the provider submits work; the evaluator attests completion or
//! rejection.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use alloy::primitives::{Address, U256};
//! use alloy::providers::ProviderBuilder;
//! use erc8183::{Erc8183, Network, types::CreateJobParams};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // 1. Connect using the built-in RPC URL
//! let network = Network::MonadMainnet;
//! let provider = ProviderBuilder::new()
//!     .connect_http(network.rpc_url().parse()?);
//!
//! // 2. Create client and get a job handle
//! let client = Erc8183::new(provider).with_network(network);
//! let job = client.job()?;
//!
//! // 3. Create a job (requires signer-enabled provider)
//! let params = CreateJobParams::new(
//!     Address::ZERO,                // deferred provider
//!     "0xEvaluator...".parse()?,    // evaluator
//!     U256::from(1_700_000_000u64), // expiredAt
//!     "Build a REST API",           // description
//! );
//! let job_id = job.create_job(&params).await?;
//!
//! // 4. Query job data
//! let data = job.get_job(job_id).await?;
//! println!("{data}"); // "Job #1 [Open] budget=0 client=0x... provider=0x..."
//! # Ok(())
//! # }
//! ```
//!
//! ## Architecture
//!
//! The SDK is designed around the alloy provider abstraction:
//!
//! - **[`Erc8183`]** — The top-level client, generic over `P: Provider`.
//!   Accepts any alloy provider the user has already configured.
//!   Use [`with_network`](Erc8183::with_network) for built-in deployments or
//!   [`with_address`](Erc8183::with_address) for custom contracts.
//! - **[`JobHandle`](job::JobHandle)** — Job lifecycle + view + admin.
//!   Core operations use the standard [`IERC8183`](contracts::IERC8183) binding
//!   (portable); view/admin use [`AgenticCommerce`](contracts::AgenticCommerce).
//! - **[`Network`]** — Pre-configured addresses for live deployments.
//! - **[`types`]** — Domain types: [`JobStatus`](types::JobStatus),
//!   [`Job`](types::Job), [`CreateJobParams`](types::CreateJobParams), etc.
//! - **[`contracts`]** — Three-layer `sol!` bindings:
//!   [`IERC8183`](contracts::IERC8183) (standard, portable),
//!   [`AgenticCommerce`](contracts::AgenticCommerce) (QNTX implementation),
//!   [`IACPHook`](contracts::IACPHook) (hook interface).
//! - **[`hooks`]** — Hook selector constants matching `BaseACPHook`.

pub mod client;
pub mod contracts;
pub mod error;
pub mod hooks;
pub mod job;
pub mod networks;
pub mod types;

// Re-export primary public API at crate root.
pub use client::Erc8183;
pub use error::{Error, Result};
pub use networks::Network;
