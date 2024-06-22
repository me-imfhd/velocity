use std::{ error::Error, sync::Arc, time::{ SystemTime, UNIX_EPOCH } };
use scylla::{
    frame::{ value::Counter, Compression },
    host_filter::DcHostFilter,
    load_balancing::{ self, LoadBalancingPolicy },
    transport::errors::QueryError,
    ExecutionProfile,
    Session,
    SessionBuilder,
};

pub mod schema;
pub mod scylla_tables;

pub struct ScyllaDb {
    pub session: Session,
}

impl ScyllaDb {
    pub async fn create_session(uri: &str) -> Result<ScyllaDb, Box<dyn Error>> {
        let policy = Arc::new(load_balancing::DefaultPolicy::default());
        let profile = ExecutionProfile::builder().load_balancing_policy(policy).build();
        let handle = profile.into_handle();
        let session = SessionBuilder::new()
            .known_node(format!("{}:9042",uri))
            .known_node(format!("{}:9043",uri))
            .known_node(format!("{}:9044",uri))
            .default_execution_profile_handle(handle)
            .compression(Some(Compression::Lz4))
            .build().await?;

        Ok(ScyllaDb {
            session,
        })
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
