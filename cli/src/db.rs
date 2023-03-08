use chrono::offset::Utc;
use dotenv::dotenv;
use rust_decimal::prelude::{Decimal, ToPrimitive};
use std::env;
use tokio_postgres::{Client, Config, Error, NoTls};
use uuid::Uuid;

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

pub async fn retrieve_payments() -> Result<(Vec<String>, Vec<u128>), Error> {
    let client = connect().await.unwrap();

    let date = Utc::now();
    println!("{:#?}", date);
    let res = client
        .query(
            "
       SELECT * from payments
       WHERE time_stamp <= $1::TIMESTAMP WITH TIME ZONE
       LIMIT 10
    ",
            &[&date],
        )
        .await
        .unwrap();

    let mut payees: Vec<String> = Vec::new();
    let mut shares: Vec<u128> = Vec::new();
    for row in res {
        let payee: Uuid = row.get(1);
        let share: Decimal = row.get(3);

        payees.push(payee.to_string());
        shares.push(share.to_u128().unwrap());
    }

    println!("{:#?}", payees);
    println!("{:#?}", shares);
    Ok((payees, shares))
}
