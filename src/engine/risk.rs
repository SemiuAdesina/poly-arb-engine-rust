use crate::types::orderbook::OrderBook;
#[allow(dead_code)]
pub fn validate_liquidity(orderbook: &OrderBook, size: f64) -> bool {
    println!("--> Pre-flight check: Validating liquidity for size {}", size);
    let yes_liquidity: f64 = orderbook.bids.iter()
        .take(5)
        .map(|level| level.size.parse::<f64>().unwrap_or(0.0))
        .sum();
    let no_liquidity: f64 = orderbook.asks.iter()
        .take(5)
        .map(|level| level.size.parse::<f64>().unwrap_or(0.0))
        .sum();
    let has_yes_liquidity = yes_liquidity >= size;
    let has_no_liquidity = no_liquidity >= size;
    println!("YES liquidity: {:.2}, required: {:.2}", yes_liquidity, size);
    println!("NO liquidity: {:.2}, required: {:.2}", no_liquidity, size);
    if has_yes_liquidity && has_no_liquidity {
        println!("Liquidity check passed");
        true
    } else {
        println!("Insufficient liquidity");
        false
    }
}