use chrono::{offset::Utc, DateTime, Datelike, NaiveDate};
use dotenv::dotenv;
use rust_decimal::prelude::{Decimal, ToPrimitive};
use std::env;
use tokio_postgres::{Client, Config, Error, NoTls, Row};

/// Creates a new postgres database connection and returns a Postgres Client.
///
/// Requires the following environment variables to be setup:
///     PG_PASSWORD - Password of the postgres user that is used to connect
///	    PG_HOST - The host for postgres (eg 127.0.0.1)
///     PG_DATABASE - Database name using to connect.
///	    PG_PORT - Port that the database is connected to (eg 5432)
///	    PG_USER - Username of the postgres user that is used to connect
///
/// Usage:
/// ```no_run
/// use tokio_postgres::{Client}
/// use db::{connect};
///
/// fn connect_example() {
///     let client: Client = connect().await.unwrap();
///     let query_result = client.query("SELECT * from table", &[]).await.unwrap();
/// }
///
/// ```
///
async fn connect() -> Result<Client, Error> {
    dotenv().ok();
    let pg_pass = env::var("PG_PASSWORD").expect("PG_PASSWORD must be set");
    let pg_host = env::var("PG_HOST").expect("PG_HOST must be set");
    let pg_db = env::var("PG_DATABASE").expect("PG_DATABASE must be set");
    let pg_port: u16 = env::var("PG_PORT")
        .expect("PG_PORT must be set")
        .parse()
        .unwrap();
    let pg_user = env::var("PG_USER").expect("PG_USER must be set");

    let (client, connection) = Config::new()
        .password(pg_pass)
        .host(pg_host.as_str())
        .dbname(pg_db.as_str())
        .port(pg_port.into())
        .user(pg_user.as_str())
        .application_name("Saturn Contract Payments")
        .connect(NoTls)
        .await
        .unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    Ok(client)
}

#[derive(Debug, Clone)]
pub struct PayoutRecords {
    pub payees: Vec<String>,
    pub shares: Vec<f64>,
}

/// Formats a vector tokio_postgres `Row` type to native rust types.
///
/// Specifically, the postgres `Row` must be formatted such that:
///     - The first index is a postgres text/char type.
///     - The second row is a postgres numeric/int (any variant) / float type.
///
fn format_payout_res(res: Vec<Row>) -> Result<PayoutRecords, Error> {
    let mut payees: Vec<String> = Vec::new();
    let mut shares: Vec<f64> = Vec::new();

    for row in res {
        let payee: String = row.get(0);
        let share: Decimal = row.get(1);

        payees.push(payee.to_string());
        shares.push(share.to_f64().unwrap());
    }
    Ok(PayoutRecords { payees, shares })
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
fn format_date(date: &str) -> Result<DateTime<Utc>, Error> {
    let date = NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap();
    let naive_datetime = date.and_hms_opt(0, 0, 0);
    let date = match naive_datetime {
        None => panic!("Error parsing date"),
        Some(naive_datetime) => DateTime::<Utc>::from_utc(naive_datetime, Utc),
    };
    Ok(date)
}

/// Retrieves and aggregates payment information from the `payment_aggregation`
/// table.
pub async fn get_payment_records(
    date: &str,
    include_cassini: bool,
) -> Result<PayoutRecords, Error> {
    let client = connect().await.unwrap();

    let date = format_date(date).unwrap();

    let res = client
        .query(
            "
        SELECT
            fil_wallet_address, sum(fil_earned)
        FROM  payment_aggregation
        INNER JOIN
            nodes on payment_aggregation.node_id = nodes.id
            AND core = false
            AND cassini = $1
            AND banned_at is NULL
        WHERE
            date_trunc('month',time_stamp)::date =
                date_trunc('month', $2::TIMESTAMP WITH TIME ZONE)::date
        GROUP BY fil_wallet_address
        ORDER BY sum(fil_earned) desc
    ",
            &[&include_cassini, &date],
        )
        .await
        .unwrap();

    let payout = format_payout_res(res).unwrap();

    Ok(payout)
}

/// Generates the final monthly payout for each month.
///
/// Currently includes a final row with the factory address and total for cassini members.
pub async fn get_payment_records_for_finance(
    date: &str,
    factory_address: &str,
) -> Result<PayoutRecords, Error> {
    let client = connect().await.unwrap();
    let mut node_payouts = get_payment_records(date, true).await.unwrap();
    let date = format_date(date).unwrap();

    let res = client
        .query(
            "
            SELECT
                sum(fil_earned)
            FROM  payment_aggregation
            INNER JOIN
                nodes on payment_aggregation.node_id = nodes.id
                AND core = false
                AND cassini = true
                AND banned_at is NULL
            WHERE
                date_trunc('month',time_stamp)::date =
                    date_trunc('month', $1::TIMESTAMP WITH TIME ZONE)::date
            ",
            &[&date],
        )
        .await
        .unwrap();

    assert_eq!(res.len(), 1);
    let cassini_payout: Decimal = res[0].get(0);

    node_payouts.payees.push(factory_address.to_string());
    node_payouts.shares.push(cassini_payout.to_f64().unwrap());

    Ok(node_payouts)
}
