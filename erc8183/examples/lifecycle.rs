//! Complete lifecycle on Monad Mainnet: `Open → Funded → Submitted → Completed`.
//!
//! A single private key plays all three roles (client, provider, evaluator).
//! This is valid because the contract checks `msg.sender == role_address`,
//! not that the roles are distinct addresses.
//!
//! ```sh
//! export PRIVATE_KEY=0xac0974...
//! cargo run --example lifecycle
//! ```

use std::{env, time::SystemTime};

use alloy::{
    network::EthereumWallet,
    primitives::{FixedBytes, U256},
    providers::ProviderBuilder,
    sol,
};
use erc8183::{Erc8183, Network, types::CreateJobParams};

sol! {
    #[sol(rpc)]
    interface IERC20 {
        function approve(address spender, uint256 amount) external returns (bool);
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("system clock before epoch")
        .as_secs()
}

#[allow(clippy::print_stdout)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let network = Network::MonadMainnet;
    let signer: alloy::signers::local::PrivateKeySigner = env::var("PRIVATE_KEY")?.parse()?;
    let addr = signer.address();

    let provider = ProviderBuilder::new()
        .wallet(EthereumWallet::from(signer))
        .connect_http(network.rpc_url().parse()?);

    let sdk = Erc8183::new(&provider).with_network(network);
    let job = sdk.job()?;

    println!("Network:  {network}");
    println!("Contract: {}", job.contract_address());
    println!("Wallet:   {addr}\n");

    // 1. Create job — one wallet is client, provider, and evaluator
    let expires_at = U256::from(now_secs() + 3600);
    let params = CreateJobParams::new(addr, addr, expires_at, "Analyze market data");
    let id = job.create_job(&params).await?;
    println!("[1/5] Created job #{id}");

    // 2. Set budget (1 USDC = 1_000_000 units with 6 decimals)
    let budget = U256::from(1_000_000u64);
    job.set_budget(id, budget, None).await?;
    println!("[2/5] Budget set: {budget} (1 USDC)");

    // 3. Approve USDC spend, then fund escrow
    let usdc = job.payment_token().await?;
    IERC20::new(usdc, &provider)
        .approve(job.contract_address(), budget)
        .send()
        .await?
        .get_receipt()
        .await?;
    job.fund(id, budget, None).await?;
    println!("[3/5] Funded escrow");

    // 4. Submit work deliverable
    job.submit(id, FixedBytes::from([0xAB; 32]), None).await?;
    println!("[4/5] Submitted deliverable");

    // 5. Evaluator completes — releases payment
    job.complete(id, FixedBytes::ZERO, None).await?;
    println!("[5/5] Completed — payment released");

    // Query final state
    let data = job.get_job(id).await?;
    println!("\n{data}");
    println!("Terminal: {}", data.status.is_terminal());

    Ok(())
}
