use std::collections::{BTreeMap, HashMap};
use log::{debug, info, warn};

use crate::models::order::{Order, OrderSide, OrderStatus, OrderType};
use crate::models::trade::Trade;
use crate::models::stats::OrderBookStats;
use crate::core::matcher::Matcher;

/// The core order book data structure that maintains bid and ask orders
pub struct OrderBook {
    /// Symbol/ticker this order book represents
    symbol: String,
    
    /// Price-sorted buy orders (highest price first)
    /// BTreeMap<price, Vec<Order>>
    bids: BTreeMap<u64, Vec<Order>>,
    
    /// Price-sorted sell orders (lowest price first)
    /// BTreeMap<price, Vec<Order>>
    asks: BTreeMap<u64, Vec<Order>>,
    
    /// Fast lookup of orders by ID
    orders_by_id: HashMap<u64, Order>,
    
    /// Current statistics
    stats: OrderBookStats,
    
    /// Matching engine
    matcher: Matcher,
}

impl OrderBook {
    /// Creates a new, empty order book for the given symbol
    pub fn new(symbol: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            orders_by_id: HashMap::new(),
            stats: OrderBookStats::new(symbol),
            matcher: Matcher::new(),
        }
    }
    
    /// Returns the symbol this order book represents
    pub fn symbol(&self) -> &str {
        &self.symbol
    }
    
    /// Returns the current statistics of the order book
    pub fn stats(&self) -> &OrderBookStats {
        &self.stats
    }
    
    /// Gets the best bid price
    pub fn best_bid(&self) -> Option<u64> {
        self.bids.keys().next_back().copied()
    }
    
    /// Gets the best ask price
    pub fn best_ask(&self) -> Option<u64> {
        self.asks.keys().next().copied()
    }
    
    /// Gets the current spread (difference between best ask and best bid)
    pub fn spread(&self) -> Option<u64> {
        match (self.best_ask(), self.best_bid()) {
            (Some(ask), Some(bid)) => Some(ask.saturating_sub(bid)),
            _ => None,
        }
    }
    
    /// Adds a new order to the book and attempts to match it
    /// Returns a vector of executed trades
    pub fn process_order(&mut self, order: Order) -> Vec<Trade> {
        let order_id = order.id;
        let order_side = order.side;
        
        // Ensure the order is for this symbol
        if order.symbol != self.symbol {
            warn!("Order symbol mismatch: {} != {}", order.symbol, self.symbol);
            return Vec::new();
        }
        
        // Place the order in the book
        self.orders_by_id.insert(order_id, order.clone());
        
        // Update the current timestamp
        self.stats.last_update_time = order.timestamp;
        
        let mut trades = Vec::new();
        
        // Handle different order types
        match order.order_type {
            OrderType::Market => {
                // Market orders are executed immediately
                trades = self.matcher.match_market_order(
                    order,
                    &mut self.bids,
                    &mut self.asks,
                    &mut self.orders_by_id,
                );
            },
            OrderType::Limit => {
                // Limit orders may be matched immediately or placed in the book
                trades = self.match_limit_order(order);
            },
            OrderType::IOC => {
                // IOC orders are executed immediately and any unfilled portion is canceled
                trades = self.match_limit_order(order.clone());
                
                // Cancel any remaining quantity
                if let Some(mut remaining_order) = self.orders_by_id.get_mut(&order_id) {
                    if remaining_order.remaining_quantity > 0 {
                        remaining_order.cancel();
                        self.remove_order(order_id);
                    }
                }
            },
            OrderType::FOK => {
                // FOK orders must be fully executed or entirely canceled
                let potential_trades = self.matcher.simulate_order_match(
                    &order,
                    &self.bids,
                    &self.asks,
                );
                
                let total_matched = potential_trades.iter().map(|t| t.quantity).sum::<u64>();
                
                if total_matched == order.quantity {
                    // Can be fully executed
                    trades = self.match_limit_order(order);
                } else {
                    // Cancel the order
                    if let Some(mut remaining_order) = self.orders_by_id.get_mut(&order_id) {
                        remaining_order.cancel();
                    }
                    self.remove_order(order_id);
                }
            },
            OrderType::Stop(stop_price) => {
                // Stop orders become market orders when the stop price is reached
                let trigger_condition = match order_side {
                    OrderSide::Buy => self.best_ask().map_or(false, |ask| ask <= stop_price),
                    OrderSide::Sell => self.best_bid().map_or(false, |bid| bid >= stop_price),
                };
                
                if trigger_condition {
                    // Convert to market order and execute
                    let mut market_order = order.clone();
                    market_order.order_type = OrderType::Market;
                    trades = self.matcher.match_market_order(
                        market_order,
                        &mut self.bids,
                        &mut self.asks,
                        &mut self.orders_by_id,
                    );
                } else {
                    // Wait for stop price to be triggered
                    // (in a real system, we'd have a trigger watching for price changes)
                }
            },
            OrderType::StopLimit(stop_price, limit_price) => {
                // Stop-limit orders become limit orders when the stop price is reached
                let trigger_condition = match order_side {
                    OrderSide::Buy => self.best_ask().map_or(false, |ask| ask <= stop_price),
                    OrderSide::Sell => self.best_bid().map_or(false, |bid| bid >= stop_price),
                };
                
                if trigger_condition {
                    // Convert to limit order and execute
                    let mut limit_order = order.clone();
                    limit_order.order_type = OrderType::Limit;
                    limit_order.price = limit_price;
                    trades = self.match_limit_order(limit_order);
                } else {
                    // Wait for stop price to be triggered
                    // (in a real system, we'd have a trigger watching for price changes)
                }
            },
        }
        
        // Update stats
        self.update_stats();
        
        // Update statistics with trade information
        for trade in &trades {
            self.stats.update_with_trade(trade.price, trade.quantity);
        }
        
        trades
    }
    
    /// Cancels an order by ID
    /// Returns true if the order was found and canceled
    pub fn cancel_order(&mut self, order_id: u64) -> bool {
        if let Some(mut order) = self.orders_by_id.get_mut(&order_id) {
            order.cancel();
            self.remove_order(order_id);
            self.update_stats();
            true
        } else {
            false
        }
    }
    
    /// Removes an order from the book
    fn remove_order(&mut self, order_id: u64) -> bool {
        if let Some(order) = self.orders_by_id.remove(&order_id) {
            let level_map = match order.side {
                OrderSide::Buy => &mut self.bids,
                OrderSide::Sell => &mut self.asks,
            };
            
            if let Some(orders) = level_map.get_mut(&order.price) {
                // Find and remove the order
                if let Some(pos) = orders.iter().position(|o| o.id == order_id) {
                    orders.remove(pos);
                    
                    // If the price level is now empty, remove it
                    if orders.is_empty() {
                        level_map.remove(&order.price);
                    }
                    
                    return true;
                }
            }
        }
        
        false
    }
    
    /// Matches a limit order (wrapper around the matcher method)
    fn match_limit_order(&mut self, mut order: Order) -> Vec<Trade> {
        let trades = self.matcher.match_limit_order(
            order.clone(),
            &mut self.bids,
            &mut self.asks,
            &mut self.orders_by_id,
        );
        
        // If the order is not completely filled, add it to the book
        if let Some(updated_order) = self.orders_by_id.get(&order.id) {
            if updated_order.remaining_quantity > 0 {
                // We need to clone because we can't mutably borrow from orders_by_id
                // while it's being used by add_to_book
                let order_to_add = updated_order.clone();
                self.add_to_book(order_to_add);
            }
        }
        
        trades
    }
    
    /// Adds an order to the appropriate side of the book
    fn add_to_book(&mut self, order: Order) {
        // Select the right side of the book
        let level_map = match order.side {
            OrderSide::Buy => &mut self.bids,
            OrderSide::Sell => &mut self.asks,
        };
        
        // Get or create the price level
        let orders = level_map.entry(order.price).or_insert_with(Vec::new);
        
        // Add the order to this price level
        orders.push(order);
        
        // Orders at the same price level are sorted by timestamp (time priority)
        orders.sort_by_key(|o| o.timestamp);
    }
    
    /// Updates the order book statistics
    fn update_stats(&mut self) {
        self.stats.best_bid = self.best_bid();
        self.stats.best_ask = self.best_ask();
        
        // Count orders
        let bid_count = self.bids.values().map(|orders| orders.len()).sum();
        let ask_count = self.asks.values().map(|orders| orders.len()).sum();
        self.stats.update_order_counts(bid_count, ask_count);
    }
    
    /// Returns the current market depth up to the specified number of levels
    pub fn market_depth(&self, levels: usize) -> (Vec<(u64, u64)>, Vec<(u64, u64)>) {
        let mut bids = Vec::new();
        let mut asks = Vec::new();
        
        // Collect bid levels (highest price first)
        for (&price, orders) in self.bids.iter().rev().take(levels) {
            let total_quantity: u64 = orders.iter().map(|o| o.remaining_quantity).sum();
            bids.push((price, total_quantity));
        }
        
        // Collect ask levels (lowest price first)
        for (&price, orders) in self.asks.iter().take(levels) {
            let total_quantity: u64 = orders.iter().map(|o| o.remaining_quantity).sum();
            asks.push((price, total_quantity));
        }
        
        (bids, asks)
    }
    
    /// Returns all orders in the book
    pub fn all_orders(&self) -> Vec<&Order> {
        self.orders_by_id.values().collect()
    }
    
    /// Returns an order by ID
    pub fn get_order(&self, order_id: u64) -> Option<&Order> {
        self.orders_by_id.get(&order_id)
    }
    
    /// Calculate the theoretical slippage for a market order of the given size
    pub fn calculate_slippage(&self, side: OrderSide, quantity: u64) -> Option<(u64, f64)> {
        // Determine which side of the book to match against
        let opposite_levels = match side {
            OrderSide::Buy => &self.asks,
            OrderSide::Sell => &self.bids,
        };
        
        if opposite_levels.is_empty() {
            return None;
        }
        
        // For buy orders: start from lowest ask
        // For sell orders: start from highest bid
        let price_time_iter = match side {
            OrderSide::Buy => opposite_levels.iter(),
            OrderSide::Sell => opposite_levels.iter().rev(),
        };
        
        let mut remaining = quantity;
        let mut total_cost = 0u64;
        let mut total_volume = 0u64;
        
        for (&price, orders) in price_time_iter {
            for order in orders {
                let match_qty = std::cmp::min(remaining, order.remaining_quantity);
                
                total_cost += price * match_qty;
                total_volume += match_qty;
                
                remaining -= match_qty;
                
                if remaining == 0 {
                    // Calculate average execution price
                    let avg_price = total_cost as f64 / total_volume as f64;
                    
                    // Calculate slippage from best price
                    let best_price = match side {
                        OrderSide::Buy => self.best_ask().unwrap_or(price),
                        OrderSide::Sell => self.best_bid().unwrap_or(price),
                    };
                    
                    let slippage_percent = match side {
                        OrderSide::Buy => (avg_price - best_price as f64) / best_price as f64 * 100.0,
                        OrderSide::Sell => (best_price as f64 - avg_price) / best_price as f64 * 100.0,
                    };
                    
                    return Some((total_cost / total_volume, slippage_percent));
                }
            }
        }
        
        // Not enough liquidity to fill the order completely
        None
    }
    
    /// Prints a formatted representation of the order book
    pub fn print_book(&self, depth: usize) {
        println!("\nOrder Book for {}", self.symbol);
        println!("{:-^40}", "");
        
        let (bids, asks) = self.market_depth(depth);
        
        // Print asks (sell orders) in reverse order (highest to lowest)
        for (i, (price, quantity)) in asks.iter().rev().enumerate() {
            println!("{:2} | ${:>10.2} | {:>10} |", 
                     asks.len() - i,
                     *price as f64 / 100.0, 
                     quantity);
        }
        
        // Print the spread
        if let Some(spread) = self.spread() {
            println!("{:-^40}", format!(" Spread: ${:.2} ", spread as f64 / 100.0));
        } else {
            println!("{:-^40}", " No Spread ");
        }
        
        // Print bids (buy orders)
        for (i, (price, quantity)) in bids.iter().enumerate() {
            println!("{:2} | ${:>10.2} | {:>10} |", 
                     i + 1,
                     *price as f64 / 100.0, 
                     quantity);
        }
        
        // Print statistics
        println!("\nStatistics:");
        println!("Best Bid: {}", self.stats.formatted_best_bid());
        println!("Best Ask: {}", self.stats.formatted_best_ask());
        println!("Spread: {}", self.stats.formatted_spread());
        println!("Bid Orders: {}", self.stats.bid_order_count);
        println!("Ask Orders: {}", self.stats.ask_order_count);
        println!("Trade Count: {}", self.stats.trade_count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_order_book_creation() {
        let book = OrderBook::new("BTC-USD");
        assert_eq!(book.symbol(), "BTC-USD");
        assert!(book.best_bid().is_none());
        assert!(book.best_ask().is_none());
    }
    
    #[test]
    fn test_limit_order_placement() {
        let mut book = OrderBook::new("BTC-USD");
        
        // Add a buy limit order
        let buy_order = Order::new_limit(
            1, 100, 10, OrderSide::Buy, 1001, 100, None, "BTC-USD".to_string()
        );
        let trades = book.process_order(buy_order);
        
        assert!(trades.is_empty()); // No trades executed yet
        assert_eq!(book.best_bid(), Some(100)); // Best bid is now 100
        
        // Add a sell limit order above the bid
        let sell_order = Order::new_limit(
            2, 110, 5, OrderSide::Sell, 1002, 200, None, "BTC-USD".to_string()
        );
        let trades = book.process_order(sell_order);
        
        assert!(trades.is_empty()); // No trades (prices don't cross)
        assert_eq!(book.best_ask(), Some(110)); // Best ask is now 110
        
        // Add a sell limit order that crosses with the buy
        let matching_sell = Order::new_limit(
            3, 90, 5, OrderSide::Sell, 1003, 300, None, "BTC-USD".to_string()
        );
        let trades = book.process_order(matching_sell);
        
        assert_eq!(trades.len(), 1); // One trade executed
        assert_eq!(trades[0].price, 100); // Execute at the resting price (100)
        assert_eq!(trades[0].quantity, 5); // Trade for 5 units
    }
    
    // More tests would go here...
}
