use rust_decimal_macros::dec;
use super::orderbook::{ Orderbook, Order, Price };
use super::error::MatchingEngineErrors;
use super::users::Users;
use super::{ Asset, Id };
use std::collections::HashMap;
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Exchange {
    pub base: Asset,
    pub quote: Asset,
}

impl Exchange {
    pub fn new(base: Asset, quote: Asset) -> Exchange {
        Exchange {
            base,
            quote,
        }
    }
}
#[derive(Debug)]
pub struct MatchingEngine {
    orderbooks: HashMap<Exchange, Orderbook>,
}
impl MatchingEngine {
    pub fn init() -> MatchingEngine {
        MatchingEngine {
            orderbooks: HashMap::new(),
        }
    }
    pub fn add_new_market(&mut self, exchange: Exchange) {
        self.orderbooks.insert(exchange, Orderbook::new());
    }
}

fn setup_engine_and_users() -> (MatchingEngine, Exchange, Orderbook, Users, Vec<Id>) {
    let mut engine = MatchingEngine::init();
    let exchange = Exchange::new(Asset::SOL, Asset::USDT);
    let mut orderbook = Orderbook::new();
    engine.add_new_market(exchange.clone());
    let mut users = Users::init();

    let ids: Vec<_> = (0..6).map(|_| users.new_user()).collect();
    for id in &ids {
        users.deposit(&Asset::USDT, dec!(1_000_000), *id);
        users.deposit(&Asset::SOL, dec!(100), *id);
    }
    (engine, exchange, orderbook, users, ids)
}

#[cfg(test)]
pub mod tests {
    use std::sync::atomic::Ordering;

    use rust_decimal_macros::dec;
    use crate::OrderSide;

    use super::*;
    #[test]
    fn is_sorting_working() {
        let mut orderbook = Orderbook::new();
        orderbook.add_limit_order(dec!(110), Order::new(OrderSide::Ask, dec!(20), true, 1));
        orderbook.add_limit_order(dec!(100), Order::new(OrderSide::Ask, dec!(20), true, 2));
        orderbook.add_limit_order(dec!(99), Order::new(OrderSide::Ask, dec!(20), true, 3));
        orderbook.add_limit_order(dec!(200), Order::new(OrderSide::Ask, dec!(20), true, 4));

        orderbook.add_limit_order(dec!(99), Order::new(OrderSide::Bid, dec!(20), true, 5));
        orderbook.add_limit_order(dec!(100), Order::new(OrderSide::Bid, dec!(20), true, 6));
        orderbook.add_limit_order(dec!(88), Order::new(OrderSide::Bid, dec!(20), true, 7));
        orderbook.add_limit_order(dec!(101), Order::new(OrderSide::Bid, dec!(20), true, 8));

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
        let (mut engine, exchange, mut orderbook, mut users, ids) = setup_engine_and_users();
        // dummy limit orders in orderbook
        orderbook.add_limit_order(dec!(110), Order::new(OrderSide::Ask, dec!(20), true, ids[0]));
        orderbook.add_limit_order(dec!(100), Order::new(OrderSide::Ask, dec!(20), true, ids[2]));
        orderbook.add_limit_order(dec!(99), Order::new(OrderSide::Ask, dec!(20), true, ids[3]));
        orderbook.add_limit_order(dec!(200), Order::new(OrderSide::Ask, dec!(20), true, ids[1]));
        orderbook.add_limit_order(dec!(99), Order::new(OrderSide::Bid, dec!(20), true, ids[2]));
        orderbook.add_limit_order(dec!(100), Order::new(OrderSide::Bid, dec!(20), true, ids[3]));
        orderbook.add_limit_order(dec!(88), Order::new(OrderSide::Bid, dec!(20), true, ids[1]));
        orderbook.add_limit_order(dec!(101), Order::new(OrderSide::Bid, dec!(20), true, ids[2]));

        let bob_order = Order::new(OrderSide::Bid, dec!(10), true, ids[4]);
        orderbook.fill_limit_order(dec!(50), bob_order, &mut users, &exchange);
        assert_eq!(orderbook.bids.contains_key(&dec!(50)), true); // failed to match at best ask(88) so it should be added to orderbook

        let alice_order = Order::new(OrderSide::Ask, dec!(10), true, ids[5]);
        orderbook.fill_limit_order(dec!(201), alice_order, &mut users, &exchange);
        assert_eq!(orderbook.asks.contains_key(&dec!(201)), true); // failed to match at best bid(101) so it should be added to orderbook
    }

    #[test]
    fn if_matched_but_not_filled_bid_order() {
        let (mut engine, exchange, mut orderbook, mut users, ids) = setup_engine_and_users();

        let ask_price_limit_1 = dec!(200);
        let ask_price_limit_2 = dec!(400);
        let ask_price_limit_3 = dec!(600);
        // dummy limit orders in orderbook
        orderbook.add_limit_order(
            ask_price_limit_1,
            Order::new(OrderSide::Ask, dec!(20), true, ids[1])
        );
        orderbook.add_limit_order(
            ask_price_limit_1,
            Order::new(OrderSide::Ask, dec!(5), true, ids[2])
        );
        orderbook.add_limit_order(
            ask_price_limit_2,
            Order::new(OrderSide::Ask, dec!(5), true, ids[3])
        );
        orderbook.add_limit_order(
            ask_price_limit_3,
            Order::new(OrderSide::Ask, dec!(10), true, ids[4])
        );

        let bid_order = Order::new(OrderSide::Bid, dec!(40), true, ids[5]);
        let bid_price_limit_1 = dec!(500);
        orderbook.fill_limit_order(bid_price_limit_1, bid_order, &mut users, &exchange);
        // Checkk all orders for that partically price limit is filled
        assert_eq!(
            orderbook.asks
                .get(&ask_price_limit_1)
                .unwrap()
                .orders.iter()
                .all(|order| order.quantity == dec!(0)),
            true
        );
        // For the Remaining Quantity a new order should be added for the price limit made by the order
        assert_eq!(
            orderbook.bids.get(&bid_price_limit_1).unwrap().orders.get(0).unwrap().quantity,
            dec!(40) - (dec!(20) + dec!(5) + dec!(5))
        );
    }
    #[test]
    fn if_matched_but_not_filled_ask_order() {
        let (mut engine, exchange, mut orderbook, mut users, ids) = setup_engine_and_users();

        let bid_price_limit_1 = dec!(500);
        let bid_price_limit_2 = dec!(400);
        let bid_price_limit_3 = dec!(200);
        // dummy limit orders in orderbook
        orderbook.add_limit_order(
            bid_price_limit_1,
            Order::new(OrderSide::Bid, dec!(20), true, ids[1])
        );
        orderbook.add_limit_order(
            bid_price_limit_1,
            Order::new(OrderSide::Bid, dec!(5), true, ids[2])
        );
        orderbook.add_limit_order(
            bid_price_limit_2,
            Order::new(OrderSide::Bid, dec!(5), true, ids[3])
        );
        orderbook.add_limit_order(
            bid_price_limit_3,
            Order::new(OrderSide::Bid, dec!(10), true, ids[4])
        );

        let ask_order = Order::new(OrderSide::Ask, dec!(40), true, ids[5]);
        let ask_price_limit_1 = dec!(300);
        orderbook.fill_limit_order(ask_price_limit_1, ask_order, &mut users, &exchange);
        // Checkk all orders for that partically price limit is filled
        // println!("{:?}", orderbook.bids.get(&bid_price_limit_1).unwrap().orders);
        println!("{:?}", orderbook.trades);
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
        let (mut engine, exchange, mut orderbook, mut users, ids) = setup_engine_and_users();

        let ask_price_limit_1 = dec!(200);
        let ask_price_limit_2 = dec!(400);
        let ask_price_limit_3 = dec!(600);
        // dummy limit orders in orderbook
        orderbook.add_limit_order(
            ask_price_limit_1,
            Order::new(OrderSide::Ask, dec!(20), true, ids[1])
        );
        orderbook.add_limit_order(
            ask_price_limit_1,
            Order::new(OrderSide::Ask, dec!(5), true, ids[2])
        );
        orderbook.add_limit_order(
            ask_price_limit_2,
            Order::new(OrderSide::Ask, dec!(5), true, ids[3])
        );
        orderbook.add_limit_order(
            ask_price_limit_3,
            Order::new(OrderSide::Ask, dec!(15), true, ids[4])
        );

        let market_order = Order::new(OrderSide::Bid, dec!(40), false, ids[5]);
        orderbook.fill_market_order(market_order, &mut users, &exchange);
        println!("{:?}", orderbook.get_trades());

        assert_eq!(
            orderbook.asks.get(&ask_price_limit_3).unwrap().orders.get(0).unwrap().quantity,
            dec!(5)
        );
        assert_eq!(orderbook.bids.is_empty(), true);
    }

    #[test]
    fn check_balance_limit_order_bid() {
        let (mut engine, exchange, mut orderbook, mut users, ids) = setup_engine_and_users();

        let ask_price_limit_1 = dec!(200);
        let ask_price_limit_2 = dec!(400);
        let ask_price_limit_3 = dec!(600);
        // dummy limit orders in orderbook
        orderbook.add_limit_order(
            ask_price_limit_1,
            Order::new(OrderSide::Ask, dec!(20), true, ids[1])
        );
        orderbook.add_limit_order(
            ask_price_limit_1,
            Order::new(OrderSide::Ask, dec!(5), true, ids[2])
        );
        orderbook.add_limit_order(
            ask_price_limit_2,
            Order::new(OrderSide::Ask, dec!(5), true, ids[3])
        );
        orderbook.add_limit_order(
            ask_price_limit_3,
            Order::new(OrderSide::Ask, dec!(10), true, ids[4])
        );

        let bid_order = Order::new(OrderSide::Bid, dec!(40), true, ids[5]);
        let bid_price_limit_1 = dec!(500);
        orderbook.fill_limit_order(bid_price_limit_1, bid_order, &mut users, &exchange);

        {
            // askers
            assert_eq!(
                users.balance(&Asset::USDT, ids[1]).unwrap(),
                &(dec!(1_000_000) + ask_price_limit_1 * dec!(20))
            );
            assert_eq!(users.balance(&Asset::SOL, ids[1]).unwrap(), &(dec!(100) - dec!(20)));

            assert_eq!(
                users.balance(&Asset::USDT, ids[2]).unwrap(),
                &(dec!(1_000_000) + ask_price_limit_1 * dec!(5))
            );
            assert_eq!(users.balance(&Asset::SOL, ids[2]).unwrap(), &(dec!(100) - dec!(5)));

            assert_eq!(
                users.balance(&Asset::USDT, ids[3]).unwrap(),
                &(dec!(1_000_000) + ask_price_limit_2 * dec!(5))
            );
            assert_eq!(users.balance(&Asset::SOL, ids[3]).unwrap(), &(dec!(100) - dec!(5)));

            // order will not get matched in this case
            assert_eq!(users.balance(&Asset::USDT, ids[4]).unwrap(), &dec!(1_000_000));
            assert_eq!(users.balance(&Asset::SOL, ids[4]).unwrap(), &dec!(100));
        }
        {
            // bidder
            assert_eq!(users.balance(&Asset::SOL, ids[5]).unwrap(), &(dec!(100) + dec!(30)));
            assert_eq!(
                users.balance(&Asset::USDT, ids[5]).unwrap(),
                &(
                    dec!(1_000_000) -
                    (ask_price_limit_1 * dec!(20) +
                        ask_price_limit_1 * dec!(5) +
                        ask_price_limit_2 * dec!(5))
                )
            );
        }
    }

    #[test]
    fn check_balance_limit_order_ask() {
        let (mut engine, exchange, mut orderbook, mut users, ids) = setup_engine_and_users();

        let bid_price_limit_1 = dec!(500);
        let bid_price_limit_2 = dec!(400);
        let bid_price_limit_3 = dec!(200);
        // dummy limit orders in orderbook
        orderbook.add_limit_order(
            bid_price_limit_1,
            Order::new(OrderSide::Bid, dec!(20), true, ids[1])
        );
        orderbook.add_limit_order(
            bid_price_limit_1,
            Order::new(OrderSide::Bid, dec!(5), true, ids[2])
        );
        orderbook.add_limit_order(
            bid_price_limit_2,
            Order::new(OrderSide::Bid, dec!(5), true, ids[3])
        );
        orderbook.add_limit_order(
            bid_price_limit_3,
            Order::new(OrderSide::Bid, dec!(10), true, ids[4])
        );

        let ask_order = Order::new(OrderSide::Ask, dec!(40), true, ids[5]);
        let ask_price_limit_1 = dec!(300);
        orderbook.fill_limit_order(ask_price_limit_1, ask_order, &mut users, &exchange);

        {
            // bidders
            assert_eq!(
                users.balance(&Asset::USDT, ids[1]).unwrap(),
                &(dec!(1_000_000) - ask_price_limit_1 * dec!(20))
            );
            assert_eq!(users.balance(&Asset::SOL, ids[1]).unwrap(), &(dec!(100) + dec!(20)));

            assert_eq!(
                users.balance(&Asset::USDT, ids[2]).unwrap(),
                &(dec!(1_000_000) - ask_price_limit_1 * dec!(5))
            );
            assert_eq!(users.balance(&Asset::SOL, ids[2]).unwrap(), &(dec!(100) + dec!(5)));

            assert_eq!(
                users.balance(&Asset::USDT, ids[3]).unwrap(),
                &(dec!(1_000_000) - ask_price_limit_1 * dec!(5))
            );
            assert_eq!(users.balance(&Asset::SOL, ids[3]).unwrap(), &(dec!(100) + dec!(5)));

            // order will not get matched in this case
            assert_eq!(users.balance(&Asset::USDT, ids[4]).unwrap(), &dec!(1_000_000));
            assert_eq!(users.balance(&Asset::SOL, ids[4]).unwrap(), &dec!(100));
        }
        {
            // asker
            assert_eq!(users.balance(&Asset::SOL, ids[5]).unwrap(), &(dec!(100) - dec!(30)));
            assert_eq!(
                users.balance(&Asset::USDT, ids[5]).unwrap(),
                &(
                    dec!(1_000_000) +
                    (ask_price_limit_1 * dec!(20) +
                        ask_price_limit_1 * dec!(5) +
                        ask_price_limit_1 * dec!(5))
                )
            );
        }
    }
    #[test]
    fn check_balance_market_order() {
        let (mut engine, exchange, mut orderbook, mut users, ids) = setup_engine_and_users();

        let ask_price_limit_1 = dec!(200);
        let ask_price_limit_2 = dec!(400);
        let ask_price_limit_3 = dec!(600);
        // dummy limit orders in orderbook
        orderbook.add_limit_order(
            ask_price_limit_1,
            Order::new(OrderSide::Ask, dec!(20), true, ids[1])
        );
        orderbook.add_limit_order(
            ask_price_limit_1,
            Order::new(OrderSide::Ask, dec!(5), true, ids[2])
        );
        orderbook.add_limit_order(
            ask_price_limit_2,
            Order::new(OrderSide::Ask, dec!(5), true, ids[3])
        );
        orderbook.add_limit_order(
            ask_price_limit_3,
            Order::new(OrderSide::Ask, dec!(15), true, ids[4])
        );

        let market_order = Order::new(OrderSide::Bid, dec!(40), false, ids[5]);
        orderbook.fill_market_order(market_order, &mut users, &exchange);

        {
            // askers
            assert_eq!(
                users.balance(&Asset::USDT, ids[1]).unwrap(),
                &(dec!(1_000_000) + ask_price_limit_1 * dec!(20))
            );
            assert_eq!(users.balance(&Asset::SOL, ids[1]).unwrap(), &(dec!(100) - dec!(20)));

            assert_eq!(
                users.balance(&Asset::USDT, ids[2]).unwrap(),
                &(dec!(1_000_000) + ask_price_limit_1 * dec!(5))
            );
            assert_eq!(users.balance(&Asset::SOL, ids[2]).unwrap(), &(dec!(100) - dec!(5)));

            assert_eq!(
                users.balance(&Asset::USDT, ids[3]).unwrap(),
                &(dec!(1_000_000) + ask_price_limit_2 * dec!(5))
            );
            assert_eq!(users.balance(&Asset::SOL, ids[3]).unwrap(), &(dec!(100) - dec!(5)));

            assert_eq!(
                users.balance(&Asset::USDT, ids[4]).unwrap(),
                &(dec!(1_000_000) + ask_price_limit_3 * dec!(10))
            );
            assert_eq!(users.balance(&Asset::SOL, ids[4]).unwrap(), &(dec!(100) - dec!(10)));
        }

        {
            // taker
            assert_eq!(users.balance(&Asset::SOL, ids[5]).unwrap(), &(dec!(100) + dec!(40)));
            assert_eq!(
                users.balance(&Asset::USDT, ids[5]).unwrap(),
                &(
                    dec!(1_000_000) -
                    (ask_price_limit_1 * dec!(20) +
                        ask_price_limit_1 * dec!(5) +
                        ask_price_limit_2 * dec!(5) +
                        ask_price_limit_3 * dec!(10))
                )
            );
        }
    }
}
