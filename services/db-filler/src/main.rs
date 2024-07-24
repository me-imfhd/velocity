use std::time::{ Duration, Instant };

use db_filler::{ Filler, ScyllaDb };
use redis::{ Connection, Value };
use serde_json::from_str;

#[tokio::main]
async fn main() {
    let uri = "127.0.0.1";
    let redis_uri = "redis://127.0.0.1:6379";
    let mut con = connect_redis(&redis_uri);
    let scylla_db = ScyllaDb::create_session(uri).await.unwrap();
    loop {
        let con = &mut con;
        let result = redis::cmd("RPOP").arg("filler").query::<String>(con);
        match result {
            Ok(queue_trade_string) => {
                let queue_trade: Filler = from_str(&queue_trade_string).unwrap();
                let start = Instant::now();
                let result = scylla_db.batch_update(queue_trade).await;
                match result {
                    Ok(trade) => {
                        println!(
                            "Balance Exchanged, Orders Updated, Trade Id : {} in {} ms",
                            trade.id,
                            start.elapsed().as_millis()
                        );
                    }
                    Err(err) => {
                        eprintln!("{}", err);
                        tokio::time::sleep(Duration::from_millis(30)).await; // wait for order to be saved, to avoid unnecessary refilling the queue
                        redis
                            ::cmd("RPUSH")
                            .arg("filler")
                            .arg(queue_trade_string)
                            .query::<Value>(con)
                            .unwrap();
                    }
                }
            }
            Err(_) => {
                // println!("No balances to exchange");
            }
        }
    }
}

fn connect_redis(url: &str) -> Connection {
    let client = redis::Client::open(url).expect("Could not create client.");
    let connection = client.get_connection().expect("Could not connect to the client");
    return connection;
}
