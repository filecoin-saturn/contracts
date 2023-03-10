use ::ethers::contract::Contract;
use blake2::digest::{Update, VariableOutput};
use blake2::Blake2bVar;
use csv::Error as CsvError;
use ethers::abi::AbiEncode;
use ethers::core::k256::{ecdsa::SigningKey, elliptic_curve::sec1::ToEncodedPoint, PublicKey};
use ethers::core::{types::Address, utils::keccak256};
use ethers::signers::Signer;
use ethers::signers::{coins_bip39::English, MnemonicBuilder};
use ethers::types::transaction::eip2718::TypedTransaction;
use ethers::types::{Bytes, TransactionReceipt, H160, H256, U256};
use ethers::{
    prelude::{Middleware, SignerMiddleware},
    providers::{Http, Provider},
    signers::Wallet,
};
use leb128 as leb;
use log::{debug, info};
use serde::Deserialize;
use serde_json::ser;
use std::fmt::Write;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio_postgres::Error as DbError;

use crate::db::retrieve_payments;

const DEFAULT_DERIVATION_PATH_PREFIX: &str = "m/44'/60'/0'/0/";
const GAS_LIMIT_MULTIPLIER: i32 = 130;
const ATTO_FIL: u128 = 10_u128.pow(18);

// The hash length used for calculating address checksums.
const CHECKSUM_HASH_LENGTH: usize = 4;

// The maximum length of `int64` as a string.
const MAX_INT64_STRING_LENGTH: usize = 19;

// The maximum length of a delegated address's sub-address.
const MAX_SUBADDRESS_LEN: usize = 54;

const ETH_ADDRESS_LENGTH: usize = 20;

enum CoinType {
    MAIN,
    TEST,
}

impl CoinType {
    fn possible_values() -> [char; 2] {
        ['f', 't']
    }
}

impl From<char> for CoinType {
    fn from(a: char) -> Self {
        match a {
            'f' => CoinType::MAIN,
            't' => CoinType::TEST,
            _ => panic!(),
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
enum Protocol {
    ID = 0,
    SECP256K1 = 1,
    ACTOR = 2,
    BLS = 3,
    DELEGATED = 4,
}

impl Protocol {
    fn possible_values() -> [u64; 5] {
        [0, 1, 2, 3, 4]
    }
}

impl From<u64> for Protocol {
    fn from(a: u64) -> Self {
        match a {
            0 => Protocol::ID,
            1 => Protocol::SECP256K1,
            2 => Protocol::ACTOR,
            3 => Protocol::BLS,
            4 => Protocol::DELEGATED,
            _ => panic!(),
        }
    }
}

pub struct AddressData {
    protocol: Protocol,
    payload: Vec<u8>,
}

#[derive(thiserror::Error, Debug)]
pub enum AddressError {
    #[error(
        "Address cointype should be one of: {:#?}",
        CoinType::possible_values()
    )]
    InvalidCointype,
    #[error(
        "Address protocol should be one of: {:#?}",
        Protocol::possible_values()
    )]
    InvalidProtocol,
    #[error("invalid address")]
    InvalidAddress,
    #[error("invalid base32")]
    InvalidBase32,
    #[error("invalid leb128")]
    InvalidLeb128,
    #[error("invalid checksum")]
    InvalidChecksum,
    #[error("can only convert delegated addresses to ETH")]
    OnlyConvertDelegated,
}

#[derive(thiserror::Error, Debug)]
pub enum CLIError {
    #[error(
        "did not receive receipt, but check a hyperspace explorer to check if tx was successful (hash: ${0})"
    )]
    NoReceipt(H256),
    #[error("contract failed to deploy")]
    ContractNotDeployed,
}

#[derive(Deserialize, Debug)]
struct Payment {
    payee: String,
    shares: u128,
}

/// Sets gas for a constructed tx
pub fn set_tx_gas(tx: &mut TypedTransaction, gas_estimate: U256, gas_price: U256) {
    let gas_estimate = gas_estimate * GAS_LIMIT_MULTIPLIER / 100;
    tx.set_gas(gas_estimate);
    tx.set_gas_price(gas_price);
}

/// Sends a constructed tx
pub async fn send_tx(
    tx: &TypedTransaction,
    client: SignerMiddleware<Arc<Provider<Http>>, Wallet<SigningKey>>,
    retries: usize,
) -> Result<TransactionReceipt, Box<dyn std::error::Error>> {
    let pending_tx = client.send_transaction(tx.clone(), None).await?;

    let hash = pending_tx.tx_hash();
    let receipt = pending_tx.retries(retries).await?;
    if receipt.is_some() {
        let receipt = receipt.unwrap();
        debug!("call receipt: {:#?}", receipt);
        Ok(receipt)
    } else {
        Err(Box::new(CLIError::NoReceipt(hash)))
    }
}

fn derive_key(mnemonic: &str, path: &str, index: u32) -> Result<U256, Bytes> {
    let derivation_path = if path.ends_with('/') {
        format!("{path}{index}")
    } else {
        format!("{path}/{index}")
    };

    let wallet = MnemonicBuilder::<English>::default()
        .phrase(mnemonic)
        .derivation_path(&derivation_path)
        .map_err(|err| err.to_string().encode())?
        .build()
        .map_err(|err| err.to_string().encode())?;

    info!("wallet address: {:#?}", wallet.address());

    let private_key = U256::from_big_endian(wallet.signer().to_bytes().as_slice());

    Ok(private_key)
}

fn parse_private_key(private_key: U256) -> Result<SigningKey, Bytes> {
    if private_key.is_zero() {
        return Err("Private key cannot be 0.".to_string().encode().into());
    }
    let mut bytes: [u8; 32] = [0; 32];
    private_key.to_big_endian(&mut bytes);
    SigningKey::from_bytes(&bytes).map_err(|err| err.to_string().encode().into())
}

fn secret_key_to_address(secret_key: &SigningKey) -> Address {
    let public_key = PublicKey::from(&secret_key.verifying_key());
    let public_key = public_key.to_encoded_point(/* compress = */ false);
    let public_key = public_key.as_bytes();
    debug_assert_eq!(public_key[0], 0x04);
    let hash = keccak256(&public_key[1..]);

    let mut bytes = [0u8; 20];
    bytes.copy_from_slice(&hash[12..]);
    Address::from(bytes)
}

pub fn addr(mnemonic: &str) -> Result<H160, Bytes> {
    let private_key = derive_key(mnemonic, DEFAULT_DERIVATION_PATH_PREFIX, 0).unwrap();
    let key = parse_private_key(private_key)?;
    let addr = secret_key_to_address(&key);
    Ok(addr)
}

fn get_signing_wallet(private_key: U256, chain_id: u64) -> Wallet<SigningKey> {
    let private_key = parse_private_key(private_key).unwrap();
    let wallet: Wallet<ethers::core::k256::ecdsa::SigningKey> = private_key.into();

    wallet.with_chain_id(chain_id)
}

pub async fn get_signing_provider(
    mnemonic: &str,
    rpc_url: &str,
) -> SignerMiddleware<Arc<Provider<Http>>, Wallet<SigningKey>> {
    let provider =
        Provider::<Http>::try_from(rpc_url).expect("could not instantiate HTTP Provider");
    debug!("{:#?}", provider);
    let chain_id = provider.get_chainid().await.unwrap();
    let private_key = derive_key(mnemonic, DEFAULT_DERIVATION_PATH_PREFIX, 0).unwrap();
    let signing_wallet = get_signing_wallet(private_key, chain_id.as_u64());

    let provider = Arc::new(provider);

    SignerMiddleware::new(provider, signing_wallet)
}

pub fn write_abi(contract: Contract<SignerMiddleware<Arc<Provider<Http>>, Wallet<SigningKey>>>) {
    let abi = contract.abi();
    let string_abi = ser::to_string(abi).unwrap();
    fs::write("./factoryAbi.json", string_abi).expect("Unable to write file");
}

pub fn parse_payouts_from_csv(payout_csv: &PathBuf) -> Result<(Vec<Address>, Vec<U256>), CsvError> {
    let mut reader = csv::Reader::from_path(payout_csv)?;
    let mut shares: Vec<U256> = Vec::new();
    let mut payees: Vec<Address> = Vec::new();
    for record in reader.deserialize() {
        let record: Payment = record?;
        let payee = filecoin_to_eth_address(record.payee.as_str()).unwrap();
        let payee = payee.parse::<Address>().unwrap();
        let share: U256 = (record.shares * ATTO_FIL).into();
        payees.push(payee);
        shares.push(share);
    }

    Ok((payees, shares))
}

pub async fn parse_payouts_from_db() -> Result<(Vec<Address>, Vec<U256>), DbError> {
    let (payees, shares) = retrieve_payments().await.unwrap();

    let payees: Vec<Address> = payees
        .iter()
        .map(|payee| {
            let eth_address = filecoin_to_eth_address(payee).unwrap();
            eth_address.parse::<Address>().unwrap()
        })
        .collect();

    let shares: Vec<U256> = shares
        .iter()
        .map(|share| U256::try_from(share * ATTO_FIL).unwrap())
        .collect();

    Ok((payees, shares))
}

fn validate_checksum(bytes: &[u8], checksum_bytes: &[u8]) -> bool {
    let mut hasher = Blake2bVar::new(CHECKSUM_HASH_LENGTH).unwrap();
    hasher.update(bytes);
    let mut buf = [0u8; CHECKSUM_HASH_LENGTH];
    hasher.finalize_variable(&mut buf).unwrap();
    buf == checksum_bytes
}

fn check_address_string(address: &str) -> Result<AddressData, AddressError> {
    info!("converting {} to ETH equivalent", address);
    let base32_alphabet = base32::Alphabet::RFC4648 { padding: false };
    if address.len() < 3 {
        return Err(AddressError::InvalidAddress);
    }

    let coin_type = address.chars().nth(0).unwrap();
    if !CoinType::possible_values().contains(&coin_type) {
        return Err(AddressError::InvalidCointype);
    }

    let protocol = address.chars().nth(1).unwrap();
    // It works because the ASCII (and thus UTF-8) encodings have the Arabic numerals 0-9 ordered in ascending order.
    // You can get the scalar values and subtract them.
    let protocol = protocol as u64 - '0' as u64;
    if !Protocol::possible_values().contains(&protocol) {
        return Err(AddressError::InvalidProtocol);
    }
    let protocol = (protocol as u64).into();

    let raw = &address[2..];
    println!("{:#?} adr", protocol);
    let addr = match protocol {
        Protocol::ID => {
            if raw.len() > MAX_INT64_STRING_LENGTH {
                return Err(AddressError::InvalidAddress);
            }
            if raw.parse::<u64>().is_err() {
                return Err(AddressError::InvalidAddress);
            }
            let mut buf: [u8; 6] = [0; 6];
            let payload_num_bytes = {
                let mut writable = &mut buf[..];
                println!("raw parsed {}", raw.parse::<u64>().unwrap());
                leb::write::unsigned(&mut writable, raw.parse::<u64>().unwrap())
                    .map_err(|_| AddressError::InvalidLeb128)?
            };
            AddressData {
                protocol,
                payload: buf[..payload_num_bytes].to_vec(),
            }
        }
        Protocol::DELEGATED => {
            let split_index = raw.find('f').unwrap();
            let namespace_str = &raw[..split_index];
            if namespace_str.len() > MAX_INT64_STRING_LENGTH {
                return Err(AddressError::InvalidAddress);
            }
            let sub_addr_cksm_str = &raw[split_index + 1..];
            let sub_addr_cksm_bytes = base32::decode(base32_alphabet, sub_addr_cksm_str)
                .ok_or(AddressError::InvalidBase32)?;
            if sub_addr_cksm_bytes.len() < CHECKSUM_HASH_LENGTH {
                return Err(AddressError::InvalidAddress);
            }
            let sub_addr_bytes =
                &sub_addr_cksm_bytes[..sub_addr_cksm_bytes.len() - CHECKSUM_HASH_LENGTH];
            let checksum_bytes = &sub_addr_cksm_bytes[sub_addr_bytes.len()..];
            if sub_addr_bytes.len() > MAX_SUBADDRESS_LEN {
                return Err(AddressError::InvalidAddress);
            }
            let mut protocol_buf: [u8; 1024] = [0; 1024];
            let protocol_byte_num = {
                let mut writable = &mut protocol_buf[..];
                leb::write::unsigned(&mut writable, protocol.clone() as u64)
                    .map_err(|_| AddressError::InvalidLeb128)?
            };
            if protocol_byte_num != 1 {
                return Err(AddressError::InvalidLeb128);
            }
            let protocol_byte = protocol_buf[0..protocol_byte_num].to_vec();

            let mut namespace_buf: [u8; 1024] = [0; 1024];
            let namespace_number = namespace_str.parse::<u64>().unwrap();
            let namespace_byte_num = {
                let mut writable = &mut namespace_buf[..];
                leb::write::unsigned(&mut writable, namespace_number)
                    .map_err(|_| AddressError::InvalidLeb128)?
            };
            if namespace_byte_num != 1 {
                return Err(AddressError::InvalidLeb128);
            }
            let namespace_byte = namespace_buf[0..namespace_byte_num].to_vec();

            let bytes = [
                protocol_byte.as_slice(),
                namespace_byte.as_slice(),
                sub_addr_bytes,
            ]
            .concat();

            if !validate_checksum(&bytes, checksum_bytes) {
                return Err(AddressError::InvalidChecksum);
            }
            let namespace_buf = namespace_number.to_be_bytes();
            let payload = [&namespace_buf, sub_addr_bytes].concat();

            AddressData { protocol, payload }
        }
        _ => {
            unimplemented!()
        }
    };
    Ok(addr)
}

/// Converts a filecoin address to a corresponding ETH address
///
///```
/// use cli::utils::filecoin_to_eth_address;
///

/// // test ID type addresses
/// let addr = "t01";
/// assert_eq!(filecoin_to_eth_address(addr).unwrap(), "0xff00000000000000000000000000000000000001");
/// let addr = "t0100";
/// assert_eq!(filecoin_to_eth_address(addr).unwrap(), "0xff00000000000000000000000000000000000064");
/// let addr = "t05088";
/// assert_eq!(filecoin_to_eth_address(addr).unwrap(), "0xff000000000000000000000000000000000013e0");
///
/// // test delegated addresses
/// let addr = "t410fkkld55ioe7qg24wvt7fu6pbknb56ht7pt4zamxa";
/// assert_eq!(filecoin_to_eth_address(addr).unwrap(), "0x52963ef50e27e06d72d59fcb4f3c2a687be3cfef");
///

/// ```
///
pub fn filecoin_to_eth_address(address: &str) -> Result<String, AddressError> {
    let address_data = check_address_string(address)?;
    let addr_buffer = if matches!(address_data.protocol, Protocol::DELEGATED) {
        let sub_addr = &address_data.payload[8..];
        sub_addr.to_vec()
    } else if matches!(address_data.protocol, Protocol::ID) {
        let id = leb::read::unsigned(&mut &address_data.payload[..])
            .map_err(|_| AddressError::InvalidLeb128)?;
        let mut addr_buffer: Vec<u8> = vec![0; ETH_ADDRESS_LENGTH];
        addr_buffer[0] = 255_u8.to_be_bytes()[0];
        let id_bytes = &id.to_be_bytes()[0..8];
        for i in 12..20 {
            addr_buffer[i] = id_bytes[i - 12];
        }
        addr_buffer
    } else {
        unimplemented!()
    };
    let mut s = String::with_capacity(ETH_ADDRESS_LENGTH * 2);
    write!(&mut s, "0x").unwrap();
    for b in addr_buffer {
        write!(&mut s, "{:02x}", b).unwrap();
    }

    info!("ETH equivalent is {}", s);

    Ok(s)
}

pub fn banner() {
    info!(
        "{}",
        format!(
            "
            _|_|_|              _|
            _|          _|_|_|  _|_|_|_|  _|    _|  _|  _|_|  _|_|_|
              _|_|    _|    _|    _|      _|    _|  _|_|      _|    _|
                  _|  _|    _|    _|      _|    _|  _|        _|    _|
            _|_|_|      _|_|_|      _|_|    _|_|_|  _|        _|    _|

        -----------------------------------------------------------
        Saturn smart contracts 🪐.
        -----------------------------------------------------------
        "
        )
    );
}
