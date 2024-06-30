use enum_stringify::EnumStringify;
use redis::{ Commands, Connection, FromRedisValue, Value };
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use scylla::Session;
use serde::{ Deserialize, Serialize };
use serde_json::from_str;
use strum::IntoEnumIterator;
use strum_macros::{ EnumIter, FromRepr };
use crate::matching_engine::Symbol;

use super::orderbook::{ Limit, Order, OrderSide, OrderType, Orderbook, Price, Trade };
use super::error::MatchingEngineErrors;
use super::{ Asset, Id, OrderId, Quantity, RegisteredSymbols };
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::ops::Deref;
use std::str::FromStr;
#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize, Serialize)]
pub struct Exchange {
    pub base: Asset,
    pub quote: Asset,
    pub symbol: String,
}
#[derive(Debug, Serialize)]
pub enum SymbolError {
    InvalidSymbol,
}
impl Exchange {
    pub fn new(base: Asset, quote: Asset) -> Exchange {
        let base_string = base.to_string();
        let quote_string = quote.to_string();
        let symbol = format!("{}_{}", base_string, quote_string);
        Exchange {
            base,
            quote,
            symbol,
        }
    }
    pub fn from_symbol(symbol: Symbol) -> Result<Exchange, SymbolError> {
        let symbols: Vec<&str> = symbol.split("_").collect();
        let base_str = symbols.get(0).ok_or(SymbolError::InvalidSymbol)?;
        let quote_str = symbols.get(1).ok_or(SymbolError::InvalidSymbol)?;
        let base = Asset::from_str(&base_str).ok_or(SymbolError::InvalidSymbol)?;
        let quote = Asset::from_str(&quote_str).ok_or(SymbolError::InvalidSymbol)?;
        let exchange = Exchange::new(base, quote);
        Ok(exchange)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MatchingEngine {
    orderbooks: HashMap<Exchange, Orderbook>,
}
impl MatchingEngine {
    pub fn init() -> MatchingEngine {
        MatchingEngine {
            orderbooks: HashMap::new(),
        }
    }
    pub async fn recover_all_orderbooks(
        &mut self,
        session: &Session,
        redis_connection: &mut redis::Connection
    ) {
        let symbols = self.registered_exchanges();
        let mut orderbooks = &mut self.orderbooks;
        for symbol in symbols {
            println!("Recovering {:?} orderbook...", symbol);
            let exchange = Exchange::from_symbol(symbol.to_string()).unwrap();
            let mut orderbook = Orderbook::new(exchange.clone());
            orderbook.recover_orderbook(session, redis_connection).await;
            orderbooks.insert(exchange, orderbook);
        }
        println!("\nOrderbook recovering complete.")
    }
    pub fn registered_exchanges(&self) -> Vec<Symbol> {
        let exchanges: Vec<Symbol> = RegisteredSymbols::iter()
            .map(|s| s.to_string())
            .collect();
        exchanges
    }
    pub fn increment_order_id(&mut self, exchange: &Exchange) -> OrderId {
        self.get_orderbook(exchange).unwrap().order_id += 1;
        self.get_orderbook(exchange).unwrap().order_id as i64
    }
    pub fn process_order(
        &mut self,
        recieved_order: RecievedOrder,
        order_id: i64,
        exchange: &Exchange,
        rc: &mut Connection
    ) {
        match recieved_order.order_type {
            OrderType::Market => {
                let order = Order::new(
                    order_id,
                    recieved_order.timestamp,
                    recieved_order.order_side,
                    recieved_order.initial_quantity,
                    false,
                    recieved_order.user_id
                );
                self.fill_market_order(order, &exchange, rc);
            }
            OrderType::Limit => {
                let order = Order::new(
                    order_id,
                    recieved_order.timestamp,
                    recieved_order.order_side,
                    recieved_order.initial_quantity,
                    true,
                    recieved_order.user_id
                );
                self.fill_limit_order(recieved_order.price, order, &exchange, rc);
            }
        }
    }
    pub fn get_quote(
        &mut self,
        order_side: &OrderSide,
        order_quantity: Quantity,
        exchange: &Exchange
    ) -> Result<Decimal, MatchingEngineErrors> {
        let orderbook = self.get_orderbook(exchange)?;
        orderbook.get_quote(&order_side, order_quantity)
    }

    pub fn add_new_market(
        &mut self,
        exchange: Exchange
    ) -> Result<&mut Self, MatchingEngineErrors> {
        let exists = self.orderbooks.contains_key(&exchange);
        if let true = exists {
            return Err(MatchingEngineErrors::ExchangeAlreadyExist);
        }
        self.orderbooks.insert(exchange.clone(), Orderbook::new(exchange));
        Ok(self)
    }
    pub fn get_orderbook(
        &mut self,
        exchange: &Exchange
    ) -> Result<&mut Orderbook, MatchingEngineErrors> {
        let orderbook = self.orderbooks
            .get_mut(&exchange)
            .ok_or(MatchingEngineErrors::ExchangeDoesNotExist)?;
        Ok(orderbook)
    }
    pub fn fill_market_order(
        &mut self,
        mut order: Order,
        exchange: &Exchange,
        rc: &mut Connection
    ) -> Result<(), MatchingEngineErrors> {
        let mut orderbook = self.get_orderbook(exchange)?;
        Ok(orderbook.fill_market_order(order, exchange, rc, true))
    }
    pub fn fill_limit_order(
        &mut self,
        price: Price,
        mut order: Order,
        exchange: &Exchange,
        rc: &mut Connection
    ) -> Result<(), MatchingEngineErrors> {
        let mut orderbook = self.get_orderbook(exchange)?;
        orderbook.fill_limit_order(price, order, exchange, rc, true);
        Ok(())
    }
    // pub fn add_limit_order(
    //     &mut self,
    //     price: Price,
    //     order: Order,
    //     exchange: &Exchange
    // ) -> Result<(), MatchingEngineErrors> {
    //     let orderbook = self.get_orderbook(exchange)?;
    //     Ok(orderbook.add_limit_order(price, order))
    // }
    pub fn get_asks(
        &mut self,
        exchange: &Exchange
    ) -> Result<Vec<&mut Limit>, MatchingEngineErrors> {
        let orderbook = self.get_orderbook(exchange)?;
        Ok(Orderbook::ask_limits(&mut orderbook.asks))
    }
    pub fn get_bids(
        &mut self,
        exchange: &Exchange
    ) -> Result<Vec<&mut Limit>, MatchingEngineErrors> {
        let orderbook = self.get_orderbook(exchange)?;
        Ok(Orderbook::bid_limits(&mut orderbook.bids))
    }
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RecievedOrder {
    pub user_id: Id,
    pub symbol: Symbol,
    pub price: Price,
    pub initial_quantity: Quantity,
    pub filled_quantity: Quantity,
    pub order_type: OrderType,
    pub order_side: OrderSide,
    pub order_status: OrderStatus,
    pub timestamp: u64,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SaveOrder {
    pub id: OrderId,
    pub recieved_order: RecievedOrder,
}
#[derive(Debug, Clone, Deserialize, Serialize, EnumIter, EnumStringify)]
pub enum OrderStatus {
    InProgress,
    Filled,
    PartiallyFilled,
    Failed,
}
impl OrderStatus {
    pub fn from_str(asset_to_match: &str) -> Result<Self, ()> {
        for asset in OrderStatus::iter() {
            let current_asset = asset.to_string();
            if asset_to_match.to_string() == current_asset {
                return Ok(asset);
            }
        }
        Err(())
    }
}

fn setup_engine_and_users() -> (MatchingEngine, Exchange, Orderbook, Vec<Id>, Connection) {
    let mut engine = MatchingEngine::init();
    let exchange = Exchange::new(Asset::SOL, Asset::USDT);
    let mut orderbook = Orderbook::new(exchange.clone());
    engine.add_new_market(exchange.clone());
    let client = redis::Client::open("redis://127.0.0.1:6379").expect("Could not create client.");
    let mut connection = client.get_connection().expect("Could not connect to the client");
    let ids: Vec<_> = [1, 2, 3, 4, 5, 6, 7, 8].to_vec();
    (engine, exchange, orderbook, ids, connection)
}

#[cfg(test)]
pub mod tests {
    use std::sync::atomic::Ordering;
    use rust_decimal_macros::dec;

    use crate::matching_engine::orderbook::OrderSide;

    use super::*;
    #[test]
    fn is_sorting_working() {
        let mut orderbook = Orderbook::new(Exchange::new(Asset::SOL, Asset::USDT));
        orderbook.add_limit_order(dec!(110), Order::new(1, 1, OrderSide::Ask, dec!(20), true, 1));
        orderbook.add_limit_order(dec!(100), Order::new(2, 2, OrderSide::Ask, dec!(20), true, 2));
        orderbook.add_limit_order(dec!(99), Order::new(3, 3, OrderSide::Ask, dec!(20), true, 3));
        orderbook.add_limit_order(dec!(200), Order::new(4, 4, OrderSide::Ask, dec!(20), true, 4));

        orderbook.add_limit_order(dec!(99), Order::new(5, 5, OrderSide::Bid, dec!(20), true, 5));
        orderbook.add_limit_order(dec!(100), Order::new(6, 6, OrderSide::Bid, dec!(20), true, 6));
        orderbook.add_limit_order(dec!(88), Order::new(7, 7, OrderSide::Bid, dec!(20), true, 7));
        orderbook.add_limit_order(dec!(101), Order::new(8, 8, OrderSide::Bid, dec!(20), true, 8));

        let best_ask_price = Orderbook::ask_limits(&mut orderbook.asks).first().unwrap().price; // first element
        assert_eq!(orderbook.asks.get(&dec!(99)).unwrap().price, best_ask_price);
        let worst_ask_price = Orderbook::ask_limits(&mut orderbook.asks).last().unwrap().price; // last element
        assert_eq!(orderbook.asks.get(&dec!(200)).unwrap().price, worst_ask_price);

        let best_bid_price = Orderbook::bid_limits(&mut orderbook.bids).first().unwrap().price;
        assert_eq!(orderbook.bids.get(&dec!(101)).unwrap().price, best_bid_price);
        let worst_bid_price = Orderbook::bid_limits(&mut orderbook.bids).last().unwrap().price;
        assert_eq!(orderbook.bids.get(&dec!(88)).unwrap().price, worst_bid_price);
    }
    #[test]
    fn adds_to_orderbook_if_didnot_match() {
        let (mut engine, exchange, mut orderbook, ids, mut rc) = setup_engine_and_users();
        // dummy limit orders in orderbook
        orderbook.add_limit_order(
            dec!(110),
            Order::new(1, 1, OrderSide::Ask, dec!(20), true, ids[0])
        );
        orderbook.add_limit_order(
            dec!(100),
            Order::new(2, 2, OrderSide::Ask, dec!(20), true, ids[2])
        );
        orderbook.add_limit_order(
            dec!(99),
            Order::new(3, 3, OrderSide::Ask, dec!(20), true, ids[3])
        );
        orderbook.add_limit_order(
            dec!(200),
            Order::new(4, 4, OrderSide::Ask, dec!(20), true, ids[1])
        );
        orderbook.add_limit_order(
            dec!(99),
            Order::new(5, 5, OrderSide::Bid, dec!(20), true, ids[2])
        );
        orderbook.add_limit_order(
            dec!(100),
            Order::new(6, 6, OrderSide::Bid, dec!(20), true, ids[3])
        );
        orderbook.add_limit_order(
            dec!(88),
            Order::new(7, 7, OrderSide::Bid, dec!(20), true, ids[1])
        );
        orderbook.add_limit_order(
            dec!(101),
            Order::new(8, 8, OrderSide::Bid, dec!(20), true, ids[2])
        );

        let bob_order = Order::new(9, 9, OrderSide::Bid, dec!(10), true, ids[4]);
        orderbook.fill_limit_order(dec!(50), bob_order, &exchange, &mut rc, false);
        assert_eq!(orderbook.bids.contains_key(&dec!(50)), true); // failed to match at best ask(88) so it should be added to orderbook

        let alice_order = Order::new(10, 10, OrderSide::Ask, dec!(10), true, ids[5]);
        orderbook.fill_limit_order(dec!(201), alice_order, &exchange, &mut rc, false);
        assert_eq!(orderbook.asks.contains_key(&dec!(201)), true); // failed to match at best bid(101) so it should be added to orderbook
    }

    #[test]
    fn if_matched_but_not_filled_bid_order() {
        let (mut engine, exchange, mut orderbook, ids, mut rc) = setup_engine_and_users();

        let ask_price_limit_1 = dec!(200);
        let ask_price_limit_2 = dec!(400);
        let ask_price_limit_3 = dec!(600);
        // dummy limit orders in orderbook
        orderbook.add_limit_order(
            ask_price_limit_1,
            Order::new(1, 1, OrderSide::Ask, dec!(20), true, ids[1])
        );
        orderbook.add_limit_order(
            ask_price_limit_1,
            Order::new(2, 2, OrderSide::Ask, dec!(5), true, ids[2])
        );
        orderbook.add_limit_order(
            ask_price_limit_2,
            Order::new(3, 3, OrderSide::Ask, dec!(5), true, ids[3])
        );
        orderbook.add_limit_order(
            ask_price_limit_3,
            Order::new(4, 4, OrderSide::Ask, dec!(10), true, ids[4])
        );

        let bid_order = Order::new(5, 5, OrderSide::Bid, dec!(40), true, ids[5]);
        let bid_price_limit_1 = dec!(500);
        orderbook.fill_limit_order(bid_price_limit_1, bid_order, &exchange, &mut rc, false);
        // For the Remaining Quantity a new order should be added for the price limit made by the order
        assert_eq!(
            orderbook.bids.get(&bid_price_limit_1).unwrap().orders.get(0).unwrap().quantity,
            dec!(40) - (dec!(20) + dec!(5) + dec!(5))
        );
    }
    #[test]
    fn if_matched_but_not_filled_ask_order() {
        let (mut engine, exchange, mut orderbook, ids, mut rc) = setup_engine_and_users();

        let bid_price_limit_1 = dec!(500);
        let bid_price_limit_2 = dec!(400);
        let bid_price_limit_3 = dec!(200);
        // dummy limit orders in orderbook
        orderbook.add_limit_order(
            bid_price_limit_1,
            Order::new(1, 1, OrderSide::Bid, dec!(20), true, ids[1])
        );
        orderbook.add_limit_order(
            bid_price_limit_1,
            Order::new(2, 2, OrderSide::Bid, dec!(5), true, ids[2])
        );
        orderbook.add_limit_order(
            bid_price_limit_2,
            Order::new(3, 3, OrderSide::Bid, dec!(5), true, ids[3])
        );
        orderbook.add_limit_order(
            bid_price_limit_3,
            Order::new(4, 4, OrderSide::Bid, dec!(10), true, ids[4])
        );

        let ask_order = Order::new(5, 5, OrderSide::Ask, dec!(40), true, ids[5]);
        let ask_price_limit_1 = dec!(300);
        orderbook.fill_limit_order(ask_price_limit_1, ask_order, &exchange, &mut rc, false);
        // Checkk all orders for that partically price limit is filled
        // println!("{:?}", orderbook.bids.get(&bid_price_limit_1).unwrap().orders);
        assert_eq!(
            orderbook.bids
                .get(&bid_price_limit_1)
                .unwrap()
                .orders.iter()
                .all(|order| order.quantity == dec!(0)),
            true
        );
        // For the Remaining Quantity a new order should be added for the price limit made by the order
        assert_eq!(
            orderbook.asks.get(&ask_price_limit_1).unwrap().orders.get(0).unwrap().quantity,
            dec!(40) - (dec!(20) + dec!(5) + dec!(5))
        );
    }
    #[test]
    fn fill_market_order() {
        let (mut engine, exchange, mut orderbook, ids, mut rc) = setup_engine_and_users();

        let ask_price_limit_1 = dec!(200);
        let ask_price_limit_2 = dec!(400);
        let ask_price_limit_3 = dec!(600);
        // dummy limit orders in orderbook
        orderbook.add_limit_order(
            ask_price_limit_1,
            Order::new(1, 1, OrderSide::Ask, dec!(20), true, ids[1])
        );
        orderbook.add_limit_order(
            ask_price_limit_1,
            Order::new(2, 2, OrderSide::Ask, dec!(5), true, ids[2])
        );
        orderbook.add_limit_order(
            ask_price_limit_2,
            Order::new(3, 3, OrderSide::Ask, dec!(5), true, ids[3])
        );
        orderbook.add_limit_order(
            ask_price_limit_3,
            Order::new(4, 4, OrderSide::Ask, dec!(15), true, ids[4])
        );

        let market_order = Order::new(5, 5, OrderSide::Bid, dec!(40), false, ids[5]);
        orderbook.fill_market_order(market_order, &exchange, &mut rc, false);
        dbg!(&orderbook.asks);
        assert_eq!(
            orderbook.asks.get(&ask_price_limit_3).unwrap().orders.get(0).unwrap().quantity,
            dec!(5)
        );
        assert_eq!(orderbook.bids.is_empty(), true);
    }
}
