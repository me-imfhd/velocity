use std::f32::INFINITY;

use super::{
    enums::{ OrderSideEn, OrderTypeEn },
    schema::{ MarketSchema, OrderSchema, TickerSchema, TradeSchema, UserSchema },
    ScyllaDb,
};

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
            MarketSchema::new(
                "SOL_USDT".to_string(),
                INFINITY,
                0.01,
                0.01,
                INFINITY,
                0.0001,
                0.0001
            )
        ).await
        .unwrap();
    scylla_db
        .new_order(
            OrderSchema::new(1, 100.0, OrderSideEn::Ask, OrderTypeEn::Limit, "SOL_USDT".to_string())
        ).await
        .unwrap();
    scylla_db.new_ticker(TickerSchema::new("SOL_USDT".to_string())).await.unwrap();
    scylla_db.new_user(UserSchema::new()).await.unwrap();
    scylla_db.new_trade(TradeSchema::new(true, 0.0, 0.0)).await.unwrap();
}
#[tokio::test]
async fn get_user() {
    let uri = "127.0.0.1:9042";
    let scylla_db = ScyllaDb::create_session(uri).await.unwrap();
    scylla_db.initialize().await.unwrap();
    scylla_db.new_user(UserSchema::new()).await.unwrap();
    scylla_db.get_user(1).await.unwrap();
}
