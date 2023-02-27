use std::sync::Arc;
use ::ethers::contract::Contract;
use ethers::abi::AbiEncode;
use ethers::core::k256::{ecdsa::SigningKey, elliptic_curve::sec1::ToEncodedPoint, PublicKey};
use ethers::core::{types::Address, utils::keccak256};
use ethers::signers::Signer;
use ethers::signers::{coins_bip39::English, MnemonicBuilder};
use ethers::types::{Bytes, H160, U256};
use ethers::{
    prelude::{Middleware, SignerMiddleware},
    providers::{Http, Provider},
    signers::Wallet,
};
use eyre::Result;
use serde_json::ser;
use std::fs;
use log::{debug, info};


const DEFAULT_DERIVATION_PATH_PREFIX: &str = "m/44'/60'/0'/0/";

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

    info!("Wallet key we use: {:#?}", wallet);

    let private_key = U256::from_big_endian(wallet.signer().to_bytes().as_slice());

    info!("Private key we use: {:#?}", private_key);

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
    // provider.for_chain(Chain::try_from(3141));
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

