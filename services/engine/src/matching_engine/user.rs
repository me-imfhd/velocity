use rust_decimal::Decimal;
use serde::{ Deserialize, Serialize };
use strum::IntoEnumIterator;
use std::{ collections::HashMap, str::FromStr, sync::atomic::Ordering };

use rust_decimal_macros::dec;

use crate::{ engine::MatchingEngine, Exchange, OrderSide, Price, ScyllaUser, User, Users };

use super::{ error::MatchingEngineErrors, Asset, Id, Quantity };

impl ScyllaUser {
    pub fn from_scylla_user(&self) -> User {
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
            id: self.id as u64,
            balance: balance_map,
            locked_balance: locked_balance_map,
        }
    }
}

impl Users {
    pub fn validate_and_lock_limit(
        &mut self,
        order_side: OrderSide,
        exchange: &Exchange,
        user_id: Id,
        price: Price,
        quantity: Quantity
    ) -> Result<(Asset, Quantity), MatchingEngineErrors> {
        match order_side {
            OrderSide::Bid => {
                let locked_amount = self.validate_and_lock(
                    &exchange.quote,
                    user_id,
                    price * quantity
                )?;
                return Ok((exchange.quote, locked_amount));
            }
            OrderSide::Ask => {
                let locked_amount = self.validate_and_lock(&exchange.base, user_id, quantity)?;
                return Ok((exchange.base, locked_amount));
            }
        }
    }
    pub fn validate_and_lock_market(
        &mut self,
        quote: Result<Decimal, MatchingEngineErrors>,
        order_side: &OrderSide,
        exchange: &Exchange,
        user_id: Id,
        quantity: Quantity
    ) -> Result<(Asset, Quantity), MatchingEngineErrors> {
        let quote = quote?;
        match order_side {
            OrderSide::Bid => {
                let locked_balance = self.validate_and_lock(&exchange.quote, user_id, quote)?;
                Ok((exchange.quote, locked_balance))
            }
            OrderSide::Ask => {
                let locked_balance = self.validate_and_lock(&exchange.base, user_id, quantity)?;
                Ok((exchange.base, locked_balance))
            }
        }
    }
    pub fn lock_amount(&mut self, asset: &Asset, user_id: Id, quantity: Quantity) -> Quantity {
        let user = self.users.get_mut(&user_id).unwrap();
        let mut locked_balance = user.locked_balance.get_mut(asset);
        match locked_balance {
            None => {
                user.locked_balance.insert(*asset, dec!(0));
                let locked_balance = user.locked_balance.get_mut(asset).unwrap();
                *locked_balance += quantity;
            }
            Some(mut balance) => {
                *balance += quantity;
            }
        }
        self.users.get(&user_id).unwrap().locked_balance.get(asset).unwrap().clone()
    }
    pub fn unlock_amount(&mut self, asset: &Asset, user_id: Id, quantity: Quantity) -> &User {
        let user = self.users.get_mut(&user_id).unwrap();
        let mut locked_balance = user.locked_balance.get_mut(asset).unwrap();
        locked_balance -= quantity;
        user
    }
    pub fn does_exist(&self, user_id: Id) -> bool {
        self.users.contains_key(&user_id)
    }
    pub fn new_user(&mut self, id: Id) -> Id {
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
    pub fn recover_user(&mut self, user: User) {
        self.users.insert(user.id, user);
    }
    pub fn deposit(
        &mut self,
        asset: &Asset,
        quantity: Quantity,
        user_id: Id
    ) -> Result<&User, MatchingEngineErrors> {
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
        Ok(user)
    }
    pub fn withdraw(
        &mut self,
        asset: &Asset,
        quantity: Quantity,
        user_id: Id
    ) -> Result<&User, MatchingEngineErrors> {
        let mut user = self.users.get_mut(&user_id).ok_or(MatchingEngineErrors::UserNotFound)?;
        let mut assets_balance = user.balance.get_mut(asset).unwrap();
        if quantity > *assets_balance {
            return Err(MatchingEngineErrors::OverWithdrawl);
        }
        assets_balance -= quantity;
        Ok(user)
    }
    pub fn locked_balance(
        &self,
        asset: &Asset,
        user_id: Id
    ) -> Result<&Quantity, MatchingEngineErrors> {
        let mut user = self.users.get(&user_id).ok_or(MatchingEngineErrors::UserNotFound)?;
        let assets_balance = user.locked_balance.get(asset).unwrap();
        Ok(assets_balance)
    }
    pub fn balance(&self, asset: &Asset, user_id: Id) -> Result<&Quantity, MatchingEngineErrors> {
        let mut user = self.users.get(&user_id).ok_or(MatchingEngineErrors::UserNotFound)?;
        let assets_balance = user.balance.get(asset).unwrap();
        Ok(assets_balance)
    }
    pub fn available_balance(
        &self,
        asset: &Asset,
        user_id: Id
    ) -> Result<Quantity, MatchingEngineErrors> {
        let locked_balance = self.locked_balance(asset, user_id)?;
        let balance = self.balance(asset, user_id)?;
        Ok(balance - locked_balance)
    }
    pub fn validate_and_lock(
        &mut self,
        asset: &Asset,
        user_id: Id,
        cmp_quantity: Quantity
    ) -> Result<Quantity, MatchingEngineErrors> {
        let available_balance = self.available_balance(asset, user_id)?;
        if available_balance < cmp_quantity {
            return Err(MatchingEngineErrors::InsufficientBalance);
        }
        let locked_balance = self.lock_amount(asset, user_id, cmp_quantity);
        Ok(locked_balance)
    }
    pub fn my_assets(&self, user_id: Id) -> Result<Vec<&Asset>, MatchingEngineErrors> {
        let mut user = self.users.get(&user_id).ok_or(MatchingEngineErrors::UserNotFound)?;
        Ok(Vec::from_iter(user.balance.keys()))
    }
}
