use chrono::offset::Utc;
use dotenv::dotenv;
use rust_decimal::prelude::{Decimal, ToPrimitive};
use std::env;
use tokio_postgres::{Client, Config, Error, NoTls};

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

pub async fn retrieve_payments() -> Result<(Vec<String>, Vec<f64>), Error> {
    let client = connect().await.unwrap();

    let date = Utc::now();
    let res = client
        .query(
            "
       SELECT fil_wallet_address, fil_earned from payments
       INNER JOIN
            nodes on payments.node_id = nodes.id
            AND cassini = true
            AND core = false
            AND banned_at is NULL
       WHERE time_stamp <= $1::TIMESTAMP WITH TIME ZONE
    ",
            &[&date],
        )
        .await
        .unwrap();

    let mut payees: Vec<String> = Vec::new();
    let mut shares: Vec<f64> = Vec::new();
    for row in res {
        let payee: String = row.get(0);
        let share: Decimal = row.get(1);

        payees.push(payee.to_string());
        shares.push(share.to_f64().unwrap());
    }
    Ok((payees, shares))
}
