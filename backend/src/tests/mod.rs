use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::db::{schema::*, ScyllaDb};

async fn init() -> ScyllaDb {
    let uri = "127.0.0.1";
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
                scylla_db.new_order_id().await.unwrap(),
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
    scylla_db.new_user(User::new(scylla_db.new_user_id().await.unwrap())).await.unwrap();
    scylla_db.new_trade(Trade::new(1, true, dec!(0.0), dec!(0.0),"SOL_USDT".to_string())).await.unwrap();
}
#[tokio::test]
async fn get_trade() {
        let scylla_db = init().await;
        let trade = Trade::new(1, true, dec!(10.21), dec!(20.1),"SOL_USDT".to_string());
        scylla_db.new_trade(trade).await.unwrap();
        let trade = scylla_db.get_trade(1).await.unwrap();
        assert_eq!(trade.quote_quantity, dec!(10.21) * dec!(20.1) );
}
#[tokio::test]
async fn update_user() {
    let scylla_db = init().await;
    let amount = dec!(20.0);
    let user_id = scylla_db.new_user_id().await.unwrap();
    scylla_db.new_user(User::new(user_id)).await.unwrap();
    let mut user = scylla_db.get_user(user_id).await.unwrap();
    user.deposit(&Asset::SOL, amount);
    scylla_db.update_user(&mut user).await.unwrap();
    let updated_user = scylla_db.get_user(user_id).await.unwrap();

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
    let scylla_db = init().await;
    let order_id = scylla_db.new_order_id().await.unwrap();
    let user_id = scylla_db.new_user_id().await.unwrap();
    let order = Order::new(
        order_id,
        user_id,
        dec!(100.0),
        dec!(1000),
        OrderSide::Ask,
        OrderType::Limit,
        "SOL_USDT".to_string()
    );
    let amount = dec!(90);
    scylla_db.new_order(order).await.unwrap();
    let mut order = scylla_db.get_order(order_id).await.unwrap();
    order.filled_quantity += amount;
    order.order_status = OrderStatus::PartiallyFilled;
    scylla_db.update_order(&mut order).await.unwrap();

    let updated_order = scylla_db.get_order(order_id).await.unwrap();
    let users_order = scylla_db.get_users_orders(user_id).await.unwrap();
    println!("{:#?}", users_order);
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
