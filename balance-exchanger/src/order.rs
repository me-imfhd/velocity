use std::{ error::Error, str::FromStr };
use rust_decimal::Decimal;
use scylla::{
    query::{ self, Query },
    statement::{ Consistency, SerialConsistency },
    transport::errors::QueryError,
};

use crate::{
    Id,
    Order,
    OrderId,
    OrderSide,
    OrderStatus,
    OrderType,
    Price,
    Quantity,
    ScyllaDb,
    ScyllaOrder,
    Symbol,
};

impl Order {
    pub fn to_scylla_order(&self) -> ScyllaOrder {
        ScyllaOrder {
            id: self.id,
            timestamp: self.timestamp,
            user_id: self.user_id,
            symbol: self.symbol.to_string(),
            filled_quantity: self.filled_quantity.to_string(),
            price: self.price.to_string(),
            initial_quantity: self.initial_quantity.to_string(),
            order_side: self.order_side.to_string(),
            order_status: self.order_status.to_string(),
            order_type: self.order_type.to_string(),
        }
    }
}
impl ScyllaOrder {
    pub fn from_scylla_order(&self) -> Order {
        Order {
            id: self.id,
            timestamp: self.timestamp,
            user_id: self.user_id,
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

impl ScyllaDb {
    pub async fn get_order(
        &self,
        order_id: OrderId,
        symbol: &Symbol
    ) -> Result<Order, Box<dyn Error>> {
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
            WHERE id = ? AND symbol = ?;
        "#;
        let res = self.session.query(s, (order_id, symbol)).await?;
        let mut orders = res.rows_typed::<ScyllaOrder>()?;
        let scylla_order = orders
            .next()
            .transpose()?
            .ok_or(QueryError::InvalidMessage("Order does not exist in db".to_string()))?;
        let order = scylla_order.from_scylla_order();
        Ok(order)
    }
    pub fn update_order_statement(&self) -> &str {
        let s =
            r#"
            UPDATE keyspace_1.order_table 
            SET
                filled_quantity = ?, 
                order_status = ?
                WHERE id = ? AND symbol = ?;
        "#;
        s
    }
}
