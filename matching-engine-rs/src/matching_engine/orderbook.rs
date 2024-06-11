use std::{
    borrow::BorrowMut,
    cell::Cell,
    sync::{ atomic::Ordering, Mutex },
    time::{ self, SystemTime, UNIX_EPOCH },
};
use rust_decimal_macros::dec;
use rust_decimal::prelude::*;
use serde::{ Deserialize, Serialize };
use std::{ clone, collections::HashMap };
use super::{
    engine::Exchange,
    error::MatchingEngineErrors,
    users::Users,
    Asset,
    Id,
    Quantity,
    ORDER_ID,
    TRADE_ID,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orderbook {
    pub asks: HashMap<Price, Limit>,
    pub bids: HashMap<Price, Limit>,
    pub trades: Vec<Trade>,
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
    pub fn new() -> Orderbook {
        Orderbook {
            asks: HashMap::new(),
            bids: HashMap::new(),
            trades: Vec::new(),
        }
    }
    pub fn get_trades(&self) -> &Vec<Trade> {
        &self.trades
    }
    pub fn get_quote(
        &mut self,
        order_side: &OrderSide,
        mut order_quantity: Quantity,
        users: &mut Users
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
    pub fn fill_market_order(&mut self, mut order: Order, users: &mut Users, exchange: &Exchange) {
        let sorted_orders = match order.order_side {
            OrderSide::Ask => Orderbook::bid_limits(&mut self.bids),
            OrderSide::Bid => Orderbook::ask_limits(&mut self.asks),
        };
        println!("Recieved an market order");
        for limit_order in sorted_orders {
            let price = limit_order.price.clone();
            order = limit_order.fill_order(order, &mut self.trades, users, exchange, price);
            if order.is_filled() {
                break;
            }
        }
    }
    pub fn fill_limit_order(
        &mut self,
        price: Price,
        mut order: Order,
        users: &mut Users,
        exchange: &Exchange
    ) {
        let initial_quantity = order.quantity;
        match order.order_side {
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
                        &mut self.trades,
                        users,
                        exchange,
                        price
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
                        &mut self.trades,
                        users,
                        exchange,
                        price
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
    pub fn add_limit_order(&mut self, price: Price, order: Order) {
        match order.order_side {
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
    pub timestamp: u128,
}

fn get_epoch_ms() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
}
impl Order {
    pub fn new(
        order_side: OrderSide,
        quantity: Quantity,
        is_market_maker: bool,
        user_id: Id
    ) -> Order {
        ORDER_ID.fetch_add(1, Ordering::SeqCst);
        let id = ORDER_ID.load(Ordering::SeqCst);
        let timestamp = get_epoch_ms();
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
    fn trade(
        users: &mut Users,
        user_id_1: Id,
        user_id_2: Id,
        exchange: &Exchange,
        base_quantity: Quantity,
        price: Price,
        is_market_maker: bool
    ) -> Trade {
        // Perform the asset flips
        users.unlock_amount(&exchange.base, user_id_1, base_quantity);
        users.withdraw(&exchange.base, base_quantity, user_id_1);
        users.deposit(&exchange.base, base_quantity, user_id_2);

        users.unlock_amount(&exchange.quote, user_id_2, base_quantity * price);
        users.withdraw(&exchange.quote, base_quantity * price, user_id_2);
        users.deposit(&exchange.quote, base_quantity * price, user_id_1);

        TRADE_ID.fetch_add(1, Ordering::SeqCst);
        let id = TRADE_ID.load(Ordering::SeqCst);
        let timestamp = get_epoch_ms();
        println!("Trade occurred.");
        Trade {
            id,
            quantity: base_quantity,
            is_market_maker,
            timestamp,
            quote_quantity: base_quantity * price,
            price,
        }
    }
    fn fill_order(
        &mut self,
        mut order: Order,
        trades: &mut Vec<Trade>,
        users: &mut Users,
        exchange: &Exchange,
        exchange_price: Price
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
                            Limit::trade(
                                users,
                                limit_order.user_id,
                                order.user_id,
                                exchange,
                                remaining_quantity,
                                exchange_price,
                                order.is_market_maker
                            ),
                        OrderSide::Ask =>
                            Limit::trade(
                                users,
                                order.user_id,
                                limit_order.user_id,
                                exchange,
                                remaining_quantity,
                                exchange_price,
                                order.is_market_maker
                            ),
                    };
                    // 1) queue this, 2) update the order request db and then publish it.
                    trades.insert(0, trade);
                }
                false => {
                    remaining_quantity -= limit_order.quantity;
                    order.quantity -= limit_order.quantity;
                    let trade = match order.order_side {
                        OrderSide::Bid =>
                            Limit::trade(
                                users,
                                limit_order.user_id,
                                order.user_id,
                                exchange,
                                limit_order.quantity,
                                exchange_price,
                                order.is_market_maker
                            ),
                        OrderSide::Ask =>
                            Limit::trade(
                                users,
                                order.user_id,
                                limit_order.user_id,
                                exchange,
                                limit_order.quantity,
                                exchange_price,
                                order.is_market_maker
                            ),
                    };
                    self.orders.remove(i);
                    // 1) queue this, 2) update the order request db and then publish it.
                    trades.insert(0, trade);
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
