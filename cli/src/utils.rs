use std::error::Error;
use std::path::PathBuf;

use chrono::DateTime;
use chrono::NaiveDate;
use chrono::Utc;
use contract_bindings::shared_types::FilAddress;
use ethers::types::U256;

use fevm_utils::check_address_string;
use serde::Deserialize;
use tokio_postgres::Error as DbError;

use crate::db::get_payment_records_for_finance;
use crate::db::PayoutRecords;
use csv::{Error as CsvError, Writer};

use once_cell::sync::Lazy;

static ATTO_FIL: Lazy<f64> = Lazy::new(|| 10_f64.powf(18.0));

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct Payment {
    Recipient: String,
    FIL: f64,
}

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
