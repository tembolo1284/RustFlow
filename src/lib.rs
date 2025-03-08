//! RustFlow - High-frequency trading engine in Rust
//!
//! This library provides components for building a high-performance 
//! trading system with order books, matching engines, and various order types.

// Core modules
pub mod models;
pub mod core;
pub mod persistence;
pub mod utils;

// Re-export commonly used types
pub use models::order::{Order, OrderSide, OrderType, OrderStatus};
pub use models::trade::Trade;
pub use models::stats::OrderBookStats;
pub use core::order_book::OrderBook;
pub use core::matcher::Matcher;
pub use persistence::trade_store::TradeStore;
pub use persistence::order_store::OrderStore;
