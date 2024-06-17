use std::sync::atomic::Ordering;

use scylla::transport::errors::QueryError;

use super::{ enums::*, get_epoch_ms, schema::*, ScyllaDb, ORDER_ID };

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
            order_status: OrderStatusEn::InProgress.to_string(),
            order_type: order_type.to_string(),
            symbol,
            timestamp: timestamp as i64,
        }
    }
}

impl ScyllaDb {
    pub async fn new_order(&self, order: OrderSchema) -> Result<(), QueryError> {
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
        let res = self.session.query(s, order).await?;
        Ok(())
    }
}