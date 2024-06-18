use std::{ error::Error, sync::atomic::AtomicU64, time::{ SystemTime, UNIX_EPOCH } };

use rust_decimal::Decimal;
use scylla::{ Session, SessionBuilder };

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
pub static ORDER_ID: AtomicU64 = AtomicU64::new(0);
pub static TRADE_ID: AtomicU64 = AtomicU64::new(0);
pub static USER_ID: AtomicU64 = AtomicU64::new(0);

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
}
pub fn get_epoch_ms() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
}
