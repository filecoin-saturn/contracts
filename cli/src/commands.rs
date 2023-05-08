use clap::{Parser, Subcommand};
use contract_bindings::payout_factory_native_addr::PAYOUTFACTORYNATIVEADDR_ABI;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::middleware::SignerMiddleware;
use ethers::prelude::{Http, Middleware, Provider};
use ethers::signers::Wallet;
use ethers::utils::__serde_json::{ser, Value};
use fevm_utils::{get_ledger_signing_provider, get_provider, get_wallet_signing_provider};
use filecoin_signer::PrivateKey;
use fvm_ipld_encoding::RawBytes;
use fvm_shared::bigint::BigInt;
use fvm_shared::econ::TokenAmount;
use fvm_shared::message::Message;
use log::info;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::{self, read_to_string};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use crate::utils::{
    claim_earnings, deploy_factory_contract, fund_factory_contract, generate_monthly_payout,
    new_payout,
};

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

// #[derive(Debug, Deserialize, Clone, Serialize)]
// struct ApproveParams(&str, &str, &str);

// #[derive(Debug, Deserialize, Clone, Serialize)]
// struct ProposeParams(&str, &str, &str, i128, &str, &str);

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
            let client: SignerMiddleware<Arc<Provider<Http>>, Wallet<SigningKey>> =
                get_wallet_signing_provider(provider, &mnemonic).await?;
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
                    claim_earnings(
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
                    claim_earnings(
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
                fund_factory_contract(
                    factory_addr,
                    amount,
                    self.secret,
                    provider,
                    self.retries,
                    gas_price,
                )
                .await
            }
            Commands::WriteAbi { path } => {
                let string_abi = ser::to_string(&PAYOUTFACTORYNATIVEADDR_ABI.clone())?;
                fs::write(&path, string_abi)?;
            }
            Commands::GenerateMonthlyPayout {
                date,
                factory_address,
            } => generate_monthly_payout(date, factory_address).await,
            Commands::MultisigInspect { actor_id } => {
                let params: (&str, ()) = (actor_id.as_str(), ());
                let result: Value = provider
                    .request::<(&str, ()), Value>("Filecoin.StateReadState", params)
                    .await
                    .unwrap();
                println!("{:#?}", result);
            }
            Commands::ProposeNewPayout {
                actor_id,
                factory_address,
                proposer_address,
            } => {
                let params = (
                    "f1mtkndd5nczhq4s7ld36a6m7sn2mcoxkpjyt57oq",
                    3000000000_i128,
                    0,
                    "",
                );

                let mm = Message {
                    version: 0,
                    to: fvm_shared::address::Address::from_str("f02183663").unwrap(),
                    from: fvm_shared::address::Address::from_str(
                        "f15lpcnqqr7cyemknve3wpeqhtniirwhhwguhkwky",
                    )
                    .unwrap(),
                    sequence: 1,
                    value: TokenAmount::from_atto(BigInt::from_str("100000").unwrap()),
                    gas_limit: 25000,
                    gas_fee_cap: TokenAmount::from_atto(BigInt::from_str("1000000").unwrap()),
                    gas_premium: TokenAmount::from_atto(BigInt::from_str("1000000").unwrap()),
                    method_num: 2, // Propose is method no 2
                    params: RawBytes::try_from(vec![]).unwrap(),
                };

                let private_key = String::try_from("insert private key");
                let private_key: Result<PrivateKey, filecoin_signer::error::SignerError> =
                    PrivateKey::try_from(private_key.unwrap());

                let privy = private_key.unwrap();

                println!("{:#?}", privy);

                let result: Value = provider
                    .request::<Message, Value>("Filecoin.MpoolPush", params)
                    .await
                    .unwrap();
                println!("{:#?}", result);
            }
            Commands::ApproveNewPayout {
                actor_id,
                transaction_id,
                proposer_address,
            } => {
                let params = (
                    actor_id.as_str(),
                    transaction_id.as_str(),
                    proposer_address.as_str(),
                );
                let result: Value = provider
                    .request::<(&str, &str, &str), Value>("Filecoin.MsigApprove", params)
                    .await
                    .unwrap();
                println!("{:#?}", result);
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
        amount: i128,
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
    #[command(arg_required_else_help = true)]
    MultisigInspect {
        /// Multisig actor id
        #[arg(short = 'A', long)]
        actor_id: String,
    },
    #[command(arg_required_else_help = true)]
    ProposeNewPayout {
        /// Multisig actor id
        #[arg(short = 'A', long)]
        actor_id: String,
        /// Payout Factory Filecoin Address
        #[arg(short = 'F', long)]
        factory_address: String,
        /// Sender Filecoin Address
        #[arg(short = 'S', long)]
        proposer_address: String,
    },
    #[command(arg_required_else_help = true)]
    ApproveNewPayout {
        /// Multisig actor id
        #[arg(short = 'A', long)]
        actor_id: String,
        /// Transaction Id
        #[arg(short = 'T', long)]
        transaction_id: String,
        /// Proposer Address
        #[arg(short = 'P', long)]
        proposer_address: String,
    },
}
