use std::{ borrow::Borrow, ops::Deref, time::Instant };

use redis::{ Connection, Value };
use serde::{ Deserialize, Serialize };
use serde_json::to_string;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    error::MatchingEngineErrors,
    orderbook::{ Order, Orderbook },
    EventTranmitter,
    Exchange,
    Id,
    OrderCancelInfo,
    OrderId,
    OrderSide,
    OrderStatus,
    OrderType,
    PersistCancel,
    PersistCancelAll,
    PersistOrderRequest,
    Price,
    Quantity,
    RecievedOrder,
    SaveOrder,
    Symbol,
    USERS,
};

#[derive(Debug, Serialize, Deserialize)]
pub enum EngineRequests {
    ExecuteOrder(RecievedOrder),
    CancelOrder(CancelOrder),
    CancelAll(CancelAll),
    OpenOrders(OpenOrders),
    OpenOrder(OpenOrder),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelOrder {
    pub id: OrderId,
    pub user_id: Id,
    pub symbol: Symbol,
    pub price: Price,
    pub order_side: OrderSide,
    sub_id: i64,
    pub timestamp: i64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CancelAll {
    user_id: Id,
    symbol: Symbol,
    sub_id: i64,
    timestamp: i64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenOrders {
    user_id: Id,
    symbol: Symbol,
    sub_id: i64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenOrder {
    user_id: Id,
    order_id: OrderId,
    symbol: Symbol,
    sub_id: i64,
}

impl EngineRequests {
    pub fn execute_order(
        start: Instant,
        mut recieved_order: RecievedOrder,
        orderbook: &mut Orderbook,
        con: &mut Connection,
        tx: UnboundedSender<PersistOrderRequest>,
        event_tx: EventTranmitter
    ) {
        println!("Recieved Order");
        let sub_id = recieved_order.id;
        let exchange = Exchange::from_symbol(recieved_order.symbol.clone()).unwrap();
        let (asset, locked_balance) = match recieved_order.order_type {
            OrderType::Market => {
                let quote = orderbook.get_quote(
                    &recieved_order.order_side,
                    recieved_order.initial_quantity
                );
                let mut users = USERS.lock().unwrap();
                let result = users.validate_and_lock_market(
                    quote,
                    &recieved_order.order_side,
                    &exchange,
                    recieved_order.user_id as u64,
                    recieved_order.initial_quantity
                );
                match result {
                    Ok(val) => { val }
                    Err(err) => {
                        redis
                            ::cmd("LPUSH")
                            .arg(sub_id)
                            .arg(err.to_string())
                            .query::<Value>(con)
                            .unwrap();
                        return;
                    }
                }
            }
            OrderType::Limit => {
                let mut users = USERS.lock().unwrap();
                let result = users.validate_and_lock_limit(
                    recieved_order.order_side.clone(),
                    &exchange,
                    recieved_order.user_id as u64,
                    recieved_order.price,
                    recieved_order.initial_quantity
                );
                match result {
                    Ok(val) => { val }
                    Err(err) => {
                        redis
                            ::cmd("LPUSH")
                            .arg(sub_id)
                            .arg(err.to_string())
                            .query::<Value>(con)
                            .unwrap();
                        return;
                    }
                }
            }
        };
        let order_id = orderbook.increment_order_id();
        recieved_order.id = order_id as i64;
        tx.send(
            PersistOrderRequest::Save(SaveOrder {
                locked_balance,
                asset,
                recieved_order: recieved_order.clone(),
            })
        );
        let (filled_quantity, filled_quote_quantity, order_status) = orderbook.process_order(
            recieved_order.clone(),
            order_id,
            event_tx
        );
        let response = RecievedOrder {
            id: order_id as i64,
            filled_quantity,
            filled_quote_quantity,
            initial_quantity: recieved_order.initial_quantity,
            quote_quantity: recieved_order.quote_quantity,
            user_id: recieved_order.user_id,
            order_status,
            order_type: recieved_order.order_type,
            order_side: recieved_order.order_side,
            price: recieved_order.price,
            symbol: recieved_order.symbol,
            timestamp: recieved_order.timestamp,
        };
        println!("Processed order in {} ms", start.elapsed().as_millis());
        redis
            ::cmd("LPUSH")
            .arg(sub_id)
            .arg(to_string(&response).unwrap())
            .query::<Value>(con)
            .unwrap();
    }
    pub fn cancel_order(
        start: Instant,
        cancel_order: CancelOrder,
        orderbook: &mut Orderbook,
        con: &mut Connection,
        tx: UnboundedSender<PersistOrderRequest>
    ) {
        let asset = match cancel_order.order_side {
            OrderSide::Bid => { orderbook.exchange.quote }
            OrderSide::Ask => { orderbook.exchange.base }
        };
        let result = orderbook.cancel_order(
            cancel_order.id,
            cancel_order.user_id,
            &cancel_order.order_side,
            &cancel_order.price
        );
        println!("Canceled order in {}ms", start.elapsed().as_millis());
        match result {
            Ok(order) => {
                let sub_id = cancel_order.sub_id;
                let mut users = USERS.lock().unwrap();
                let quantity = match order.order_side {
                    OrderSide::Bid => order.quantity * cancel_order.price,
                    OrderSide::Ask => order.quantity,
                };
                let updated_locked_balance = *users
                    .unlock_amount(&asset, order.user_id, quantity)
                    .locked_balance.get(&asset)
                    .unwrap();
                drop(users);
                tx.send(
                    PersistOrderRequest::Cancel(PersistCancel {
                        id: cancel_order.id,
                        order_side: cancel_order.order_side,
                        price: cancel_order.price,
                        symbol: cancel_order.symbol.clone(),
                        timestamp: cancel_order.timestamp,
                        updated_locked_balance,
                        asset,
                        user_id: cancel_order.user_id,
                    })
                );
                redis
                    ::cmd("LPUSH")
                    .arg(sub_id)
                    .arg(
                        to_string(
                            &(RecievedOrder {
                                id: order.id as i64,
                                filled_quantity: order.initial_quantity - order.quantity,
                                filled_quote_quantity: order.filled_quote_quantity,
                                initial_quantity: order.initial_quantity,
                                order_side: order.order_side,
                                order_status: OrderStatus::Cancelled,
                                order_type: order.order_type,
                                price: cancel_order.price,
                                quote_quantity: order.initial_quantity * cancel_order.price,
                                symbol: cancel_order.symbol,
                                timestamp: order.timestamp as i64,
                                user_id: order.user_id as i64,
                            })
                        ).unwrap()
                    )
                    .query::<Value>(con)
                    .unwrap();
            }
            Err(err) => {
                redis
                    ::cmd("LPUSH")
                    .arg(cancel_order.sub_id)
                    .arg(err.to_string())
                    .query::<Value>(con)
                    .unwrap();
            }
        }
    }
    pub fn cancel_all_order(
        start: Instant,
        cancel_all: CancelAll,
        orderbook: &mut Orderbook,
        con: &mut Connection,
        tx: UnboundedSender<PersistOrderRequest>
    ) {
        let (orders, locked_balances) = orderbook.cancel_all_orders(cancel_all.user_id);
        println!("Canceled all order in {}ms", start.elapsed().as_millis());
        if orders.len() != 0 {
            tx.send(
                PersistOrderRequest::CancelAll(PersistCancelAll {
                    locked_balances,
                    symbol: cancel_all.symbol,
                    timestamp: cancel_all.timestamp,
                    user_id: cancel_all.user_id as i64,
                    data: orders
                        .iter()
                        .map(|o| OrderCancelInfo {
                            id: o.id,
                            order_side: o.order_side.clone(),
                            price: o.price,
                        })
                        .collect(),
                })
            );
        }
        println!("Canceled all orders in {}ms", start.elapsed().as_millis());
        redis
            ::cmd("LPUSH")
            .arg(cancel_all.sub_id)
            .arg(to_string(&orders).unwrap())
            .query::<Value>(con)
            .unwrap();
    }
    pub fn open_order(
        start: Instant,
        o_order: OpenOrder,
        orderbook: &mut Orderbook,
        con: &mut Connection
    ) {
        let open_orders = orderbook.get_open_orders(o_order.user_id);
        let order = open_orders.iter().find(|(_, o)| o.id == o_order.order_id);
        match order {
            Some((price, order)) => {
                redis
                    ::cmd("LPUSH")
                    .arg(o_order.sub_id)
                    .arg(
                        to_string(
                            &(RecievedOrder {
                                id: order.id as i64,
                                filled_quantity: order.initial_quantity - order.quantity,
                                filled_quote_quantity: order.filled_quote_quantity,
                                initial_quantity: order.initial_quantity,
                                order_side: order.order_side.clone(),
                                order_status: order.order_status.clone(),
                                order_type: order.order_type.clone(),
                                price: *price,
                                quote_quantity: order.initial_quantity * price,
                                symbol: o_order.symbol,
                                timestamp: order.timestamp as i64,
                                user_id: order.user_id as i64,
                            })
                        ).unwrap()
                    )
                    .query::<Value>(con)
                    .unwrap();
            }
            None => {
                redis
                    ::cmd("LPUSH")
                    .arg(o_order.sub_id)
                    .arg(MatchingEngineErrors::InvalidOrderId.to_string())
                    .query::<Value>(con)
                    .unwrap();
            }
        }
    }
    pub fn open_orders(
        start: Instant,
        open_orders: OpenOrders,
        orderbook: &mut Orderbook,
        con: &mut Connection
    ) {
        let mut get_open_orders: Vec<RecievedOrder> = orderbook
            .get_open_orders(open_orders.user_id)
            .iter()
            .map(|(price, order)| {
                RecievedOrder {
                    id: order.id as i64,
                    filled_quantity: order.initial_quantity - order.quantity,
                    initial_quantity: order.initial_quantity,
                    order_side: order.order_side.clone(),
                    order_type: order.order_type.clone(),
                    order_status: order.order_status.clone(),
                    price: *price,
                    quote_quantity: price * order.initial_quantity,
                    symbol: open_orders.symbol.clone(),
                    timestamp: order.timestamp as i64,
                    user_id: order.user_id as i64,
                    filled_quote_quantity: order.filled_quote_quantity,
                }
            })
            .collect();
        redis
            ::cmd("LPUSH")
            .arg(open_orders.sub_id)
            .arg(to_string(&get_open_orders).unwrap())
            .query::<Value>(con)
            .unwrap();
    }
}
