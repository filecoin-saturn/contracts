use clap::{Parser, Subcommand};
use contract_bindings::payout_factory_native_addr::{
    PayoutFactoryNativeAddr as PayoutFactory, PAYOUTFACTORYNATIVEADDR_ABI,
};
use contract_bindings::shared_types::FilAddress;
use ethers::abi::Address;
use ethers::prelude::Middleware;
use ethers::utils::__serde_json::ser;
use fevm_utils::{check_address_string, get_signing_provider, send_tx, set_tx_gas};
use log::info;
use serde::{Deserialize, Serialize};
use std::fs::{self, read_to_string};
use std::path::PathBuf;
use std::str::FromStr;

use crate::db::{get_payment_records, get_payment_records_for_finance, PayoutRecords};
use crate::utils::{parse_payouts_from_csv, parse_payouts_from_db, write_payout_csv};

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

#[derive(thiserror::Error, Debug)]
pub enum CLIError {
    #[error("contract failed to deploy")]
    ContractNotDeployed,
}

impl Cli {
    /// Create a configuration
    pub fn create() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Cli::parse())
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mnemonic = read_to_string(self.secret.clone())?;
        let client = get_signing_provider(&mnemonic, &self.rpc_url).await?;

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
                db_deploy,
            } => {
                let addr = Address::from_str(factory_addr)?;
                let payees;
                let shares;

                if *db_deploy {
                    (payees, shares) = parse_payouts_from_db("2023-03-02".into(), &factory_addr)
                        .await
                        .unwrap();
                } else {
                    (payees, shares) = match payout_csv {
                        Some(csv_path) => parse_payouts_from_csv(csv_path).await.unwrap(),
                        None => {
                            panic!("Either payout-csv or db-deployment must be defined as CLI args")
                        }
                    }
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
            Commands::WriteAbi { path } => {
                let string_abi = ser::to_string(&PAYOUTFACTORYNATIVEADDR_ABI.clone())?;
                fs::write(&path, string_abi)?;
            }
            Commands::GenerateMonthlyPayouts {
                date,
                factory_address,
            } => {
                let PayoutRecords { payees, shares } =
                    get_payment_records_for_finance(date.as_str(), factory_address)
                        .await
                        .unwrap();

                let csv_title = format!("Saturn-Finance-Payouts-{}.csv", date);
                let path = PathBuf::from_str(&csv_title.as_str()).unwrap();
                write_payout_csv(&path, &payees, &shares).unwrap();

                let PayoutRecords { payees, shares } =
                    get_payment_records(date.as_str(), false).await.unwrap();

                let payout_sum: f64 = shares.iter().sum();
                info!("Sum from payouts {:#?}", payout_sum);
                let csv_title = format!("Saturn-Payouts-{}.csv", date);
                let path = PathBuf::from_str(&csv_title.as_str()).unwrap();
                write_payout_csv(&path, &payees, &shares).unwrap();
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
    #[command()]
    NewPayout {
        /// PayoutFactory ethereum address.
        #[arg(short = 'F', long, required = true)]
        factory_addr: String,
        #[arg(short = 'P', long)]
        payout_csv: Option<PathBuf>,
        // Flag to determine if this is a db deployment.
        #[arg(
            short = 'D',
            long,
            conflicts_with = "payout_csv",
            default_value_t = false
        )]
        db_deploy: bool,
    },
    /// Claims all available funds for a given address
    #[command(arg_required_else_help = true)]
    Claim {
        /// PayoutFactory ethereum address.
        #[arg(short = 'F', long)]
        factory_addr: String,
        // Address to claim for
        #[arg(short = 'A', long)]
        addr_to_claim: String,
    },
    /// Writes the abi of the PayouFactory contract.
    #[command(arg_required_else_help = true)]
    WriteAbi {
        /// Path to write the abi
        #[arg(short = 'P', long)]
        path: String,
    },
    /// Generates monthly payout and stores relevant csv's.
    #[command(arg_required_else_help = true)]
    GenerateMonthlyPayouts {
        /// Date formatted YYYY-MM-DD
        #[arg(short = 'P', long)]
        date: String,
        /// PayoutFactory ethereum address.
        #[arg(short = 'F', long)]
        factory_address: String,
    },
}
