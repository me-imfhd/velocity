use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub enum MatchingEngineErrors {
    ExchangeDoesNotExist,
    ExchangeAlreadyExist,
    AskedMoreThanTradeable,
    UserNotFound,
    AssetNotFound,
    OverWithdrawl
}