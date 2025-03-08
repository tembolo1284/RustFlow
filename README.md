# RustFlow

A high-frequency trading engine implemented in Rust, optimized for low latency and high throughput.

## Features

- **Order Management**: Create and manage various order types including limit, market, stop, IOC, and FOK orders
- **Order Book**: Efficient price-time priority order book implemented with B-tree data structures
- **Matching Engine**: Fast order matching with support for partial fills and cancellations
- **Market Analysis**: Calculate spread, market depth, and slippage
- **Persistence**: Store and retrieve order and trade history
- **Performance Metrics**: Track execution times and system performance
- **Thread Safety**: Concurrent access to shared components

## Project Structure

```
rustflow/
├── Cargo.toml                         # Project configuration
├── README.md                          # This file
├── examples/                          # Example usage scripts
│   └── basic_trading.rs               # Basic trading example
└── src/
    ├── core/                          # Core trading engine components
    │   ├── matcher.rs                 # Matching engine
    │   ├── mod.rs                     # Module exports
    │   └── order_book.rs              # OrderBook implementation
    ├── lib.rs                         # Library entry point
    ├── models/                        # Core data models
    │   ├── mod.rs                     # Module exports
    │   ├── order.rs                   # Order structure
    │   ├── stats.rs                   # Statistics structure
    │   └── trade.rs                   # Trade structure
    ├── persistence/                   # Data storage and retrieval
    │   ├── mod.rs                     # Module exports
    │   ├── order_store.rs             # Order history storage
    │   └── trade_store.rs             # Trade history storage
    └── utils/                         # Utility functions
        ├── metrics.rs                 # Performance metrics
        ├── mod.rs                     # Module exports
        └── time.rs                    # Time-related utilities
```

## Building the Project

### Prerequisites

- Rust and Cargo (1.54.0 or newer): [Install Rust](https://www.rust-lang.org/tools/install)

### Build Steps

1. Clone the repository:
   ```bash
   git clone https://github.com/tembolo1284/rustflow.git
   cd rustflow
   ```

2. Build the library in debug mode:
   ```bash
   cargo build
   ```

3. Build with optimizations for production use:
   ```bash
   cargo build --release
   ```

## Running Examples

The project includes example applications that demonstrate RustFlow's functionality:

### Basic Trading Example

Demonstrates creating an order book, submitting various order types, and executing trades:

```bash
cargo run --example basic_trading
```

## Running Tests

### Unit Tests

Run all unit tests:

```bash
cargo test
```

Run tests for a specific module:

```bash
cargo test --package rustflow --lib models::order
```

### Documentation Tests

Run documentation examples to ensure they're correct:

```bash
cargo test --doc
```

### Test Coverage (Optional)

If you have [tarpaulin](https://github.com/xd009642/tarpaulin) installed:

```bash
cargo tarpaulin --ignore-tests
```

## Benchmarks

To measure performance (requires nightly Rust):

```bash
rustup run nightly cargo bench
```

## Documentation

Generate and open the documentation:

```bash
cargo doc --no-deps --open
```

## Usage Example

```rust
use rustflow::{Order, OrderBook, OrderSide, OrderType};
use rustflow::utils::time::current_timestamp_nanos;

fn main() {
    // Create a new order book
    let mut book = OrderBook::new("BTC-USD");
    
    // Create and add a buy limit order
    let buy_order = Order::new_limit(
        1,               // Order ID
        10000,           // Price (cents)
        2,               // Quantity
        OrderSide::Buy,  // Side
        1001,            // User ID
        current_timestamp_nanos(),  // Timestamp
        None,            // Client order ID
        "BTC-USD".to_string()  // Symbol
    );
    
    book.process_order(buy_order);
    
    // Create and add a sell limit order
    let sell_order = Order::new_limit(
        2, 
        10200, 
        1, 
        OrderSide::Sell, 
        1002, 
        current_timestamp_nanos(), 
        None, 
        "BTC-USD".to_string()
    );
    
    book.process_order(sell_order);
    
    // Create a market order that will match
    let market_buy = Order::new_market(
        3,               // Order ID
        1,               // Quantity
        OrderSide::Buy,  // Side
        1003,            // User ID
        current_timestamp_nanos(),  // Timestamp
        None,            // Client order ID
        "BTC-USD".to_string()  // Symbol
    );
    
    // Process the order and get executed trades
    let trades = book.process_order(market_buy);
    
    // Print trade information
    for trade in trades {
        println!(
            "Trade: {} {} @ ${:.2}", 
            trade.quantity, 
            trade.symbol, 
            trade.price as f64 / 100.0
        );
    }
    
    // Display the order book
    book.print_book(5);
}
```

## Adding RustFlow to Your Project

Add to your Cargo.toml:

```toml
[dependencies]
rustflow = { git = "https://github.com/yourusername/rustflow.git" }
```

## Library Components

### Models

- **Order**: Represents a trading order (limit, market, etc.)
- **Trade**: Represents an executed trade between orders
- **OrderBookStats**: Statistics about the order book state

### Core

- **OrderBook**: Central component that maintains bids and asks
- **Matcher**: Matches buy and sell orders based on price-time priority

### Persistence

- **TradeStore**: Stores and retrieves trade history
- **OrderStore**: Stores and retrieves order history

### Utils

- **time**: Utilities for timestamp generation and formatting
- **metrics**: Performance measurement tools

## Performance Considerations

RustFlow is designed for high-frequency trading applications:

1. **Low Latency**: Optimized for minimal processing time
2. **Memory Efficiency**: Careful data structure selection to minimize allocations
3. **Concurrency**: Support for parallel processing of orders
4. **Persistence**: Flexible storage options with both in-memory and file-based storage
5. **Metrics**: Built-in performance measurement tools

## Performance Tuning

For maximum performance:

1. Always use `--release` mode for production
2. Consider using `RUSTFLAGS="-C target-cpu=native"` for CPU-specific optimizations
3. Profile your application using the built-in metrics or external tools
4. Configure persistence to use batch operations rather than individual writes

## Future Enhancements

- WebSocket API for real-time order submission and market data
- Support for multiple assets and cross-asset trading
- Advanced order types (trailing stop, OCO, bracket orders)
- Risk management features (position limits, margin requirements)
- Backtesting engine for strategy development
- Integration with market data providers
- FIX protocol support

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

Please ensure your code passes all tests and includes appropriate documentation.

## License

MIT# RustFlow

A high-frequency trading engine implemented in Rust, optimized for low latency and high throughput.

## Features

- **Order Management**: Create and manage various order types including limit, market, stop, IOC, and FOK orders
- **Order Book**: Efficient price-time priority order book implemented with B-tree data structures
- **Matching Engine**: Fast order matching with support for partial fills and cancellations
- **Market Analysis**: Calculate spread, market depth, and slippage
- **Persistence**: Store and retrieve order and trade history
- **Performance Metrics**: Track execution times and system performance
- **Thread Safety**: Concurrent access to shared components

## Project Structure

```
rustflow/
├── src/
│   ├── models/                        # Core data models
│   │   ├── order.rs                   # Order structure
│   │   ├── trade.rs                   # Trade structure
│   │   └── stats.rs                   # Statistics structure
│   │
│   ├── core/                          # Core trading engine components
│   │   ├── order_book.rs              # OrderBook implementation
│   │   └── matcher.rs                 # Matching engine
│   │
│   ├── persistence/                   # Data storage and retrieval
│   │   ├── trade_store.rs             # Trade history storage
│   │   └── order_store.rs             # Order history storage
│   │
│   ├── utils/                         # Utility functions
│   │   ├── time.rs                    # Time-related utilities
│   │   └── metrics.rs                 # Performance metrics
│   │
│   └── lib.rs                         # Library entry point
│
├── examples/                          # Example usage scripts
│   └── basic_trading.rs               # Basic trading example
│
├── Cargo.toml
└── README.md
```

## Installation

Add to your Cargo.toml:

```toml
[dependencies]
rustflow = { git = "https://github.com/yourusername/rustflow.git" }
```

Or for local development:

```bash
git clone https://github.com/yourusername/rustflow.git
cd rustflow
cargo build
```

## Usage Example

```rust
use rustflow::{Order, OrderBook, OrderSide, OrderType};
use rustflow::utils::time::current_timestamp_nanos;

fn main() {
    // Create a new order book
    let mut book = OrderBook::new("BTC-USD");
    
    // Create and add a buy limit order
    let buy_order = Order::new_limit(
        1,               // Order ID
        10000,           // Price (cents)
        2,               // Quantity
        OrderSide::Buy,  // Side
        1001,            // User ID
        current_timestamp_nanos(),  // Timestamp
        None,            // Client order ID
        "BTC-USD".to_string()  // Symbol
    );
    
    book.process_order(buy_order);
    
    // Create and add a sell limit order
    let sell_order = Order::new_limit(
        2, 
        10200, 
        1, 
        OrderSide::Sell, 
        1002, 
        current_timestamp_nanos(), 
        None, 
        "BTC-USD".to_string()
    );
    
    book.process_order(sell_order);
    
    // Create a market order that will match
    let market_buy = Order::new_market(
        3,               // Order ID
        1,               // Quantity
        OrderSide::Buy,  // Side
        1003,            // User ID
        current_timestamp_nanos(),  // Timestamp
        None,            // Client order ID
        "BTC-USD".to_string()  // Symbol
    );
    
    // Process the order and get executed trades
    let trades = book.process_order(market_buy);
    
    // Print trade information
    for trade in trades {
        println!(
            "Trade: {} {} @ ${:.2}", 
            trade.quantity, 
            trade.symbol, 
            trade.price as f64 / 100.0
        );
    }
    
    // Display the order book
    book.print_book(5);
}
```

## Running Examples

```bash
cargo run --example basic_trading
```

## Performance Considerations

RustFlow is designed for high-frequency trading applications:

1. **Low Latency**: Optimized for minimal processing time
2. **Memory Efficiency**: Careful data structure selection to minimize allocations
3. **Concurrency**: Support for parallel processing of orders
4. **Persistence**: Flexible storage options with both in-memory and file-based storage
5. **Metrics**: Built-in performance measurement tools

## Future Enhancements

- WebSocket API for real-time order submission and market data
- Support for multiple assets and cross-asset trading
- Advanced order types (trailing stop, OCO, bracket orders)
- Risk management features (position limits, margin requirements)
- Backtesting engine for strategy development
- Integration with market data providers
- FIX protocol support

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT
# RustFlow

A high-frequency trading engine implemented in Rust.

## Features

- **Order Management**: Create and manage various order types including limit, market, stop, IOC, and FOK orders
- **Order Book**: Efficient price-time priority order book implemented with B-tree data structures
- **Matching Engine**: Fast order matching with support for partial fills and cancellations
- **Market Analysis**: Calculate spread, market depth, and slippage
- **Performance**: Designed for high-performance, low-latency trading operations

```

## Usage

```rust
use rustflow::{Order, OrderBook, OrderSide, OrderType};

fn main() {
    // Create a new order book for BTC-USD
    let mut book = OrderBook::new("BTC-USD");
    
    // Add some buy limit orders
    let buy_order = Order::new_limit(1, 10000, 1, OrderSide::Buy, 1001, timestamp(), None);
    book.process_order(buy_order);
    
    // Add a sell limit order
    let sell_order = Order::new_limit(2, 10500, 1, OrderSide::Sell, 1002, timestamp(), None);
    book.process_order(sell_order);
    
    // Execute a market order
    let market_order = Order::new_market(3, 1, OrderSide::Buy, 1003, timestamp(), None);
    let trades = book.process_order(market_order);
    
    // Print trades
    for trade in trades {
        println!("Trade: {} @ {}", trade.quantity, trade.price);
    }
}

fn timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}
```

## Performance Considerations

RustFlow is designed for high-frequency trading applications with a focus on:

1. **Low Latency**: Optimized for minimal processing time
2. **Memory Efficiency**: Careful data structure selection to minimize allocations
3. **Concurrency**: Support for parallel processing of orders
4. **Error Handling**: Robust error handling suitable for financial applications

## Future Enhancements

- Multi-threading support for higher throughput
- Network server for remote order submission
- Additional order types and market mechanics
- Persistence and recovery mechanisms
- Risk management features
