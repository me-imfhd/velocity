pub mod schema;
pub mod enums;

use std::sync::atomic::AtomicU64;

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
    pub async fn create_session(uri: &str) -> Result<ScyllaDb> {
        let session = SessionBuilder::new().known_node(uri).build().await.map_err(From::from);
        match session {
            Err(err) => Err(err),
            Ok(session) =>
                Ok(ScyllaDb {
                    session,
                }),
        }
    }
    pub async fn initialize(&self) -> Result<()> {
        self.create_keyspace().await?;
        self.create_user_table().await?;
        self.create_order_table().await?;
        self.create_trade_table().await?;
        self.create_market_table().await?;
        self.create_ticker_table().await?;

        Ok(())
    }
    async fn create_keyspace(&self) -> Result<()> {
        let create_keyspace =
            r#"CREATE KEYSPACE IF NOT EXISTS keyspace_1 
            WITH REPLICATION = 
        {
            'class' : 'NetworkTopologyStrategy', 
            'replication_factor' : 1
        }"#;

        self.session.query(create_keyspace, &[]).await?;
        Ok(())
    }
    async fn create_user_table(&self) -> Result<()> {
        let create_user_table: &str =
            r#"
        CREATE TABLE IF NOT EXISTS keyspace_1.user_table (
            id bigint PRIMARY KEY,
            balance map<text, float>,
            locked_balance map<text, float>
        );
      "#;
        self.session.query(create_user_table, &[]).await?;
        Ok(())
    }
    async fn create_order_table(&self) -> Result<()> {
        let create_order_table: &str =
            r#"
        CREATE TABLE IF NOT EXISTS keyspace_1.order_table (
            id bigint PRIMARY KEY,
            user_id bigint,
            symbol text,
            initial_quantity float,
            filled_quantity float, 
            order_type text,
            order_side text,
            order_status text,
            timestamp bigint
        );
      "#;
        self.session.query(create_order_table, &[]).await?;
        Ok(())
    }
    async fn create_trade_table(&self) -> Result<()> {
        let create_trade_table: &str =
            r#"
        CREATE TABLE IF NOT EXISTS keyspace_1.trade_table (
            id bigint PRIMARY KEY,
            quantity float,
            quote_quantity float,
            is_market_maker boolean,
            price float,
            timestamp bigint
        );
      "#;
        self.session.query(create_trade_table, &[]).await?;
        Ok(())
    }
    async fn create_market_table(&self) -> Result<()> {
        let create_market_table: &str =
            r#"
        CREATE TABLE IF NOT EXISTS keyspace_1.market_table (
            symbol text PRIMARY KEY,
            base text,
            quote text,
            max_price float,
            min_price float,
            tick_size float,
            max_quantity float,
            min_quantity float,
            step_size float
        );
      "#;
        self.session.query(create_market_table, &[]).await?;
        Ok(())
    }
    async fn create_ticker_table(&self) -> Result<()> {
        let create_ticker_table: &str =
            r#"
        CREATE TABLE IF NOT EXISTS keyspace_1.ticker_table (
            symbol text PRIMARY KEY,
            base_volume float,
            quote_volume float,
            price_change float,
            price_change_percent float,
            high_price float,
            low_price float,
            last_price float
        );
      "#;
        self.session.query(create_ticker_table, &[]).await?;
        Ok(())
    }
}

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
