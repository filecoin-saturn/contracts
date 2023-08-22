use crate::utils::format_date;
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
/// ```ignore
/// use tokio_postgres::{Client};
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
        .port(pg_port)
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

/// Retrieves and aggregates payment information from the `payment_aggregation`
/// table.
pub async fn get_payment_records(date: &str) -> Result<PayoutRecords, Error> {
    let client = connect().await.unwrap();

    let date = format_date(date).unwrap();

    let res = client
        .query(
            "
        SELECT
            fil_wallet_address, sum(fil_earned)
        FROM  payments
        INNER JOIN
            nodes on payments.node_id = nodes.id
            AND core = false
            AND banned_at is NULL
        WHERE
            date_trunc('month',time_stamp)::date =
                date_trunc('month', $1::TIMESTAMP WITH TIME ZONE)::date
        GROUP BY fil_wallet_address
        ORDER BY sum(fil_earned) desc
    ",
            &[&date],
        )
        .await
        .unwrap();

    let payout = format_payout_res(res).unwrap();

    Ok(payout)
}
