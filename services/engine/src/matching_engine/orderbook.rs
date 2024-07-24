use std::{
    borrow::BorrowMut,
    cell::Cell,
    ops::Deref,
    sync::{ atomic::Ordering, Arc, Mutex },
    time::{ self, SystemTime, UNIX_EPOCH },
};
use enum_stringify::EnumStringify;
use error::MatchingEngineErrors;
use futures::SinkExt;
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
use tokio::sync::mpsc::UnboundedSender;
use std::{ clone, collections::HashMap };
use crate::{ EventTranmitter, RedisEmit };

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
    pub async fn recover_orderbook(&mut self, session: &Session) {
        self.recover_trade_id(&session).await;
        self.recover_order_id(&session).await;
        self.replay_orders(&session).await;
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
    async fn replay_orders(&mut self, session: &Session) {
        let current_time = get_epoch_ms() as i64;
        let since = 1000 * 60 * 60 * 24; // 24 hours in millis
        let from_time = current_time - since;
        let canceled_order_s =
            r#"
        SELECT 
            id,
            user_id,
            order_side,
            symbol,
            price,
            timestamp
        FROM keyspace_1.cancel_order_table
        WHERE timestamp > ? AND symbol = ? ALLOW FILTERING;
            "#;
        let normal_order_s =
            r#"
        SELECT 
            id,
            user_id,
            symbol,
            price,
            initial_quantity,
            filled_quantity, 
            quote_quantity,
            filled_quote_quantity,
            order_type,
            order_side,
            order_status,
            timestamp
        FROM keyspace_1.order_table
        WHERE timestamp > ? AND symbol = ? ALLOW FILTERING;
            "#;
        enum OrderRequest {
            Cancel(ScyllaCancelOrder),
            Normal(RecievedOrder),
        }
        let symbol = &self.exchange.symbol;
        let res = session.query(normal_order_s, (from_time, symbol)).await.unwrap();
        let cancel_res = session.query(canceled_order_s, (from_time, symbol)).await.unwrap();
        let mut orders = res.rows_typed::<ScyllaOrder>().unwrap();
        let mut canceled_orders = cancel_res.rows_typed::<ScyllaCancelOrder>().unwrap();
        let mut replay_orders: Vec<OrderRequest> = orders
            .map(|order| {
                let order = order.unwrap().from_scylla_order();
                OrderRequest::Normal(order)
            })
            .collect();
        let mut canceled_orders: Vec<OrderRequest> = canceled_orders
            .map(|order| {
                let order = order.unwrap();
                OrderRequest::Cancel(order)
            })
            .collect();
        replay_orders.extend(canceled_orders);
        replay_orders.sort_by(|r1, r2| {
            let r1_timestamp = match r1 {
                OrderRequest::Cancel(c_order) => c_order.timestamp,
                OrderRequest::Normal(n_order) => n_order.timestamp,
            };
            let r2_timestamp = match r2 {
                OrderRequest::Cancel(c_order) => c_order.timestamp,
                OrderRequest::Normal(n_order) => n_order.timestamp,
            };
            r1_timestamp.cmp(&r2_timestamp)
        });
        for replay_order in replay_orders {
            match replay_order {
                OrderRequest::Cancel(c_order) => {
                    self.cancel_order(
                        c_order.id as u64,
                        c_order.user_id as u64,
                        &OrderSide::from_str(&c_order.order_side).unwrap(),
                        &Decimal::from_str(&c_order.price).unwrap()
                    ).unwrap();
                    println!("Cancelled an {} Open order", c_order.order_side);
                }
                OrderRequest::Normal(replay_order) => {
                    let order = Order::new(
                        replay_order.id as u64,
                        replay_order.timestamp as u64,
                        replay_order.order_side,
                        replay_order.initial_quantity,
                        replay_order.order_type.clone(),
                        replay_order.user_id as u64
                    );
                    let _ = match replay_order.order_type {
                        OrderType::Market => self.fill_market_order(order, false, None),
                        OrderType::Limit =>
                            self.fill_limit_order(replay_order.price, order, false, None),
                    };
                }
            }
        }
    }

    pub fn process_order(
        &mut self,
        recieved_order: RecievedOrder,
        order_id: OrderId,
        event_tx: EventTranmitter
    ) -> (Decimal, Decimal, OrderStatus) {
        let order = Order::new(
            order_id as u64,
            recieved_order.timestamp as u64,
            recieved_order.order_side,
            recieved_order.initial_quantity,
            recieved_order.order_type.clone(),
            recieved_order.user_id as u64
        );
        match recieved_order.order_type {
            OrderType::Market => { self.fill_market_order(order, true, Some(event_tx)) }
            OrderType::Limit => {
                self.fill_limit_order(recieved_order.price, order, true, Some(event_tx))
            }
        }
    }
    pub fn increment_order_id(&mut self) -> OrderId {
        let mut order_id = &mut self.order_id;
        *order_id += 1;
        *order_id
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
        should_exectute_trade: bool,
        event_tx: Option<EventTranmitter>
    ) -> (Decimal, Decimal, OrderStatus) {
        let sorted_orders = match order.order_side {
            OrderSide::Ask => Orderbook::bid_limits(&mut self.bids),
            OrderSide::Bid => Orderbook::ask_limits(&mut self.asks),
        };
        let mut executed_quantity = dec!(0);
        let mut executed_quote_quantity = dec!(0);
        let mut order_status = order.order_status.clone();
        println!("Recived an {} Market order", order.order_side);
        for limit_order in sorted_orders {
            let price = limit_order.price.clone();
            order = limit_order.fill_order(
                order,
                &self.exchange,
                price,
                &mut self.trade_id,
                should_exectute_trade,
                event_tx.clone()
            );
            let executed_quantity_limit = order.initial_quantity - order.quantity;
            executed_quantity += executed_quantity_limit;
            executed_quote_quantity += executed_quantity_limit * price;
            order_status = order.order_status.clone();
            if order.is_filled() {
                break;
            }
        }
        (executed_quantity, executed_quote_quantity, order_status)
    }
    pub fn fill_limit_order(
        &mut self,
        price: Price,
        mut order: Order,
        should_exectute_trade: bool,
        event_tx: Option<EventTranmitter>
    ) -> (Decimal, Decimal, OrderStatus) {
        println!("Recived an {} Limit order", order.order_side);
        let mut executed_quantity = dec!(0);
        let mut executed_quote_quantity = dec!(0);
        let mut order_status = order.order_status.clone();
        let result = match order.order_side {
            OrderSide::Ask => {
                let sorted_bids = &mut Orderbook::bid_limits(&mut self.bids);
                let mut i = 0;
                if sorted_bids.len() == 0 {
                    self.add_limit_order(price, order);
                    return (executed_quantity, executed_quote_quantity, order_status);
                }
                while i < sorted_bids.len() {
                    if price > sorted_bids[i].price {
                        self.add_limit_order(price, order);
                        break;
                    }
                    order = sorted_bids[i].fill_order(
                        order,
                        &self.exchange,
                        price,
                        &mut self.trade_id,
                        should_exectute_trade,
                        event_tx.clone()
                    );
                    let executed_quantity_limit = order.initial_quantity - order.quantity;
                    executed_quantity += executed_quantity_limit;
                    executed_quote_quantity += executed_quantity_limit * price;
                    order_status = order.order_status.clone();
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
                    return (executed_quantity, executed_quote_quantity, order_status);
                }
                while i < sorted_asks.len() {
                    if price < sorted_asks[i].price {
                        self.add_limit_order(price, order);
                        break;
                    }
                    let price = sorted_asks[i].price.clone();
                    order = sorted_asks[i].fill_order(
                        order,
                        &self.exchange,
                        price,
                        &mut self.trade_id,
                        should_exectute_trade,
                        event_tx.clone()
                    );
                    let executed_quantity_limit = order.initial_quantity - order.quantity;
                    executed_quantity += executed_quantity_limit;
                    executed_quote_quantity += executed_quantity_limit * price;
                    order_status = order.order_status.clone();
                    if order.quantity > dec!(0) && sorted_asks.get(i + 1).is_none() {
                        self.add_limit_order(price, order);
                        break;
                    }
                    i += 1;
                }
            }
        };
        (executed_quantity, executed_quote_quantity, order_status)
    }
    pub fn users_orders(asks: &mut HashMap<Price, Limit>, user_id: Id) -> Vec<(Price, &mut Order)> {
        asks.values_mut()
            .flat_map(|limit| {
                limit.orders
                    .iter_mut()
                    .filter(|order| order.user_id == user_id)
                    .map(|order| (limit.price, order))
                    .collect::<Vec<(Price, &mut Order)>>()
            })
            .collect::<Vec<(Price, &mut Order)>>()
    }
    pub fn get_open_orders(&mut self, user_id: Id) -> Vec<(Price, &mut Order)> {
        let mut open_orders = Orderbook::users_orders(&mut self.asks, user_id);
        open_orders.extend(Orderbook::users_orders(&mut self.bids, user_id));
        open_orders
    }
    pub fn cancel_all_orders(
        &mut self,
        user_id: Id
    ) -> (Vec<RecievedOrder>, HashMap<String, String>) {
        let quote = self.exchange.quote;
        let base = self.exchange.base;
        let symbol = self.exchange.symbol.clone();
        let mut open_orders = self.get_open_orders(user_id);
        let mut users = USERS.lock().unwrap();
        let orders: Vec<RecievedOrder> = open_orders
            .iter()
            .map(|(price, order)| {
                match order.order_side {
                    OrderSide::Bid => {
                        users.unlock_amount(&quote, user_id, order.quantity * price);
                    }
                    OrderSide::Ask => {
                        users.unlock_amount(&base, user_id, order.quantity);
                    }
                }
                RecievedOrder {
                    id: order.id as i64,
                    filled_quantity: order.initial_quantity - order.quantity,
                    filled_quote_quantity: order.filled_quote_quantity,
                    initial_quantity: order.initial_quantity,
                    order_side: order.order_side.clone(),
                    order_status: OrderStatus::Cancelled,
                    order_type: order.order_type.clone(),
                    price: *price,
                    quote_quantity: order.initial_quantity * price,
                    symbol: symbol.clone(),
                    timestamp: order.timestamp as i64,
                    user_id: order.user_id as i64,
                }
            })
            .collect();
        self.asks
            .values_mut()
            .for_each(|limit| limit.orders.retain(|order| order.user_id != user_id));
        self.bids
            .values_mut()
            .for_each(|limit| limit.orders.retain(|order| order.user_id != user_id));
        let locked_balances: &HashMap<String, String> = &users.users
            .get(&user_id)
            .unwrap()
            .locked_balance.iter()
            .map(|(asset, balance)| (asset.to_string(), balance.to_string()))
            .collect();
        (orders, locked_balances.clone())
    }
    // More perfomant
    pub fn cancel_order(
        &mut self,
        order_id: OrderId,
        user_id: Id,
        order_side: &OrderSide,
        price: &Price
    ) -> Result<Order, MatchingEngineErrors> {
        match order_side {
            OrderSide::Bid => {
                let mut limit = self.bids.get_mut(price);
                match limit {
                    Some(limit) => {
                        let index = limit.orders.iter().position(|order| order.id == order_id);
                        match index {
                            Some(index) => {
                                let order = limit.orders.get(index).unwrap().clone();
                                limit.orders.remove(index);
                                Ok(order)
                            }
                            None => { Err(MatchingEngineErrors::InvalidOrderId) }
                        }
                    }
                    None => { Err(MatchingEngineErrors::InvalidPriceLimitOrOrderSide) }
                }
            }
            OrderSide::Ask => {
                let mut limit = self.asks.get_mut(price);
                match limit {
                    Some(limit) => {
                        let index = limit.orders.iter().position(|order| order.id == order_id);
                        match index {
                            Some(index) => {
                                let order = limit.orders.get(index).unwrap().clone();
                                limit.orders.remove(index);
                                Ok(order)
                            }
                            None => { Err(MatchingEngineErrors::InvalidOrderId) }
                        }
                    }
                    None => { Err(MatchingEngineErrors::InvalidPriceLimitOrOrderSide) }
                }
            }
        }
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

    pub fn get_depth(&mut self) -> (HashMap<Price, Quantity>, HashMap<Price, Quantity>) {
        let sorted_bids = Orderbook::bid_limits(&mut self.bids);
        let sorted_asks = Orderbook::bid_limits(&mut self.asks);
        let bids: HashMap<Price, Quantity> = sorted_bids
            .iter()
            .map(|limit| (limit.price, limit.total_volume()))
            .collect();
        let asks: HashMap<Price, Quantity> = sorted_asks
            .iter()
            .map(|limit| (limit.price, limit.total_volume()))
            .collect();
        return (bids, asks);
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
    pub filled_quote_quantity: Quantity,
    pub quantity: Quantity,
    pub order_side: OrderSide,
    pub order_type: OrderType,
    pub order_status: OrderStatus,
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
        order_type: OrderType,
        user_id: Id
    ) -> Order {
        Order {
            id,
            user_id,
            order_side,
            initial_quantity: quantity,
            filled_quote_quantity: dec!(0),
            quantity,
            order_status: OrderStatus::InProgress,
            order_type,
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
        mut trade_id: &mut u64,
        should_exectute_trade: bool,
        event_tx: Option<EventTranmitter>
    ) -> Order {
        let mut remaining_quantity = order.quantity.clone();
        let mut i = 0;
        while i < self.orders.len() {
            if remaining_quantity == dec!(0) {
                break;
            }
            let limit_order = &mut self.orders[i];
            let event_tx = event_tx.clone();
            match limit_order.quantity > remaining_quantity {
                true => {
                    println!("\tAn order was matched");
                    limit_order.quantity -= remaining_quantity;
                    order.quantity = dec!(0);
                    order.order_status = OrderStatus::Filled;
                    limit_order.order_status = OrderStatus::PartiallyFilled;
                    limit_order.filled_quote_quantity += exchange_price * remaining_quantity;
                    if should_exectute_trade == true {
                        *trade_id += 1;
                        let timestamp = get_epoch_micro();
                        let user_ids = match order.order_side {
                            OrderSide::Bid => (limit_order.user_id, order.user_id),
                            OrderSide::Ask => (order.user_id, limit_order.user_id),
                        };
                        let post_users = exchange_balance(
                            &exchange,
                            remaining_quantity,
                            exchange_price,
                            user_ids.0,
                            user_ids.1
                        );
                        let is_buyer_maker = if
                            order.order_type == OrderType::Market &&
                            order.order_side == OrderSide::Bid
                        {
                            true
                        } else {
                            false
                        };
                        let trade = Filler {
                            trade_id: *trade_id,
                            post_users,
                            exchange: exchange.clone(),
                            quantity: remaining_quantity,
                            exchange_price,
                            is_buyer_maker,
                            order_status: order.order_status.clone(),
                            client_order_status: limit_order.order_status.clone(),
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
                            is_buyer_maker: trade.is_buyer_maker,
                            price: trade.exchange_price,
                            quantity: trade.quantity,
                            quote_quantity: trade.exchange_price * trade.quantity,
                            timestamp,
                        };
                        let serialized_filler = to_string(&trade).unwrap();
                        let serialized_order_update_1 = to_string(&order_update_1).unwrap();
                        let serialized_order_update_2 = to_string(&order_update_2).unwrap();
                        let serialized_publish_trade = to_string(&publish_trade).unwrap();
                        event_tx.unwrap().send(
                            vec![
                                RedisEmit {
                                    cmd: "PUBLISH".to_string(),
                                    arg_1: format!("order_update:{}", trade.exchange.symbol),
                                    arg_2: serialized_order_update_1,
                                },
                                RedisEmit {
                                    cmd: "PUBLISH".to_string(),
                                    arg_1: format!("order_update:{}", trade.exchange.symbol),
                                    arg_2: serialized_order_update_2,
                                },

                                RedisEmit {
                                    cmd: "PUBLISH".to_string(),
                                    arg_1: format!("trade:{}", trade.exchange.symbol),
                                    arg_2: serialized_publish_trade,
                                },
                                RedisEmit {
                                    cmd: "LPUSH".to_string(),
                                    arg_1: "filler".to_string(),
                                    arg_2: serialized_filler,
                                }
                            ]
                        );
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
                    order.order_status = order_status;
                    limit_order.order_status = OrderStatus::Filled;
                    limit_order.filled_quote_quantity += exchange_price * limit_order.quantity;
                    if should_exectute_trade == true {
                        *trade_id += 1;
                        let timestamp = get_epoch_micro();
                        let user_ids = match order.order_side {
                            OrderSide::Bid => (limit_order.user_id, order.user_id),
                            OrderSide::Ask => (order.user_id, limit_order.user_id),
                        };
                        let post_users = exchange_balance(
                            &exchange,
                            limit_order.quantity,
                            exchange_price,
                            user_ids.0,
                            user_ids.1
                        );
                        let is_buyer_maker = if
                            order.order_type == OrderType::Market &&
                            order.order_side == OrderSide::Bid
                        {
                            true
                        } else {
                            false
                        };
                        let trade = Filler {
                            trade_id: *trade_id,
                            post_users,
                            exchange: exchange.clone(),
                            quantity: limit_order.quantity,
                            exchange_price,
                            is_buyer_maker,
                            order_status: order.order_status.clone(),
                            client_order_status: limit_order.order_status.clone(),
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
                            is_buyer_maker: trade.is_buyer_maker,
                            price: trade.exchange_price,
                            quantity: trade.quantity,
                            quote_quantity: trade.exchange_price * trade.quantity,
                            timestamp: timestamp,
                        };
                        let serialized_filler = to_string(&trade).unwrap();
                        let serialized_order_update_1 = to_string(&order_update_1).unwrap();
                        let serialized_order_update_2 = to_string(&order_update_2).unwrap();
                        let serialized_publish_trade = to_string(&publish_trade).unwrap();
                        event_tx.unwrap().send(
                            vec![
                                RedisEmit {
                                    cmd: "PUBLISH".to_string(),
                                    arg_1: format!("order_update:{}", trade.exchange.symbol),
                                    arg_2: serialized_order_update_1,
                                },
                                RedisEmit {
                                    cmd: "PUBLISH".to_string(),
                                    arg_1: format!("order_update:{}", trade.exchange.symbol),
                                    arg_2: serialized_order_update_2,
                                },

                                RedisEmit {
                                    cmd: "PUBLISH".to_string(),
                                    arg_1: format!("trade:{}", trade.exchange.symbol),
                                    arg_2: serialized_publish_trade,
                                },
                                RedisEmit {
                                    cmd: "LPUSH".to_string(),
                                    arg_1: "filler".to_string(),
                                    arg_2: serialized_filler,
                                }
                            ]
                        );
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
    exchange: &Exchange,
    quantity: Quantity,
    exchange_price: Price,
    user_id: Id,
    client_user_id: Id
) -> PostUsers {
    let mut users = USERS.lock().unwrap();
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
