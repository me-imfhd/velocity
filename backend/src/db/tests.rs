use std::{ f32::INFINITY, ops::Deref };
use rust_decimal_macros::dec;
use schema::{ Asset, Market, Order, OrderSide, OrderStatus, OrderType, Ticker, Trade, User };
use super::*;

async fn init() -> ScyllaDb {
    let uri = "127.0.0.1:9042";
    let scylla_db = ScyllaDb::create_session(uri).await.unwrap();
    scylla_db.initialize().await.unwrap();
    scylla_db
}
#[tokio::test]
async fn is_able_to_create_tables() {
    init().await;
}
#[tokio::test]
async fn insert_tables_all() {
    let scylla_db = init().await;
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
            Order::new(
                1,
                dec!(100.0),
                dec!(1000),
                OrderSide::Ask,
                OrderType::Limit,
                "SOL_USDT".to_string()
            )
        ).await
        .unwrap();
    scylla_db.new_ticker(Ticker::new("SOL_USDT".to_string())).await.unwrap();
    scylla_db.new_user(User::new()).await.unwrap();
    scylla_db.new_trade(Trade::new(true, dec!(0.0), dec!(0.0))).await.unwrap();
}
#[tokio::test]
async fn get_trade() {
    let trade = Trade::new(true, dec!(10.21), dec!(20.1));
    let scylla_db = init().await;
    scylla_db.new_trade(trade).await.unwrap();
    let mut trade = scylla_db.get_trade(1).await.unwrap();
    assert_eq!(trade.quote_quantity, dec!(10.21) * dec!(20.1) );
}
#[tokio::test]
async fn update_user() {
    let scylla_db = init().await;
    let amount = dec!(20.0);
    scylla_db.new_user(User::new()).await.unwrap();
    let mut user = scylla_db.get_user(1).await.unwrap();
    user.deposit(&Asset::SOL, amount);
    scylla_db.update_user(&mut user).await.unwrap();
    let updated_user = scylla_db.get_user(1).await.unwrap();

    let updated_balance = *updated_user.balance.get(&Asset::SOL).unwrap();
    assert_eq!(updated_balance, amount);
}
#[tokio::test]
async fn update_market() {
    let market = Market::new(
        "SOL_USDT".to_string(),
        Decimal::NEGATIVE_ONE,
        dec!(0.01),
        dec!(0.01),
        Decimal::NEGATIVE_ONE,
        dec!(0.0001),
        dec!(0.0001)
    );
    let scylla_db = init().await;
    let amount = dec!(210);
    scylla_db.new_market(market).await.unwrap();
    let mut market = scylla_db.get_market("SOL_USDT".to_string()).await.unwrap();
    market.min_price += amount;
    scylla_db.update_market(&mut market).await.unwrap();
    let updated_market = scylla_db.get_market("SOL_USDT".to_string()).await.unwrap();

    let updated_balance = updated_market.min_price;
    assert_eq!(updated_balance, amount + dec!(0.01));
}
#[tokio::test]
async fn update_order() {
    let order = Order::new(
        1,
        dec!(100.0),
        dec!(1000),
        OrderSide::Ask,
        OrderType::Limit,
        "SOL_USDT".to_string()
    );
    let scylla_db = init().await;
    let amount = dec!(90);
    scylla_db.new_order(order).await.unwrap();
    let mut order = scylla_db.get_order(1).await.unwrap();
    order.filled_quantity += amount;
    order.order_status = OrderStatus::PartiallyFilled;
    scylla_db.update_order(&mut order).await.unwrap();

    let updated_order = scylla_db.get_order(1).await.unwrap();
    assert_eq!(updated_order.order_status.to_string(), OrderStatus::PartiallyFilled.to_string());
}

#[tokio::test]
async fn update_ticker() {
    let ticker = Ticker::new("SOL_USDT".to_string());
    let scylla_db = init().await;
    let amount = dec!(210);
    scylla_db.new_ticker(ticker).await.unwrap();
    let mut ticker = scylla_db.get_ticker("SOL_USDT".to_string()).await.unwrap();
    ticker.last_price = amount;
    scylla_db.update_ticker(&mut ticker).await.unwrap();

    let updated_ticker = scylla_db.get_ticker("SOL_USDT".to_string()).await.unwrap();
    assert_eq!(updated_ticker.last_price, ticker.last_price);
}
