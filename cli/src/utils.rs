use std::path::PathBuf;

use contract_bindings::shared_types::FilAddress;
use ethers::types::U256;

use fevm_utils::check_address_string;
use serde::Deserialize;
use tokio_postgres::Error as DbError;

use crate::db::retrieve_payments;
use csv::Error as CsvError;

use once_cell::sync::Lazy;

static ATTO_FIL: Lazy<f64> = Lazy::new(|| 10_f64.powf(18.0));

#[derive(Deserialize, Debug)]
struct Payment {
    payee: String,
    shares: f64,
}

pub async fn parse_payouts_from_csv(
    payout_csv: &PathBuf,
) -> Result<(Vec<FilAddress>, Vec<U256>), CsvError> {
    let mut reader = csv::Reader::from_path(payout_csv)?;
    let mut shares: Vec<U256> = Vec::new();
    let mut payees: Vec<FilAddress> = Vec::new();

    for record in reader.deserialize() {
        let record: Payment = record?;
        let addr = check_address_string(&record.payee).unwrap();

        let payee = FilAddress {
            data: addr.bytes.into(),
        };

        let share: U256 = ((record.shares * &*ATTO_FIL) as u128).into();
        payees.push(payee);
        shares.push(share);
    }

    Ok((payees, shares))
}

pub async fn parse_payouts_from_db() -> Result<(Vec<FilAddress>, Vec<U256>), DbError> {
    let (payees, shares) = retrieve_payments().await.unwrap();

    let payees = payees
        .iter()
        .map(|payee| {
            let addr = check_address_string(&payee).unwrap();
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
