use chrono::{Datelike, Month};
use clap::{Parser, Subcommand};
use contract_bindings::payout_factory_native_addr::{
    PayoutFactoryNativeAddr as PayoutFactory, PAYOUTFACTORYNATIVEADDR_ABI,
};
use contract_bindings::shared_types::FilAddress;
use ethers::abi::Address;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::middleware::SignerMiddleware;
use ethers::prelude::{Http, Middleware, Provider, U256};
use ethers::signers::Wallet;
use ethers::types::transaction::eip2718::TypedTransaction;
use ethers::types::Eip1559TransactionRequest;
use ethers::utils::__serde_json::ser;
use fevm_utils::{
    check_address_string, get_ledger_signing_provider, get_provider, get_wallet_signing_provider,
    send_tx, set_tx_gas,
};
use log::info;
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::{self, read_to_string};
use std::io::{self, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use crate::db::{get_payment_records, get_payment_records_for_finance, PayoutRecords};
use crate::utils::{format_date, parse_payouts_from_csv, parse_payouts_from_db, write_payout_csv};

#[allow(missing_docs)]
#[derive(Parser, Debug, Clone, Deserialize, Serialize)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    #[allow(missing_docs)]
    pub command: Commands,
    /// Path to the wallet mnemonic
    #[arg(short = 'S', long)]
    secret: Option<PathBuf>,
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

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let provider = get_provider(&self.rpc_url)?;
        let gas_price = provider.get_gas_price().await?;
        let chain_id = provider.get_chainid().await?;
        info!("current gas price: {:#?}", gas_price);
        info!("using {} retries", self.retries);

        async fn get_wallet(
            secret: PathBuf,
            provider: Provider<Http>,
        ) -> Result<Arc<SignerMiddleware<Arc<Provider<Http>>, Wallet<SigningKey>>>, Box<dyn Error>>
        {
            let mnemonic = read_to_string(secret)?;
            let client = get_wallet_signing_provider(provider, &mnemonic).await?;
            let client = Arc::new(client);
            Ok(client)
        }

        match &self.command {
            Commands::Deploy {} => {
                if self.secret.is_some() {
                    let client = get_wallet(self.secret.unwrap(), provider).await?;
                    deploy_factory_contract(
                        client.clone(),
                        self.retries,
                        gas_price,
                        client.address(),
                    )
                    .await?;
                } else {
                    let client = get_ledger_signing_provider(provider, chain_id.as_u64()).await?;
                    let client = Arc::new(client);
                    deploy_factory_contract(
                        client.clone(),
                        self.retries,
                        gas_price,
                        client.address(),
                    )
                    .await?;
                }
            }
            Commands::NewPayout {
                factory_addr,
                payout_csv,
                db_deploy,
                date,
            } => {
                if self.secret.is_some() {
                    let client = get_wallet(self.secret.unwrap(), provider).await?;
                    new_payout(
                        client.clone(),
                        self.retries,
                        gas_price,
                        factory_addr,
                        payout_csv,
                        db_deploy,
                        date,
                    )
                    .await?;
                } else {
                    let client = get_ledger_signing_provider(provider, chain_id.as_u64()).await?;
                    let client = Arc::new(client);
                    new_payout(
                        client.clone(),
                        self.retries,
                        gas_price,
                        factory_addr,
                        payout_csv,
                        db_deploy,
                        date,
                    )
                    .await?;
                }
            }
            Commands::Claim {
                factory_addr,
                addr_to_claim,
            } => {
                if self.secret.is_some() {
                    let client = get_wallet(self.secret.unwrap(), provider).await?;
                    claim_reward(
                        client.clone(),
                        self.retries,
                        gas_price,
                        factory_addr,
                        addr_to_claim,
                    )
                    .await?;
                } else {
                    let client = get_ledger_signing_provider(provider, chain_id.as_u64()).await?;
                    let client = Arc::new(client);
                    claim_reward(
                        client.clone(),
                        self.retries,
                        gas_price,
                        factory_addr,
                        addr_to_claim,
                    )
                    .await?;
                }
            }
            Commands::Fund {
                factory_addr,
                amount,
            } => {
                let client = get_wallet(self.secret.unwrap(), provider).await?;
                let addr = Address::from_str(factory_addr)?;
                // craft the tx (Filecoin doesn't support legacy transactions)
                let mut fund_tx: TypedTransaction = Eip1559TransactionRequest::new()
                    .to(addr)
                    .value(amount)
                    .from(client.address())
                    .into(); // specify the `from` field so that the client knows which account to use

                let tx = fund_tx.clone();
                set_tx_gas(
                    &mut fund_tx,
                    client.estimate_gas(&tx, None).await?,
                    gas_price,
                );

                info!("estimated fund gas cost {:#?}", fund_tx.gas().unwrap());

                send_tx(&fund_tx, client, self.retries).await?;
            }
            Commands::WriteAbi { path } => {
                let string_abi = ser::to_string(&PAYOUTFACTORYNATIVEADDR_ABI.clone())?;
                fs::write(&path, string_abi)?;
            }
            Commands::GenerateMonthlyPayout {
                date,
                factory_address,
            } => {
                let formatted_date = format_date(date).unwrap();

                let mut confirmation = String::new();

                let month =
                    Month::from_u32(formatted_date.month()).expect("Invalid MM format in date");
                info!(
                    "Type 'yes' to confirm you are generating payouts for {} {}",
                    month.name(),
                    formatted_date.year(),
                );
                let _ = io::stdout().flush();
                let _ = std::io::stdin().read_line(&mut confirmation).unwrap();

                if confirmation.trim().ne(&String::from("yes")) {
                    panic!("User rejected current date");
                }

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
                let csv_title = format!("Saturn-Global-Payouts-{}.csv", date);
                let path = PathBuf::from_str(&csv_title.as_str()).unwrap();
                write_payout_csv(&path, &payees, &shares).unwrap();

                let PayoutRecords { payees, shares } =
                    get_payment_records(date.as_str(), true).await.unwrap();

                let payout_sum: f64 = shares.iter().sum();
                info!("Sum from cassini only payouts {:#?}", payout_sum);
                let csv_title = format!("Saturn-Cassini-Payouts-{}.csv", date);
                let path = PathBuf::from_str(&csv_title.as_str()).unwrap();
                write_payout_csv(&path, &payees, &shares).unwrap();
            }
        }
        Ok(())
    }
}

async fn claim_reward<S: Middleware + 'static>(
    client: Arc<S>,
    retries: usize,
    gas_price: U256,
    factory_addr: &String,
    addr_to_claim: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = Address::from_str(factory_addr)?;
    let factory = PayoutFactory::new(addr, client.clone());
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

    send_tx(&claim_tx.tx, client, retries).await?;
    Ok(())
}

async fn new_payout<S: Middleware + 'static>(
    client: Arc<S>,
    retries: usize,
    gas_price: U256,
    factory_addr: &String,
    payout_csv: &Option<PathBuf>,
    db_deploy: &bool,
    date: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = Address::from_str(factory_addr)?;
    let payees;
    let shares;

    if *db_deploy {
        (payees, shares) = parse_payouts_from_db(&date.as_str(), &factory_addr.as_str())
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

    let factory = PayoutFactory::new(addr, client.clone());
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

    send_tx(&payout_tx.tx, client, retries).await?;
    Ok(())
}

async fn deploy_factory_contract<S: Middleware + 'static>(
    client: Arc<S>,
    retries: usize,
    gas_price: U256,
    address: Address,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut contract = PayoutFactory::deploy(client.clone(), address)?;
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

    let receipt = send_tx(&contract.deployer.tx, client, retries).await?;

    let address = receipt
        .contract_address
        .ok_or(CLIError::ContractNotDeployed)?;
    info!("contract address: {:#?}", address);
    Ok(())
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
        #[arg(long, conflicts_with = "payout_csv", default_value_t = false)]
        db_deploy: bool,
        // Date for the payout period month.
        #[arg(short = 'D', long, default_value = "")]
        date: String,
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
    /// Fund a factory contract
    #[command(arg_required_else_help = true)]
    Fund {
        /// PayoutFactory ethereum address.
        #[arg(short = 'F', long)]
        factory_addr: String,
        // Amount to send
        #[arg(short = 'A', long)]
        amount: U256,
    },
    /// Path to write the abi
    WriteAbi {
        /// Path to write the abi
        #[arg(short = 'P', long)]
        path: String,
    },
    /// Generates monthly payout and stores relevant csv's.
    #[command(arg_required_else_help = true)]
    GenerateMonthlyPayout {
        /// Date formatted YYYY-MM
        #[arg(short = 'D', long)]
        date: String,
        /// PayoutFactory ethereum address.
        #[arg(short = 'F', long)]
        factory_address: String,
    },
}
