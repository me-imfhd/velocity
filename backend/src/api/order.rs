use std::{ error::Error, str::FromStr };
use rust_decimal::Decimal;
use scylla::transport::errors::QueryError;

use crate::db::{
    get_epoch_ms,
    schema::{ Id, Order, OrderSide, OrderStatus, OrderType, Price, Quantity, Symbol },
    scylla_tables::ScyllaOrder,
    ScyllaDb,
};

impl Order {
    pub fn new(
        id: Id,
        user_id: Id,
        initial_quantity: Quantity,
        price: Price,
        order_side: OrderSide,
        order_type: OrderType,
        symbol: Symbol
    ) -> Order {
        let timestamp = get_epoch_ms();
        Order {
            id,
            user_id,
            filled_quantity: rust_decimal_macros::dec!(0.0),
            initial_quantity,
            price,
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
            price: self.price.to_string(),
            initial_quantity: self.initial_quantity.to_string(),
            order_side: self.order_side.to_string(),
            order_status: self.order_status.to_string(),
            order_type: self.order_type.to_string(),
        }
    }
}
impl ScyllaOrder {
    fn from_scylla_order(&self) -> Order {
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
    pub async fn new_order(&self, order: Order) -> Result<(), QueryError> {
        let s =
            r#"
            INSERT INTO keyspace_1.order_table (
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
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
        "#;
        let order = order.to_scylla_order();
        self.session.query(s, order).await?;
        Ok(())
    }
    pub async fn get_order(&self, order_id: i64) -> Result<Order, Box<dyn Error>> {
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
            WHERE id = ? ;
        "#;
        let res = self.session.query(s, (order_id,)).await?;
        let mut orders = res.rows_typed::<ScyllaOrder>()?;
        let scylla_order = orders
            .next()
            .transpose()?
            .ok_or(QueryError::InvalidMessage("Order does not exist in db".to_string()))?;
        let order = scylla_order.from_scylla_order();
        Ok(order)
    }
    pub async fn update_order(&self, order: &mut Order) -> Result<(), Box<dyn Error>> {
        let order = order.to_scylla_order();
        let s =
            r#"
                UPDATE keyspace_1.order_table 
                SET
                    price = ?,
                    initial_quantity = ?,
                    filled_quantity = ?, 
                    order_type = ?,
                    order_side = ?,
                    order_status = ?
                WHERE id = ? ;
            "#;

        self.session.query(s, (
            order.price,
            order.initial_quantity,
            order.filled_quantity,
            order.order_type,
            order.order_side,
            order.order_status,
            order.id,
        )).await?;
        Ok(())
    }
}
