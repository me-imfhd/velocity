use std::{ collections::HashMap, sync::atomic::Ordering };

use strum::IntoEnumIterator;

use super::{ enums::AssetEn, schema::{ Asset, Quantity, UserSchema }, USER_ID };

impl UserSchema {
    pub fn new() -> UserSchema {
        USER_ID.fetch_add(1, Ordering::SeqCst);
        let id = USER_ID.load(Ordering::SeqCst);
        let mut balance: HashMap<Asset, Quantity> = HashMap::new();
        let mut locked_balance: HashMap<Asset, Quantity> = HashMap::new();
        for asset in AssetEn::iter() {
            balance.insert(asset.to_string(), 0.0);
        }
        for asset in AssetEn::iter() {
            locked_balance.insert(asset.to_string(), 0.0);
        }
        UserSchema {
            id: id as i64,
            balance,
            locked_balance,
        }
    }
}
