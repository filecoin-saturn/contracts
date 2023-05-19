use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;

use chrono::{DateTime, Datelike, Month, NaiveDate, Utc};
use contract_bindings::shared_types::FilAddress;
use ethers::types::{Eip1559TransactionRequest, U256};

use csv::{Error as CsvError, Writer};
use extras::json::tokenamount;
use extras::signed_message::ref_fvm::SignedMessage;
use fevm_utils::filecoin_to_eth_address;
use fil_actor_multisig::{ProposeParams, TxnID, TxnIDParams};
use filecoin_signer::api::{MessageParams, MessageTxAPI};
use fvm_ipld_encoding::to_vec;
use fvm_ipld_encoding::RawBytes;
use fvm_shared::address::Address as FilecoinAddress;
use fvm_shared::bigint::BigInt;
use fvm_shared::crypto::signature::Signature as FilSignature;
use fvm_shared::econ::TokenAmount;
use fvm_shared::message::Message;
use ledger_filecoin::{BIP44Path, FilecoinApp};
use ledger_transport_hid::{hidapi::HidApi, TransportNativeHID};
use serde_json::Value;
use tokio_postgres::Error as DbError;

use once_cell::sync::Lazy;

use contract_bindings::payout_factory_native_addr::PayoutFactoryNativeAddr as PayoutFactory;
use ethers::abi::Address;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::middleware::SignerMiddleware;
use ethers::providers::{Http, Middleware, Provider};
use ethers::signers::Wallet;
use ethers::types::transaction::eip2718::TypedTransaction;
use fevm_utils::{check_address_string, get_wallet_signing_provider, send_tx, set_tx_gas};
use log::info;
use num_traits::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
// use serde_derive::Deserialize;
// use serde_derive::Serialize;
use std::fs::read_to_string;
use std::io::{self, Write};
use std::str::FromStr;
use std::sync::Arc;
use tabled::{settings::object::Object, Table, Tabled};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionDetails {
    #[serde(rename = "/")]
    pub field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingTxns {
    #[serde(rename = "/")]
    pub field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Code {
    #[serde(rename = "/")]
    pub field: String,
}

fn display_vector<T: std::fmt::Debug>(v: &Vec<T>) -> String {
    if !v.is_empty() {
        format!("{:?}", v)
    } else {
        String::new()
    }
}

fn display_txns(v: &PendingTxns) -> String {
    format!("{:?}", v.field)
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Tabled)]
#[serde(rename_all = "PascalCase")]
pub struct State {
    start_epoch: u64,
    unlock_duration: u64,
    #[serde(rename = "PendingTxns")]
    #[tabled(display_with = "display_txns")]
    pub pending_txns: PendingTxns,
    #[tabled(display_with = "display_vector")]
    pub signers: Vec<String>,
    #[serde(rename = "InitialBalance")]
    pub init_bal: String,
    #[serde(rename = "NumApprovalsThreshold")]
    pub threshold: u64,
    #[serde(rename = "NextTxnID")]
    pub next_txn_id: u64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MultiSigActor {
    pub balance: String,
    pub code: Code,
    pub state: State,
}

use crate::db::{get_payment_records, get_payment_records_for_finance, PayoutRecords};

pub static ATTO_FIL: Lazy<f64> = Lazy::new(|| 10_f64.powf(18.0));

pub const MAX_PAYEES_PER_PAYOUT: usize = 700;

// MaxFee is set to zero when using MpoolPush
const MAX_FEE: &str = "0";

const BIP44_PATH: BIP44Path = BIP44Path {
    purpose: 0x8000_0000 | 44,
    coin: 0x8000_0000 | 461,
    account: 0,
    change: 0,
    index: 0,
};

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

pub async fn propose_payout(
    actor_address: &str,
    receiver_address: &str,
    payout_csv: &Option<PathBuf>,
    db_deploy: &bool,
    date: &str,
    provider: &Provider<Http>,
    rpc_url: &str,
    filecoin_ledger_app: &FilecoinApp<TransportNativeHID>,
) -> Result<(), Box<dyn std::error::Error>> {
    let factory_addr_eth = filecoin_to_eth_address(&receiver_address, &rpc_url).await?;

    let propose_call_data = propose_new_payout_callbytes(
        Arc::new(provider.clone()),
        &factory_addr_eth,
        payout_csv,
        db_deploy,
        date,
    )
    .await?;

    let proposer_address = filecoin_ledger_app
        .address(&BIP44_PATH, false)
        .await
        .unwrap()
        .addr_string;

    let params: ProposeParams = ProposeParams {
        to: FilecoinAddress::from_str(&receiver_address).unwrap(),
        // no transfer of value
        value: TokenAmount::from_atto(BigInt::from_str("0").unwrap()),
        method: fil_actor_evm::Method::InvokeContract as u64,
        params: RawBytes::new(propose_call_data),
    };

    let nonce = get_nonce(&proposer_address, provider.clone()).await;

    let mut message = Message {
        version: 0,
        to: FilecoinAddress::from_str(&actor_address).unwrap(),
        from: FilecoinAddress::from_str(&proposer_address).unwrap(),
        sequence: nonce,
        value: TokenAmount::from_atto(BigInt::from_str("0").unwrap()),
        gas_limit: 0,
        gas_fee_cap: TokenAmount::from_atto(BigInt::from_str("0").unwrap()),
        gas_premium: TokenAmount::from_atto(BigInt::from_str("0").unwrap()),
        method_num: 2, // Propose is method no 2
        params: MessageParams::ProposeParams(params).serialize().unwrap(),
    };

    let signed_message: MessageTxAPI =
        sign_message(provider, filecoin_ledger_app, &mut message).await?;

    push_mpool_message(provider, signed_message).await?;
    Ok(())
}

pub async fn cancel_payout(
    actor_address: &str,
    provider: &Provider<Http>,
    filecoin_ledger_app: &FilecoinApp<TransportNativeHID>,
    transaction_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let params: TxnIDParams = TxnIDParams {
        id: TxnID(i64::from_str(&transaction_id).unwrap()),
        proposal_hash: vec![],
    };

    let approver_address = filecoin_ledger_app
        .address(&BIP44_PATH, false)
        .await
        .unwrap()
        .addr_string;

    let nonce = get_nonce(&approver_address, provider.clone()).await;

    let mut message = Message {
        version: 0,
        to: FilecoinAddress::from_str(&actor_address)?,
        from: FilecoinAddress::from_str(&approver_address)?,
        sequence: nonce,
        value: TokenAmount::from_atto(BigInt::from_str("0")?),
        gas_limit: 0,
        gas_fee_cap: TokenAmount::from_atto(BigInt::from_str("0")?),
        gas_premium: TokenAmount::from_atto(BigInt::from_str("0")?),
        method_num: 3365893656, // Cancel is method no 3365893656
        params: MessageParams::TxnIDParams(params).serialize()?,
    };

    let signed_message: MessageTxAPI =
        sign_message(provider, filecoin_ledger_app, &mut message).await?;

    push_mpool_message(provider, signed_message).await?;
    Ok(())
}

pub async fn approve_payout(
    actor_address: &str,
    provider: &Provider<Http>,
    filecoin_ledger_app: &FilecoinApp<TransportNativeHID>,
    transaction_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let params: TxnIDParams = TxnIDParams {
        id: TxnID(i64::from_str(&transaction_id).unwrap()),
        proposal_hash: vec![],
    };

    let approver_address = filecoin_ledger_app
        .address(&BIP44_PATH, false)
        .await
        .unwrap()
        .addr_string;

    let nonce = get_nonce(&approver_address, provider.clone()).await;

    let mut message = Message {
        version: 0,
        to: FilecoinAddress::from_str(&actor_address)?,
        from: FilecoinAddress::from_str(&approver_address)?,
        sequence: nonce,
        value: TokenAmount::from_atto(BigInt::from_str("0")?),
        gas_limit: 0,
        gas_fee_cap: TokenAmount::from_atto(BigInt::from_str("0")?),
        gas_premium: TokenAmount::from_atto(BigInt::from_str("0")?),
        method_num: 3, // Approve is method no 3
        params: MessageParams::TxnIDParams(params).serialize()?,
    };

    let signed_message: MessageTxAPI =
        sign_message(provider, filecoin_ledger_app, &mut message).await?;

    push_mpool_message(provider, signed_message).await?;
    Ok(())
}

pub async fn sign_message(
    provider: &Provider<Http>,
    filecoin_ledger_app: &FilecoinApp<TransportNativeHID>,
    message: &mut Message,
) -> Result<MessageTxAPI, Box<dyn std::error::Error>> {
    let gas_info = get_gas_info(message.clone(), provider.clone(), MAX_FEE).await;

    message.gas_limit = gas_info.gas_limit;
    message.gas_fee_cap = gas_info.gas_fee_cap;
    message.gas_premium = gas_info.gas_premium;

    let message_bytes = to_vec(&message)?;
    let signature = filecoin_ledger_app
        .sign(&BIP44_PATH, &message_bytes)
        .await
        .unwrap();
    let sig = signature.sig.to_vec();

    let signature = FilSignature::new_secp256k1(sig);

    let signed_message: SignedMessage = SignedMessage {
        message: Message::default(),
        signature,
    };
    let signed_message = MessageTxAPI::SignedMessage(signed_message);

    Ok(signed_message)
}

pub async fn push_mpool_message(
    provider: &Provider<Http>,
    signed_message: MessageTxAPI,
) -> Result<(), Box<dyn std::error::Error>> {
    let result: Value = provider
        .request::<[MessageTxAPI; 1], Value>("Filecoin.MpoolPush", [signed_message])
        .await?;

    println!("{:#?}", result);
    Ok(())
}

pub async fn inspect_multisig(
    provider: &Provider<Http>,
    actor_id: &str,
) -> Result<MultiSigActor, Box<dyn std::error::Error>> {
    let params: (&str, ()) = (actor_id, ());
    let result: Value = provider
        .request::<(&str, ()), Value>("Filecoin.StateReadState", params)
        .await?;

    let result: MultiSigActor = serde_json::from_value(result)?;

    let mut table = Table::new(vec![result.clone().state].iter());
    table.with(tabled::settings::Style::modern());
    table.with(
        tabled::settings::Modify::new(
            tabled::settings::object::Rows::new(1..)
                .not(tabled::settings::object::Columns::first()),
        )
        .with(tabled::settings::Alignment::center()),
    );
    table.with(tabled::settings::Shadow::new(1));

    let string = format!(
        "\n\n  MultiSig {} with balance {} \n\n{}",
        actor_id,
        result.balance,
        table.to_string()
    );

    info!("{}", string);
    Ok(result)
}

pub async fn claim_earnings<S: ::ethers::providers::Middleware + 'static>(
    client: Arc<S>,
    retries: usize,
    gas_price: U256,
    offset: U256,
    factory_addr: &str,
    addr_to_claim: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = Address::from_str(factory_addr)?;
    let factory = PayoutFactory::new(addr, client.clone());
    let addr_to_claim = check_address_string(addr_to_claim)?;
    let claim_addr = FilAddress {
        data: addr_to_claim.bytes.into(),
    };
    let mut claim_tx = factory.release_all(claim_addr, offset);
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

pub async fn deploy_payout_batch<S: Middleware + 'static>(
    start_index: usize,
    payees: &Vec<FilAddress>,
    shares: &Vec<U256>,
    factory_contract: PayoutFactory<S>,
    client: Arc<S>,
    gas_price: U256,
    retries: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let payouts_size = payees.len();

    if start_index >= payouts_size {
        return Ok(());
    }

    let end_index = if start_index + MAX_PAYEES_PER_PAYOUT >= payouts_size {
        payouts_size
    } else {
        start_index + MAX_PAYEES_PER_PAYOUT
    };
    info!(
        "Deploying payouts with index range {:?} ... {:?}",
        start_index, end_index
    );

    let payees = Vec::from(&payees[start_index..end_index]);
    let shares = Vec::from(&shares[start_index..end_index]);

    let total_sum = shares.clone().iter().fold(U256::from(0), |acc, x| acc + x);

    let mut payout_tx = factory_contract.payout(payees, shares, total_sum);
    let tx = payout_tx.tx.clone();

    let gas_estimate_result = client.estimate_gas(&tx, None).await;
    let gas_estimate = match gas_estimate_result {
        Ok(gas) => gas,
        Err(error) => panic!(
            "Error estimating gas for batch payout at index range {:?} .. {:?}:  {:?}",
            start_index, end_index, error
        ),
    };
    set_tx_gas(&mut payout_tx.tx, gas_estimate, gas_price);

    info!(
        "Estimated batch payout gas cost {:#?}",
        payout_tx.tx.gas().unwrap()
    );

    let receipt_result = send_tx(&payout_tx.tx, client, retries).await;

    let receipt = match receipt_result {
        Ok(receipt) => receipt,
        Err(error) => panic!(
            "Error deploying batch payout at index range {:?} .. {:?}:  {:?}",
            start_index, end_index, error
        ),
    };

    info!(
        "Batch Deployment Successful, TxId: {:?} \n",
        receipt.transaction_hash
    );
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

    let total_sum = shares.clone().iter().fold(U256::from(0), |acc, x| acc + x);

    info!(
        "Total Sum from Payouts: {:?}",
        total_sum.as_u128() as f64 / ATTO_FIL.to_f64().unwrap()
    );
    info!("Total Payee Count: {:?}", payees.len());

    let factory: PayoutFactory<S> = PayoutFactory::new(addr, client.clone());

    let payout_size: i32 = payees.len() as i32;
    let batches = if payout_size % (MAX_PAYEES_PER_PAYOUT as i32) == 0 {
        payout_size / (MAX_PAYEES_PER_PAYOUT as i32)
    } else {
        payout_size / (MAX_PAYEES_PER_PAYOUT as i32) + 1
    };

    info!("Deploying Payouts in {:?} batch deployments \n ", batches);
    for i in 0..(batches as usize) {
        let start_index = i * MAX_PAYEES_PER_PAYOUT;

        let payout_result = deploy_payout_batch(
            start_index,
            &payees,
            &shares,
            factory.clone(),
            client.clone(),
            gas_price,
            retries,
        )
        .await;

        let _ = match payout_result {
            Ok(payout) => payout,
            Err(error) => panic!(
                "Error deploying batch payout at start index range {:?}:  {:?}",
                start_index, error
            ),
        };
    }

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

    let total_sum = shares.clone().iter().fold(U256::from(0), |acc, x| acc + x);

    let call_bytes = factory
        .payout(payees, shares, total_sum)
        .calldata()
        .unwrap()
        .to_vec();

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

pub async fn get_filecoin_ledger() -> FilecoinApp<TransportNativeHID> {
    let hid_api: Lazy<HidApi> = Lazy::new(|| HidApi::new().expect("Failed to create Hidapi"));

    let app =
        FilecoinApp::new(TransportNativeHID::new(&hid_api).expect("unable to create transport"));
    let path = BIP44Path {
        purpose: 0x8000_0000 | 44,
        coin: 0x8000_0000 | 461,
        account: 0,
        change: 0,
        index: 0,
    };
    let addr = app.address(&path, false).await.unwrap();
    info!(
        "Connected to Filecoin Ledger on address: {:#?}",
        addr.addr_string
    );
    app
}
