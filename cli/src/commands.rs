use clap::{Parser, Subcommand};
use contract_bindings::payout_factory_native_addr::PAYOUTFACTORYNATIVEADDR_ABI;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::middleware::SignerMiddleware;
use ethers::prelude::{Http, Middleware, Provider};
use ethers::signers::Wallet;
use ethers::utils::__serde_json::ser;
use fevm_utils::{get_ledger_signing_provider, get_provider, get_wallet_signing_provider};
use log::info;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::{self, read_to_string};
use std::path::PathBuf;
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
}

#[cfg(test)]
pub mod cli_tests {

    use crate::utils::ATTO_FIL;
    use assert_cmd::prelude::*;
    use assert_fs::fixture::FileWriteStr;
    use assert_fs::NamedTempFile;
    use once_cell::sync::Lazy;
    use regex::Regex;
    use std::env;
    use std::process::Command;
    use std::sync::Mutex;

    static RPC_URL: Lazy<String> =
        Lazy::new(|| env::var("TEST_RPC_URL").expect("TEST_RPC_URL must be set"));
    static MNEMONIC: Lazy<String> =
        Lazy::new(|| env::var("TEST_MNEMONIC").expect("TEST_MNEMONIC must be set"));
    static FACTORY_ADDRESS: Lazy<Mutex<String>> = Lazy::new(|| {
        Mutex::new(env::var("TEST_FACTORY_ADDRESS").expect("TEST_FACTORY_ADDRESS must be set"))
    });

    static SECRETS_FILE: Lazy<NamedTempFile> = Lazy::new(|| {
        let file = assert_fs::NamedTempFile::new("secrets.txt").unwrap();
        file.write_str(&MNEMONIC.as_str()).unwrap();
        file
    });

    const RECIPIENT_ADDRESS: &str = "t1ypi542zmmgaltijzw4byonei5c267ev5iif2liy";

    const PAYOUT: &str = "Recipient,FIL\nt1ypi542zmmgaltijzw4byonei5c267ev5iif2liy,0.01\n";

    fn get_const_cli_args() -> Vec<&'static str> {
        vec![
            "--secret",
            SECRETS_FILE.path().to_str().unwrap(),
            "--rpc-url",
            &RPC_URL,
            "--retries",
            "10",
        ]
    }

    /// This function extract the contract address from the deploy output of the cli.
    fn extract_contract_addr(command_output: &str) -> String {
        // this regex extracts the line containing '[*] contract address:'
        let contract_addr_regex = Regex::new(r"\[\*\] contract address:.*\b").unwrap();
        let contract_addr_match = contract_addr_regex
            .find(command_output)
            .expect("No contract address line found in command output")
            .as_str();

        // this regex extracts 0x addresses from a given str.
        let regex = Regex::new(r"\b0x[a-zA-Z0-9]*").unwrap();
        let contract_addr = regex.find(contract_addr_match).unwrap();
        contract_addr.as_str().to_string()
    }

    #[test]
    fn cli_1_deploy() -> Result<(), Box<dyn std::error::Error>> {
        let mut args = get_const_cli_args();
        args.push("deploy");

        let mut cmd = Command::cargo_bin("saturn-contracts")?;
        cmd.args(&args);

        let output = cmd
            // execute the command, wait for it to complete, then capture the output
            .output()
            .expect("Ok");

        // extract the CLI terminal output and parse to string
        let out_data = String::from_utf8(output.clone().stderr).unwrap();

        let addr = extract_contract_addr(&out_data.as_str());
        let mut data = FACTORY_ADDRESS.lock().unwrap();
        *data = addr;
        Ok(())
    }

    #[test]
    fn cli_2_fund() -> Result<(), Box<dyn std::error::Error>> {
        let mut args = get_const_cli_args();

        let factory_addr = &FACTORY_ADDRESS.lock().unwrap().to_string();
        let amount = (0.01 * &*ATTO_FIL) as u128;
        let amount = amount.to_string();

        let mut new_payout_args = vec![
            "fund",
            "--factory-addr",
            factory_addr.as_str(),
            "--amount",
            amount.as_str(),
        ];
        args.append(&mut new_payout_args);

        let mut cmd = Command::cargo_bin("saturn-contracts")?;
        cmd.args(&args);
        cmd.output().ok();

        Ok(())
    }

    #[test]
    fn cli_3_new_payout() -> Result<(), Box<dyn std::error::Error>> {
        let mut args = get_const_cli_args();

        let payouts_csv = assert_fs::NamedTempFile::new("payout.csv")?;
        payouts_csv.write_str(PAYOUT)?;

        let factory_addr = &FACTORY_ADDRESS.lock().unwrap();

        let mut new_payout_args = vec![
            "new-payout",
            "--factory-addr",
            factory_addr,
            "--payout-csv",
            payouts_csv.path().to_str().unwrap(),
        ];
        args.append(&mut new_payout_args);

        let mut cmd = Command::cargo_bin("saturn-contracts")?;
        cmd.args(&args);
        cmd.output().ok();

        Ok(())
    }

    #[test]
    fn cli_4_claim() -> Result<(), Box<dyn std::error::Error>> {
        let mut args = get_const_cli_args();

        let factory_addr = &FACTORY_ADDRESS.lock().unwrap();
        let mut new_payout_args = vec![
            "claim",
            "--factory-addr",
            factory_addr,
            "--addr-to-claim",
            &RECIPIENT_ADDRESS,
        ];
        args.append(&mut new_payout_args);

        let mut cmd = Command::cargo_bin("saturn-contracts")?;
        cmd.args(&args);
        cmd.output().ok();
        Ok(())
    }
}
