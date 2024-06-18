use std::sync::atomic::Ordering;

use scylla::transport::errors::QueryError;

use super::{ get_epoch_ms, schema::*, scylla_tables::ScyllaOrder, ScyllaDb, ORDER_ID };

impl Order {
    pub fn new(
        user_id: Id,
        initial_quantity: Quantity,
        order_side: OrderSide,
        order_type: OrderType,
        symbol: Symbol
    ) -> Order {
        ORDER_ID.fetch_add(1, Ordering::SeqCst);
        let id = ORDER_ID.load(Ordering::SeqCst);
        let timestamp = get_epoch_ms();
        Order {
            id: id as i64,
            user_id,
            filled_quantity: rust_decimal_macros::dec!(0.0),
            initial_quantity,
            order_side,
            order_status: OrderStatus::InProgress,
            order_type,
            symbol,
            timestamp: timestamp as i64,
        }
    }
    fn to_scylla_order(&self) -> ScyllaOrder {
        ScyllaOrder {
            id: self.id,
            timestamp: self.timestamp,
            user_id: self.user_id,
            symbol: self.symbol.to_string(),
            filled_quantity: self.filled_quantity.to_string(),
            initial_quantity: self.initial_quantity.to_string(),
            order_side: self.order_side.to_string(),
            order_status: self.order_status.to_string(),
            order_type: self.order_type.to_string(),
        }
    }
}

impl ScyllaDb {
    pub async fn new_order(&self, order: Order) -> Result<(), QueryError> {
        let s =
            r#"
            INSERT INTO keyspace_1.order_table (
                id,
                user_id,
                symbol,
                initial_quantity,
                filled_quantity, 
                order_type,
                order_side,
                order_status,
                timestamp
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?);
        "#;
        let order = order.to_scylla_order();
        let res = self.session.query(s, order).await?;
        Ok(())
    }
}
