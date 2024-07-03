use std::{
    borrow::BorrowMut,
    cell::Cell,
    sync::{ atomic::Ordering, Arc, Mutex },
    time::{ self, SystemTime, UNIX_EPOCH },
};
use enum_stringify::EnumStringify;
use error::MatchingEngineErrors;
use redis::{ Connection, PubSub, Value };
use rust_decimal_macros::dec;
use rust_decimal::prelude::*;
use scylla::{
    frame::Compression,
    load_balancing,
    ExecutionProfile,
    FromRow,
    SerializeRow,
    Session,
    SessionBuilder,
};
use serde::{ Deserialize, Serialize };
use serde_json::to_string;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use std::{ clone, collections::HashMap };
use super::*;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orderbook {
    pub trade_id: u64,
    pub order_id: u64,
    pub exchange: Exchange,
    pub asks: HashMap<Price, Limit>,
    pub bids: HashMap<Price, Limit>,
}
impl Orderbook {
    pub fn new(exchange: Exchange) -> Orderbook {
        Orderbook {
            trade_id: 0,
            order_id: 0,
            exchange,
            asks: HashMap::new(),
            bids: HashMap::new(),
        }
    }
    pub fn get_quote(
        &mut self,
        order_side: &OrderSide,
        mut order_quantity: Quantity
    ) -> Result<Decimal, MatchingEngineErrors> {
        let sorted_orders = match order_side {
            OrderSide::Ask => Orderbook::bid_limits(&mut self.bids),
            OrderSide::Bid => Orderbook::ask_limits(&mut self.asks),
        };
        let mut orderbook_quote = dec!(0);
        for limit_order in sorted_orders {
            let total_quantity = limit_order.total_volume();
            if total_quantity >= order_quantity {
                orderbook_quote += order_quantity * limit_order.price;
                return Ok(orderbook_quote);
            }
            orderbook_quote += total_quantity * limit_order.price;
            order_quantity -= total_quantity;
        }
        Err(MatchingEngineErrors::AskedMoreThanTradeable)
    }
    pub fn fill_market_order(
        &mut self,
        mut order: Order,
        exchange: &Exchange,
        rc: &mut Connection,
        users: &mut Users,
        should_exectute_trade: bool
    ) {
        let sorted_orders = match order.order_side {
            OrderSide::Ask => Orderbook::bid_limits(&mut self.bids),
            OrderSide::Bid => Orderbook::ask_limits(&mut self.asks),
        };
        println!("Recived an {} Market order", order.order_side);
        for limit_order in sorted_orders {
            let price = limit_order.price.clone();
            order = limit_order.fill_order(
                order,
                exchange,
                price,
                rc,
                &mut self.trade_id,
                users,
                should_exectute_trade
            );
            if order.is_filled() {
                break;
            }
        }
    }
    pub fn fill_limit_order(
        &mut self,
        price: Price,
        mut order: Order,
        exchange: &Exchange,
        rc: &mut Connection,
        users: &mut Users,
        should_exectute_trade: bool
    ) {
        println!("Recived an {} Limit order", order.order_side);
        let initial_quantity = order.quantity;
        let result = match order.order_side {
            OrderSide::Ask => {
                let sorted_bids = &mut Orderbook::bid_limits(&mut self.bids);
                let mut i = 0;
                if sorted_bids.len() == 0 {
                    self.add_limit_order(price, order);
                    return;
                }
                while i < sorted_bids.len() {
                    if price > sorted_bids[i].price {
                        self.add_limit_order(price, order);
                        break;
                    }
                    order = sorted_bids[i].fill_order(
                        order,
                        exchange,
                        price,
                        rc,
                        &mut self.trade_id,
                        users,
                        should_exectute_trade
                    );
                    if order.quantity > dec!(0) && sorted_bids.get(i + 1).is_none() {
                        self.add_limit_order(price, order);
                        break;
                    }
                    i += 1;
                }
            }
            OrderSide::Bid => {
                let sorted_asks = &mut Orderbook::ask_limits(&mut self.asks);
                let mut i = 0;
                if sorted_asks.len() == 0 {
                    self.add_limit_order(price, order);
                    return;
                }
                while i < sorted_asks.len() {
                    if price < sorted_asks[i].price {
                        self.add_limit_order(price, order);
                        break;
                    }
                    let price = sorted_asks[i].price.clone();
                    order = sorted_asks[i].fill_order(
                        order,
                        exchange,
                        price,
                        rc,
                        &mut self.trade_id,
                        users,
                        should_exectute_trade
                    );

                    if order.quantity > dec!(0) && sorted_asks.get(i + 1).is_none() {
                        self.add_limit_order(price, order);
                        break;
                    }
                    i += 1;
                }
            }
        };
    }
    pub fn bids_by_user(&self, user_id: Id) -> Vec<Limit> {
        let mut bids = &self.bids;
        let mut user_limit_vec: Vec<Limit> = Vec::new();

        for limit in bids.values() {
            let mut user_limit_orders = Limit::new(limit.price);
            for order in &limit.orders {
                if order.user_id == user_id {
                    user_limit_orders.orders.push(order.clone());
                }
            }
            user_limit_vec.push(user_limit_orders);
        }

        user_limit_vec
    }
    pub fn asks_by_user(&self, user_id: Id) -> Vec<Limit> {
        let mut asks = &self.asks;
        let mut user_limit_vec: Vec<Limit> = Vec::new();

        for limit in asks.values() {
            let mut user_limit_orders = Limit::new(limit.price);
            for order in &limit.orders {
                if order.user_id == user_id {
                    user_limit_orders.orders.push(order.clone());
                }
            }
            user_limit_vec.push(user_limit_orders);
        }

        user_limit_vec
    }
    // sorted from lowest to highest
    pub fn ask_limits(asks: &mut HashMap<Price, Limit>) -> Vec<&mut Limit> {
        let mut asks = asks.values_mut().collect::<Vec<&mut Limit>>();
        asks.sort_by(|a, b| a.price.cmp(&b.price));
        asks
    }
    // sorted from highest to lowest
    pub fn bid_limits(bids: &mut HashMap<Price, Limit>) -> Vec<&mut Limit> {
        let mut bids = bids.values_mut().collect::<Vec<&mut Limit>>();
        bids.sort_by(|a, b| b.price.cmp(&a.price));
        bids
    }
    async fn recover_trade_id(&mut self, session: &Session) {
        let s = r#"
            SELECT COUNT(*) FROM keyspace_1.trade_table;
                "#;
        let res = session.query(s, &[]).await.unwrap();
        let mut res = res.rows_typed::<(i64,)>().unwrap();
        let trade_id = res.next().transpose().unwrap().unwrap().0;
        self.trade_id = trade_id as u64;
    }
    async fn recover_order_id(&mut self, session: &Session) {
        let s = r#"
            SELECT COUNT(*) FROM keyspace_1.order_table;
                "#;
        let res = session.query(s, &[]).await.unwrap();
        let mut res = res.rows_typed::<(i64,)>().unwrap();
        let order_id = res.next().transpose().unwrap().unwrap().0;
        self.order_id = order_id as u64;
    }
    async fn replay_orders(
        &mut self,
        rc: &mut redis::Connection,
        session: &Session,
        users: &mut Users
    ) {
        let current_time = get_epoch_ms() as i64;
        let since = 1000 * 60 * 60 * 24; // 24 hours in millis
        let from_time = current_time - since;
        let s =
            r#"
        SELECT
            id,
            user_id,
            symbol,
            price,
            initial_quantity,
            filled_quantity, 
            order_type,
            order_side,
            order_status,
            timestamp
        FROM keyspace_1.order_table
        WHERE timestamp > ? AND symbol = ? ALLOW FILTERING;
    "#;
        let symbol = &self.exchange.symbol;
        let res = session.query(s, (from_time, symbol)).await.unwrap();
        let mut orders = res.rows_typed::<ScyllaOrder>().unwrap();
        let mut replay_orders: Vec<ReplayOrder> = orders
            .map(|order| order.unwrap().from_scylla_order())
            .collect();
        replay_orders.reverse();
        for replay_order in replay_orders {
            match replay_order.order_type {
                OrderType::Market => {
                    let order = Order::new(
                        replay_order.id,
                        replay_order.timestamp as u64,
                        replay_order.order_side,
                        replay_order.initial_quantity,
                        true,
                        replay_order.user_id as u64
                    );
                    self.fill_market_order(
                        order,
                        &Exchange::from_symbol(replay_order.symbol).unwrap(),
                        rc,
                        users,
                        false
                    );
                }
                OrderType::Limit => {
                    let order = Order::new(
                        replay_order.id,
                        replay_order.timestamp as u64,
                        replay_order.order_side,
                        replay_order.initial_quantity,
                        true,
                        replay_order.user_id as u64
                    );
                    self.fill_limit_order(
                        replay_order.price,
                        order,
                        &Exchange::from_symbol(replay_order.symbol).unwrap(),
                        rc,
                        users,
                        false
                    );
                }
            }
        }
    }
    pub async fn recover_orderbook(
        &mut self,
        session: &Session,
        rc: &mut redis::Connection,
        users: &mut Users
    ) {
        self.recover_trade_id(&session).await;
        self.recover_order_id(&session).await;
        self.replay_orders(rc, &session, users).await;
    }
    pub fn add_limit_order(&mut self, price: Price, order: Order) {
        let order_side = &order.order_side.clone();
        match order_side {
            OrderSide::Bid => {
                let limit = self.bids.get_mut(&price);
                match limit {
                    Some(limit) => limit.add_order(order),
                    None => {
                        let mut limit = Limit::new(price);
                        limit.add_order(order);
                        self.bids.insert(price, limit);
                    }
                }
            }
            OrderSide::Ask => {
                let limit = self.asks.get_mut(&price);
                match limit {
                    Some(limit) => limit.add_order(order),
                    None => {
                        let mut limit = Limit::new(price);
                        limit.add_order(order);
                        self.asks.insert(price, limit);
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: OrderId,
    pub user_id: Id,
    pub initial_quantity: Quantity,
    pub quantity: Quantity,
    pub order_side: OrderSide,
    pub is_market_maker: bool,
    pub timestamp: u64,
}

fn get_epoch_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}
fn get_epoch_micro() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros()
}
impl Order {
    pub fn new(
        id: OrderId,
        timestamp: u64,
        order_side: OrderSide,
        quantity: Quantity,
        is_market_maker: bool,
        user_id: Id
    ) -> Order {
        Order {
            id,
            user_id,
            order_side,
            initial_quantity: quantity,
            quantity,
            is_market_maker,
            timestamp,
        }
    }
    pub fn is_filled(&self) -> bool {
        self.quantity == dec!(0)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Limit {
    pub price: Price,
    pub orders: Vec<Order>,
}

impl Limit {
    pub fn new(price: Price) -> Limit {
        Limit {
            price,
            orders: Vec::new(),
        }
    }

    fn add_order(&mut self, order: Order) {
        self.orders.push(order)
    }
    fn fill_order(
        &mut self,
        mut order: Order,
        exchange: &Exchange,
        exchange_price: Price,
        rc: &mut Connection,
        mut trade_id: &mut u64,
        users: &mut Users,
        should_exectute_trade: bool
    ) -> Order {
        let mut remaining_quantity = order.quantity.clone();
        let mut i = 0;
        while i < self.orders.len() {
            if remaining_quantity == dec!(0) {
                break;
            }
            let limit_order = &mut self.orders[i];
            match limit_order.quantity > remaining_quantity {
                true => {
                    println!("\tAn order was matched");
                    limit_order.quantity -= remaining_quantity;
                    order.quantity = dec!(0);
                    if should_exectute_trade == true {
                        *trade_id += 1;
                        let timestamp = get_epoch_micro();
                        let user_ids = match order.order_side {
                            OrderSide::Bid => (limit_order.user_id, order.user_id),
                            OrderSide::Ask => (order.user_id, limit_order.user_id),
                        };
                        let post_users = exchange_balance(
                            users,
                            &exchange,
                            remaining_quantity,
                            exchange_price,
                            user_ids.0,
                            user_ids.1
                        );
                        let trade = Filler {
                            trade_id: *trade_id,
                            post_users,
                            exchange: exchange.clone(),
                            quantity: remaining_quantity,
                            exchange_price,
                            is_market_maker: order.is_market_maker,
                            order_status: OrderStatus::Filled,
                            client_order_status: OrderStatus::PartiallyFilled,
                            order_id: order.id,
                            client_order_id: limit_order.id,
                            timestamp,
                        };

                        let order_update_1 = OrderUpdate {
                            order_id: trade.order_id,
                            client_order_id: trade.client_order_id,
                            executed_quantity: trade.quantity,
                            executed_quote_quantity: trade.exchange_price * trade.quantity,
                            order_side: order.order_side.clone(),
                            order_status: trade.order_status.clone(),
                            price: trade.exchange_price,
                            symbol: trade.exchange.symbol.clone(),
                            trade_id: trade.trade_id,
                            trade_timestamp: timestamp,
                            user_id: order.user_id,
                        };
                        let order_update_2 = OrderUpdate {
                            order_id: trade.order_id,
                            client_order_id: trade.client_order_id,
                            executed_quantity: trade.quantity,
                            executed_quote_quantity: trade.exchange_price * trade.quantity,
                            order_side: limit_order.order_side.clone(),
                            order_status: trade.order_status.clone(),
                            price: trade.exchange_price,
                            symbol: trade.exchange.symbol.clone(),
                            trade_id: trade.trade_id,
                            trade_timestamp: timestamp,
                            user_id: limit_order.user_id,
                        };
                        let publish_trade = Trade {
                            id: trade.trade_id,
                            is_market_maker: trade.is_market_maker,
                            price: trade.exchange_price,
                            quantity: trade.quantity,
                            quote_quantity: trade.exchange_price * trade.quantity,
                            timestamp,
                        };
                        let serialized_filler = to_string(&trade).unwrap();
                        let serialized_order_update_1 = to_string(&order_update_1).unwrap();
                        let serialized_order_update_2 = to_string(&order_update_2).unwrap();
                        let serialized_publish_trade = to_string(&publish_trade).unwrap();

                        redis
                            ::cmd("PUBLISH")
                            .arg(format!("order_update:{}", trade.exchange.symbol))
                            .arg(serialized_order_update_1)
                            .query::<Value>(rc);
                        redis
                            ::cmd("PUBLISH")
                            .arg(format!("order_update:{}", trade.exchange.symbol))
                            .arg(serialized_order_update_2)
                            .query::<Value>(rc);
                        redis
                            ::cmd("PUBLISH")
                            .arg(format!("trades:{}", trade.exchange.symbol))
                            .arg(serialized_publish_trade)
                            .query::<Value>(rc);
                        redis::cmd("LPUSH").arg("filler").arg(serialized_filler).query::<Value>(rc);
                    }
                }
                false => {
                    println!("\tAn order was matched");
                    let order_status = match limit_order.quantity == remaining_quantity {
                        true => OrderStatus::Filled,
                        false => OrderStatus::PartiallyFilled,
                    };
                    remaining_quantity -= limit_order.quantity;
                    order.quantity -= limit_order.quantity;
                    if should_exectute_trade == true {
                        *trade_id += 1;

                        let timestamp = get_epoch_micro();
                        let user_ids = match order.order_side {
                            OrderSide::Bid => (limit_order.user_id, order.user_id),
                            OrderSide::Ask => (order.user_id, limit_order.user_id),
                        };
                        let post_users = exchange_balance(
                            users,
                            &exchange,
                            limit_order.quantity,
                            exchange_price,
                            user_ids.0,
                            user_ids.1
                        );
                        let trade = Filler {
                            trade_id: *trade_id,
                            post_users,
                            exchange: exchange.clone(),
                            quantity: limit_order.quantity,
                            exchange_price,
                            is_market_maker: order.is_market_maker,
                            order_status,
                            client_order_status: OrderStatus::Filled,
                            order_id: order.id,
                            client_order_id: limit_order.id,
                            timestamp,
                        };
                        let order_update_1 = OrderUpdate {
                            order_id: trade.order_id,
                            client_order_id: trade.client_order_id,
                            executed_quantity: trade.quantity,
                            executed_quote_quantity: trade.exchange_price * trade.quantity,
                            order_side: order.order_side.clone(),
                            order_status: trade.order_status.clone(),
                            price: trade.exchange_price,
                            symbol: trade.exchange.symbol.clone(),
                            trade_id: trade.trade_id,
                            trade_timestamp: timestamp,
                            user_id: order.user_id,
                        };
                        let order_update_2 = OrderUpdate {
                            order_id: trade.client_order_id,
                            client_order_id: trade.order_id,
                            executed_quantity: trade.quantity,
                            executed_quote_quantity: trade.exchange_price * trade.quantity,
                            order_side: limit_order.order_side.clone(),
                            order_status: trade.order_status.clone(),
                            price: trade.exchange_price,
                            symbol: trade.exchange.symbol.clone(),
                            trade_id: trade.trade_id,
                            trade_timestamp: timestamp,
                            user_id: limit_order.user_id,
                        };
                        let publish_trade = Trade {
                            id: trade.trade_id,
                            is_market_maker: trade.is_market_maker,
                            price: trade.exchange_price,
                            quantity: trade.quantity,
                            quote_quantity: trade.exchange_price * trade.quantity,
                            timestamp: timestamp,
                        };
                        let serialized_filler = to_string(&trade).unwrap();
                        let serialized_order_update_1 = to_string(&order_update_1).unwrap();
                        let serialized_order_update_2 = to_string(&order_update_2).unwrap();
                        let serialized_publish_trade = to_string(&publish_trade).unwrap();
                        redis
                            ::cmd("PUBLISH")
                            .arg(format!("order_update:{}", trade.exchange.symbol))
                            .arg(serialized_order_update_1)
                            .query::<Value>(rc);
                        redis
                            ::cmd("PUBLISH")
                            .arg(format!("order_update:{}", trade.exchange.symbol))
                            .arg(serialized_order_update_2)
                            .query::<Value>(rc);
                        redis
                            ::cmd("PUBLISH")
                            .arg(format!("trades:{}", trade.exchange.symbol))
                            .arg(serialized_publish_trade)
                            .query::<Value>(rc);
                        redis::cmd("LPUSH").arg("filler").arg(serialized_filler).query::<Value>(rc);
                    }

                    self.orders.remove(i);
                    continue;
                }
            }
            if order.is_filled() {
                break;
            }
            i += 1;
        }
        order
    }
    fn total_volume(&self) -> Decimal {
        self.orders
            .iter()
            .map(|order| order.quantity)
            .reduce(|a, b| a + b)
            .unwrap_or(dec!(0))
    }
}
pub fn exchange_balance(
    users: &mut Users,
    exchange: &Exchange,
    quantity: Quantity,
    exchange_price: Price,
    user_id: Id,
    client_user_id: Id
) -> PostUsers {
    users.unlock_amount(&exchange.base, user_id, quantity);
    users.withdraw(&exchange.base, quantity, user_id);
    users.deposit(&exchange.base, quantity, client_user_id);

    users.unlock_amount(&exchange.quote, client_user_id, quantity * exchange_price);
    users.withdraw(&exchange.quote, quantity * exchange_price, client_user_id);
    users.deposit(&exchange.quote, quantity * exchange_price, user_id);

    let user = users.users.get(&user_id).unwrap().clone();
    let client = users.users.get(&client_user_id).unwrap().clone();
    PostUsers {
        client,
        user,
    }
}
