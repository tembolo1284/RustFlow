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

## Adding RustFlow to Your Project

Add to your Cargo.toml:

```toml
[dependencies]
rustflow = { git = "https://github.com/tembolo1284/rustflow.git" }
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
