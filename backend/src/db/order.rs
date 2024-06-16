use std::sync::atomic::Ordering;

use super::{enums::*, get_epoch_ms, schema::*, ORDER_ID};

impl OrderSchema {
    pub fn new(
        user_id: Id,
        initial_quantity: Quantity,
        order_side: OrderSideEn,
        order_type: OrderTypeEn,
        symbol: Symbol
    ) -> OrderSchema {
        ORDER_ID.fetch_add(1, Ordering::SeqCst);
        let id = ORDER_ID.load(Ordering::SeqCst);
        let timestamp = get_epoch_ms();
        OrderSchema {
            id: id as i64,
            user_id,
            filled_quantity: 0.0,
            initial_quantity,
            order_side: order_side.to_string(),
            order_status: OrderStatusEn::Processing.to_string(),
            order_type: order_type.to_string(),
            symbol,
            timestamp: timestamp as i64,
        }
    }
}