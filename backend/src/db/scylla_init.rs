use scylla::SessionBuilder;

use crate::result::Result;

use super::ScyllaDb;

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
