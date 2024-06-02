#[derive(Debug)]
pub enum MatchingEngineErrors {
    OrderbookDoesNotExist(String)
}
#[derive(Debug)]
pub enum UserError {
    AssetNotFound,
    UserNotFound
}