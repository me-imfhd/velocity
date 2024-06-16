use super::schema::{Exchange, MarketSchema, Symbol};

impl MarketSchema {
    pub fn new(
        symbol: Symbol,
        max_price: f32,
        min_price: f32,
        tick_size: f32,
        max_quantity: f32,
        min_quantity: f32,
        step_size: f32
    ) -> MarketSchema {
        let exchange = Exchange::from_symbol(symbol.clone());
        MarketSchema {
            symbol,
            base: exchange.base.to_string(),
            quote: exchange.quote.to_string(),
            max_price,
            min_price,
            tick_size,
            max_quantity,
            min_quantity,
            step_size,
        }
    }
}