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
use ethers::utils::__serde_json::ser;
use fevm_utils::{
    check_address_string, get_ledger_signing_provider, get_provider, get_wallet_signing_provider,
    send_tx, set_tx_gas,
};
use log::info;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::{self, read_to_string};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use crate::utils::{parse_payouts_from_csv, parse_payouts_from_db};

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
            Commands::WriteAbi { path } => {
                let string_abi = ser::to_string(&PAYOUTFACTORYNATIVEADDR_ABI.clone())?;
                fs::write(&path, string_abi)?;
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
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = Address::from_str(factory_addr)?;
    let payees;
    let shares;

    if *db_deploy {
        (payees, shares) = parse_payouts_from_db().await.unwrap();
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
    /// Path to write the abi
    WriteAbi {
        #[arg(short = 'P', long)]
        path: String,
    },
}
