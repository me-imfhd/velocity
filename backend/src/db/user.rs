use std::{
    borrow::BorrowMut,
    collections::HashMap,
    error::Error,
    str::FromStr,
    sync::atomic::Ordering,
};

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use scylla::transport::errors::QueryError;
use strum::IntoEnumIterator;

use super::{ schema::{ Asset, Quantity, User }, scylla_tables::ScyllaUser, ScyllaDb, USER_ID };
pub enum UserError {
    OverWithdrawl,
    AssetNotFound,
    UserNotFound,
}

impl ScyllaUser {
    fn from_scylla_user(&self) -> User {
        let mut balance_map: HashMap<Asset, Quantity> = HashMap::new();
        for (asset_str, balance) in &self.balance {
            let asset = Asset::from_str(&asset_str).unwrap();
            let balance = Decimal::from_str(&balance).unwrap();
            balance_map.insert(asset, balance);
        }
        let mut locked_balance_map: HashMap<Asset, Quantity> = HashMap::new();
        for (asset_str, locked_balance) in &self.locked_balance {
            let asset = Asset::from_str(&asset_str).unwrap();
            let locked_balance = Decimal::from_str(&locked_balance).unwrap();
            locked_balance_map.insert(asset, locked_balance);
        }

        User {
            id: self.id,
            balance: balance_map,
            locked_balance: locked_balance_map,
        }
    }
}
impl User {
    pub fn new() -> User {
        USER_ID.fetch_add(1, Ordering::SeqCst);
        let id = USER_ID.load(Ordering::SeqCst);
        let mut balance: HashMap<Asset, Quantity> = HashMap::new();
        let mut locked_balance: HashMap<Asset, Quantity> = HashMap::new();
        for asset in Asset::iter() {
            balance.insert(asset, dec!(0.0));
        }
        for asset in Asset::iter() {
            locked_balance.insert(asset, dec!(0.0));
        }
        User {
            id: id as i64,
            balance,
            locked_balance,
        }
    }
    pub fn lock_amount(&mut self, asset: &Asset, quantity: Quantity) {
        let mut locked_balance = self.locked_balance.get_mut(asset);
        match locked_balance {
            None => {
                self.locked_balance.insert(asset.clone(), dec!(0.0));
                *self.locked_balance.get_mut(asset).unwrap() += quantity;
            }
            Some(mut balance) => {
                balance += quantity;
            }
        }
    }
    pub fn unlock_amount(&mut self, asset: &Asset, quantity: Quantity) {
        let mut locked_balance = self.locked_balance.get_mut(asset).unwrap();
        locked_balance -= quantity;
    }
    pub fn deposit(&mut self, asset: &Asset, quantity: Quantity) {
        let mut assets_balance = self.balance.get_mut(asset);
        match assets_balance {
            None => {
                self.balance.insert(asset.clone(), quantity);
            }
            Some(mut balance) => {
                balance += quantity;
            }
        }
    }
    pub fn withdraw(&mut self, asset: &Asset, quantity: Quantity) -> Result<(), UserError> {
        let mut assets_balance = self.balance.get_mut(asset).ok_or(UserError::AssetNotFound)?;
        if quantity > *assets_balance {
            return Err(UserError::OverWithdrawl);
        }
        assets_balance -= quantity;
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
    fn to_scylla_user(&self) -> ScyllaUser {
        let mut scylla_balance: HashMap<String, String> = HashMap::new();
        for (asset, balance) in &self.balance {
            scylla_balance.insert(asset.to_string(), balance.to_string());
        }
        let mut scylla_locked_balance: HashMap<String, String> = HashMap::new();
        for (asset, balance) in &self.locked_balance {
            scylla_locked_balance.insert(asset.to_string(), balance.to_string());
        }
        ScyllaUser {
            id: self.id,
            balance: scylla_balance,
            locked_balance: scylla_locked_balance,
        }
    }
}

impl ScyllaDb {
    pub async fn new_user(&self, user: User) -> Result<(), QueryError> {
        let s =
            r#"
            INSERT INTO keyspace_1.user_table (
                id,
                balance,
                locked_balance
            ) VALUES (?, ?, ?);
        "#;
        let user = user.to_scylla_user();
        let res = self.session.query(s, user).await?;
        Ok(())
    }
    pub async fn get_user(&self, user_id: i64) -> Result<User, Box<dyn Error>> {
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
        let mut users = res.rows_typed::<ScyllaUser>()?;
        let scylla_user = users
            .next()
            .transpose()?
            .ok_or(QueryError::InvalidMessage("User does not exist in db".to_string()))?;
        let user = scylla_user.from_scylla_user();
        Ok(user)
    }
}
