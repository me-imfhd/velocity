use enum_stringify::EnumStringify;
use serde::{ Deserialize, Serialize };
use strum::IntoEnumIterator;
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
impl AssetEn {
    pub fn from_str(asset_to_match: &str) -> Result<Self, ()> {
        for asset in AssetEn::iter() {
            let current_asset = asset.to_string();
            if asset_to_match.to_string() == current_asset {
                return Ok(asset);
            }
        }
        Err(())
    }
}
