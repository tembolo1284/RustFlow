use std::collections::{BTreeMap, HashMap};
use log::{debug, info, warn};

use crate::models::order::{Order, OrderSide, OrderStatus, OrderType};
use crate::models::trade::Trade;

/// The matching engine component that pairs buy and sell orders
pub struct Matcher {
    /// Last generated trade ID
    last_trade_id: u64,
}

impl Matcher {
    /// Creates a new matcher
    pub fn new() -> Self {
        Self {
            last_trade_id: 0,
        }
    }

    /// Generate the next trade ID
    pub fn next_trade_id(&mut self) -> u64 {
        self.last_trade_id += 1;
        self.last_trade_id
    }

    /// Matches a market order immediately against the provided order book sides
    pub fn match_market_order(
        &mut self,
        mut order: Order,
        bids: &mut BTreeMap<u64, Vec<Order>>,
        asks: &mut BTreeMap<u64, Vec<Order>>,
        orders_by_id: &mut HashMap<u64, Order>,
    ) -> Vec<Trade> {
        let mut trades = Vec::new();
        
        // Determine which side of the book to match against
        let opposite_levels = match order.side {
            OrderSide::Buy => asks,
            OrderSide::Sell => bids,
        };
        
        // Keep matching until the order is filled or the opposite side is exhausted
        while order.remaining_quantity > 0 && !opposite_levels.is_empty() {
            let best_price = match order.side {
                OrderSide::Buy => *opposite_levels.keys().next().unwrap(),
                OrderSide::Sell => *opposite_levels.keys().next_back().unwrap(),
            };
            
            let level_orders = opposite_levels.get_mut(&best_price).unwrap();
            
            if level_orders.is_empty() {
                opposite_levels.remove(&best_price);
                continue;
            }
            
            // Match with the first order at this price level
            let mut opposite_order = &mut level_orders[0];
            
            // Calculate the match quantity
            let match_qty = std::cmp::min(order.remaining_quantity, opposite_order.remaining_quantity);
            
            // Create the trade
            let trade = Trade {
                id: self.next_trade_id(),
                price: best_price,
                quantity: match_qty,
                timestamp: std::cmp::max(order.timestamp, opposite_order.timestamp),
                buy_order_id: if order.is_buy() { order.id } else { opposite_order.id },
                sell_order_id: if order.is_sell() { order.id } else { opposite_order.id },
                buy_user_id: if order.is_buy() { order.user_id } else { opposite_order.user_id },
                sell_user_id: if order.is_sell() { order.user_id } else { opposite_order.user_id },
                symbol: order.symbol.clone(),
            };
            
            // Update the orders
            order.fill_partial(match_qty);
            
            // We need to update the order in the orders_by_id map
            if let Some(stored_order) = orders_by_id.get_mut(&order.id) {
                stored_order.fill_partial(match_qty);
            }
            
            // Update the opposite order
            opposite_order.fill_partial(match_qty);
            
            // Add the trade to the results
            trades.push(trade);
            
            // If the opposite order is now filled, remove it
            if opposite_order.is_filled() {
                // We need to clone the ID because we can't mutably borrow the order
                // and then remove it by ID in the same scope
                let opposite_id = opposite_order.id;
                
                // Mark it as filled in the orders_by_id map as well
                if let Some(stored_order) = orders_by_id.get_mut(&opposite_id) {
                    stored_order.fill_complete();
                }
                
                // Now remove the first order
                level_orders.remove(0);
                
                // If the level is now empty, we'll remove it in the next iteration
            }
        }
        
        // For market orders, we don't add any remaining quantity to the book
        // It's either filled completely or filled as much as possible
        if order.remaining_quantity > 0 {
            // In a real system, we might report "unable to fill completely" here
            warn!(
                "Market order {} could not be filled completely. Remaining: {}",
                order.id, order.remaining_quantity
            );
        }
        
        trades
    }
    
    /// Matches a limit order against the opposite side of the book
    /// Returns a vector of executed trades
    pub fn match_limit_order(
        &mut self,
        mut order: Order,
        bids: &mut BTreeMap<u64, Vec<Order>>,
        asks: &mut BTreeMap<u64, Vec<Order>>,
        orders_by_id: &mut HashMap<u64, Order>,
    ) -> Vec<Trade> {
        let mut trades = Vec::new();
        
        // Determine which side of the book to match against
        let opposite_levels = match order.side {
            OrderSide::Buy => asks,
            OrderSide::Sell => bids,
        };
        
        // Keep matching while there's a favorable price on the opposite side
        while order.remaining_quantity > 0 && !opposite_levels.is_empty() {
            let best_opposite_price = match order.side {
                OrderSide::Buy => {
                    if let Some(&price) = opposite_levels.keys().next() {
                        if price <= order.price {
                            price
                        } else {
                            break; // No more favorable prices
                        }
                    } else {
                        break; // No more orders
                    }
                },
                OrderSide::Sell => {
                    if let Some(&price) = opposite_levels.keys().next_back() {
                        if price >= order.price {
                            price
                        } else {
                            break; // No more favorable prices
                        }
                    } else {
                        break; // No more orders
                    }
                },
            };
            
            let level_orders = opposite_levels.get_mut(&best_opposite_price).unwrap();
            
            if level_orders.is_empty() {
                opposite_levels.remove(&best_opposite_price);
                continue;
            }
            
            // Match with the first order at this price level
            let mut opposite_order = &mut level_orders[0];
            
            // Calculate the match quantity
            let match_qty = std::cmp::min(order.remaining_quantity, opposite_order.remaining_quantity);
            
            // Create the trade
            let trade = Trade {
                id: self.next_trade_id(),
                price: best_opposite_price,
                quantity: match_qty,
                timestamp: std::cmp::max(order.timestamp, opposite_order.timestamp),
                buy_order_id: if order.is_buy() { order.id } else { opposite_order.id },
                sell_order_id: if order.is_sell() { order.id } else { opposite_order.id },
                buy_user_id: if order.is_buy() { order.user_id } else { opposite_order.user_id },
                sell_user_id: if order.is_sell() { order.user_id } else { opposite_order.user_id },
                symbol: order.symbol.clone(),
            };
            
            // Update the orders
            order.fill_partial(match_qty);
            
            // We need to update the order in the orders_by_id map
            if let Some(stored_order) = orders_by_id.get_mut(&order.id) {
                stored_order.fill_partial(match_qty);
            }
            
            // Update the opposite order
            opposite_order.fill_partial(match_qty);
            
            // Add the trade to the results
            trades.push(trade);
            
            // If the opposite order is now filled, remove it
            if opposite_order.is_filled() {
                // We need to clone the ID because we can't mutably borrow the order
                // and then remove it by ID in the same scope
                let opposite_id = opposite_order.id;
                
                // Mark it as filled in the orders_by_id map as well
                if let Some(stored_order) = orders_by_id.get_mut(&opposite_id) {
                    stored_order.fill_complete();
                }
                
                // Now remove the first order
                level_orders.remove(0);
                
                // If the level is now empty, we'll remove it in the next iteration
            }
        }
        
        trades
    }
    
    /// Simulates matching an order without actually executing it
    /// Used for FOK orders to see if they can be fully filled
    pub fn simulate_order_match(
        &self,
        order: &Order,
        bids: &BTreeMap<u64, Vec<Order>>,
        asks: &BTreeMap<u64, Vec<Order>>,
    ) -> Vec<Trade> {
        let mut simulated_trades = Vec::new();
        let mut remaining_qty = order.remaining_quantity;
        
        // Clone the opposite side's price levels for simulation
        let opposite_levels = match order.side {
            OrderSide::Buy => asks.clone(),
            OrderSide::Sell => bids.clone(),
        };
        
        // Simulate matching against the opposite side
        for (&price, level_orders) in match order.side {
            OrderSide::Buy => opposite_levels.iter(),
            OrderSide::Sell => opposite_levels.iter().rev(),
        } {
            // For a buy order, only match if the ask price is <= order price
            // For a sell order, only match if the bid price is >= order price
            let price_matches = match order.side {
                OrderSide::Buy => price <= order.price,
                OrderSide::Sell => price >= order.price,
            };
            
            if !price_matches {
                break;
            }
            
            // Simulate matching with each order at this price level
            for opposite_order in level_orders {
                let match_qty = std::cmp::min(remaining_qty, opposite_order.remaining_quantity);
                
                // Create a simulated trade
                let trade = Trade {
                    id: 0, // Placeholder ID for simulation
                    price,
                    quantity: match_qty,
                    timestamp: std::cmp::max(order.timestamp, opposite_order.timestamp),
                    buy_order_id: if order.is_buy() { order.id } else { opposite_order.id },
                    sell_order_id: if order.is_sell() { order.id } else { opposite_order.id },
                    buy_user_id: if order.is_buy() { order.user_id } else { opposite_order.user_id },
                    sell_user_id: if order.is_sell() { order.user_id } else { opposite_order.user_id },
                    symbol: order.symbol.clone(),
                };
                
                simulated_trades.push(trade);
                
                remaining_qty -= match_qty;
                
                if remaining_qty == 0 {
                    return simulated_trades;
                }
            }
        }
        
        simulated_trades
    }
}
