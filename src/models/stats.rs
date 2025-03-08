use serde::{Deserialize, Serialize};

/// Statistics about the current state of the order book
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrderBookStats {
    /// Symbol/ticker these statistics are for
    pub symbol: String,
    /// Current best bid price
    pub best_bid: Option<u64>,
    /// Current best ask price
    pub best_ask: Option<u64>,
    /// Last trade price
    pub last_trade_price: Option<u64>,
    /// Total volume traded
    pub volume: u64,
    /// Total number of trades executed
    pub trade_count: u64,
    /// Number of buy orders in the book
    pub bid_order_count: usize,
    /// Number of ask orders in the book
    pub ask_order_count: usize,
    /// Timestamp of the last update
    pub last_update_time: u64,
}

impl OrderBookStats {
    /// Creates a new OrderBookStats for the given symbol
    pub fn new(symbol: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
            ..Default::default()
        }
    }
    
    /// Returns the current spread (difference between best ask and best bid)
    pub fn spread(&self) -> Option<u64> {
        match (self.best_ask, self.best_bid) {
            (Some(ask), Some(bid)) => Some(ask.saturating_sub(bid)),
            _ => None,
        }
    }

    /// Returns the midpoint price (average of best bid and best ask)
    pub fn midpoint(&self) -> Option<f64> {
        match (self.best_ask, self.best_bid) {
            (Some(ask), Some(bid)) => Some((ask as f64 + bid as f64) / 2.0),
            _ => None,
        }
    }

    /// Updates the statistics with a new trade
    pub fn update_with_trade(&mut self, price: u64, quantity: u64) {
        self.last_trade_price = Some(price);
        self.volume += quantity;
        self.trade_count += 1;
    }

    /// Updates the order counts
    pub fn update_order_counts(&mut self, bid_count: usize, ask_count: usize) {
        self.bid_order_count = bid_count;
        self.ask_order_count = ask_count;
    }
    
    /// Format the best bid price for display
    pub fn formatted_best_bid(&self) -> String {
        match self.best_bid {
            Some(price) => format!("${:.2}", price as f64 / 100.0),
            None => "None".to_string(),
        }
    }
    
    /// Format the best ask price for display
    pub fn formatted_best_ask(&self) -> String {
        match self.best_ask {
            Some(price) => format!("${:.2}", price as f64 / 100.0),
            None => "None".to_string(),
        }
    }
    
    /// Format the spread for display
    pub fn formatted_spread(&self) -> String {
        match self.spread() {
            Some(spread) => format!("${:.2}", spread as f64 / 100.0),
            None => "None".to_string(),
        }
    }
    
    /// Generate a summary of the current market state
    pub fn summary(&self) -> String {
        format!(
            "{} - Bid: {}, Ask: {}, Spread: {}, Volume: {}, Trades: {}",
            self.symbol,
            self.formatted_best_bid(),
            self.formatted_best_ask(),
            self.formatted_spread(),
            self.volume,
            self.trade_count
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_creation() {
        let stats = OrderBookStats::new("BTC-USD");
        assert_eq!(stats.symbol, "BTC-USD");
        assert_eq!(stats.volume, 0);
        assert_eq!(stats.trade_count, 0);
    }

    #[test]
    fn test_spread_calculation() {
        let mut stats = OrderBookStats::new("BTC-USD");
        
        // No spread when no prices exist
        assert!(stats.spread().is_none());
        
        stats.best_bid = Some(9900);
        stats.best_ask = Some(10100);
        
        // Spread should be 200
        assert_eq!(stats.spread(), Some(200));
    }

    #[test]
    fn test_midpoint_calculation() {
        let mut stats = OrderBookStats::new("BTC-USD");
        
        // No midpoint when no prices exist
        assert!(stats.midpoint().is_none());
        
        stats.best_bid = Some(9900);
        stats.best_ask = Some(10100);
        
        // Midpoint should be 10000.0
        assert_eq!(stats.midpoint(), Some(10000.0));
    }

    #[test]
    fn test_trade_update() {
        let mut stats = OrderBookStats::new("BTC-USD");
        
        stats.update_with_trade(10000, 5);
        
        assert_eq!(stats.last_trade_price, Some(10000));
        assert_eq!(stats.volume, 5);
        assert_eq!(stats.trade_count, 1);
        
        stats.update_with_trade(10100, 3);
        
        assert_eq!(stats.last_trade_price, Some(10100));
        assert_eq!(stats.volume, 8);
        assert_eq!(stats.trade_count, 2);
    }
}
