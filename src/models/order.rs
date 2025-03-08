use std::cmp::Ordering;
use std::fmt;
use serde::{Deserialize, Serialize};

/// Represents the side of an order (buy or sell)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

impl fmt::Display for OrderSide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderSide::Buy => write!(f, "Buy"),
            OrderSide::Sell => write!(f, "Sell"),
        }
    }
}

/// Different types of orders that can be placed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderType {
    /// Execute at the specified price or better
    Limit,
    /// Execute immediately at the best available price
    Market,
    /// Becomes a market order when the stop price is reached
    Stop(u64),
    /// Becomes a limit order when the stop price is reached
    StopLimit(u64, u64), // (stop price, limit price)
    /// Immediate-or-Cancel: Execute immediately and cancel any unfilled portion
    IOC,
    /// Fill-or-Kill: Execute the entire order immediately or cancel
    FOK,
}

impl fmt::Display for OrderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderType::Limit => write!(f, "Limit"),
            OrderType::Market => write!(f, "Market"),
            OrderType::Stop(price) => write!(f, "Stop({})", price),
            OrderType::StopLimit(stop, limit) => write!(f, "StopLimit({}, {})", stop, limit),
            OrderType::IOC => write!(f, "IOC"),
            OrderType::FOK => write!(f, "FOK"),
        }
    }
}

/// Current status of an order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderStatus {
    /// New order, not yet processed
    New,
    /// Partially filled order
    PartiallyFilled,
    /// Completely filled order
    Filled,
    /// Canceled order
    Canceled,
    /// Rejected order (e.g., invalid parameters)
    Rejected,
}

/// Represents a trading order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// Unique order identifier
    pub id: u64,
    /// Price in the smallest currency unit (e.g., cents)
    pub price: u64,
    /// Original quantity of the order
    pub quantity: u64,
    /// Remaining quantity to be filled
    pub remaining_quantity: u64,
    /// Order side (buy or sell)
    pub side: OrderSide,
    /// Type of the order (limit, market, etc.)
    pub order_type: OrderType,
    /// Timestamp when the order was created (in nanoseconds)
    pub timestamp: u64,
    /// Current status of the order
    pub status: OrderStatus,
    /// User or account identifier
    pub user_id: u64,
    /// Optional client-provided order identifier
    pub client_order_id: Option<String>,
    /// Symbol/ticker this order is for (e.g., "BTC-USD")
    pub symbol: String,
}

impl Order {
    /// Creates a new limit order
    pub fn new_limit(
        id: u64,
        price: u64,
        quantity: u64,
        side: OrderSide,
        user_id: u64,
        timestamp: u64,
        client_order_id: Option<String>,
        symbol: String,
    ) -> Self {
        Self {
            id,
            price,
            quantity,
            remaining_quantity: quantity,
            side,
            order_type: OrderType::Limit,
            timestamp,
            status: OrderStatus::New,
            user_id,
            client_order_id,
            symbol,
        }
    }

    /// Creates a new market order
    pub fn new_market(
        id: u64,
        quantity: u64,
        side: OrderSide,
        user_id: u64,
        timestamp: u64,
        client_order_id: Option<String>,
        symbol: String,
    ) -> Self {
        Self {
            id,
            // Market orders don't have a specific price, but we set a default
            // For buy orders: u64::MAX (willing to pay any price)
            // For sell orders: 0 (willing to sell at any price)
            price: match side {
                OrderSide::Buy => u64::MAX,
                OrderSide::Sell => 0,
            },
            quantity,
            remaining_quantity: quantity,
            side,
            order_type: OrderType::Market,
            timestamp,
            status: OrderStatus::New,
            user_id,
            client_order_id,
            symbol,
        }
    }

    /// Check if the order is fully filled
    pub fn is_filled(&self) -> bool {
        self.remaining_quantity == 0
    }

    /// Check if the order is a buy order
    pub fn is_buy(&self) -> bool {
        self.side == OrderSide::Buy
    }

    /// Check if the order is a sell order
    pub fn is_sell(&self) -> bool {
        self.side == OrderSide::Sell
    }

    /// Mark the order as partially filled
    pub fn fill_partial(&mut self, filled_quantity: u64) {
        assert!(
            filled_quantity <= self.remaining_quantity,
            "Cannot fill more than remaining quantity"
        );
        
        self.remaining_quantity -= filled_quantity;
        
        if self.remaining_quantity > 0 {
            self.status = OrderStatus::PartiallyFilled;
        } else {
            self.status = OrderStatus::Filled;
        }
    }

    /// Mark the order as fully filled
    pub fn fill_complete(&mut self) {
        self.remaining_quantity = 0;
        self.status = OrderStatus::Filled;
    }

    /// Mark the order as canceled
    pub fn cancel(&mut self) {
        if self.status != OrderStatus::Filled {
            self.status = OrderStatus::Canceled;
        }
    }

    /// Check if this order can match with another order
    pub fn can_match_with(&self, other: &Self) -> bool {
        if self.side == other.side || self.symbol != other.symbol {
            return false; // Same side orders or different symbols don't match
        }
        
        match (self.side, other.side) {
            (OrderSide::Buy, OrderSide::Sell) => self.price >= other.price,
            (OrderSide::Sell, OrderSide::Buy) => self.price <= other.price,
        }
    }
}

/// Orders are equal if they have the same ID
impl PartialEq for Order {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Order {}

/// Orders are compared by price-time priority
/// For buy orders: higher price first, then earlier timestamp
/// For sell orders: lower price first, then earlier timestamp
impl PartialOrd for Order {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Order {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.side, other.side) {
            // For buy orders: higher price comes first
            (OrderSide::Buy, OrderSide::Buy) => {
                match other.price.cmp(&self.price) {
                    Ordering::Equal => self.timestamp.cmp(&other.timestamp),
                    ordering => ordering,
                }
            },
            // For sell orders: lower price comes first
            (OrderSide::Sell, OrderSide::Sell) => {
                match self.price.cmp(&other.price) {
                    Ordering::Equal => self.timestamp.cmp(&other.timestamp),
                    ordering => ordering,
                }
            },
            // Different sides shouldn't be compared directly
            _ => Ordering::Equal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_creation() {
        let buy_limit = Order::new_limit(
            1, 100, 10, OrderSide::Buy, 1001, 123456789, None, "BTC-USD".to_string()
        );
        assert_eq!(buy_limit.side, OrderSide::Buy);
        assert_eq!(buy_limit.order_type, OrderType::Limit);
        assert_eq!(buy_limit.remaining_quantity, 10);
        
        let sell_market = Order::new_market(
            2, 5, OrderSide::Sell, 1002, 123456790, None, "BTC-USD".to_string()
        );
        assert_eq!(sell_market.side, OrderSide::Sell);
        assert_eq!(sell_market.order_type, OrderType::Market);
        assert_eq!(sell_market.price, 0);  // Sell at any price
    }

    #[test]
    fn test_order_fill() {
        let mut order = Order::new_limit(
            1, 100, 10, OrderSide::Buy, 1001, 123456789, None, "BTC-USD".to_string()
        );
        
        order.fill_partial(4);
        assert_eq!(order.remaining_quantity, 6);
        assert_eq!(order.status, OrderStatus::PartiallyFilled);
        
        order.fill_partial(6);
        assert_eq!(order.remaining_quantity, 0);
        assert_eq!(order.status, OrderStatus::Filled);
    }

    #[test]
    fn test_order_matching() {
        let buy = Order::new_limit(
            1, 100, 10, OrderSide::Buy, 1001, 123456789, None, "BTC-USD".to_string()
        );
        let sell_low = Order::new_limit(
            2, 90, 5, OrderSide::Sell, 1002, 123456790, None, "BTC-USD".to_string()
        );
        let sell_high = Order::new_limit(
            3, 110, 5, OrderSide::Sell, 1003, 123456791, None, "BTC-USD".to_string()
        );
        
        assert!(buy.can_match_with(&sell_low)); // Buy at 100, sell at 90 - should match
        assert!(!buy.can_match_with(&sell_high)); // Buy at 100, sell at 110 - shouldn't match
    }

    #[test]
    fn test_order_comparison() {
        // Buy orders with same timestamp, different prices
        let buy1 = Order::new_limit(
            1, 100, 10, OrderSide::Buy, 1001, 100, None, "BTC-USD".to_string()
        );
        let buy2 = Order::new_limit(
            2, 110, 10, OrderSide::Buy, 1002, 100, None, "BTC-USD".to_string()
        );
        
        // Higher buy price should come first
        assert!(buy2 > buy1);
        
        // Buy orders with same price, different timestamps
        let buy3 = Order::new_limit(
            3, 100, 10, OrderSide::Buy, 1003, 100, None, "BTC-USD".to_string()
        );
        let buy4 = Order::new_limit(
            4, 100, 10, OrderSide::Buy, 1004, 200, None, "BTC-USD".to_string()
        );
        
        // Earlier timestamp should come first
        assert!(buy3 < buy4);
        
        // Sell orders with same timestamp, different prices
        let sell1 = Order::new_limit(
            5, 100, 10, OrderSide::Sell, 1005, 100, None, "BTC-USD".to_string()
        );
        let sell2 = Order::new_limit(
            6, 90, 10, OrderSide::Sell, 1006, 100, None, "BTC-USD".to_string()
        );
        
        // Lower sell price should come first
        assert!(sell2 < sell1);
    }
}
