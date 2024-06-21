use std::{ collections::HashMap, error::Error };
use scylla::{ batch::Batch, SessionBuilder };
use serde_json::from_str;

use crate::{ QueueTrade, ScyllaDb, User };

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
    pub fn get_order_batch_values(&self) {
        todo!()
    }
    pub fn get_trade_batch_values(&self) {
        todo!()
    }
    pub fn get_user_batch_values(
        &self,
        mut user_1: User,
        mut user_2: User,
        queue_trade: QueueTrade
    ) -> (
        (HashMap<String, String>, HashMap<String, String>, i64),
        (HashMap<String, String>, HashMap<String, String>, i64),
    ) {
        let exchange = queue_trade.exchange;
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
        let mut user_1 = self.get_user(queue_trade.user_id_1 as i64).await.unwrap();
        let mut user_2 = self.get_user(queue_trade.user_id_2 as i64).await.unwrap();
        let mut batch: Batch = Default::default();
        let user_statement_1 = self.update_user_statement();
        let user_statement_2 = self.update_user_statement();

        let order_statement_1 = self.update_order_statement();
        let order_statement_2 = self.update_order_statement();

        let trade_statement = self.trade_entry_statement();
        batch.append_statement(user_statement_1);
        batch.append_statement(user_statement_2);
        // batch.append_statement(order_statement_1);
        // batch.append_statement(order_statement_2);
        // batch.append_statement(trade_statement);

        let prepared_batch: Batch = self.session.prepare_batch(&batch).await?;

        let (user_1_values, user_2_values) = self.get_user_batch_values(
            user_1,
            user_2,
            queue_trade
        );

        self.session.batch(&prepared_batch, (user_1_values, user_2_values)).await.unwrap();
        Ok(())
    }
}
