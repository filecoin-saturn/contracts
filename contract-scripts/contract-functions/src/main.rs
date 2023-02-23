use std::fs::read_to_string;
mod utils;
use utils::addr;

use ethers::{
    prelude::{k256::ecdsa::SigningKey, Middleware, SignerMiddleware},
    providers::{Http, Provider},
    signers::Wallet,
    types::Address,
};

use contract_bindings::payout_factory::PayoutFactory;
// use ethers::signers::Signer;
use eyre::Result;
// use serde_json::ser;
// use std::fs;
// use std::fs::File;
// use std::sync::Arc;
use utils::get_signing_provider;

const GAS_LIMIT_MULTIPLIER: i32 = 130;

#[tokio::main]
async fn main() -> Result<()> {
    let mnemonic = read_to_string("./secrets/.secret").unwrap();
    let rpc_url: &str = "https://api.hyperspace.node.glif.io/rpc/v1";
    let client = get_signing_provider(&mnemonic, rpc_url).await;
    let addr: Address = addr(&mnemonic).unwrap().into();

    let gas = client.provider().get_gas_price().await.unwrap();
    let gas_limit = gas * GAS_LIMIT_MULTIPLIER / 100;

    let deployer = PayoutFactory::deploy(client.clone().into(), addr).unwrap();
    let deployer = deployer.gas_price(gas);
    let deployer = deployer.gas(gas_limit);

    let mut tx = deployer.deployer.tx.clone();
    let tx = tx.set_chain_id(3141);
    println!(
        "Estimated deployment gas cost {:#?}",
        client.estimate_gas(tx, None).await.unwrap()
    );
    deployer.send().await.unwrap();

    Ok(())
}