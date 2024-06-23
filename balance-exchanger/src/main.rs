use balance_exchanger::{ QueueTrade, ScyllaDb };
use redis::{ Connection, Value };
use serde_json::{ from_str, to_string };

#[tokio::main]
async fn main() {
    let uri = "127.0.0.1";
    let redis_uri = "redis://127.0.0.1:6379";
    let mut con = connect_redis(&redis_uri);
    let scylla_db = ScyllaDb::create_session(uri).await.unwrap();
    loop {
        let con = &mut con;
        let result = redis::cmd("RPOP").arg("queues:trade").query::<String>(con);
        match result {
            Ok(queue_trade_string) => {
                let queue_trade: QueueTrade = from_str(&queue_trade_string).unwrap();
                let result = scylla_db.exchange_balances(queue_trade).await;
                match result {
                    Ok(trade) => {
                        let trade_string = to_string(&trade).unwrap();
                        redis
                            ::cmd("PUBLISH")
                            .arg(format!("trades:{}", trade.symbol))
                            .arg(trade_string)
                            .query::<Value>(con)
                            .unwrap();
                        println!("Balance Exchanged, Orders Updated, Trade Id : {}", trade.id);
                    }
                    Err(err) => {
                        dbg!(err);
                        redis
                            ::cmd("LPUSH")
                            .arg("queues:trade")
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
