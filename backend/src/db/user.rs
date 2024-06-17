use std::{ borrow::BorrowMut, collections::HashMap, error::Error, sync::atomic::Ordering };

use scylla::transport::errors::QueryError;
use strum::IntoEnumIterator;

use super::{
    add, enums::AssetEn, from_f32, schema::{ Asset, Quantity, UserSchema }, sub, to_f32, ScyllaDb, USER_ID
};
pub enum UserError {
    OverWithdrawl,
    AssetNotFound,
    UserNotFound
}
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
    pub fn lock_amount(&mut self, asset: &Asset, quantity: Quantity) {
        let mut locked_balance = self.locked_balance.get_mut(asset);
        match locked_balance {
            None => {
                self.locked_balance.insert(asset.clone(), 0.0);
                *self.locked_balance.get_mut(asset).unwrap() += quantity;
            }
            Some(mut balance) => {
                let sum = from_f32(*balance) + from_f32(quantity);
                balance = &mut to_f32(&sum);
            }
        }
    }
    pub fn unlock_amount(&mut self, asset: &Asset, quantity: Quantity) {
        let mut locked_balance = self.locked_balance.get_mut(asset).unwrap();
        let diff = from_f32(*locked_balance) - from_f32(quantity);
        locked_balance = &mut to_f32(&diff);
    }
    pub fn deposit(&mut self, asset: &Asset, quantity: Quantity) {
        let mut assets_balance = self.balance.get_mut(asset);
        match assets_balance {
            None => {
                self.balance.insert(asset.clone(), quantity);
            }
            Some(mut balance) => {
                balance = &mut add(*balance, quantity);
            }
        }
    }
    pub fn withdraw(&mut self, asset: &Asset, quantity: Quantity) -> Result<(), UserError> {
        let mut assets_balance = self.balance.get_mut(asset).ok_or(UserError::AssetNotFound)?;
        if quantity > *assets_balance {
            return Err(UserError::OverWithdrawl);
        }
        assets_balance = &mut sub(*assets_balance, quantity);
        Ok(())
    }
    pub fn locked_balance(&self, asset: &Asset) -> Result<&Quantity, UserError> {
        let assets_balance = self.locked_balance.get(asset).ok_or(UserError::AssetNotFound)?;
        Ok(assets_balance)
    }
    pub fn balance(&self, asset: &Asset) -> Result<&Quantity, UserError> {
        let assets_balance = self.balance.get(asset).ok_or(UserError::AssetNotFound)?;
        Ok(assets_balance)
    }
    pub fn available_balance(&self, asset: &Asset) -> Result<Quantity, UserError> {
        let locked_balance = self.locked_balance(asset)?;
        let balance = self.balance(asset)?;
        Ok(balance - locked_balance)
    }
}

impl ScyllaDb {
    pub async fn new_user(&self, user: UserSchema) -> Result<(), QueryError> {
        let s =
            r#"
            INSERT INTO keyspace_1.user_table (
                id,
                balance,
                locked_balance
            ) VALUES (?, ?, ?);
        "#;
        let res = self.session.query(s, user).await?;
        Ok(())
    }
    pub async fn get_user(&self, user_id: i64) -> Result<UserSchema, Box<dyn Error>> {
        let s =
            r#"
            SELECT
                id,
                balance,
                locked_balance
            FROM keyspace_1.user_table
            WHERE id = ? ;
        "#;
        let res = self.session.query(s, (user_id,)).await?;
        let mut users = res.rows_typed::<UserSchema>()?;
        let user = users
            .next()
            .transpose()?
            .ok_or(QueryError::InvalidMessage("User does not exist in db".to_string()))?;
        Ok(user)
    }
}