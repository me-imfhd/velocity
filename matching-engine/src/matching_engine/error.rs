use serde::{ Deserialize, Serialize };

#[derive(Debug, Serialize)]
pub enum MatchingEngineErrors {
    ExchangeAlreadyExist,
    AskedMoreThanTradeable,
    UserNotFound,
    OverWithdrawl,
    InsufficientBalance
}
