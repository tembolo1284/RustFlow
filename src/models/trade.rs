use serde::{Deserialize, Serialize};

/// Represents a completed trade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    /// Unique trade identifier
    pub id: u64,
    /// Price at which the trade executed
    pub price: u64,
    /// Quantity traded
    pub quantity: u64,
    /// Timestamp of the trade
    pub timestamp: u64,
    /// Buy order ID
    pub buy_order_id: u64,
    /// Sell order ID
    pub sell_order_id: u64,
    /// Buy order user ID
    pub buy_user_id: u64,
    /// Sell order user ID
    pub sell_user_id: u64,
    /// Symbol/ticker this trade is for (e.g., "BTC-USD")
    pub symbol: String,
}

impl Trade {
    /// Creates a new trade
    pub fn new(
        id: u64,
        price: u64,
        quantity: u64,
        timestamp: u64,
        buy_order_id: u64,
        sell_order_id: u64,
        buy_user_id: u64,
        sell_user_id: u64,
        symbol: String,
    ) -> Self {
        Self {
            id,
            price,
            quantity,
            timestamp,
            buy_order_id,
            sell_order_id,
            buy_user_id,
            sell_user_id,
            symbol,
        }
    }

    /// Returns the total value of the trade (price * quantity)
    pub fn value(&self) -> u64 {
        self.price * self.quantity
    }
    
    /// Convert price from internal representation (e.g., cents) to display format (dollars)
    pub fn formatted_price(&self) -> f64 {
        self.price as f64 / 100.0
    }
    
    /// Generate a simple string representation of the trade
    pub fn summary(&self) -> String {
        format!(
            "Trade #{}: {} {} @ ${:.2} (B: #{}, S: #{})",
            self.id,
            self.quantity,
            self.symbol,
            self.formatted_price(),
            self.buy_order_id,
            self.sell_order_id
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trade_creation() {
        let trade = Trade::new(
            1, 10000, 5, 123456789, 101, 102, 1001, 1002, "BTC-USD".to_string()
        );
        
        assert_eq!(trade.id, 1);
        assert_eq!(trade.price, 10000);
        assert_eq!(trade.quantity, 5);
        assert_eq!(trade.buy_order_id, 101);
        assert_eq!(trade.sell_order_id, 102);
    }

    #[test]
    fn test_trade_value() {
        let trade = Trade::new(
            1, 10000, 5, 123456789, 101, 102, 1001, 1002, "BTC-USD".to_string()
        );
        
        assert_eq!(trade.value(), 50000);
    }

    #[test]
    fn test_formatted_price() {
        let trade = Trade::new(
            1, 10000, 5, 123456789, 101, 102, 1001, 1002, "BTC-USD".to_string()
        );
        
        assert_eq!(trade.formatted_price(), 100.0);
    }
}
