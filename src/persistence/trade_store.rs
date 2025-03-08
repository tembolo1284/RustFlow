use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

use crate::models::trade::Trade;

/// Represents a store for persisting and retrieving trade data
pub struct TradeStore {
    /// In-memory cache of trades, indexed by trade ID
    trades: HashMap<u64, Trade>,
    /// Optional file path for persistence
    file_path: Option<String>,
    /// Whether to automatically flush to disk on each write
    auto_flush: bool,
}

impl TradeStore {
    /// Creates a new in-memory trade store
    pub fn new() -> Self {
        Self {
            trades: HashMap::new(),
            file_path: None,
            auto_flush: false,
        }
    }

    /// Creates a new trade store with file persistence
    pub fn with_file(file_path: &str, auto_flush: bool) -> io::Result<Self> {
        let mut store = Self {
            trades: HashMap::new(),
            file_path: Some(file_path.to_string()),
            auto_flush,
        };

        // Try to load existing trades from file
        if Path::new(file_path).exists() {
            store.load_from_file()?;
        }

        Ok(store)
    }

    /// Adds a trade to the store
    pub fn add_trade(&mut self, trade: Trade) -> io::Result<()> {
        let trade_id = trade.id;
        self.trades.insert(trade_id, trade);

        if self.auto_flush {
            self.flush()?;
        }

        Ok(())
    }

    /// Adds multiple trades to the store
    pub fn add_trades(&mut self, trades: Vec<Trade>) -> io::Result<()> {
        for trade in trades {
            self.trades.insert(trade.id, trade);
        }

        if self.auto_flush {
            self.flush()?;
        }

        Ok(())
    }

    /// Retrieves a trade by ID
    pub fn get_trade(&self, trade_id: u64) -> Option<&Trade> {
        self.trades.get(&trade_id)
    }

    /// Returns all trades
    pub fn get_all_trades(&self) -> Vec<&Trade> {
        self.trades.values().collect()
    }

    /// Returns all trades for a given symbol
    pub fn get_trades_by_symbol(&self, symbol: &str) -> Vec<&Trade> {
        self.trades
            .values()
            .filter(|trade| trade.symbol == symbol)
            .collect()
    }

    /// Returns trades for a specific user
    pub fn get_trades_by_user(&self, user_id: u64) -> Vec<&Trade> {
        self.trades
            .values()
            .filter(|trade| trade.buy_user_id == user_id || trade.sell_user_id == user_id)
            .collect()
    }

    /// Loads trades from the configured file
    fn load_from_file(&mut self) -> io::Result<()> {
        if let Some(file_path) = &self.file_path {
            let file = File::open(file_path)?;
            let reader = BufReader::new(file);

            match serde_json::from_reader::<_, Vec<Trade>>(reader) {
                Ok(trades) => {
                    for trade in trades {
                        self.trades.insert(trade.id, trade);
                    }
                    info!("Loaded {} trades from {}", self.trades.len(), file_path);
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to parse trades from {}: {}", file_path, e);
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

    /// Writes all trades to the configured file
    pub fn flush(&self) -> io::Result<()> {
        if let Some(file_path) = &self.file_path {
            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(file_path)?;
            
            let writer = BufWriter::new(file);
            let trades: Vec<&Trade> = self.trades.values().collect();
            
            match serde_json::to_writer_pretty(writer, &trades) {
                Ok(_) => {
                    debug!("Wrote {} trades to {}", trades.len(), file_path);
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to write trades to {}: {}", file_path, e);
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

    /// Returns the count of trades in the store
    pub fn count(&self) -> usize {
        self.trades.len()
    }

    /// Clears all trades from the store
    pub fn clear(&mut self) -> io::Result<()> {
        self.trades.clear();
        
        if self.auto_flush {
            self.flush()?;
        }
        
        Ok(())
    }
    
    /// Get statistics about total volume by symbol
    pub fn volume_by_symbol(&self) -> HashMap<String, u64> {
        let mut volumes = HashMap::new();
        
        for trade in self.trades.values() {
            *volumes.entry(trade.symbol.clone()).or_insert(0) += trade.quantity;
        }
        
        volumes
    }
    
    /// Get average price by symbol
    pub fn average_price_by_symbol(&self) -> HashMap<String, f64> {
        let mut total_values = HashMap::new();
        let mut total_quantities = HashMap::new();
        
        for trade in self.trades.values() {
            *total_values.entry(trade.symbol.clone()).or_insert(0) += trade.price * trade.quantity;
            *total_quantities.entry(trade.symbol.clone()).or_insert(0) += trade.quantity;
        }
        
        let mut avg_prices = HashMap::new();
        for (symbol, total_value) in total_values {
            if let Some(&quantity) = total_quantities.get(&symbol) {
                if quantity > 0 {
                    avg_prices.insert(symbol, total_value as f64 / quantity as f64);
                }
            }
        }
        
        avg_prices
    }
}

impl Default for TradeStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe wrapper around TradeStore
pub struct ThreadSafeTradeStore {
    store: Arc<Mutex<TradeStore>>,
}

impl ThreadSafeTradeStore {
    /// Creates a new thread-safe trade store
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(TradeStore::new())),
        }
    }

    /// Creates a new thread-safe trade store with file persistence
    pub fn with_file(file_path: &str, auto_flush: bool) -> io::Result<Self> {
        Ok(Self {
            store: Arc::new(Mutex::new(TradeStore::with_file(file_path, auto_flush)?)),
        })
    }

    /// Adds a trade to the store
    pub fn add_trade(&self, trade: Trade) -> io::Result<()> {
        match self.store.lock() {
            Ok(mut store) => store.add_trade(trade),
            Err(e) => {
                error!("Failed to acquire lock: {}", e);
                Err(io::Error::new(io::ErrorKind::Other, "Lock acquisition failed"))
            }
        }
    }

    /// Adds multiple trades to the store
    pub fn add_trades(&self, trades: Vec<Trade>) -> io::Result<()> {
        match self.store.lock() {
            Ok(mut store) => store.add_trades(trades),
            Err(e) => {
                error!("Failed to acquire lock: {}", e);
                Err(io::Error::new(io::ErrorKind::Other, "Lock acquisition failed"))
            }
        }
    }

    /// Writes all trades to the configured file
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
