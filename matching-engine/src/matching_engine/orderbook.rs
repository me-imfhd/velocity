use std::{
    borrow::BorrowMut,
    cell::Cell,
    sync::{ atomic::Ordering, Mutex },
    time::{ self, SystemTime, UNIX_EPOCH },
};
use enum_stringify::EnumStringify;
use redis::{ Connection, PubSub, Value };
use rust_decimal_macros::dec;
use rust_decimal::prelude::*;
use serde::{ Deserialize, Serialize };
use serde_json::to_string;
use std::{ clone, collections::HashMap };
use super::{ engine::{ Exchange, OrderStatus }, error::MatchingEngineErrors, Asset, Id, Quantity };

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orderbook {
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
        rc: &mut Connection
    ) {
        let sorted_orders = match order.order_side {
            OrderSide::Ask => Orderbook::bid_limits(&mut self.bids),
            OrderSide::Bid => Orderbook::ask_limits(&mut self.asks),
        };
        println!("Recieved an market order");
        for limit_order in sorted_orders {
            let price = limit_order.price.clone();
            order = limit_order.fill_order(order, exchange, price, rc);
            if order.is_filled() {
                break;
            }
        }
        self.save_orderbook(rc);
    }
    pub fn fill_limit_order(
        &mut self,
        price: Price,
        mut order: Order,
        exchange: &Exchange,
        rc: &mut Connection
    ) {
        let initial_quantity = order.quantity;
        let result = match order.order_side {
            OrderSide::Ask => {
                println!("Recieved an ask order");
                let sorted_bids = &mut Orderbook::bid_limits(&mut self.bids);
                let mut i = 0;
                if sorted_bids.len() == 0 {
                    self.add_limit_order(price, order);
                    self.save_orderbook(rc);
                    return;
                }
                while i < sorted_bids.len() {
                    if price > sorted_bids[i].price {
                        self.add_limit_order(price, order);
                        break;
                    }
                    order = sorted_bids[i].fill_order(order, exchange, price, rc);
                    if order.quantity > dec!(0) && sorted_bids.get(i + 1).is_none() {
                        self.add_limit_order(price, order);
                        break;
                    }
                    i += 1;
                }
                self.save_orderbook(rc);
            }
            OrderSide::Bid => {
                println!("Recieved an bid order");
                let sorted_asks = &mut Orderbook::ask_limits(&mut self.asks);
                let mut i = 0;
                if sorted_asks.len() == 0 {
                    self.add_limit_order(price, order);
                    self.save_orderbook(rc);
                    return;
                }
                while i < sorted_asks.len() {
                    if price < sorted_asks[i].price {
                        self.add_limit_order(price, order);
                        break;
                    }
                    let price = sorted_asks[i].price.clone();
                    order = sorted_asks[i].fill_order(order, exchange, price, rc);

                    if order.quantity > dec!(0) && sorted_asks.get(i + 1).is_none() {
                        self.add_limit_order(price, order);
                        break;
                    }
                    i += 1;
                }
                self.save_orderbook(rc);
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
    pub fn save_orderbook(&mut self, rc: &mut Connection) {
        println!("Saving orderbook");
        let asks = Orderbook::ask_limits(&mut self.asks);
        let ask_str = to_string(&asks).unwrap();
        let bids = Orderbook::bid_limits(&mut self.bids);
        let bid_str = to_string(&bids).unwrap();
        redis
            ::cmd("MSET")
            .arg(format!("orderbook:{}:asks", self.exchange.symbol))
            .arg(ask_str)
            .arg(format!("orderbook:{}:bids", self.exchange.symbol))
            .arg(bid_str)
            .query::<Value>(rc).unwrap();
    }
    pub fn recover_orderbook(&mut self, redis_connection: &mut redis::Connection) {
        let symbol = &self.exchange.symbol;
        let bids_symbol = "orderbook:".to_string() + &symbol + ":bids";
        let asks_symbol = "orderbook:".to_string() + &symbol + ":asks";
        let bids_store_str = redis
            ::cmd("GET")
            .arg(bids_symbol)
            .query::<String>(redis_connection)
            .expect("Orderbook does not exist, invalid symbol");
        let asks_store_str = redis
            ::cmd("GET")
            .arg(asks_symbol)
            .query::<String>(redis_connection)
            .expect("Orderbook does not exist, invalid symbol");
        let mut bids_store: Vec<Limit> = serde_json::from_str(&bids_store_str).unwrap();
        let mut asks_store: Vec<Limit> = serde_json::from_str(&asks_store_str).unwrap();
        let mut asks = &mut self.asks;
        let mut bids = &mut self.bids;
        asks_store.iter_mut().for_each(|limit| {
            asks.insert(limit.price, limit.clone());
        });
        bids_store.iter_mut().for_each(|limit| {
            bids.insert(limit.price, limit.clone());
        });
        println!("{:#?}", self);
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
pub enum OrderType {
    Market,
    Limit,
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
        rc: &mut Connection
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
                    limit_order.quantity -= remaining_quantity;
                    order.quantity = dec!(0);
                    let trade = match order.order_side {
                        OrderSide::Bid =>
                            QueueTrade {
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
                    let limit_order_status = match limit_order.quantity == remaining_quantity {
                        true => OrderStatus::Filled,
                        false => OrderStatus::PartiallyFilled,
                    };
                    remaining_quantity -= limit_order.quantity;
                    order.quantity -= limit_order.quantity;
                    let trade = match order.order_side {
                        OrderSide::Bid =>
                            QueueTrade {
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
