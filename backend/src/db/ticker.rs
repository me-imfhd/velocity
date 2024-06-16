use super::schema::{ Symbol, TickerSchema };

impl TickerSchema {
    pub fn new(symbol: Symbol) -> TickerSchema {
        TickerSchema {
            symbol,
            base_volume: 0.0,
            high_price: 0.0,
            last_price: 0.0,
            low_price: 0.0,
            price_change: 0.0,
            price_change_percent: 0.0,
            quote_volume: 0.0,
        }
    }
}
