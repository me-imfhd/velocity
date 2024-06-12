use serde::{ Deserialize, Serialize };
use strum::IntoEnumIterator;
use std::{ collections::HashMap, sync::atomic::Ordering };

use rust_decimal_macros::dec;

use super::{ error::MatchingEngineErrors, Asset, Id, Quantity, USER_ID };

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    pub id: Id,
    pub balance: HashMap<Asset, Quantity>,
    pub locked_balance: HashMap<Asset, Quantity>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Users {
    pub users: HashMap<Id, User>,
}
impl Users {
    pub fn init() -> Users {
        Users {
            users: HashMap::new(),
        }
    }
    pub fn recover_users(&mut self, redis_connection: &mut redis::Connection) {
        let user_store_str = redis::cmd("GET").arg("users").query::<String>(redis_connection);
        match user_store_str {
            Err(_) => {}
            Ok(user_store_str) => {
                let mut user_store: Vec<User> = serde_json::from_str(&user_store_str).unwrap();
                let mut users = &mut self.users;
                user_store.iter_mut().for_each(|user| {
                    users.insert(user.id, user.clone());
                });
            }
        }
    }
    pub fn lock_amount(&mut self, asset: &Asset, user_id: Id, quantity: Quantity) {
        let user = self.users.get_mut(&user_id).unwrap();
        let mut locked_balance = user.locked_balance.get_mut(asset);
        match locked_balance {
            None => {
                user.locked_balance.insert(*asset, dec!(0));
                *user.locked_balance.get_mut(asset).unwrap() += quantity;
            }
            Some(mut balance) => {
                balance += quantity;
            }
        }
    }
    pub fn unlock_amount(&mut self, asset: &Asset, user_id: Id, quantity: Quantity) {
        let user = self.users.get_mut(&user_id).unwrap();
        let mut locked_balance = user.locked_balance.get_mut(asset).unwrap();
        locked_balance -= quantity;
    }
    pub fn does_exist(&self, user_id: Id) -> bool {
        self.users.contains_key(&user_id)
    }
    pub fn new_user(&mut self) -> Id {
        USER_ID.fetch_add(1, Ordering::SeqCst);
        let id = USER_ID.load(Ordering::SeqCst);
        let mut balance: HashMap<Asset, Quantity> = HashMap::new();
        let mut locked_balance: HashMap<Asset, Quantity> = HashMap::new();
        for asset in Asset::iter() {
            balance.insert(asset, dec!(0));
        }
        for asset in Asset::iter() {
            locked_balance.insert(asset, dec!(0));
        }
        self.users.insert(id, User {
            id,
            balance,
            locked_balance,
        });
        id
    }
    pub fn deposit(
        &mut self,
        asset: &Asset,
        quantity: Quantity,
        user_id: Id
    ) -> Result<(), MatchingEngineErrors> {
        let mut user = self.users.get_mut(&user_id).ok_or(MatchingEngineErrors::UserNotFound)?;
        let mut assets_balance = user.balance.get_mut(asset);
        match assets_balance {
            None => {
                user.balance.insert(*asset, dec!(0));
            }
            Some(mut balance) => {
                balance += quantity;
            }
        }
        Ok(())
    }
    pub fn withdraw(
        &mut self,
        asset: &Asset,
        quantity: Quantity,
        user_id: Id
    ) -> Result<(), MatchingEngineErrors> {
        let mut user = self.users.get_mut(&user_id).ok_or(MatchingEngineErrors::UserNotFound)?;
        let mut assets_balance = user.balance
            .get_mut(asset)
            .ok_or(MatchingEngineErrors::AssetNotFound)?;
        if quantity > *assets_balance {
            return Err(MatchingEngineErrors::OverWithdrawl);
        }
        assets_balance -= quantity;
        Ok(())
    }
    pub fn locked_balance(
        &self,
        asset: &Asset,
        user_id: Id
    ) -> Result<&Quantity, MatchingEngineErrors> {
        let mut user = self.users.get(&user_id).ok_or(MatchingEngineErrors::UserNotFound)?;
        let assets_balance = user.locked_balance
            .get(asset)
            .ok_or(MatchingEngineErrors::AssetNotFound)?;
        Ok(assets_balance)
    }
    pub fn balance(&self, asset: &Asset, user_id: Id) -> Result<&Quantity, MatchingEngineErrors> {
        let mut user = self.users.get(&user_id).ok_or(MatchingEngineErrors::UserNotFound)?;
        let assets_balance = user.balance.get(asset).ok_or(MatchingEngineErrors::AssetNotFound)?;
        Ok(assets_balance)
    }
    pub fn open_balance(
        &self,
        asset: &Asset,
        user_id: Id
    ) -> Result<Quantity, MatchingEngineErrors> {
        let locked_balance = self.locked_balance(asset, user_id)?;
        let balance = self.balance(asset, user_id)?;
        Ok(balance - locked_balance)
    }
    pub fn my_assets(&self, user_id: Id) -> Result<Vec<&Asset>, MatchingEngineErrors> {
        let mut user = self.users.get(&user_id).ok_or(MatchingEngineErrors::UserNotFound)?;
        Ok(Vec::from_iter(user.balance.keys()))
    }
}
