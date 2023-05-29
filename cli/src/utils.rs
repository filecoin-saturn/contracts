use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use std::process::Command;

use chrono::{DateTime, Datelike, Month, NaiveDate, Utc};
use contract_bindings::shared_types::FilAddress;
use ethers::abi::AbiDecode;
use ethers::types::{Eip1559TransactionRequest, U256};

use csv::{Error as CsvError, Writer};
use extras::json::tokenamount;
use extras::signed_message::ref_fvm::SignedMessage;
use fevm_utils::filecoin_to_eth_address;
use fil_actor_multisig::{ProposeParams, TxnID, TxnIDParams};
use filecoin_signer::api::{MessageParams, MessageTxAPI};
use filecoin_signer::{transaction_sign, PrivateKey};
use fvm_ipld_encoding::to_vec;
use fvm_ipld_encoding::RawBytes;
use fvm_shared::address::{set_current_network, Address as FilecoinAddress, SECP_PUB_LEN};
use fvm_shared::bigint::BigInt;
use fvm_shared::crypto::signature::Signature as FilSignature;
use fvm_shared::econ::TokenAmount;
use fvm_shared::message::Message;
use ledger_filecoin::{BIP44Path, FilecoinApp};
use ledger_transport_hid::{hidapi::HidApi, TransportNativeHID};
use libsecp256k1::{PublicKey, SecretKey};
use rpassword::read_password;
use serde_json::Value;

use once_cell::sync::Lazy;

use contract_bindings::payout_factory_native_addr::PayoutFactoryNativeAddr as PayoutFactory;
use ethers::abi::Address;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::middleware::SignerMiddleware;
use ethers::providers::{Http, JsonRpcClient, Middleware, Provider};
use ethers::signers::Wallet;
use ethers::types::transaction::eip2718::TypedTransaction;
use fevm_utils::{check_address_string, get_wallet_signing_provider, send_tx, set_tx_gas};
use log::{debug, error, info};
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use std::io::{self, Write};
use std::str::FromStr;
use std::sync::Arc;
use tabled::{settings::object::Object, Table, Tabled};
use url::Url;

const ADMIN_ROLE: [u8; 32] = [0; 32];

const LOTUS_RPC_URL: &str = "http://127.0.0.1:1234/rpc/v1";
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Tabled)]
#[serde(rename_all = "PascalCase")]
pub struct MultiSigTransaction {
    #[serde(rename = "ID")]
    pub id: u64,
    pub to: String,
    pub value: String,
    pub method: u64,
    #[tabled(display_with = "display_option")]
    pub params: Option<String>,
    #[tabled(display_with = "display_vector")]
    pub approved: Vec<String>,
}

fn display_vector<T: std::fmt::Debug>(v: &Vec<T>) -> String {
    if !v.is_empty() {
        format!("{:?}", v)
    } else {
        String::new()
    }
}
fn display_option<T: std::fmt::Debug>(v: &Option<T>) -> String {
    if let Some(x) = v {
        format!("{:?}", x)
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
    // The purpose of the 0x8000_0000 is to add the apostrophe(') in a BipPath
    purpose: 44 | 0x8000_0000,
    coin: 461 | 0x8000_0000,
    account: 0 | 0x8000_0000,
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

pub fn parse_payouts(payees: &Vec<String>, shares: &Vec<f64>) -> (Vec<FilAddress>, Vec<U256>) {
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

    (payees, shares)
}

/// Parses payouts from a csv file.
///
/// CSV file formatted as such:
///    Recipient,FIL
///    f1...,5
pub async fn parse_raw_payouts_from_csv(
    payout_csv: &PathBuf,
) -> Result<(Vec<String>, Vec<f64>), CsvError> {
    let mut reader = csv::Reader::from_path(payout_csv)?;
    let mut shares: Vec<f64> = Vec::new();
    let mut payees: Vec<String> = Vec::new();

    for record in reader.deserialize() {
        let record: Payment = record?;
        payees.push(record.Recipient);
        shares.push(record.FIL);
    }
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
        csv_writer.write_record(&[
            payee.to_string(),
            share.to_string(),
            String::from("0"),
            String::from("nil"),
        ])?;
    }
    csv_writer.flush()?;

    Ok(())
}

pub async fn get_signing_method_and_address(
    method: &SigningOptions,
) -> Result<(SignatureMethod, String), Box<dyn Error>> {
    let signing_method;

    match method {
        SigningOptions::Ledger => {
            let filecoin_ledger_app = get_filecoin_ledger().await;

            let address = filecoin_ledger_app
                .address(&BIP44_PATH, false)
                .await
                .unwrap()
                .addr_string;

            signing_method = SignatureMethod::LedgerApp(filecoin_ledger_app);

            info!("Signing with address: {:?}", address.clone());

            Ok((signing_method, address))
        }
        SigningOptions::Lotus => {
            let token = get_lotus_signing_token().await.unwrap();

            let url: Url = Url::parse(LOTUS_RPC_URL).unwrap();
            let lotus_node_provider =
                Http::new_with_auth(url, ethers::providers::Authorization::bearer(token.trim()))
                    .unwrap();

            let address = lotus_node_provider
                .request::<(), String>("Filecoin.WalletDefaultAddress", ())
                .await?;

            signing_method = SignatureMethod::Lotus(lotus_node_provider, address.clone());

            Ok((signing_method, address))
        }
        SigningOptions::Local => {
            info!(
                "Insert your private key to sign (it will not be displayed for security reasons): ",
            );

            let _ = io::stdout().flush().unwrap();
            let mut private_key = read_password().unwrap();
            private_key = String::from(private_key.trim());

            let private_key_res = PrivateKey::try_from(private_key);

            let private_key = match private_key_res {
                Ok(private_key) => private_key,
                Err(err) => panic!("Error parsing private key: {:?}", err),
            };

            let secret_key = SecretKey::parse_slice(&private_key.0)?;
            let public_key = PublicKey::from_secret_key(&secret_key);
            let address = FilecoinAddress::new_secp256k1(&public_key.serialize().to_vec()).unwrap();

            signing_method = SignatureMethod::PrivateKey(private_key);
            info!("Signing with address: {:?}", address.to_string());

            Ok((signing_method, address.to_string()))
        }
    }
}

pub async fn propose_payout_batch(
    actor_address: &str,
    receiver_address: &str,
    payees: &Vec<String>,
    shares: &Vec<f64>,
    start_index: usize,
    provider: &Provider<Http>,
    rpc_url: &str,
    signature_method: &SignatureMethod,
    signer_address: &str,
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
        "Proposing payouts with index range {:?} ... {:?}",
        start_index, end_index
    );

    let payees = Vec::from(&payees[start_index..end_index]);
    let shares = Vec::from(&shares[start_index..end_index]);

    let (parsed_payees, parsed_shares) = parse_payouts(&payees, &shares);

    let factory_addr_eth = filecoin_to_eth_address(&receiver_address, &rpc_url).await?;

    let propose_call_data = propose_new_payout_callbytes(
        Arc::new(provider.clone()),
        &factory_addr_eth,
        parsed_payees,
        parsed_shares,
    )
    .unwrap();

    let params: ProposeParams = ProposeParams {
        to: FilecoinAddress::from_str(&receiver_address).unwrap(),
        // no transfer of value
        value: TokenAmount::from_atto(BigInt::from_str("0").unwrap()),
        method: fil_actor_evm::Method::InvokeContract as u64,
        params: RawBytes::new(propose_call_data),
    };

    let nonce = get_nonce(&signer_address, provider.clone()).await;

    let mut message = Message {
        version: 0,
        to: FilecoinAddress::from_str(&actor_address).unwrap(),
        from: FilecoinAddress::from_str(&signer_address).unwrap(),
        sequence: nonce,
        value: TokenAmount::from_atto(BigInt::from_str("0").unwrap()),
        gas_limit: 0,
        gas_fee_cap: TokenAmount::from_atto(BigInt::from_str("0").unwrap()),
        gas_premium: TokenAmount::from_atto(BigInt::from_str("0").unwrap()),
        method_num: 2, // Propose is method no 2
        params: MessageParams::ProposeParams(params).serialize().unwrap(),
    };

    let signed_message_result = sign_message(provider, signature_method, &mut message).await;
    let signed_message = match signed_message_result {
        Ok(message) => message,
        Err(error) => {
            let date = chrono::offset::Utc::now().to_string();
            let file_path = PathBuf::from(&format!("./FailedPayouts{}", date));
            write_payout_csv(&file_path, &payees, &shares).unwrap();
            panic!(
                "Error signing multisig propose message for batch payout at index range {:?} .. {:?}:  {:?}",
                start_index, end_index, error
            )
        }
    };

    let mpool_push_result: Result<(), Box<dyn Error>> =
        push_mpool_message(provider, signed_message).await;

    let _ = match mpool_push_result {
        Ok(mpool_push) => mpool_push,
        Err(error) => {
            let date = chrono::offset::Utc::now().to_string();
            let file_path = PathBuf::from(&format!("./FailedPayouts{}", date));
            write_payout_csv(&file_path, &payees, &shares).unwrap();
            panic!(
                "MpoolPush error for proposing batch payout at index range {:?} .. {:?}:  {:?}",
                start_index, end_index, error
            )
        }
    };
    Ok(())
}

pub async fn cancel_payout(
    actor_address: &str,
    provider: &Provider<Http>,
    transaction_id: &str,
    signing_method: &SignatureMethod,
    signing_address: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let params: TxnIDParams = TxnIDParams {
        id: TxnID(i64::from_str(&transaction_id).unwrap()),
        proposal_hash: vec![],
    };

    let nonce = get_nonce(&signing_address, provider.clone()).await;

    let mut message = Message {
        version: 0,
        to: FilecoinAddress::from_str(&actor_address)?,
        from: FilecoinAddress::from_str(&signing_address)?,
        sequence: nonce,
        value: TokenAmount::from_atto(BigInt::from_str("0")?),
        gas_limit: 0,
        gas_fee_cap: TokenAmount::from_atto(BigInt::from_str("0")?),
        gas_premium: TokenAmount::from_atto(BigInt::from_str("0")?),
        method_num: 3365893656, // Cancel is method no 3365893656
        params: MessageParams::TxnIDParams(params).serialize()?,
    };

    let signed_message: MessageTxAPI = sign_message(provider, signing_method, &mut message).await?;

    push_mpool_message(provider, signed_message).await?;
    Ok(())
}

pub async fn claim_earnings_filecoin_signing(
    provider: &Provider<Http>,
    factory_addr: &str,
    release_address: &str,
    signing_method: &SignatureMethod,
    signing_address: &str,
    rpc_url: &str,
) -> Result<(), Box<dyn Error>> {
    let factory_eth_addr = filecoin_to_eth_address(factory_addr, rpc_url)
        .await
        .unwrap();

    let addr = Address::from_str(factory_eth_addr.as_str()).unwrap();
    let release_addr = FilAddress {
        data: check_address_string(release_address).unwrap().bytes.into(),
    };

    let client = Arc::new(provider.clone());
    let factory = PayoutFactory::new(addr, client);

    let call_bytes = factory
        .release_all(release_addr, U256::from(1))
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
    params.extend(call_bytes.clone());

    let nonce = get_nonce(&signing_address, provider.clone()).await;

    let mut message = Message {
        version: 0,
        to: FilecoinAddress::from_str(&factory_addr)?,
        from: FilecoinAddress::from_str(&signing_address)?,
        sequence: nonce,
        value: TokenAmount::from_atto(BigInt::from_str("0")?),
        gas_limit: 0,
        gas_fee_cap: TokenAmount::from_atto(BigInt::from_str("0")?),
        gas_premium: TokenAmount::from_atto(BigInt::from_str("0")?),
        method_num: 3844450837, // InvokeContract is method no 3844450837
        params: RawBytes::new(params),
    };

    let signed_message: MessageTxAPI = sign_message(provider, signing_method, &mut message).await?;

    push_mpool_message(provider, signed_message).await?;
    Ok(())
}

pub async fn approve_payout(
    actor_address: &str,
    provider: &Provider<Http>,
    signing_method: &SignatureMethod,
    signing_address: &str,
    transaction_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let params: TxnIDParams = TxnIDParams {
        id: TxnID(i64::from_str(&transaction_id).unwrap()),
        proposal_hash: vec![],
    };

    let nonce = get_nonce(&signing_address, provider.clone()).await;

    let mut message = Message {
        version: 0,
        to: FilecoinAddress::from_str(&actor_address)?,
        from: FilecoinAddress::from_str(&signing_address)?,
        sequence: nonce,
        value: TokenAmount::from_atto(BigInt::from_str("0")?),
        gas_limit: 0,
        gas_fee_cap: TokenAmount::from_atto(BigInt::from_str("0")?),
        gas_premium: TokenAmount::from_atto(BigInt::from_str("0")?),
        method_num: 3, // Approve is method no 3
        params: MessageParams::TxnIDParams(params).serialize()?,
    };

    let signed_message: MessageTxAPI = sign_message(provider, signing_method, &mut message).await?;

    push_mpool_message(provider, signed_message).await?;
    Ok(())
}

pub enum SignatureMethod {
    LedgerApp(FilecoinApp<TransportNativeHID>),
    PrivateKey(PrivateKey),
    Lotus(Http, String),
}

#[derive(Debug, Serialize, Deserialize, Clone, clap::ValueEnum)]
pub enum SigningOptions {
    Lotus,
    Ledger,
    Local,
}

pub async fn sign_message(
    provider: &Provider<Http>,
    signature_method: &SignatureMethod,
    message: &mut Message,
) -> Result<MessageTxAPI, Box<dyn std::error::Error>> {
    let gas_info = get_gas_info(message.clone(), provider.clone(), MAX_FEE).await;

    message.gas_limit = gas_info.gas_limit;
    message.gas_fee_cap = gas_info.gas_fee_cap;
    message.gas_premium = gas_info.gas_premium;

    let message_bytes = to_vec(&message).unwrap();

    let signed_message: MessageTxAPI;

    match signature_method {
        SignatureMethod::LedgerApp(ledger_app) => {
            let signature = ledger_app.sign(&BIP44_PATH, &message_bytes).await.unwrap();
            let recovery_id = signature.v;
            let mut sig = signature.sig.to_vec();
            sig.push(recovery_id);

            let signature = FilSignature::new_secp256k1(sig);

            signed_message = MessageTxAPI::SignedMessage(SignedMessage {
                message: message.clone(),
                signature,
            });
        }
        SignatureMethod::PrivateKey(private_key) => {
            signed_message =
                MessageTxAPI::SignedMessage(transaction_sign(&message, &private_key).unwrap());
        }
        SignatureMethod::Lotus(provider, signer_address) => {
            let message_tx: MessageTxAPI = MessageTxAPI::Message(message.clone());
            signed_message = provider
                .request::<(&str, MessageTxAPI), MessageTxAPI>(
                    "Filecoin.WalletSignMessage",
                    (signer_address, message_tx),
                )
                .await?;
        }
    }
    Ok(signed_message)
}

/// Generates a signing token from a locus local node. The token is used to sign
/// messages using the lotus node.
pub async fn get_lotus_signing_token() -> Result<String, Box<dyn Error>> {
    let output = Command::new("lotus")
        .arg("auth")
        .arg("create-token")
        .arg("--perm")
        .arg("sign")
        .output()
        .expect("Failed to extract signing token from lotus");

    if !output.status.success() {
        panic!("Failed to extract signing token from lotus");
    }

    let token = String::from_utf8_lossy(&output.stdout).to_string();

    Ok(token)
}

pub async fn push_mpool_message(
    provider: &Provider<Http>,
    signed_message: MessageTxAPI,
) -> Result<(), Box<dyn std::error::Error>> {
    let result: Value = provider
        .request::<[MessageTxAPI; 1], Value>("Filecoin.MpoolPush", [signed_message])
        .await?;

    info!("{:#?}", result);
    Ok(())
}

pub async fn get_pending_transaction_multisig(
    provider: &Provider<Http>,
    actor_id: &str,
) -> Result<Vec<MultiSigTransaction>, Box<dyn std::error::Error>> {
    let params: (&str, ()) = (actor_id, ());
    let result: Value = provider
        .request::<(&str, ()), Value>("Filecoin.MsigGetPending", params)
        .await?;

    let result: Vec<MultiSigTransaction> = serde_json::from_value(result)?;

    Ok(result)
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

    let pending_tx = get_pending_transaction_multisig(provider, actor_id).await?;

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

    for tx in pending_tx.iter() {
        let mut table = Table::new(vec![tx.clone()].iter());
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
            "\n\n  Pending Transaction {} \n\n{}",
            tx.id,
            table.to_string()
        );

        info!("{}", string);

        if let Some(params) = &tx.params {
            let params: String =
                RawBytes::from(base64::decode(params.as_bytes())?).deserialize()?;
            let params =
                contract_bindings::payout_factory_native_addr::PayoutFactoryNativeAddrCalls::decode(
                    &params.as_bytes(),
                );
            match params {
                Ok(params) => {
                    debug!("human readable params {:#?}", params);
                }
                Err(_) => {
                    error!("could not parse params");
                }
            }
        }
    }

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

pub async fn grant_admin<S: ::ethers::providers::Middleware + 'static>(
    client: Arc<S>,
    retries: usize,
    gas_price: U256,
    factory_addr: &str,
    address_to_grant: &str,
    rpc_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = Address::from_str(factory_addr)?;
    let factory: PayoutFactory<_> = PayoutFactory::new(addr, client.clone());
    let address_to_grant = filecoin_to_eth_address(address_to_grant, rpc_url)
        .await
        .unwrap();
    let address_to_grant = Address::from_str(address_to_grant.as_str())?;

    let mut claim_tx = factory.grant_role(ADMIN_ROLE.into(), address_to_grant);
    let tx = claim_tx.tx.clone();
    set_tx_gas(
        &mut claim_tx.tx,
        client.estimate_gas(&tx, None).await?,
        gas_price,
    );

    info!("estimated grant gas cost {:#?}", claim_tx.tx.gas().unwrap());

    send_tx(&claim_tx.tx, client, retries).await?;
    Ok(())
}

pub async fn propose_payout(
    actor_address: &str,
    receiver_address: &str,
    date: &str,
    db_deploy: &bool,
    payout_csv: &Option<PathBuf>,
    provider: &Provider<Http>,
    rpc_url: &str,
    signature_method: SignatureMethod,
    signer_address: &str,
) -> Result<(), Box<dyn Error>> {
    let (payees, shares) = get_payout_data(db_deploy, &payout_csv, date, receiver_address)
        .await
        .unwrap();

    let total_sum = shares.clone().iter().fold(0_f64, |acc, x| acc + x);

    info!("Total Sum from Payouts: {:?}", total_sum);
    info!("Total Payee Count: {:?}", payees.len());

    let payout_size: i32 = payees.len() as i32;
    let batches = if payout_size % (MAX_PAYEES_PER_PAYOUT as i32) == 0 {
        payout_size / (MAX_PAYEES_PER_PAYOUT as i32)
    } else {
        payout_size / (MAX_PAYEES_PER_PAYOUT as i32) + 1
    };

    info!("Proposing Payouts in {:?} batch deployments \n ", batches);
    for i in 0..(batches as usize) {
        let start_index = i * MAX_PAYEES_PER_PAYOUT;
        let propose_result = propose_payout_batch(
            actor_address,
            receiver_address,
            &payees.clone(),
            &shares.clone(),
            start_index,
            provider,
            rpc_url,
            &signature_method,
            signer_address,
        )
        .await;

        let _ = match propose_result {
            Ok(payout) => payout,
            Err(error) => panic!(
                "Error proposing batch payout at start index range {:?}:  {:?}",
                start_index, error
            ),
        };
    }
    Ok(())
}

// Deploys a PaymentSplitter batch of node operator payouts.
pub async fn deploy_payout_batch<S: Middleware + 'static>(
    start_index: usize,
    payees: &Vec<String>,
    shares: &Vec<f64>,
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

    let (parsed_payees, parsed_shares) = parse_payouts(&payees, &shares);

    let total_sum = parsed_shares
        .clone()
        .iter()
        .fold(U256::from(0), |acc, x| acc + x);

    let mut payout_tx = factory_contract.payout(parsed_payees, parsed_shares, total_sum);
    let tx = payout_tx.tx.clone();

    let gas_estimate_result = client.estimate_gas(&tx, None).await;
    let gas_estimate = match gas_estimate_result {
        Ok(gas) => gas,
        Err(error) => {
            let date = chrono::offset::Utc::now().to_string();
            let file_path = PathBuf::from(&format!("./FailedPayouts{}", date));
            write_payout_csv(&file_path, &payees, &shares).unwrap();
            panic!(
                "Error estimating gas for batch payout at index range {:?} .. {:?}:  {:?}",
                start_index, end_index, error
            )
        }
    };
    set_tx_gas(&mut payout_tx.tx, gas_estimate, gas_price);

    info!(
        "Estimated batch payout gas cost {:#?}",
        payout_tx.tx.gas().unwrap()
    );

    let receipt_result = send_tx(&payout_tx.tx, client, retries).await;

    let receipt = match receipt_result {
        Ok(receipt) => receipt,
        Err(error) => {
            let date = chrono::offset::Utc::now().to_string();
            let file_path = PathBuf::from(&format!("./FailedPayouts{}", date));
            write_payout_csv(&file_path, &payees, &shares).unwrap();
            panic!(
                "Error deploying batch payout at index range {:?} .. {:?}:  {:?}",
                start_index, end_index, error
            )
        }
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

    let (payees, shares) = get_payout_data(db_deploy, &payout_csv, date, factory_addr)
        .await
        .unwrap();

    let total_sum = shares.clone().iter().fold(0_f64, |acc, x| acc + x);

    info!("Total Sum from Payouts: {:?}", total_sum);
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

async fn get_payout_data(
    db_deploy: &bool,
    csv_path: &Option<PathBuf>,
    date: &str,
    factory_addr: &str,
) -> Result<(Vec<String>, Vec<f64>), Box<dyn Error>> {
    if *db_deploy {
        let db_payout_records = get_payment_records_for_finance(date, factory_addr)
            .await
            .unwrap();
        return Ok((db_payout_records.payees, db_payout_records.shares));
    } else {
        let (payees, shares) = match csv_path {
            Some(csv_path) => parse_raw_payouts_from_csv(csv_path).await.unwrap(),
            None => {
                panic!("Either payout-csv or db-deployment must be defined as CLI args");
            }
        };
        return Ok((payees, shares));
    }
}

pub fn propose_new_payout_callbytes<S: Middleware + 'static>(
    client: Arc<S>,
    factory_addr: &str,
    payees: Vec<FilAddress>,
    shares: Vec<U256>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let addr = Address::from_str(factory_addr)?;
    let factory = PayoutFactory::new(addr, client.clone());

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
    params.extend(call_bytes.clone());

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

    let addr = app.address(&BIP44_PATH, false).await.unwrap();
    info!(
        "Connected to Filecoin Ledger on address: {:#?}",
        addr.addr_string
    );
    app
}

pub fn random_filecoin_address(testnet: bool) -> Result<String, Box<dyn Error>> {
    let mut rng = ethers::prelude::rand::thread_rng();
    let mut bytes = [0u8; SECP_PUB_LEN];
    ethers::prelude::rand::Rng::fill(&mut rng, &mut bytes[..]);
    let addr = FilecoinAddress::new_secp256k1(&bytes)?;
    if testnet {
        set_current_network(fvm_shared::address::Network::Testnet);
    }
    Ok(addr.to_string())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_random_filecoin_address() {
        for _i in 0..100 {
            let res = super::random_filecoin_address(false);
            assert!(res.is_ok());
        }

        // generate for testnet
        for _i in 0..100 {
            let res = super::random_filecoin_address(true);
            assert!(res.is_ok());
        }

        const PAYOUT: &str = "Recipient,FIL\nt1ypi542zmmgaltijzw4byonei5c267ev5iif2liy,0.01\n";
        let mut global_payout = PAYOUT.to_string();
        for _i in 0..400 {
            let random_payee = super::random_filecoin_address(true).unwrap();
            let amount = "0.0001";
            let payout_str = format!("{},{}\n", random_payee, amount);
            global_payout = format!("{}{}", global_payout, payout_str);
        }
    }
}
