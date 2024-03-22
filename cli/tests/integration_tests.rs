use assert_cmd::prelude::*;
use assert_fs::fixture::FileWriteStr;
use assert_fs::NamedTempFile;
use cli::utils::{random_filecoin_address, ATTO_FIL};
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
    file.write_str(MNEMONIC.as_str()).unwrap();
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
    let out_data = String::from_utf8(output.stderr).unwrap();
    println!("{}", out_data);
    let addr = extract_contract_addr(out_data.as_str());
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

    let mut global_payout = PAYOUT.to_string();
    for _i in 0..5000 {
        let random_payee = random_filecoin_address(true)?;
        let amount = "0.0001";
        let payout_str = format!("{},{}\n", random_payee, amount);
        global_payout = format!("{}{}", global_payout, payout_str);
    }

    let payouts_csv = assert_fs::NamedTempFile::new("payout.csv")?;
    payouts_csv.write_str(&global_payout)?;

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
        RECIPIENT_ADDRESS,
        "--offset",
        "0",
    ];
    args.append(&mut new_payout_args);

    let mut cmd = Command::cargo_bin("saturn-contracts")?;
    cmd.args(&args);
    cmd.output().ok();
    Ok(())
}

#[test]
fn cli_5_grant_admin() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = get_const_cli_args();

    let factory_addr = &FACTORY_ADDRESS.lock().unwrap();
    let mut new_payout_args = vec![
        "grant-admin",
        "--factory-addr",
        factory_addr,
        "--address",
        RECIPIENT_ADDRESS,
    ];
    args.append(&mut new_payout_args);

    let mut cmd = Command::cargo_bin("saturn-contracts")?;
    cmd.args(&args);
    cmd.output().ok();
    Ok(())
}

#[test]
fn cli_6_revoke_admin() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = get_const_cli_args();

    let factory_addr = &FACTORY_ADDRESS.lock().unwrap();
    let mut new_payout_args = vec![
        "revoke-admin",
        "--factory-addr",
        factory_addr,
        "--address",
        RECIPIENT_ADDRESS,
    ];
    args.append(&mut new_payout_args);

    let mut cmd = Command::cargo_bin("saturn-contracts")?;
    cmd.args(&args);
    cmd.output().ok();
    Ok(())
}
