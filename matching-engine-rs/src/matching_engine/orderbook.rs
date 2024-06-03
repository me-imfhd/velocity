use std::{
    borrow::BorrowMut,
    cell::Cell,
    sync::{ atomic::Ordering, Mutex },
    time::{ self, SystemTime, UNIX_EPOCH },
};
use rust_decimal_macros::dec;
use rust_decimal::prelude::*;
use std::{ clone, collections::HashMap };
use super::{ engine::Exchange, error::UserError, users::Users, Asset, Id, Quantity, ORDER_ID, TRADE_ID };

#[derive(Debug)]
pub struct Orderbook {
    pub asks: HashMap<Price, Limit>,
    pub bids: HashMap<Price, Limit>,
    pub trades: Vec<Trade>,
}
#[derive(Debug)]
pub struct Trade {
    id: Id,
    quantity: Quantity,
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
    pub fn fill_market_order(&mut self, mut order: Order, users: &mut Users, exchange: &Exchange) {
        let sorted_orders = match order.order_side {
            OrderSide::Ask => Orderbook::bid_limits(&mut self.bids),
            OrderSide::Bid => Orderbook::ask_limits(&mut self.asks),
        };
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
                let sorted_bids = &mut Orderbook::bid_limits(&mut self.bids);
                let mut i = 0;
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
                let sorted_asks = &mut Orderbook::ask_limits(&mut self.asks);
                let mut i = 0;
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

#[derive(Debug, Clone)]
pub enum OrderSide {
    Bid,
    Ask,
}

#[derive(Debug, Clone)]
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
#[derive(Debug)]
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
    fn trade(
        users: &mut Users,
        user_id_1: Id,
        user_id_2: Id,
        exchange: &Exchange,
        base_quantity: Quantity,
        price: Price,
        is_market_maker: bool
    ) -> Trade {
        // Create a new trade with the given parameters
        TRADE_ID.fetch_add(1, Ordering::SeqCst);
        let id = TRADE_ID.load(Ordering::SeqCst);
        let timestamp = get_epoch_ms(); // Assuming get_epoch_ms() is defined elsewhere

        // Perform the asset flips
        users.withdraw(&exchange.base, base_quantity, user_id_1);
        users.deposit(&exchange.base, base_quantity, user_id_2);

        users.withdraw(&exchange.quote, base_quantity * price, user_id_2);
        users.deposit(&exchange.quote, base_quantity * price, user_id_1);

        // Return the newly created trade
        Trade {
            id,
            quantity: base_quantity,
            is_market_maker,
            timestamp,
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
        let mut remaining_quantity = order.quantity;
        let mut i = 0;
        while i < self.orders.len() {
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
                    // exchnage balance with limit_order.user_id and order.user_id
                    trades.insert(0, trade);
                    limit_order.quantity = dec!(0);
                }
            }
            if order.is_filled() {
                println!("Order filled");
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
            .unwrap()
    }
}
