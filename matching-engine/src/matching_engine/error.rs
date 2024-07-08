use enum_stringify::EnumStringify;
use serde::{ Deserialize, Serialize };

#[derive(Debug, Serialize, EnumStringify)]
pub enum MatchingEngineErrors {
    ExchangeAlreadyExist,
    AskedMoreThanTradeable,
    UserNotFound,
    OverWithdrawl,
    InsufficientBalance,
    InvalidOrderId,
    InvalidPriceLimitOrOrderSide
}
