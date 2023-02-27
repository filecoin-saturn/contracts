use clap::{Parser, Subcommand};
use log::{info};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::PathBuf;
use std::fs::read_to_string;
use ethers::{
    prelude::{Middleware},
    types::Address,
};
use contract_bindings::payout_factory::PayoutFactory;
use eyre::Result;
use crate::utils::{addr, get_signing_provider};

const GAS_LIMIT_MULTIPLIER: i32 = 130;

#[allow(missing_docs)]
#[derive(Parser, Debug, Clone, Deserialize, Serialize)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    #[allow(missing_docs)]
    pub command: Commands,
}

impl Cli {
    /// Create a configuration
    pub fn create() -> Result<Self, Box<dyn Error>> {
         Ok(Cli::parse())
         
    }

    pub async fn run(&self) {
        match &self.command {
            Commands::Deploy{secret, rpc_url} => {
                let mnemonic = read_to_string(secret).unwrap();
                let client = get_signing_provider(&mnemonic, &rpc_url).await;
                let addr: Address = addr(&mnemonic).unwrap();

                let gas = client.provider().get_gas_price().await.unwrap();
                let gas_limit = gas * GAS_LIMIT_MULTIPLIER;

                let deployer = PayoutFactory::deploy(client.clone().into(), addr).unwrap();
                let deployer = deployer.gas_price(gas);
                let deployer = deployer.gas(gas_limit);

                let mut tx = deployer.deployer.tx.clone();
                let tx = tx.set_chain_id(3141);
                info!(
                    "Estimated deployment gas cost {:#?}",
                    client.estimate_gas(tx, None).await.unwrap()
                );
                deployer.send().await.unwrap();
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
        /// The path to the .onnx model file
        #[arg(short = 'S', long)]
        secret: PathBuf,
        /// The path to the .onnx model file
        #[arg(short = 'U', long)]
        rpc_url: String
    },
}
