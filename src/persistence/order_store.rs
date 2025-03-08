use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

use crate::models::order::{Order, OrderSide, OrderStatus, OrderType};

/// Represents a store for persisting and retrieving order data
pub struct OrderStore {
    /// In-memory cache of orders, indexed by order ID
    orders: HashMap<u64, Order>,
    /// Optional file path for persistence
    file_path: Option<String>,
    /// Whether to automatically flush to disk on each write
    auto_flush: bool,
}

impl OrderStore {
    /// Creates a new in-memory order store
    pub fn new() -> Self {
        Self {
            orders: HashMap::new(),
            file_path: None,
            auto_flush: false,
        }
    }

    /// Creates a new order store with file persistence
    pub fn with_file(file_path: &str, auto_flush: bool) -> io::Result<Self> {
        let mut store = Self {
            orders: HashMap::new(),
            file_path: Some(file_path.to_string()),
            auto_flush,
        };

        // Try to load existing orders from file
        if Path::new(file_path).exists() {
            store.load_from_file()?;
        }

        Ok(store)
    }

    /// Adds or updates an order in the store
    pub fn add_or_update_order(&mut self, order: Order) -> io::Result<()> {
        let order_id = order.id;
        self.orders.insert(order_id, order);

        if self.auto_flush {
            self.flush()?;
        }

        Ok(())
    }

    /// Adds multiple orders to the store
    pub fn add_orders(&mut self, orders: Vec<Order>) -> io::Result<()> {
        for order in orders {
            self.orders.insert(order.id, order);
        }

        if self.auto_flush {
            self.flush()?;
        }

        Ok(())
    }

    /// Retrieves an order by ID
    pub fn get_order(&self, order_id: u64) -> Option<&Order> {
        self.orders.get(&order_id)
    }

    /// Returns all orders
    pub fn get_all_orders(&self) -> Vec<&Order> {
        self.orders.values().collect()
    }

    /// Returns all orders for a given symbol
    pub fn get_orders_by_symbol(&self, symbol: &str) -> Vec<&Order> {
        self.orders
            .values()
            .filter(|order| order.symbol == symbol)
            .collect()
    }

    /// Returns orders for a specific user
    pub fn get_orders_by_user(&self, user_id: u64) -> Vec<&Order> {
        self.orders
            .values()
            .filter(|order| order.user_id == user_id)
            .collect()
    }
    
    /// Returns orders with a specific status
    pub fn get_orders_by_status(&self, status: OrderStatus) -> Vec<&Order> {
        self.orders
            .values()
            .filter(|order| order.status == status)
            .collect()
    }
    
    /// Returns active orders (not filled or canceled)
    pub fn get_active_orders(&self) -> Vec<&Order> {
        self.orders
            .values()
            .filter(|order| {
                order.status != OrderStatus::Filled && order.status != OrderStatus::Canceled
            })
            .collect()
    }

    /// Loads orders from the configured file
    fn load_from_file(&mut self) -> io::Result<()> {
        if let Some(file_path) = &self.file_path {
            let file = File::open(file_path)?;
            let reader = BufReader::new(file);

            match serde_json::from_reader::<_, Vec<Order>>(reader) {
                Ok(orders) => {
                    for order in orders {
                        self.orders.insert(order.id, order);
                    }
                    info!("Loaded {} orders from {}", self.orders.len(), file_path);
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to parse orders from {}: {}", file_path, e);
                    Err(io::Error::new(io::ErrorKind::InvalidData, e))
                }
            }
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No file path configured",
            ))
        }
    }

    /// Writes all orders to the configured file
    pub fn flush(&self) -> io::Result<()> {
        if let Some(file_path) = &self.file_path {
            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(file_path)?;
            
            let writer = BufWriter::new(file);
            let orders: Vec<&Order> = self.orders.values().collect();
            
            match serde_json::to_writer_pretty(writer, &orders) {
                Ok(_) => {
                    debug!("Wrote {} orders to {}", orders.len(), file_path);
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to write orders to {}: {}", file_path, e);
                    Err(io::Error::new(io::ErrorKind::Other, e))
                }
            }
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No file path configured",
            ))
        }
    }

    /// Returns the count of orders in the store
    pub fn count(&self) -> usize {
        self.orders.len()
    }

    /// Clears all orders from the store
    pub fn clear(&mut self) -> io::Result<()> {
        self.orders.clear();
        
        if self.auto_flush {
            self.flush()?;
        }
        
        Ok(())
    }
    
    /// Get statistics about orders by symbol
    pub fn order_count_by_symbol(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        
        for order in self.orders.values() {
            *counts.entry(order.symbol.clone()).or_insert(0) += 1;
        }
        
        counts
    }
    
    /// Get statistics about orders by status
    pub fn order_count_by_status(&self) -> HashMap<OrderStatus, usize> {
        let mut counts = HashMap::new();
        
        for order in self.orders.values() {
            *counts.entry(order.status).or_insert(0) += 1;
        }
        
        counts
    }
    
    /// Get statistics about orders by side
    pub fn order_count_by_side(&self) -> HashMap<OrderSide, usize> {
        let mut counts = HashMap::new();
        
        for order in self.orders.values() {
            *counts.entry(order.side).or_insert(0) += 1;
        }
        
        counts
    }
}

impl Default for OrderStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe wrapper around OrderStore
pub struct ThreadSafeOrderStore {
    store: Arc<Mutex<OrderStore>>,
}

impl ThreadSafeOrderStore {
    /// Creates a new thread-safe order store
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(OrderStore::new())),
        }
    }

    /// Creates a new thread-safe order store with file persistence
    pub fn with_file(file_path: &str, auto_flush: bool) -> io::Result<Self> {
        Ok(Self {
            store: Arc::new(Mutex::new(OrderStore::with_file(file_path, auto_flush)?)),
        })
    }

    /// Adds or updates an order in the store
    pub fn add_or_update_order(&self, order: Order) -> io::Result<()> {
        match self.store.lock() {
            Ok(mut store) => store.add_or_update_order(order),
            Err(e) => {
                error!("Failed to acquire lock: {}", e);
                Err(io::Error::new(io::ErrorKind::Other, "Lock acquisition failed"))
            }
        }
    }

    /// Adds multiple orders to the store
    pub fn add_orders(&self, orders: Vec<Order>) -> io::Result<()> {
        match self.store.lock() {
            Ok(mut store) => store.add_orders(orders),
            Err(e) => {
                error!("Failed to acquire lock: {}", e);
                Err(io::Error::new(io::ErrorKind::Other, "Lock acquisition failed"))
            }
        }
    }

    /// Writes all orders to the configured file
    pub fn flush(&self) -> io::Result<()> {
        match self.store.lock() {
            Ok(store) => store.flush(),
            Err(e) => {
                error!("Failed to acquire lock: {}", e);
                Err(io::Error::new(io::ErrorKind::Other, "Lock acquisition failed"))
            }
        }
    }

    /// Creates a new clone of this store that can be shared with another thread
    pub fn clone(&self) -> Self {
        Self {
            store: Arc::clone(&self.store),
        }
    }
}
