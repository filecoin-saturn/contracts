const WASM_COMPILED_PATH: &str = "./build/tests/PayoutFactoryNativeAddr.bin";
const ABI_PATH: &str = "./build/tests/PayoutFactoryNativeAddr.abi";

use std::error::Error;

use ethabi::{ethereum_types::U256, ParamType};
use ethers::abi::{Address, Token};
use fevm_utils::executor::{Contract, FilAddress, TestExecutor};
use fvm_shared::econ::TokenAmount;

fn compile_contracts() -> Result<(), Box<dyn Error>> {
    let mut cmd = std::process::Command::new("solc");
    cmd.arg("solidity-cborutils=../lib/filecoin-solidity/node_modules/solidity-cborutils/");
    cmd.arg("@ensdomains=../lib/filecoin-solidity/node_modules/@ensdomains/");
    cmd.arg("../src/PayoutFactoryNativeAddr.sol");
    cmd.arg("--output-dir=./build/tests");
    cmd.arg("--overwrite");
    cmd.arg("--bin");
    cmd.arg("--hashes");
    cmd.arg("--opcodes");
    cmd.arg("--abi");
    cmd.arg("--allow-paths=../lib");
    cmd.status()?;
    Ok(())
}

async fn id_to_eth(id: u64) -> Token {
    let admin = Token::Address(Address::from(
        &hex::decode(
            fevm_utils::filecoin_to_eth_address(&format!("t0{}", id), "")
                .await
                .unwrap()
                .replace("0x", ""),
        )
        .unwrap()
        .try_into()
        .unwrap(),
    ));
    admin
}

fn id_to_bytes(id: u64) -> Vec<u8> {
    let id = format!("t0{}", id);
    let address_data = fevm_utils::check_address_string(&id).unwrap();
    address_data.bytes
}

async fn deploy_payout_factory(executor: &mut TestExecutor) -> Contract {
    let admin_id = executor.current_sender().0;
    let admin = id_to_eth(admin_id).await;
    let mut contract = executor
        .deploy(WASM_COMPILED_PATH, ABI_PATH, Some(&[admin]))
        .unwrap();

    executor.call_fn(&mut contract, "totalShares", &[]).unwrap();

    let call = contract.last_call();
    let res = call
        .decode_return_data(&vec![ParamType::Uint(256)])
        .unwrap()[0]
        .clone();

    assert_eq!(res, Token::Uint(0.into()));

    contract
}

#[tokio::test]
async fn deployment() {
    compile_contracts().unwrap();
    let mut test_executor = TestExecutor::new().unwrap();
    deploy_payout_factory(&mut test_executor).await;
}

#[tokio::test]
async fn new_payout() {
    colog::init();

    let mut test_executor = TestExecutor::new().unwrap();
    let mut contract = deploy_payout_factory(&mut test_executor).await;
    let payees = (1..10)
        .map(|idx| {
            FilAddress::new(id_to_bytes(test_executor.get_account(idx).unwrap().0)).to_eth_token()
        })
        .collect::<Vec<Token>>();
    let shares = (1..10)
        .map(|_| Token::Uint(U256::from(1)))
        .collect::<Vec<Token>>();

    test_executor
        .send_funds(contract.address, TokenAmount::from_atto(10))
        .unwrap();

    test_executor
        .call_fn(
            &mut contract,
            "payout",
            &[Token::Array(payees), Token::Array(shares)],
        )
        .unwrap();

    let call = contract.last_call();

    assert!(call.result.msg_receipt.exit_code.is_success());
}
