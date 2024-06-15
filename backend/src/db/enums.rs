use enum_stringify::EnumStringify;
use serde::{ Deserialize, Serialize };
use strum_macros::EnumIter;

#[derive(Debug, Deserialize, Serialize, EnumStringify)]
pub enum OrderStatusEn {
    Processing,
    Filled,
    PartiallyFilled,
    Failed,
}
#[derive(Debug, Deserialize, Serialize, EnumStringify)]
pub enum OrderSideEn {
    Bid,
    Ask,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumStringify)]
pub enum OrderTypeEn {
    Market,
    Limit,
}
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, EnumIter, Serialize, Deserialize, EnumStringify)]
pub enum AssetEn {
    USDT,
    BTC,
    SOL,
    ETH,
}
