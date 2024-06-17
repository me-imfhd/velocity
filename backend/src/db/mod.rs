pub mod schema;
pub mod enums;
pub mod user;
pub mod trade;
pub mod order;
pub mod ticker;
pub mod market;
pub mod scylla_init;
#[cfg(test)]
pub mod tests;
use std::{ sync::atomic::AtomicU64, time::{ SystemTime, UNIX_EPOCH } };

use bigdecimal::{ FromPrimitive, ToPrimitive };
use rust_decimal::Decimal;
use schema::{ Id, MarketSchema, OrderSchema, TickerSchema, TradeSchema, UserSchema };
use scylla::{ transport::errors::QueryError, Session, SessionBuilder };
use user::UserError;

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
