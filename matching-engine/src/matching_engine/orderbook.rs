use std::{
    borrow::BorrowMut,
    cell::Cell,
    sync::{ atomic::Ordering, Arc, Mutex },
    time::{ self, SystemTime, UNIX_EPOCH },
};
use enum_stringify::EnumStringify;
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
use super::{ engine::{ Exchange, OrderStatus }, error::MatchingEngineErrors, Asset, Id, Quantity };

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orderbook {
    pub trade_id: u64,
    pub order_id: u64,
    pub exchange: Exchange,
    pub asks: HashMap<Price, Limit>,
    pub bids: HashMap<Price, Limit>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    id: Id,
    quantity: Quantity,
    quote_quantity: Quantity,
    is_market_maker: bool,
    timestamp: u128,
    price: Price,
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
        should_exectute_trade: bool
    ) {
        let sorted_orders = match order.order_side {
            OrderSide::Ask => Orderbook::bid_limits(&mut self.bids),
            OrderSide::Bid => Orderbook::ask_limits(&mut self.asks),
        };
        println!("Recieved an market order");
        for limit_order in sorted_orders {
            let price = limit_order.price.clone();
            order = limit_order.fill_order(
                order,
                exchange,
                price,
                rc,
                self.trade_id,
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
        should_exectute_trade: bool
    ) {
        let initial_quantity = order.quantity;
        let result = match order.order_side {
            OrderSide::Ask => {
                println!("Recieved an ask order");
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
                        self.trade_id,
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
                println!("Recieved an bid order");
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
                        self.trade_id,
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
    async fn replay_orders(&mut self, rc: &mut redis::Connection, session: &Session) {
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
        WHERE timestamp > ? ALLOW FILTERING;
    "#;
        let res = session.query(s, (from_time,)).await.unwrap();
        let mut orders = res.rows_typed::<SerializedOrder>().unwrap();
        let replay_orders: Vec<ReplayOrder> = orders
            .map(|order| order.unwrap().from_scylla_order())
            .collect();
        let symbol = &self.exchange.symbol;
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
                        &Exchange::from_symbol(replay_order.symbol),
                        rc,
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
                        &Exchange::from_symbol(replay_order.symbol),
                        rc,
                        false
                    );
                }
            }
        }
    }
    pub async fn recover_orderbook(&mut self, session: &Session, rc: &mut redis::Connection) {
        self.recover_trade_id(&session).await;
        self.replay_orders(rc, &session).await;
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

#[derive(Debug, Clone, Serialize, Deserialize, EnumIter, EnumStringify)]
pub enum OrderType {
    Market,
    Limit,
}
impl OrderType {
    pub fn from_str(asset_to_match: &str) -> Result<Self, ()> {
        for asset in OrderType::iter() {
            let current_asset = asset.to_string();
            if asset_to_match.to_string() == current_asset {
                return Ok(asset);
            }
        }
        Err(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, EnumStringify)]
pub enum OrderSide {
    Bid,
    Ask,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Id,
    pub user_id: Id,
    pub quantity: Quantity,
    pub order_side: OrderSide,
    pub is_market_maker: bool,
    pub timestamp: u64,
}

fn get_epoch_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}
impl Order {
    pub fn new(
        id: Id,
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
            quantity,
            is_market_maker,
            timestamp,
        }
    }
    pub fn is_filled(&self) -> bool {
        self.quantity == dec!(0)
    }
}
pub type Price = Decimal;
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
        println!("Adding a new {:?} order to orderbook", &order.order_side);
        self.orders.push(order)
    }
    fn fill_order(
        &mut self,
        mut order: Order,
        exchange: &Exchange,
        exchange_price: Price,
        rc: &mut Connection,
        mut trade_id: u64,
        should_exectute_trade: bool
    ) -> Order {
        let mut remaining_quantity = order.quantity.clone();
        let mut i = 0;
        while i < self.orders.len() {
            if should_exectute_trade == false {
                trade_id += 1;
            }
            if remaining_quantity == dec!(0) {
                break;
            }
            let limit_order = &mut self.orders[i];
            match limit_order.quantity > remaining_quantity {
                true => {
                    limit_order.quantity -= remaining_quantity;
                    order.quantity = dec!(0);
                    if should_exectute_trade == true {
                        continue;
                    }
                    let trade = match order.order_side {
                        OrderSide::Bid =>
                            QueueTrade {
                                trade_id,
                                user_id_1: limit_order.user_id,
                                user_id_2: order.user_id,
                                exchange: exchange.clone(),
                                base_quantity: remaining_quantity,
                                price: exchange_price,
                                is_market_maker: order.is_market_maker,
                                order_status_1: OrderStatus::Filled,
                                order_status_2: OrderStatus::PartiallyFilled,
                                order_id_1: order.id,
                                order_id_2: limit_order.id,
                            },

                        OrderSide::Ask =>
                            QueueTrade {
                                trade_id,
                                user_id_1: order.user_id,
                                user_id_2: limit_order.user_id,
                                exchange: exchange.clone(),
                                base_quantity: remaining_quantity,
                                price: exchange_price,
                                is_market_maker: order.is_market_maker,
                                order_status_1: OrderStatus::Filled,
                                order_status_2: OrderStatus::PartiallyFilled,
                                order_id_1: order.id,
                                order_id_2: limit_order.id,
                            },
                    };
                    let string = to_string(&trade).unwrap();
                    // 1) queue this, 2) update the order request db and then publish it.
                    redis::cmd("LPUSH").arg("queues:trade").arg(string).query::<Value>(rc).unwrap();
                }
                false => {
                    remaining_quantity -= limit_order.quantity;
                    order.quantity -= limit_order.quantity;
                    if should_exectute_trade == true {
                        self.orders.remove(i);
                        continue;
                    }
                    let limit_order_status = match limit_order.quantity == remaining_quantity {
                        true => OrderStatus::Filled,
                        false => OrderStatus::PartiallyFilled,
                    };
                    let trade = match order.order_side {
                        OrderSide::Bid =>
                            QueueTrade {
                                trade_id,
                                user_id_1: limit_order.user_id,
                                user_id_2: order.user_id,
                                exchange: exchange.clone(),
                                base_quantity: limit_order.quantity,
                                price: exchange_price,
                                is_market_maker: order.is_market_maker,
                                order_status_1: limit_order_status,
                                order_status_2: OrderStatus::Filled,
                                order_id_1: order.id,
                                order_id_2: limit_order.id,
                            },

                        OrderSide::Ask =>
                            QueueTrade {
                                trade_id,
                                user_id_1: order.user_id,
                                user_id_2: limit_order.user_id,
                                exchange: exchange.clone(),
                                base_quantity: limit_order.quantity,
                                price: exchange_price,
                                is_market_maker: order.is_market_maker,
                                order_status_1: limit_order_status,
                                order_status_2: OrderStatus::Filled,
                                order_id_1: order.id,
                                order_id_2: limit_order.id,
                            },
                    };
                    let string = to_string(&trade).unwrap();
                    // 1) queue this, 2) update the order request db and then publish it.
                    redis::cmd("LPUSH").arg("queues:trade").arg(string).query::<Value>(rc).unwrap();
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

#[derive(Debug, Serialize)]
pub struct QueueTrade {
    trade_id: Id,
    user_id_1: Id,
    user_id_2: Id,
    exchange: Exchange,
    base_quantity: Quantity,
    price: Price,
    is_market_maker: bool,
    order_status_1: OrderStatus,
    order_status_2: OrderStatus,
    order_id_1: Id,
    order_id_2: Id,
}

#[derive(Debug, Deserialize, Serialize, SerializeRow, FromRow)]
pub struct SerializedOrder {
    pub id: i64,
    pub user_id: i64,
    pub symbol: String,
    pub price: String,
    pub initial_quantity: String,
    pub filled_quantity: String,
    pub order_type: String,
    pub order_side: String,
    pub order_status: String,
    pub timestamp: i64,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct ReplayOrder {
    pub id: u64,
    pub user_id: u64,
    pub symbol: String,
    pub price: Price,
    pub initial_quantity: Quantity,
    pub filled_quantity: Quantity,
    pub order_type: OrderType,
    pub order_side: OrderSide,
    pub order_status: OrderStatus,
    pub timestamp: u64,
}
impl SerializedOrder {
    fn from_scylla_order(&self) -> ReplayOrder {
        ReplayOrder {
            id: self.id as u64,
            timestamp: self.timestamp as u64,
            user_id: self.user_id as u64,
            symbol: self.symbol.to_string(),
            filled_quantity: Decimal::from_str(&self.filled_quantity).unwrap(),
            price: Decimal::from_str(&self.price).unwrap(),
            initial_quantity: Decimal::from_str(&self.initial_quantity).unwrap(),
            order_side: OrderSide::from_str(&self.order_side).unwrap(),
            order_status: OrderStatus::from_str(&self.order_status).unwrap(),
            order_type: OrderType::from_str(&self.order_type).unwrap(),
        }
    }
}
