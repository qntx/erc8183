//! Basic usage example for the ERC-8183 Agentic Commerce Protocol SDK.
//!
//! This example demonstrates how to:
//! 1. Create a client connected to a local Anvil node.
//! 2. Query the contract version.
//! 3. Create a job with deferred provider assignment.
//!
//! Run with:
//! ```sh
//! cargo run --example basic
//! ```

use alloy::{
    primitives::{Address, U256},
    providers::ProviderBuilder,
};
use erc8183::{Erc8183, types::CreateJobParams};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Connect to a local Anvil node (or any EVM-compatible RPC).
    let provider = ProviderBuilder::new()
        .connect_http("http://127.0.0.1:8545".parse()?);

    // 2. Create an ERC-8183 client with a custom contract address.
    //    Replace with the actual deployed address.
    let contract_addr: Address =
        "0x5FbDB2315678afecb367f032d93F642f64180aa3".parse()?;

    let client = Erc8183::new(provider).with_address(contract_addr);

    // 3. Get a job handle and query contract info.
    let job = client.job()?;

    match job.get_version().await {
        Ok(version) => println!("Contract version: {version}"),
        Err(e) => eprintln!("Failed to get version: {e}"),
    }

    // 4. Create a job (requires a signer-enabled provider in practice).
    let evaluator: Address =
        "0x70997970C51812dc3A010C7d01b50e0d17dc79C8".parse()?;

    let params = CreateJobParams::new(
        Address::ZERO,                       // deferred provider
        evaluator,                           // evaluator
        U256::from(1_900_000_000u64),        // expiredAt (far future)
        "Implement a REST API for payments", // description
    );

    println!("Job params: {params:?}");

    // Uncomment below if the contract is actually deployed:
    // let job_id = job.create_job(&params).await?;
    // println!("Created job: {job_id}");

    Ok(())
}
