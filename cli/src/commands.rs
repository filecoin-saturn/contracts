use crate::utils::{addr, get_signing_provider};
use clap::{Parser, Subcommand};
use contract_bindings::payout_factory::PayoutFactory;
use ethers::abi::Address;
use ethers::prelude::Middleware;
use ethers::types::{H256, U256};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;

const GAS_LIMIT_MULTIPLIER: i32 = 130;
const ATTO_FIL: u128 = 10_u128.pow(18);

#[derive(Error, Debug)]
pub enum CLIError {
    #[error(
        "did not receive receipt, but check a hyperspace explorer to check if tx was successful (hash: ${0})"
    )]
    NoReceipt(H256),
    #[error("contract failed to deploy")]
    ContractNotDeployed,
}

#[allow(missing_docs)]
#[derive(Parser, Debug, Clone, Deserialize, Serialize)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    #[allow(missing_docs)]
    pub command: Commands,
}

#[derive(Deserialize, Debug)]
struct Payment {
    payee: String,
    shares: u128,
}
impl Cli {
    /// Create a configuration
    pub fn create() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Cli::parse())
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        match &self.command {
            Commands::Deploy {
                secret,
                rpc_url,
                retries,
            } => {
                let mnemonic = read_to_string(secret)?;
                let client = get_signing_provider(&mnemonic, &rpc_url).await;
                let addr: Address = addr(&mnemonic).unwrap();
                let mut contract = PayoutFactory::deploy(client.clone().into(), addr)?;

                let gas = client.provider().get_gas_price().await?;
                info!("gas price {:#?}: ", gas);

                let tx = contract.deployer.tx.clone();
                let gas_estimate =
                    client.estimate_gas(&tx, None).await? * GAS_LIMIT_MULTIPLIER / 100;
                contract.deployer.tx.set_gas(gas_estimate);
                contract.deployer.tx.set_gas_price(gas);

                debug!("{:#?}", tx);
                info!(
                    "estimated deployment gas cost {:#?}",
                    client.estimate_gas(&tx, None).await?
                );

                let deployer = contract.deployer;
                let pending_tx = client.send_transaction(deployer.tx, None).await?;

                let hash = pending_tx.tx_hash();
                info!("using {} retries", retries);
                let receipt = pending_tx.retries(*retries).await?;
                if receipt.is_some() {
                    let receipt = receipt.unwrap();
                    debug!("call receipt: {:#?}", receipt);
                    let address = receipt
                        .contract_address
                        .ok_or(CLIError::ContractNotDeployed)?;
                    info!("contract address: {:#?}", address);
                } else {
                    return Err(Box::new(CLIError::NoReceipt(hash)));
                }
            }
            Commands::NewPayout {
                secret,
                rpc_url,
                factory_addr,
                payout_csv,
                retries,
            } => {
                let mnemonic = read_to_string(secret)?;
                let client = get_signing_provider(&mnemonic, &rpc_url).await;
                let addr = Address::from_str(factory_addr.as_str())?;

                let mut reader = csv::Reader::from_path(payout_csv)?;
                let mut shares: Vec<U256> = Vec::new();
                let mut payees: Vec<Address> = Vec::new();
                for record in reader.deserialize() {
                    let record: Payment = record?;
                    let payee = record.payee.parse::<Address>()?;
                    let share: U256 = (record.shares * ATTO_FIL).into();
                    payees.push(payee);
                    shares.push(share);
                }

                let factory = PayoutFactory::new(addr, client.clone().into());
                let mut payout_tx = factory.payout(payees, shares);
                let gas = client.provider().get_gas_price().await?;
                info!("gas price: {:#?}", gas);

                let gas_estimate =
                    client.estimate_gas(&payout_tx.tx, None).await? * GAS_LIMIT_MULTIPLIER / 100;
                payout_tx.tx.set_gas_price(gas);
                payout_tx.tx.set_gas(gas_estimate);

                let pending_tx = payout_tx.send().await?;
                let hash = pending_tx.tx_hash();
                info!("using {} retries", retries);
                let receipt = pending_tx.retries(*retries).await?;
                if receipt.is_some() {
                    debug!("call receipt: {:#?}", receipt.unwrap());
                } else {
                    return Err(Box::new(CLIError::NoReceipt(hash)));
                }
            }
        }
        Ok(())
    }
}

#[allow(missing_docs)]
#[derive(Debug, Subcommand, Clone, Deserialize, Serialize)]
pub enum Commands {
    /// Loads model and prints model table
    #[command(arg_required_else_help = true)]
    Deploy {
        /// The path to the wallet mnemonic
        #[arg(short = 'S', long)]
        secret: PathBuf,
        /// RPC Url
        #[arg(short = 'U', long)]
        rpc_url: String,
        // Num of retries when attempting to make a transaction.
        #[arg(long, default_value = "10")]
        retries: usize,
    },
    #[command(arg_required_else_help = true)]
    NewPayout {
        /// Path to the wallet mnemonic
        #[arg(short = 'S', long)]
        secret: PathBuf,
        /// RPC Url
        #[arg(short = 'U', long)]
        rpc_url: String,
        /// Path to the wallet mnemonic
        #[arg(short = 'F', long)]
        factory_addr: String,
        // Path to csv payout file.
        #[arg(short = 'P', long)]
        payout_csv: PathBuf,
        // Num of retries when attempting to make a transaction.
        #[arg(long, default_value = "10")]
        retries: usize,
    },
}
