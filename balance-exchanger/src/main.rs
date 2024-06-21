use balance_exchanger::{ QueueTrade, ScyllaDb };
use redis::Connection;
use serde_json::from_str;

#[tokio::main]
async fn main() {
    let uri = "127.0.0.1:9042";
    let redis_uri = "redis://127.0.0.1:6379";
    let mut con = connect_redis(&redis_uri);
    let scylla_db = ScyllaDb::create_session(uri).await.unwrap();
    loop {
        let con = &mut con;
        let result = redis::cmd("RPOP").arg("queues:trade").query::<String>(con);
        match result {
            Ok(queue_trade_string) => {
                let queue_trade: QueueTrade = from_str(&queue_trade_string).unwrap();
                let _ = scylla_db.exchange_balances(queue_trade).await;
            }
            Err(_) => {
                println!("No balances to exchange");
            }
        }
    }
}

fn connect_redis(url: &str) -> Connection {
    let client = redis::Client::open(url).expect("Could not create client.");
    let connection = client.get_connection().expect("Could not connect to the client");
    return connection;
}
