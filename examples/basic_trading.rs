use rustflow::{Order, OrderBook, OrderSide, OrderType};
use rustflow::utils::time;
use rustflow::persistence::trade_store::TradeStore;

fn main() {
    println!("RustFlow Basic Trading Example");
    println!("==============================\n");
    
    // Set up a trade store with persistence
    let mut trade_store = match TradeStore::with_file("trades.json", true) {
        Ok(store) => {
            println!("Trade store initialized with persistence");
            store
        }
        Err(e) => {
            println!("Failed to initialize trade store with persistence: {}", e);
            println!("Using in-memory trade store instead");
            TradeStore::new()
        }
    };
    
    // Create an order book for BTC-USD
    let mut book = OrderBook::new("BTC-USD");
    let mut order_id = 0;
    
    // Helper to create unique order IDs
    let next_id = || {
        order_id += 1;
        order_id
    };
    
    // Helper to create a timestamp
    let timestamp = || time::current_timestamp_nanos();
    
    // Add some buy orders
    let buy_orders = vec![
        Order::new_limit(
            next_id(), 
            10000, 
            1, 
            OrderSide::Buy, 
            1001, 
            timestamp(), 
            None, 
            "BTC-USD".to_string()
        ),
        Order::new_limit(
            next_id(), 
            9900, 
            2, 
            OrderSide::Buy, 
            1002, 
            timestamp(), 
            None, 
            "BTC-USD".to_string()
        ),
        Order::new_limit(
            next_id(), 
            9800, 
            3, 
            OrderSide::Buy, 
            1003, 
            timestamp(), 
            None, 
            "BTC-USD".to_string()
        ),
        Order::new_limit(
            next_id(), 
            10100, 
            2, 
            OrderSide::Buy, 
            1001, 
            timestamp(), 
            None, 
            "BTC-USD".to_string()
        ),
    ];
    
    for order in buy_orders {
        println!("\nAdding Buy Order: {} @ ${:.2}", order.quantity, order.price as f64 / 100.0);
        let trades = book.process_order(order);
        if !trades.is_empty() {
            println!("  - Order resulted in {} trade(s)", trades.len());
            trade_store.add_trades(trades).unwrap();
        }
    }
    
    println!("\n-- After Adding Buy Orders --");
    book.print_book(5);
    
    // Add some sell orders
    let sell_orders = vec![
        Order::new_limit(
            next_id(), 
            10300, 
            1, 
            OrderSide::Sell, 
            2001, 
            timestamp(), 
            None, 
            "BTC-USD".to_string()
        ),
        Order::new_limit(
            next_id(), 
            10400, 
            2, 
            OrderSide::Sell, 
            2002, 
            timestamp(), 
            None, 
            "BTC-USD".to_string()
        ),
        Order::new_limit(
            next_id(), 
            10500, 
            3, 
            OrderSide::Sell, 
            2003, 
            timestamp(), 
            None, 
            "BTC-USD".to_string()
        ),
        Order::new_limit(
            next_id(), 
            10200, 
            2, 
            OrderSide::Sell, 
            2001, 
            timestamp(), 
            None, 
            "BTC-USD".to_string()
        ),
    ];
    
    for order in sell_orders {
        println!("\nAdding Sell Order: {} @ ${:.2}", order.quantity, order.price as f64 / 100.0);
        let trades = book.process_order(order);
        if !trades.is_empty() {
            println!("  - Order resulted in {} trade(s)", trades.len());
            trade_store.add_trades(trades).unwrap();
        }
    }
    
    println!("\n-- After Adding Sell Orders --");
    book.print_book(5);
    
    // Add a matching order that will execute
    let matching_buy = Order::new_limit(
        next_id(), 
        10300, 
        2, 
        OrderSide::Buy, 
        1004, 
        timestamp(), 
        None, 
        "BTC-USD".to_string()
    );
    println!("\nAdding Matching Buy Order: {} @ ${:.2}", matching_buy.quantity, matching_buy.price as f64 / 100.0);
    
    let trades = book.process_order(matching_buy);
    trade_store.add_trades(trades.clone()).unwrap();
    
    println!("\n-- Trades Executed --");
    for trade in &trades {
        println!("Trade: {} @ ${:.2}", trade.quantity, trade.price as f64 / 100.0);
    }
    
    println!("\n-- After Matching --");
    book.print_book(5);
    
    // Try a market order
    let market_sell = Order::new_market(
        next_id(), 
        3, 
        OrderSide::Sell, 
        2004, 
        timestamp(), 
        None, 
        "BTC-USD".to_string()
    );
    println!("\nAdding Market Sell Order: {}", market_sell.quantity);
    
    let trades = book.process_order(market_sell);
    trade_store.add_trades(trades.clone()).unwrap();
    
    println!("\n-- Market Order Trades --");
    for trade in &trades {
        println!("Trade: {} @ ${:.2}", trade.quantity, trade.price as f64 / 100.0);
    }
    
    println!("\n-- Final Order Book --");
    book.print_book(5);
    
    // Calculate slippage for a larger order
    if let Some((avg_price, slippage)) = book.calculate_slippage(OrderSide::Buy, 10) {
        println!("\nSlippage Analysis for Buy 10 BTC:");
        println!("Average Execution Price: ${:.2}", avg_price as f64 / 100.0);
        println!("Slippage: {:.2}%", slippage);
    } else {
        println!("\nNot enough liquidity to calculate slippage");
    }
    
    // Print trade statistics
    println!("\n-- Trade Statistics --");
    let trades = trade_store.get_all_trades();
    println!("Total trades: {}", trades.len());
    
    let volumes = trade_store.volume_by_symbol();
    for (symbol, volume) in volumes {
        println!("Volume for {}: {}", symbol, volume);
    }
    
    let avg_prices = trade_store.average_price_by_symbol();
    for (symbol, price) in avg_prices {
        println!("Average price for {}: ${:.2}", symbol, price / 100.0);
    }
}
