// Note this useful idiom: importing names from outer (for mod tests) scope.
// use super::*;
const WASM_COMPILED_PATH: &str = "./build/tests/PayoutFactoryNativeAddr.bin";
const ABI_PATH: &str = "./build/tests/PayoutFactoryNativeAddr.abi";

use std::error::Error;

use ethabi::ParamType;
use ethers::abi::{Address, Token};
use fevm_utils::executor::{Contract, TestExecutor};

fn deploy_payout_factory() -> Contract {
    let addr = "0x0000000000000000000000000000000000000000"
        .parse::<Address>()
        .unwrap();

    let addr = Token::Address(addr);

    let mut test_executor = TestExecutor::new().unwrap();
    let mut contract = test_executor
        .deploy(WASM_COMPILED_PATH, ABI_PATH, Some(&[addr]))
        .unwrap();

    test_executor
        .call_fn(&mut contract, "totalShares", &[])
        .unwrap();

    let call = contract.last_call();
    let res = call
        .decode_return_data(&vec![ParamType::Uint(256)])
        .unwrap()[0]
        .clone();
    println!("{:?}", res);

    contract
}

#[test]
fn deployment() {
    deploy_payout_factory();
}

// #[test]
// fn new_payout() {
//     let mut test_executor = TestExecutor::new().unwrap();
//     let contract = deploy_payout_factory();
//     let payee = "0x0000000000000000000000000000000000000001"
//         .parse::<Address>()
//         .unwrap();

//     let addr = Token::Address(addr);
//     let share =
// }
