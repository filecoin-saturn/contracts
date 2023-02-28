use crate::utils::{addr, get_signing_provider};
use clap::{Parser, Subcommand};
use contract_bindings::payout_factory::PayoutFactory;
use ethers::abi::Address;
use ethers::prelude::Middleware;
use ethers::types::U256;
use eyre::Result;
use log::info;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::str::FromStr;

const GAS_LIMIT_MULTIPLIER: i32 = 130;
const ATTO_FIL: u128 = 10_u128.pow(18);

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
    pub fn create() -> Result<Self, Box<dyn Error>> {
        Ok(Cli::parse())
    }

    pub async fn run(&self) {
        match &self.command {
            Commands::Deploy { secret, rpc_url } => {
                let mnemonic = read_to_string(secret).unwrap();
                let client = get_signing_provider(&mnemonic, &rpc_url).await;
                let addr: Address = addr(&mnemonic).unwrap();
                let mut contract = PayoutFactory::deploy(client.clone().into(), addr).unwrap();

                // let gas1 = client.get_gas_price().await.unwrap();
                let gas = client.provider().get_gas_price().await.unwrap();
                println!("Gas Price {:#?}", gas);
                // let gas_limit = gas * GAS_LIMIT_MULTIPLIER / 100;

                let tx = contract.deployer.tx.clone();
                let gas_estimate =
                    client.estimate_gas(&tx, None).await.unwrap() * GAS_LIMIT_MULTIPLIER / 100;
                // let deployer = deployer.gas_price(gas);
                // let deployer = deployer.gas(gas_estimate);
                contract.deployer.tx.set_gas(gas_estimate);
                contract.deployer.tx.set_gas_price(gas);

                println!("{:#?}", tx);
                info!(
                    "Estimated deployment gas cost {:#?}",
                    client.estimate_gas(&tx, None).await.unwrap()
                );
                let deploy_transaction = contract.send().await.unwrap();
                println!("Deploy Receipt: {:#?}", deploy_transaction)
            }
            Commands::AddPayments {
                secret,
                rpc_url,
                factory_addresss,
                payout_csv,
            } => {
                let mnemonic = read_to_string(secret).unwrap();
                let client = get_signing_provider(&mnemonic, &rpc_url).await;
                let addr = Address::from_str(factory_addresss.as_str()).unwrap();

                let mut reader = csv::Reader::from_path(payout_csv).unwrap();
                let mut shares: Vec<U256> = Vec::new();
                let mut payees: Vec<Address> = Vec::new();
                for record in reader.deserialize() {
                    let record: Payment = record.unwrap();
                    let payee = record.payee.parse::<Address>().unwrap();
                    let share: U256 = (record.shares * ATTO_FIL).into();
                    payees.push(payee);
                    shares.push(share);
                }

                let factory = PayoutFactory::new(addr, client.clone().into());
                let mut payout_tx = factory.payout(payees, shares);
                // let gas1 = client.get_gas_price().await.unwrap();

                let gas = client.provider().get_gas_price().await.unwrap();
                println!("Gas Price {:#?}", gas);
                // let gas_limit = gas * GAS_LIMIT_MULTIPLIER / 100;

                let gas_estimate = client.estimate_gas(&payout_tx.tx, None).await.unwrap()
                    * GAS_LIMIT_MULTIPLIER
                    / 100;
                // let mut payout_tx = payout_tx.gas(gas);
                // let mut payout_tx = payout_tx.gas_price(gas_estimate);
                payout_tx.tx.set_gas_price(gas);
                payout_tx.tx.set_gas(gas_estimate);

                let pendind_tx = payout_tx.send().await;
                println!("Transaction {:#?}", pendind_tx);
            }
        }
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
    },
    #[command(arg_required_else_help = true)]
    AddPayments {
        /// Path to the wallet mnemonic
        #[arg(short = 'S', long)]
        secret: PathBuf,
        /// RPC Url
        #[arg(short = 'U', long)]
        rpc_url: String,
        /// Path to the wallet mnemonic
        #[arg(short = 'F', long)]
        factory_addresss: String,
        // Path to csv payout file.
        #[arg(short = 'P', long)]
        payout_csv: PathBuf,
    },
}
