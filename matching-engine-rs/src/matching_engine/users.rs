use strum::IntoEnumIterator;
use std::{ collections::HashMap, sync::atomic::Ordering };

use rust_decimal_macros::dec;

use super::{ error::UserError, Asset, Id, Quantity, USER_ID };

#[derive(Debug)]
struct User {
    id: Id,
    balance: HashMap<Asset, Quantity>,
}
#[derive(Debug)]
pub struct Users {
    users: HashMap<Id, User>,
}
impl Users {
    pub fn init() -> Users {
        Users {
            users: HashMap::new(),
        }
    }
    pub fn new_user(&mut self) -> Id {
        USER_ID.fetch_add(1, Ordering::SeqCst);
        let id = USER_ID.load(Ordering::SeqCst);
        let mut balance: HashMap<Asset, Quantity> = HashMap::new();
        for asset in Asset::iter() {
            balance.insert(asset, dec!(0));
        }
        self.users.insert(id, User {
            id,
            balance,
        });
        id
    }
    pub fn deposit(
        &mut self,
        asset: &Asset,
        quantity: Quantity,
        user_id: Id
    ) -> Result<(), UserError> {
        let mut user = self.users.get_mut(&user_id).ok_or(UserError::UserNotFound)?;
        let mut assets_balance = user.balance.get_mut(asset).ok_or(UserError::AssetNotFound)?;
        // add logic to deduct balance from users wallet
        assets_balance += quantity;
        Ok(())
    }
    pub fn withdraw(
        &mut self,
        asset: &Asset,
        quantity: Quantity,
        user_id: Id
    ) -> Result<(), UserError> {
        let mut user = self.users.get_mut(&user_id).ok_or(UserError::UserNotFound)?;
        let mut assets_balance = user.balance.get_mut(asset).ok_or(UserError::AssetNotFound)?;
        // add logic to add balance to users wallet
        assets_balance -= quantity;
        Ok(())
    }
    pub fn balance(&self, asset: &Asset, user_id: Id) -> Result<&Quantity, UserError> {
        let mut user = self.users.get(&user_id).ok_or(UserError::UserNotFound)?;
        let assets_balance = user.balance.get(asset).ok_or(UserError::AssetNotFound)?;
        Ok(assets_balance)
    }
    pub fn my_assets(&self, user_id: Id) -> Result<Vec<&Asset>, UserError> {
        let mut user = self.users.get(&user_id).ok_or(UserError::UserNotFound)?;
        Ok(Vec::from_iter(user.balance.keys()))
    }
}
