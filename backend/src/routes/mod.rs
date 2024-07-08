use std::collections::HashMap;

use rust_decimal::Decimal;
use serde::{ Deserialize, Serialize };

use crate::db::schema::{
    Asset,
    Id,
    Order,
    OrderId,
    OrderSide,
    OrderStatus,
    OrderType,
    Price,
    Quantity,
    Symbol,
};

pub mod order;
pub mod user;
pub mod ping;
pub mod trades;

#[derive(Debug, Serialize, Deserialize)]
pub enum EngineRequests {
    ExecuteOrder(Order),
    CancelOrder(CancelOrder),
    CancelAll(CancelAll),
    OpenOrders(OpenOrders),
    OpenOrder(OpenOrder),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CancelOrder {
    id: OrderId,
    user_id: Id,
    symbol: Symbol,
    price: Decimal,
    order_side: OrderSide,
    #[serde(skip_deserializing)]
    sub_id: i64,
    #[serde(skip_deserializing)]
    timestamp: i64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CancelAll {
    user_id: Id,
    symbol: Symbol,
    #[serde(skip_deserializing)]
    sub_id: i64,
    #[serde(skip_deserializing)]
    timestamp: i64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenOrders {
    user_id: Id,
    symbol: Symbol,
    #[serde(skip_deserializing)]
    sub_id: i64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenOrder {
    user_id: Id,
    order_id: OrderId,
    symbol: Symbol,
    #[serde(skip_deserializing)]
    sub_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UserRequests {
    NewUser(NewUser),
    Deposit(Deposit),
    Withdraw(Withdraw),
    GetUserBalances(GetUserBalances),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct NewUser {
    #[serde(skip_deserializing)]
    sub_id: i64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Deposit {
    user_id: Id,
    asset: Asset,
    quantity: Quantity,
    #[serde(skip_deserializing)]
    sub_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Withdraw {
    user_id: Id,
    asset: Asset,
    quantity: Quantity,
    #[serde(skip_deserializing)]
    sub_id: i64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct GetUserBalances {
    user_id: Id,
    #[serde(skip_deserializing)]
    sub_id: i64,
}
