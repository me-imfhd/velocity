use scylla::SessionBuilder;

use crate::result::Result;

use super::ScyllaDb;
use std::collections::HashMap;

use scylla::{ FromRow, SerializeRow };
use serde::{ Deserialize, Serialize };

pub type Id = i64;
pub type Symbol = String;
pub type Asset = String;
pub type OrderType = String;
pub type OrderSide = String;
pub type OrderStatus = String;

impl ScyllaDb {
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
            balance map<text, text>,
            locked_balance map<text, text>
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
            initial_quantity text,
            filled_quantity text, 
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
            quantity text,
            quote_quantity text,
            is_market_maker boolean,
            price text,
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
            max_price text,
            min_price text,
            tick_size text,
            max_quantity text,
            min_quantity text,
            step_size text
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
            base_volume text,
            quote_volume text,
            price_change text,
            price_change_percent text,
            high_price text,
            low_price text,
            last_price text
        );
      "#;
        self.session.query(create_ticker_table, &[]).await?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, SerializeRow)]
pub struct ScyllaOrder {
    pub id: Id,
    pub user_id: Id,
    pub symbol: Symbol,
    pub initial_quantity: String,
    pub filled_quantity: String,
    pub order_type: OrderType,
    pub order_side: OrderSide,
    pub order_status: OrderStatus,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, SerializeRow, FromRow)]
pub struct ScyllaUser {
    pub id: Id,
    pub balance: HashMap<Asset, String>,
    pub locked_balance: HashMap<Asset, String>,
}
#[derive(Debug, Serialize, Deserialize, SerializeRow)]
pub struct ScyllaTrade {
    pub id: Id,
    pub quantity: String,
    pub quote_quantity: String,
    pub is_market_maker: bool,
    pub price: String,
    pub timestamp: i64,
}

#[derive(Debug, Deserialize, Serialize, SerializeRow)]
pub struct ScyllaTicker {
    pub symbol: Symbol,
    pub base_volume: String,
    pub quote_volume: String,
    pub price_change: String,
    pub price_change_percent: String,
    pub high_price: String,
    pub low_price: String,
    pub last_price: String,
}

#[derive(Debug, Deserialize, Serialize, SerializeRow)]
pub struct ScyllaMarket {
    pub symbol: Symbol,
    pub base: Asset,
    pub quote: Asset,
    pub max_price: String,
    pub min_price: String,
    pub tick_size: String,
    pub max_quantity: String,
    pub min_quantity: String,
    pub step_size: String,
}
