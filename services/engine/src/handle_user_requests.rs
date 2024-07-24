use redis::{ Commands, Connection, Value };
use serde::{ Deserialize, Serialize };
use serde_json::to_string;

use crate::{ error::MatchingEngineErrors, Asset, Id, Quantity, Users };

#[derive(Debug, Serialize, Deserialize)]
pub enum UserRequests {
    NewUser(NewUser),
    Deposit(Deposit),
    Withdraw(Withdraw),
    GetUserBalances(GetUserBalances),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct NewUser {
    sub_id: i64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Deposit {
    user_id: Id,
    asset: Asset,
    quantity: Quantity,
    sub_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Withdraw {
    user_id: Id,
    asset: Asset,
    quantity: Quantity,
    sub_id: i64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct GetUserBalances {
    user_id: Id,
    sub_id: i64,
}

impl UserRequests {
    pub fn new_user(users: &mut Users, u: NewUser, con: &mut Connection) {
        let total_user = users.users.len() as u64;
        let new_user_id = total_user + 1;
        users.new_user(total_user + 1);
        let user = users.users.get(&new_user_id).unwrap();
        println!("New User Created");
        con.lpush::<i64, String, Value>(u.sub_id, to_string(user).unwrap()).unwrap();
    }
    pub fn get_user_balances(users: &mut Users, u: GetUserBalances, con: &mut Connection) {
        let user = users.users.get(&u.user_id).unwrap();
        con.lpush::<i64, String, Value>(u.sub_id, to_string(user).unwrap()).unwrap();
    }
    pub fn deposit(users: &mut Users, u: Deposit, con: &mut Connection) {
        let res = users.deposit(&u.asset, u.quantity, u.user_id);
        match res {
            Ok(user) => {
                println!("Deposited balance");
                con.lpush::<i64, String, Value>(u.sub_id, to_string(user).unwrap()).unwrap();
            }
            Err(err) => {
                println!("{}", err);
                con.lpush::<i64, String, Value>(u.sub_id, err.to_string()).unwrap();
            }
        }
    }
    pub fn withdraw(users: &mut Users, u: Withdraw, con: &mut Connection) {
        let ava_b = users.available_balance(&u.asset, u.user_id);
        match ava_b {
            Ok(available) => {
                if available > u.quantity {
                    con.lpush::<i64, String, Value>(
                        u.sub_id,
                        MatchingEngineErrors::OverWithdrawl.to_string()
                    ).unwrap();
                    return;
                }
                let user = users.withdraw(&u.asset, u.quantity, u.user_id).unwrap();
                println!("Withdrawn balance");
                con.lpush::<i64, String, Value>(u.sub_id, to_string(user).unwrap()).unwrap();
            }
            Err(err) => {
                con.lpush::<i64, String, Value>(u.sub_id, err.to_string()).unwrap();
            }
        }
    }
}
