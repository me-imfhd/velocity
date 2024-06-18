use std::{ error::Error, sync::atomic::AtomicU64, time::{ SystemTime, UNIX_EPOCH } };

use rust_decimal::Decimal;
use scylla::{ frame::value::Counter, transport::errors::QueryError, Session, SessionBuilder };

pub mod schema;
pub mod user;
pub mod trade;
pub mod order;
pub mod ticker;
pub mod market;
pub mod scylla_tables;
#[cfg(test)]
pub mod tests;

pub struct ScyllaDb {
    pub session: Session,
}

impl ScyllaDb {
    pub async fn create_session(uri: &str) -> Result<ScyllaDb, Box<dyn Error>> {
        let session = SessionBuilder::new().known_node(uri).build().await.map_err(From::from);
        match session {
            Err(err) => Err(err),
            Ok(session) =>
                Ok(ScyllaDb {
                    session,
                }),
        }
    }
    pub async fn new_user_id(&self) -> Result<i64, Box<dyn Error>> {
        let s =
            r#"
            UPDATE keyspace_1.counter_table 
            SET user_id = user_id + 1 
            WHERE id = 1;
        "#;
        self.session.query(s, &[]).await?;
        let s =
            r#"
            SELECT user_id
            FROM keyspace_1.counter_table
            WHERE id = 1;
            "#;
        let res = self.session.query(s, &[]).await?;
        let mut iter = res.rows_typed::<(Counter,)>()?;
        let id = iter
            .next()
            .transpose()?
            .ok_or(QueryError::InvalidMessage("Does not exist in db".to_string()))?;
        Ok(id.0.0)
    }
    pub async fn new_order_id(&self) -> Result<i64, Box<dyn Error>> {
        let s =
            r#"
            UPDATE keyspace_1.counter_table 
            SET order_id = order_id + 1 
            WHERE id = 1;
        "#;
        self.session.query(s, &[]).await?;
        let s =
            r#"
            SELECT order_id
            FROM keyspace_1.counter_table
            WHERE id = 1;
            "#;
        let res = self.session.query(s, &[]).await?;
        let mut iter = res.rows_typed::<(Counter,)>()?;
        let id = iter
            .next()
            .transpose()?
            .ok_or(QueryError::InvalidMessage("Does not exist in db".to_string()))?;
        Ok(id.0.0)
    }
    pub async fn new_trade_id(&self) -> Result<i64, Box<dyn Error>> {
        let s =
            r#"
            UPDATE keyspace_1.counter_table 
            SET trade_id = trade_id + 1 
            WHERE id = 1;
        "#;
        self.session.query(s, &[]).await?;
        let s =
            r#"
            SELECT trade_id
            FROM keyspace_1.counter_table
            WHERE id = 1;
            "#;
        let res = self.session.query(s, &[]).await?;
        let mut iter = res.rows_typed::<(Counter,)>()?;
        let id = iter
            .next()
            .transpose()?
            .ok_or(QueryError::InvalidMessage("Does not exist in db".to_string()))?;
        Ok(id.0.0)
    }
}
pub fn get_epoch_ms() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
}
