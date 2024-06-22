use std::{ collections::HashMap, error::Error };
use scylla::{ batch::Batch, SessionBuilder };
use serde_json::from_str;

use crate::{ order, Id, Order, Quantity, QueueTrade, ScyllaDb, Trade, User };

impl ScyllaDb {
    pub async fn create_session(uri: &str) -> Result<ScyllaDb, Box<dyn Error>> {
        let session = SessionBuilder::new().known_node(uri).build().await.map_err(From::from);
        match session {
            Err(err) => Err(err),
            Ok(session) =>
                Ok(ScyllaDb {
                    session,
                }),
        }
    }
    pub fn get_order_batch_values(
        &self,
        queue_trade: QueueTrade,
        mut order_1: Order,
        mut order_2: Order
    ) -> ((String, String, i64), (String, String, i64)) {
        order_1.filled_quantity += queue_trade.base_quantity;
        order_2.filled_quantity += queue_trade.base_quantity;
        order_1.order_status = queue_trade.order_status_1;
        order_2.order_status = queue_trade.order_status_2;
        let serialized_order_1 = order_1.to_scylla_order();
        let serialized_order_2 = order_2.to_scylla_order();
        (
            (
                serialized_order_1.filled_quantity,
                serialized_order_1.order_status,
                serialized_order_1.id,
            ),
            (
                serialized_order_2.filled_quantity,
                serialized_order_2.order_status,
                serialized_order_2.id,
            ),
        )
    }
    pub fn get_trade_batch_values(
        &self,
        queue_trade: &QueueTrade,
        trade_id: Id
    ) -> (i64, String, String, String, bool, String, i64) {
        let trade = Trade::new(
            trade_id,
            queue_trade.is_market_maker,
            queue_trade.price,
            queue_trade.base_quantity,
            queue_trade.exchange.symbol.to_string()
        );
        let serialized_trade = trade.to_scylla_trade();
        (
            serialized_trade.id,
            serialized_trade.symbol,
            serialized_trade.quantity,
            serialized_trade.quote_quantity,
            serialized_trade.is_market_maker,
            serialized_trade.price,
            serialized_trade.timestamp,
        )
    }
    pub fn get_user_batch_values(
        &self,
        mut user_1: User,
        mut user_2: User,
        queue_trade: &QueueTrade
    ) -> (
        (HashMap<String, String>, HashMap<String, String>, i64),
        (HashMap<String, String>, HashMap<String, String>, i64),
    ) {
        let exchange = &queue_trade.exchange;
        let base_quantity = queue_trade.base_quantity;
        let price = queue_trade.price;
        user_1.unlock_amount(&exchange.base, base_quantity);
        user_1.withdraw(&exchange.base, base_quantity);
        user_2.deposit(&exchange.base, base_quantity);

        user_2.unlock_amount(&exchange.quote, base_quantity * price);
        user_2.withdraw(&exchange.quote, base_quantity * price);
        user_1.deposit(&exchange.quote, base_quantity * price);

        let serializer_user_1 = user_1.to_scylla_user();
        let serializer_user_2 = user_2.to_scylla_user();
        (
            (serializer_user_1.balance, serializer_user_1.locked_balance, serializer_user_1.id),
            (serializer_user_2.balance, serializer_user_2.locked_balance, serializer_user_2.id),
        )
    }
    pub async fn exchange_balances(&self, queue_trade: QueueTrade) -> Result<(), Box<dyn Error>> {
        let mut user_1 = self.get_user(queue_trade.user_id_1 as i64).await?;
        let mut user_2 = self.get_user(queue_trade.user_id_2 as i64).await?;
        let mut order_1 = self.get_order(queue_trade.order_id_1).await?;
        let mut order_2 = self.get_order(queue_trade.order_id_2).await?;
        let trade_id = self.new_trade_id().await?;
        let mut batch: Batch = Default::default();
        let user_statement_1 = self.update_user_statement();
        let user_statement_2 = self.update_user_statement();

        let trade_statement = self.trade_entry_statement();

        let order_statement_1 = self.update_order_statement();
        let order_statement_2 = self.update_order_statement();

        batch.append_statement(user_statement_1);
        batch.append_statement(user_statement_2);
        batch.append_statement(trade_statement);
        batch.append_statement(order_statement_1);
        batch.append_statement(order_statement_2);

        let prepared_batch: Batch = self.session.prepare_batch(&batch).await?;

        let (user_1_values, user_2_values) = self.get_user_batch_values(
            user_1,
            user_2,
            &queue_trade
        );
        let trade_values = self.get_trade_batch_values(&queue_trade, trade_id);
        let (order_1_values, order_2_values) = self.get_order_batch_values(
            queue_trade,
            order_1,
            order_2
        );

        self.session.batch(&prepared_batch, (
            user_1_values,
            user_2_values,
            trade_values,
            order_1_values,
            order_2_values,
        )).await?;
        Ok(())
    }
}
