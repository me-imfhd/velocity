use std::f32::INFINITY;
use rust_decimal_macros::dec;
use schema::{ Market, Order, OrderSide, OrderType, Ticker, Trade, User };
use super::*;
#[tokio::test]
async fn is_able_to_create_tables() {
    let uri = "127.0.0.1:9042";
    let scylla_db = ScyllaDb::create_session(uri).await.unwrap();
    scylla_db.initialize().await.unwrap();
}
#[tokio::test]
async fn insert_tables_all() {
    let uri = "127.0.0.1:9042";
    let scylla_db = ScyllaDb::create_session(uri).await.unwrap();
    scylla_db.initialize().await.unwrap();
    scylla_db
        .new_market(
            Market::new(
                "SOL_USDT".to_string(),
                Decimal::NEGATIVE_ONE,
                dec!(0.01),
                dec!(0.01),
                Decimal::NEGATIVE_ONE,
                dec!(0.0001),
                dec!(0.0001)
            )
        ).await
        .unwrap();
    scylla_db
        .new_order(
            Order::new(1, dec!(100.0), OrderSide::Ask, OrderType::Limit, "SOL_USDT".to_string())
        ).await
        .unwrap();
    scylla_db.new_ticker(Ticker::new("SOL_USDT".to_string())).await.unwrap();
    scylla_db.new_user(User::new()).await.unwrap();
    scylla_db.new_trade(Trade::new(true, dec!(0.0), dec!(0.0))).await.unwrap();
}
#[tokio::test]
async fn get_user() {
    let uri = "127.0.0.1:9042";
    let scylla_db = ScyllaDb::create_session(uri).await.unwrap();
    scylla_db.initialize().await.unwrap();
    scylla_db.new_user(User::new()).await.unwrap();
    scylla_db.get_user(1).await.unwrap();
}
