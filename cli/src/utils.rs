use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;

use chrono::{DateTime, Datelike, Month, NaiveDate, Utc};
use contract_bindings::shared_types::FilAddress;
use ethers::types::{Eip1559TransactionRequest, U256};

use csv::{Error as CsvError, Writer};
use extras::json::tokenamount;
use filecoin_signer::api::MessageTxAPI;
use fvm_shared::econ::TokenAmount;
use fvm_shared::message::Message;
use tokio_postgres::Error as DbError;

use once_cell::sync::Lazy;

use contract_bindings::payout_factory_native_addr::PayoutFactoryNativeAddr as PayoutFactory;
use ethers::abi::Address;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::middleware::SignerMiddleware;
use ethers::prelude::{Http, Middleware, Provider};
use ethers::signers::Wallet;
use ethers::types::transaction::eip2718::TypedTransaction;
use fevm_utils::{check_address_string, get_wallet_signing_provider, send_tx, set_tx_gas};
use log::info;
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use std::io::{self, Write};
use std::str::FromStr;
use std::sync::Arc;

use crate::db::{get_payment_records, get_payment_records_for_finance, PayoutRecords};

pub static ATTO_FIL: Lazy<f64> = Lazy::new(|| 10_f64.powf(18.0));

#[derive(thiserror::Error, Debug)]
pub enum CLIError {
    #[error("contract failed to deploy")]
    ContractNotDeployed,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct Payment {
    Recipient: String,
    FIL: f64,
}

/// calldata is encoding as a byte array of variable length with length encoded by (1, 2, 4, 8 bytes)
const PARAMS_CBOR_HEADER: [&str; 4] = ["58", "59", "5a", "5b"];

/// Parses payouts from a csv file.
///
/// CSV file formatted as such:
///    Recipient,FIL
///    f1...,5
pub async fn parse_payouts_from_csv(
    payout_csv: &PathBuf,
) -> Result<(Vec<FilAddress>, Vec<U256>), CsvError> {
    let mut reader = csv::Reader::from_path(payout_csv)?;
    let mut shares: Vec<U256> = Vec::new();
    let mut payees: Vec<FilAddress> = Vec::new();

    for record in reader.deserialize() {
        let record: Payment = record?;
        let addr = check_address_string(&record.Recipient).unwrap();

        let payee = FilAddress {
            data: addr.bytes.into(),
        };

        let share: U256 = ((record.FIL * &*ATTO_FIL) as u128).into();
        payees.push(payee);
        shares.push(share);
    }
    Ok((payees, shares))
}

/// Retrieves and parses payouts from a Postgres database.
pub async fn parse_payouts_from_db(
    date: &str,
    factory_address: &str,
) -> Result<(Vec<FilAddress>, Vec<U256>), DbError> {
    let PayoutRecords { payees, shares } = get_payment_records_for_finance(date, factory_address)
        .await
        .unwrap();

    let payees = payees
        .iter()
        .map(|payee| {
            let addr = check_address_string(payee).unwrap();
            FilAddress {
                data: addr.bytes.into(),
            }
        })
        .collect();

    let shares: Vec<U256> = shares
        .iter()
        .map(|share| U256::try_from((share * &*ATTO_FIL) as u128).unwrap())
        .collect();
    Ok((payees, shares))
}

/// Formats a date str to an equivalent Postgres compatible date type using DateTime.
///
/// Usage:
/// ```no_run
/// let date = "1916-04-30";
/// let formatted_date = format_date(&date);
/// println!("Formatted Date: {:#?}", formatted_date);
///
/// ```
pub fn format_date(date: &str) -> Result<DateTime<Utc>, Box<dyn Error>> {
    let date_str = date.to_owned() + "-01";
    let date = NaiveDate::parse_from_str(date_str.as_str(), "%Y-%m-%d").unwrap();
    let naive_datetime = date.and_hms_opt(0, 0, 0);
    let date = match naive_datetime {
        None => panic!("Error parsing date"),
        Some(naive_datetime) => DateTime::<Utc>::from_utc(naive_datetime, Utc),
    };
    Ok(date)
}

/// Writes a payout csv to a given path locally.
pub fn write_payout_csv(
    path: &PathBuf,
    payees: &Vec<String>,
    shares: &Vec<f64>,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(payees.len(), shares.len());
    let mut csv_writer = Writer::from_path(path)?;
    let headers = &["Recipient", "FIL", "Method", "Params"];
    csv_writer.write_record(headers)?;

    for record in payees.iter().zip(shares.iter()) {
        let (payee, share) = record;
        csv_writer.write_record(&[payee, &share.to_string(), "0", "nil"])?;
    }

    Ok(())
}

pub async fn claim_earnings<S: Middleware + 'static>(
    client: Arc<S>,
    retries: usize,
    gas_price: U256,
    factory_addr: &str,
    addr_to_claim: &str,
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

pub async fn new_payout<S: Middleware + 'static>(
    client: Arc<S>,
    retries: usize,
    gas_price: U256,
    factory_addr: &str,
    payout_csv: &Option<PathBuf>,
    db_deploy: &bool,
    date: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = Address::from_str(factory_addr)?;
    let payees;
    let shares;

    if *db_deploy {
        (payees, shares) = parse_payouts_from_db(date, factory_addr).await.unwrap();
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

pub async fn propose_new_payout_callbytes<S: Middleware + 'static>(
    client: Arc<S>,
    factory_addr: &str,
    payout_csv: &Option<PathBuf>,
    db_deploy: &bool,
    date: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let addr = Address::from_str(factory_addr)?;
    let factory = PayoutFactory::new(addr, client.clone());

    let payees;
    let shares;

    if *db_deploy {
        (payees, shares) = parse_payouts_from_db(date, factory_addr).await.unwrap();
    } else {
        (payees, shares) = match payout_csv {
            Some(csv_path) => parse_payouts_from_csv(csv_path).await.unwrap(),
            None => {
                panic!("Either payout-csv or db-deployment must be defined as CLI args")
            }
        }
    }

    let call_bytes = factory.payout(payees, shares).calldata().unwrap().to_vec();

    let num_bytes = call_bytes.len().to_be_bytes();
    let num_bytes = num_bytes
        .iter()
        .filter(|x| **x != 0)
        .map(|x| x.clone())
        .collect::<Vec<u8>>();
    let mut params = hex::decode(PARAMS_CBOR_HEADER[num_bytes.len() - 1])?;
    params.extend(num_bytes);
    params.extend(call_bytes);

    Ok(params)
}

async fn get_wallet(
    secret: PathBuf,
    provider: Provider<Http>,
) -> Result<Arc<SignerMiddleware<Arc<Provider<Http>>, Wallet<SigningKey>>>, Box<dyn Error>> {
    let mnemonic = read_to_string(secret)?;
    let client = get_wallet_signing_provider(provider, &mnemonic).await?;
    let client = Arc::new(client);
    Ok(client)
}

pub async fn fund_factory_contract(
    factory_addr: &str,
    amount: &i128,
    secret: Option<PathBuf>,
    provider: Provider<Http>,
    retries: usize,
    gas_price: U256,
) {
    let client = get_wallet(secret.unwrap(), provider).await.unwrap();
    let addr = Address::from_str(factory_addr).unwrap();
    // craft the tx (Filecoin doesn't support legacy transactions)
    let amount = U256::from(*amount);
    let mut fund_tx: TypedTransaction = Eip1559TransactionRequest::new()
        .to(addr)
        .value(amount)
        .from(client.address())
        .into(); // specify the `from` field so that the client knows which account to use

    let tx = fund_tx.clone();
    set_tx_gas(
        &mut fund_tx,
        client.estimate_gas(&tx, None).await.unwrap(),
        gas_price,
    );

    info!("estimated fund gas cost {:#?}", fund_tx.gas().unwrap());

    send_tx(&fund_tx, client, retries).await.unwrap();
}

pub async fn deploy_factory_contract<S: Middleware + 'static>(
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

    // !! Changing this info statement will result in test failure
    info!("contract address: {:#?}", address);

    Ok(())
}

pub async fn generate_monthly_payout(date: &str, factory_address: &str) {
    let formatted_date = format_date(date).unwrap();

    let mut confirmation = String::new();

    let month = Month::from_u32(formatted_date.month()).expect("Invalid MM format in date");
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

    let PayoutRecords { payees, shares } = get_payment_records_for_finance(date, factory_address)
        .await
        .unwrap();

    let csv_title = format!("Saturn-Finance-Payouts-{}.csv", date);
    let path = PathBuf::from_str(csv_title.as_str()).unwrap();
    write_payout_csv(&path, &payees, &shares).unwrap();

    let PayoutRecords { payees, shares } = get_payment_records(date, false).await.unwrap();

    let payout_sum: f64 = shares.iter().sum();
    info!("Sum from payouts {:#?}", payout_sum);
    let csv_title = format!("Saturn-Global-Payouts-{}.csv", date);
    let path = PathBuf::from_str(csv_title.as_str()).unwrap();
    write_payout_csv(&path, &payees, &shares).unwrap();

    let PayoutRecords { payees, shares } = get_payment_records(date, true).await.unwrap();

    let payout_sum: f64 = shares.iter().sum();
    info!("Sum from cassini only payouts {:#?}", payout_sum);
    let csv_title = format!("Saturn-Cassini-Payouts-{}.csv", date);
    let path = PathBuf::from_str(csv_title.as_str()).unwrap();
    write_payout_csv(&path, &payees, &shares).unwrap();
}

#[derive(Debug)]
pub struct TransactionGasInfo {
    pub gas_limit: u64,
    pub gas_fee_cap: TokenAmount,
    pub gas_premium: TokenAmount,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct StateActorInfo {
    #[serde(skip)]
    pub code: HashMap<String, String>,
    #[serde(skip)]
    pub head: HashMap<String, String>,
    #[serde(rename = "Balance", with = "tokenamount")]
    pub balance: TokenAmount,
    #[serde(rename = "Nonce")]
    pub nonce: u64,
}

const GAS_LIMIT_MULTIPLIER: u64 = 150;

pub async fn get_gas_info(
    unsigned_message: Message,
    provider: Provider<Http>,
    max_fee: &str,
) -> TransactionGasInfo {
    let max_fee = HashMap::from([("MaxFee", max_fee)]);

    let gas_info: MessageTxAPI = provider
        .request::<(MessageTxAPI, HashMap<&str, &str>, ()), MessageTxAPI>(
            "Filecoin.GasEstimateMessageGas",
            (MessageTxAPI::Message(unsigned_message), max_fee, ()),
        )
        .await
        .unwrap();

    let gas_info = gas_info.get_message();
    TransactionGasInfo {
        gas_limit: gas_info.gas_limit * (GAS_LIMIT_MULTIPLIER / 100),
        gas_premium: gas_info.gas_premium,
        gas_fee_cap: gas_info.gas_fee_cap,
    }
}

pub async fn get_nonce(address: &str, provider: Provider<Http>) -> u64 {
    let result: StateActorInfo = provider
        .request::<(&str, ()), StateActorInfo>("Filecoin.StateGetActor", (address, ()))
        .await
        .unwrap();

    result.nonce
}
