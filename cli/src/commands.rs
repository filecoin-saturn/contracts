use crate::utils::{check_address_string, get_signing_provider, send_tx, set_tx_gas, CLIError};
use clap::{Parser, Subcommand};
use contract_bindings::payout_factory_native_addr::PayoutFactoryNativeAddr as PayoutFactory;
use contract_bindings::shared_types::FilAddress;
use ethers::abi::Address;
use ethers::prelude::Middleware;
use ethers::types::U256;
use log::info;
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use std::path::PathBuf;
use std::str::FromStr;

const ATTO_FIL: u128 = 10_u128.pow(18);

#[allow(missing_docs)]
#[derive(Parser, Debug, Clone, Deserialize, Serialize)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    #[allow(missing_docs)]
    pub command: Commands,
    /// Path to the wallet mnemonic
    #[arg(short = 'S', long)]
    secret: PathBuf,
    /// RPC Url
    #[arg(short = 'U', long)]
    rpc_url: String,
    // Num of retries when attempting to make a transaction.
    #[arg(long, default_value = "10")]
    retries: usize,
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
        let mnemonic = read_to_string(self.secret.clone())?;
        let client = get_signing_provider(&mnemonic, &self.rpc_url).await;

        let gas_price = client.provider().get_gas_price().await?;

        info!("current gas price: {:#?}", gas_price);

        info!("using {} retries", self.retries);

        match &self.command {
            Commands::Deploy {} => {
                let mut contract = PayoutFactory::deploy(client.clone().into(), client.address())?;
                let tx = contract.deployer.tx.clone();
                set_tx_gas(
                    &mut contract.deployer.tx,
                    client.estimate_gas(&tx, None).await?,
                    gas_price,
                );

                info!(
                    "estimated deployment gas cost: {:#?}",
                    contract.deployer.tx.gas().unwrap()
                );

                let receipt = send_tx(&contract.deployer.tx, client, self.retries).await?;

                let address = receipt
                    .contract_address
                    .ok_or(CLIError::ContractNotDeployed)?;
                info!("contract address: {:#?}", address);
            }
            Commands::NewPayout {
                factory_addr,
                payout_csv,
            } => {
                let addr = Address::from_str(factory_addr)?;
                let mut reader = csv::Reader::from_path(payout_csv)?;
                let mut shares: Vec<U256> = Vec::new();
                let mut payees: Vec<FilAddress> = Vec::new();
                for record in reader.deserialize() {
                    let record: Payment = record?;
                    let addr = check_address_string(&record.payee)?;

                    let payee = FilAddress {
                        data: addr.bytes.into(),
                    };
                    let share: U256 = (record.shares * ATTO_FIL).into();
                    payees.push(payee);
                    shares.push(share);
                }

                let factory = PayoutFactory::new(addr, client.clone().into());
                let mut payout_tx = factory.payout(payees, shares);
                let tx = payout_tx.tx.clone();
                set_tx_gas(
                    &mut payout_tx.tx,
                    client.estimate_gas(&tx, None).await?,
                    gas_price,
                );

                info!(
                    "estimated payout gas cost {:#?}",
                    payout_tx.tx.gas().unwrap()
                );

                send_tx(&payout_tx.tx, client, self.retries).await?;
            }
            Commands::Claim {
                factory_addr,
                addr_to_claim,
            } => {
                let addr = Address::from_str(factory_addr)?;
                let factory = PayoutFactory::new(addr, client.clone().into());
                let addr_to_claim = check_address_string(addr_to_claim)?;
                let claim_addr = FilAddress {
                    data: addr_to_claim.bytes.into(),
                };
                let mut claim_tx = factory.release_all(claim_addr);
                let tx = claim_tx.tx.clone();
                set_tx_gas(
                    &mut claim_tx.tx,
                    client.estimate_gas(&tx, None).await?,
                    gas_price,
                );

                info!("estimated claim gas cost {:#?}", claim_tx.tx.gas().unwrap());

                send_tx(&claim_tx.tx, client, self.retries).await?;
            }
        }
        Ok(())
    }
}

#[allow(missing_docs)]
#[derive(Debug, Subcommand, Clone, Deserialize, Serialize)]
pub enum Commands {
    /// Deploys a new payout factory contract
    Deploy,
    /// Creates a new paymentsplitter based payout
    #[command(arg_required_else_help = true)]
    NewPayout {
        /// Path to the wallet mnemonic
        #[arg(short = 'F', long)]
        factory_addr: String,
        // Path to csv payout file.
        #[arg(short = 'P', long)]
        payout_csv: PathBuf,
    },
    /// Claims all available funds for a given address
    #[command(arg_required_else_help = true)]
    Claim {
        /// Path to the wallet mnemonic
        #[arg(short = 'F', long)]
        factory_addr: String,
        // Address to claim for
        #[arg(short = 'A', long)]
        addr_to_claim: String,
    },
}
