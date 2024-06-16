pub mod schema;
pub mod enums;
pub mod user;
pub mod trade;
pub mod order;
pub mod ticker;
pub mod market;
pub mod scylla_init;
use std::{ sync::atomic::AtomicU64, time::{ SystemTime, UNIX_EPOCH } };

use bigdecimal::{ FromPrimitive, ToPrimitive };
use rust_decimal::Decimal;
use schema::{ MarketSchema, OrderSchema, TickerSchema, TradeSchema, UserSchema };
use scylla::{ Session, SessionBuilder };

use crate::result::Result;

pub struct ScyllaDb {
    pub session: Session,
}
pub static ORDER_ID: AtomicU64 = AtomicU64::new(0);
pub static TRADE_ID: AtomicU64 = AtomicU64::new(0);
pub static USER_ID: AtomicU64 = AtomicU64::new(0);

impl ScyllaDb {
    pub async fn new_user(&self, user: UserSchema) -> Result<()> {
        let s =
            r#"
            INSERT INTO keyspace_1.user_table (
                id,
                balance,
                locked_balance
            ) VALUES (?, ?, ?);
        "#;
        let res = self.session.query(s, user).await?;
        Ok(())
    }
    pub async fn new_order(&self, order: OrderSchema) -> Result<()> {
        let s =
            r#"
            INSERT INTO keyspace_1.order_table (
                id,
                user_id,
                symbol,
                initial_quantity,
                filled_quantity, 
                order_type,
                order_side,
                order_status,
                timestamp
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?);
        "#;
        let res = self.session.query(s, order).await?;
        Ok(())
    }
    pub async fn new_trade(&self, trade: TradeSchema) -> Result<()> {
        let s =
            r#"
            INSERT INTO keyspace_1.trade_table (
                id,
                quantity,
                quote_quantity,
                is_market_maker,
                price,
                timestamp
            ) VALUES (?, ?, ?, ?, ?, ?);
        "#;
        let res = self.session.query(s, trade).await?;
        Ok(())
    }
    pub async fn new_market(&self, market: MarketSchema) -> Result<()> {
        let s =
            r#"
            INSERT INTO keyspace_1.market_table (
                symbol,
                base,
                quote,
                max_price,
                min_price,
                tick_size,
                max_quantity,
                min_quantity,
                step_size
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?);
        "#;
        let res = self.session.query(s, market).await?;
        Ok(())
    }
    pub async fn new_ticker(&self, ticker: TickerSchema) -> Result<()> {
        let s =
            r#"
            INSERT INTO keyspace_1.ticker_table (
                symbol,
                base_volume,
                quote_volume,
                price_change,
                price_change_percent,
                high_price,
                low_price,
                last_price
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?);
        "#;
        let res = self.session.query(s, ticker).await?;
        Ok(())
    }
}
pub fn get_epoch_ms() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
}
pub fn from_f32(val: f32) -> Decimal {
    Decimal::from_f32(val).unwrap()
}
pub fn to_f32(val: &Decimal) -> f32 {
    Decimal::to_f32(val).unwrap()
}
pub fn add(first: f32, second: f32) -> f32 {
    let res = from_f32(first) + from_f32(second);
    to_f32(&res)
}
pub fn sub(from: f32, by: f32) -> f32 {
    let res = from_f32(from) - from_f32(by);
    to_f32(&res)
}
pub fn mul(first: f32, second: f32) -> f32 {
    let res = from_f32(first) * from_f32(second);
    to_f32(&res)
}
#[cfg(test)]
pub mod tests {
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
    async fn insert_in_tables() {
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
                OrderSchema::new(
                    1,
                    100.0,
                    OrderSideEn::Ask,
                    OrderTypeEn::Limit,
                    "SOL_USDT".to_string()
                )
            ).await
            .unwrap();
        scylla_db.new_ticker(TickerSchema::new("SOL_USDT".to_string())).await.unwrap();
        scylla_db.new_user(UserSchema::new()).await.unwrap();
        scylla_db.new_trade(TradeSchema::new(true, 0.0, 0.0)).await.unwrap();
    }
}
