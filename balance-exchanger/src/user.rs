use std::{ collections::HashMap, error::Error, str::FromStr };

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use scylla::transport::errors::QueryError;

use crate::{ Asset, Quantity, ScyllaDb, ScyllaUser, User };

#[derive(Debug)]
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
    pub fn to_scylla_user(&self) -> ScyllaUser {
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
    pub fn unlock_amount(&mut self, asset: &Asset, quantity: Quantity) {
        let mut locked_balance = self.locked_balance.get_mut(asset).unwrap();
        locked_balance -= quantity;
    }
    pub fn deposit(&mut self, asset: &Asset, quantity: Quantity) {
        let assets_balance = self.balance.get_mut(asset);
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
}

impl ScyllaDb {
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
    pub fn update_user_statement(&self) -> &str {
        let s =
            r#"
            UPDATE keyspace_1.user_table 
            SET
                balance = ?,
                locked_balance = ?
            WHERE id = ?;
        "#;
        s
    }
}
