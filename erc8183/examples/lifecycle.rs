//! Complete lifecycle on Monad Mainnet: `Open → Funded → Submitted → Completed`.
//!
//! ```sh
//! export RPC_URL=https://rpc.monad.xyz
//! export CLIENT_KEY=0xac0974...
//! export PROVIDER_KEY=0x59c699...
//! export EVALUATOR_KEY=0x5de411...
//! cargo run --example lifecycle
//! ```

use std::env;

use alloy::{
    network::EthereumWallet,
    primitives::{Address, FixedBytes, U256},
    providers::{Provider, ProviderBuilder},
    signers::local::PrivateKeySigner,
};
use erc8183::{Erc8183, Network, job::JobHandle, types::CreateJobParams};

fn connect(
    rpc: &str,
    key_env: &str,
) -> Result<(Erc8183<impl Provider>, Address), Box<dyn std::error::Error>> {
    let signer: PrivateKeySigner = env::var(key_env)?.parse()?;
    let addr = signer.address();
    let sdk = Erc8183::new(
        ProviderBuilder::new()
            .wallet(EthereumWallet::from(signer))
            .connect_http(rpc.parse()?),
    )
    .with_network(Network::MonadMainnet);
    Ok((sdk, addr))
}

/// Client: create job → set budget → fund escrow.
async fn client_create_and_fund<P: Provider>(
    job: &JobHandle<P>,
    provider: Address,
    evaluator: Address,
) -> erc8183::Result<U256> {
    let params = CreateJobParams::new(
        provider,
        evaluator,
        U256::from(u64::MAX),
        "Analyze market data",
    );
    let id = job.create_job(&params).await?;
    let budget = U256::from(1_000_000u64);
    job.set_budget(id, budget, None).await?;
    job.fund(id, budget, None).await?;
    Ok(id)
}

/// Provider: submit work deliverable.
async fn provider_submit<P: Provider>(job: &JobHandle<P>, id: U256) -> erc8183::Result<()> {
    job.submit(id, FixedBytes::from([0xAB; 32]), None).await
}

/// Evaluator: attest completion → release escrow.
async fn evaluator_complete<P: Provider>(job: &JobHandle<P>, id: U256) -> erc8183::Result<()> {
    job.complete(id, FixedBytes::ZERO, None).await
}

#[allow(clippy::print_stdout)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpc = env::var("RPC_URL")?;

    let (client_sdk, _) = connect(&rpc, "CLIENT_KEY")?;
    let (provider_sdk, provider_addr) = connect(&rpc, "PROVIDER_KEY")?;
    let (evaluator_sdk, evaluator_addr) = connect(&rpc, "EVALUATOR_KEY")?;

    let client = client_sdk.job()?;
    let provider = provider_sdk.job()?;
    let evaluator = evaluator_sdk.job()?;

    // Open → Funded → Submitted → Completed
    let id = client_create_and_fund(&client, provider_addr, evaluator_addr).await?;
    println!("[Client]    Created & funded job #{id}");

    provider_submit(&provider, id).await?;
    println!("[Provider]  Submitted deliverable");

    evaluator_complete(&evaluator, id).await?;
    println!("[Evaluator] Completed — payment released");

    let job = client.get_job(id).await?;
    println!(
        "\nJob #{}: {} (terminal: {})",
        job.id,
        job.status,
        job.status.is_terminal()
    );

    Ok(())
}
