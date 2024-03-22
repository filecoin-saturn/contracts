use clap::{Parser, Subcommand};
use contract_bindings::payout_factory_native_addr::PAYOUTFACTORYNATIVEADDR_ABI;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::middleware::SignerMiddleware;
use ethers::providers::{Http, Middleware, Provider};
use ethers::signers::Wallet;
use ethers::utils::__serde_json::ser;
use fevm_utils::{
    filecoin_to_eth_address, get_ledger_signing_provider, get_provider, get_wallet_signing_provider,
};

use log::info;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::{self, read_to_string};
use std::path::PathBuf;
use std::sync::Arc;

use crate::utils::{
    approve_payout, cancel_payout, deploy_factory_contract, fund_factory_contract,
    generate_monthly_payout, get_pending_transaction_multisig, get_signing_method_and_address,
    get_unreleased_payout_contracts, grant_admin, inspect_earnings, inspect_multisig, new_payout,
    propose_payout, release_selected_payouts, release_selected_payouts_filecoin_signing,
    revoke_admin, SigningOptions,
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
    #[arg(short = 'U', long, default_value = "https://api.node.glif.io/rpc/v1")]
    rpc_url: String,
    /// Num of retries when attempting to make a transaction.
    #[arg(long, default_value = "10")]
    retries: usize,
    /// Ledger account index in Bip44 Path.
    #[arg(long, default_value = "0")]
    ledger_account: u32,
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
                method,
            } => {
                let releasable_contract_indices = get_unreleased_payout_contracts(
                    &factory_addr,
                    &addr_to_claim,
                    &self.rpc_url,
                    &provider.clone(),
                )
                .await
                .unwrap();
                match method {
                    Some(option) => {
                        let (signing_method, signer_address) =
                            get_signing_method_and_address(option, self.ledger_account.clone())
                                .await
                                .unwrap();

                        release_selected_payouts_filecoin_signing(
                            &provider.clone(),
                            factory_addr,
                            addr_to_claim,
                            releasable_contract_indices.clone(),
                            &signing_method,
                            &signer_address,
                            &self.rpc_url,
                        )
                        .await?
                    }
                    None => {
                        let factory_eth_addr =
                            filecoin_to_eth_address(&factory_addr, &self.rpc_url).await?;
                        if self.secret.is_some() {
                            let client = get_wallet(self.secret.unwrap(), provider).await?;
                            release_selected_payouts(
                                client.clone(),
                                self.retries,
                                gas_price,
                                &factory_eth_addr,
                                addr_to_claim,
                                releasable_contract_indices.clone(),
                            )
                            .await?;
                        } else {
                            let client =
                                get_ledger_signing_provider(provider, chain_id.as_u64()).await?;
                            let client = Arc::new(client);
                            release_selected_payouts(
                                client.clone(),
                                self.retries,
                                gas_price,
                                &factory_eth_addr,
                                addr_to_claim,
                                releasable_contract_indices.clone(),
                            )
                            .await?;
                        }
                    }
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
                inspect_multisig(&provider, actor_id).await?;
            }
            Commands::ProposeNewPayout {
                actor_address,
                receiver_address,
                payout_csv,
                db_deploy,
                date,
                method,
            } => {
                let (signing_method, signer_address) =
                    get_signing_method_and_address(method, self.ledger_account.clone())
                        .await
                        .unwrap();

                propose_payout(
                    actor_address,
                    receiver_address,
                    date,
                    db_deploy,
                    payout_csv,
                    &provider,
                    &self.rpc_url,
                    signing_method,
                    &signer_address,
                )
                .await?;
            }
            Commands::CancelPayout {
                actor_address,
                transaction_id,
                method,
            } => {
                let (signing_method, signer_address) =
                    get_signing_method_and_address(method, self.ledger_account.clone())
                        .await
                        .unwrap();

                cancel_payout(
                    actor_address,
                    &provider,
                    &transaction_id,
                    &signing_method,
                    &signer_address,
                )
                .await?;
            }
            Commands::CancelAll {
                actor_address,
                method,
            } => {
                let tx = get_pending_transaction_multisig(&provider, actor_address).await?;
                let (signing_method, signer_address) =
                    get_signing_method_and_address(method, self.ledger_account.clone())
                        .await
                        .unwrap();
                for transaction in tx.iter() {
                    cancel_payout(
                        actor_address,
                        &provider,
                        &format!("{}", transaction.id),
                        &signing_method,
                        &signer_address,
                    )
                    .await?;
                }
            }
            Commands::ApproveNewPayout {
                actor_address,
                transaction_id,
                method,
            } => {
                let (signing_method, signer_address) =
                    get_signing_method_and_address(method, self.ledger_account.clone())
                        .await
                        .unwrap();

                approve_payout(
                    &actor_address,
                    &provider,
                    &signing_method,
                    &signer_address,
                    transaction_id,
                )
                .await?;
            }
            Commands::ApproveAll {
                actor_address,
                method,
            } => {
                let tx = get_pending_transaction_multisig(&provider, actor_address).await?;
                let (signing_method, signer_address) =
                    get_signing_method_and_address(method, self.ledger_account.clone())
                        .await
                        .unwrap();
                for transaction in tx.iter() {
                    approve_payout(
                        &actor_address,
                        &provider,
                        &signing_method,
                        &signer_address,
                        &format!("{}", transaction.id),
                    )
                    .await?;
                }
            }
            Commands::GrantAdmin {
                address,
                factory_addr,
            } => {
                if self.secret.is_some() {
                    let client = get_wallet(self.secret.unwrap(), provider).await?;
                    grant_admin(
                        client.clone(),
                        self.retries,
                        gas_price,
                        factory_addr,
                        address,
                        &self.rpc_url,
                    )
                    .await?;
                } else {
                    let client = get_ledger_signing_provider(provider, chain_id.as_u64()).await?;
                    let client = Arc::new(client);
                    grant_admin(
                        client.clone(),
                        self.retries,
                        gas_price,
                        factory_addr,
                        address,
                        &self.rpc_url,
                    )
                    .await?;
                }
            }
            Commands::RevokeAdmin {
                address,
                factory_addr,
            } => {
                if self.secret.is_some() {
                    let client = get_wallet(self.secret.unwrap(), provider).await?;
                    revoke_admin(
                        client.clone(),
                        self.retries,
                        gas_price,
                        factory_addr,
                        address,
                        &self.rpc_url,
                    )
                    .await?;
                } else {
                    let client = get_ledger_signing_provider(provider, chain_id.as_u64()).await?;
                    let client = Arc::new(client);
                    revoke_admin(
                        client.clone(),
                        self.retries,
                        gas_price,
                        factory_addr,
                        address,
                        &self.rpc_url,
                    )
                    .await?;
                }
            }
            Commands::InspectEarnings {
                address,
                factory_address,
            } => {
                let factory_eth_addr =
                    filecoin_to_eth_address(&factory_address, &self.rpc_url).await?;
                let provider = get_provider(&self.rpc_url).unwrap();
                inspect_earnings(&provider, address, &factory_eth_addr).await;
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
        /// PayoutFactory Filecoin address.
        #[arg(short = 'F', long)]
        factory_addr: String,
        // Address to claim for
        #[arg(short = 'A', long)]
        addr_to_claim: String,
        #[arg(short = 'M', long, required = false)]
        method: Option<SigningOptions>,
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
    /// Returns State and Pending Transactions of a Multisig Contract
    #[command(arg_required_else_help = true)]
    MultisigInspect {
        /// Multisig actor id
        #[arg(short = 'A', long)]
        actor_id: String,
    },
    /// Returns Payout stats for a given node filecoin address
    #[command(arg_required_else_help = true)]
    InspectEarnings {
        /// Address to insepct
        #[arg(short = 'A', long)]
        address: String,
        /// Multisig Filecoin Actor Id or Address
        #[arg(short = 'F', long)]
        factory_address: String,
    },
    /// Proposes a new payout deployment to a multisig address and
    /// a factory contract address
    #[command(arg_required_else_help = true)]
    ProposeNewPayout {
        /// Multisig Filecoin Actor Id or Address
        #[arg(short = 'A', long)]
        actor_address: String,
        /// Payout Factory Filecoin Address
        #[arg(short = 'M', long)]
        receiver_address: String,
        #[arg(short = 'C', long)]
        payout_csv: Option<PathBuf>,
        /// Flag to determine if this is a db deployment.
        #[arg(long, conflicts_with = "payout_csv", default_value_t = false)]
        db_deploy: bool,
        /// Date for the payout period month.
        #[arg(short = 'D', long, default_value = "")]
        date: String,
        /// Signing Method for the command.
        #[arg(long, default_value = "local", value_enum)]
        method: SigningOptions,
    },
    /// Cancels a proposed payout on a multisig actor identified by its transaction Id
    #[command(arg_required_else_help = true)]
    CancelPayout {
        /// Multisig Filecoin Actor Id or Address
        #[arg(short = 'A', long)]
        actor_address: String,
        /// Transaction Id
        #[arg(short = 'T', long)]
        transaction_id: String,
        /// Signing Method for the command.
        #[arg(long, default_value = "local", value_enum)]
        method: SigningOptions,
    },
    /// Cancels all proposed payouts on a multisig actor
    #[command(arg_required_else_help = true)]
    CancelAll {
        /// Multisig Filecoin Actor Id or Address
        #[arg(short = 'A', long)]
        actor_address: String,
        /// Signing Method for the command.
        #[arg(long, default_value = "local", value_enum)]
        method: SigningOptions,
    },
    /// Approves a proposed payout on a multisig address by its transaction Id
    #[command(arg_required_else_help = true)]
    ApproveNewPayout {
        /// Multisig Filecoin Actor Id or Address
        #[arg(short = 'A', long)]
        actor_address: String,
        /// Transaction Id
        #[arg(short = 'T', long)]
        transaction_id: String,
        /// Signing Method for the command.
        #[arg(long, default_value = "local", value_enum)]
        method: SigningOptions,
    },
    /// Approves all proposed payouts on a multisig actor
    #[command(arg_required_else_help = true)]
    ApproveAll {
        /// Multisig Filecoin Actor Id or Address
        #[arg(short = 'A', long)]
        actor_address: String,
        /// Signing Method for the command.
        #[arg(long, default_value = "local", value_enum)]
        method: SigningOptions,
    },
    /// Grants an admin role to a payout factory contract. The issuing address
    /// has to be an admin on the contract.
    #[command(arg_required_else_help = true)]
    GrantAdmin {
        /// Address to grant role to
        #[arg(short = 'A', long)]
        address: String,
        /// PayoutFactory Ethereum address.
        #[arg(short = 'F', long)]
        factory_addr: String,
    },
    /// Revokes an admin role from a payout factory contract. The issuing address
    /// has to be an admin on the contract.
    #[command(arg_required_else_help = true)]
    RevokeAdmin {
        /// Address to revoke role from
        #[arg(short = 'A', long)]
        address: String,
        /// PayoutFactory Ethereum address.
        #[arg(short = 'F', long)]
        factory_addr: String,
    },
}
