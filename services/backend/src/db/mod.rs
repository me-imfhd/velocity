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
            .known_node(format!("{}:9042", uri))
            .known_node(format!("{}:9043", uri))
            .known_node(format!("{}:9044", uri))
            .default_execution_profile_handle(handle)
            .compression(Some(Compression::Lz4))
            .build().await?;

        Ok(ScyllaDb {
            session,
        })
    }
    pub async fn new_user_id(&self) -> Result<i64, Box<dyn Error>> {
        let s = r#"
        SELECT COUNT(*) FROM keyspace_1.user_table;
            "#;
        let res = self.session.query(s, &[]).await?;
        let mut res = res.rows_typed::<(i64,)>()?;
        let total_users = res.next().transpose()?.unwrap().0;
        Ok(total_users + 1)
    }
}
pub fn get_epoch_micros() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros()
}
