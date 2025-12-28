pub fn calculate_fishing_prices(
    best_yes_bid: f64,
    best_no_bid: f64,
    price_offset: f64,
) -> (f64, f64, f64) {
    let yes_limit = (best_yes_bid - price_offset).max(0.001);
    let no_limit = (best_no_bid - price_offset).max(0.001);
    let total_cost = yes_limit + no_limit;
    let expected_value = 1.0;
    let expected_profit = expected_value - total_cost;
    let expected_profit_pct = (expected_profit / total_cost) * 100.0;
    (yes_limit, no_limit, expected_profit_pct)
}
pub fn should_place_fishing_orders(
    best_yes_bid: f64,
    best_no_bid: f64,
    min_profit_pct: f64,
    test_mode: bool,
) -> (bool, f64, f64, f64) {
    println!("👀 Market analysis: YES bid=${:.4}, NO bid=${:.4}, Sum=${:.4}",
             best_yes_bid, best_no_bid, best_yes_bid + best_no_bid);
    let (price_offset, use_percentage) = if test_mode {
        (0.05, true)
    } else {
        (0.001, false)
    };
    let (yes_limit, no_limit, profit_pct) = if use_percentage {
        let yes_limit = (best_yes_bid * price_offset).max(0.001);
        let no_limit = (best_no_bid * price_offset).max(0.001);
        let total_cost = yes_limit + no_limit;
        let expected_value = 1.0;
        let expected_profit = expected_value - total_cost;
        let expected_profit_pct = (expected_profit / total_cost) * 100.0;
        (yes_limit, no_limit, expected_profit_pct)
    } else {
        calculate_fishing_prices(best_yes_bid, best_no_bid, price_offset)
    };
    println!("Calculated limits: YES=${:.4}, NO=${:.4}, Total=${:.4}, Profit={:.2}%",
             yes_limit, no_limit, yes_limit + no_limit, profit_pct);
    let should_fish = if test_mode {
        println!("TEST MODE: Forcing order placement (ignoring profit threshold)");
        true
    } else {
        profit_pct >= min_profit_pct
    };
    if should_fish {
        println!("Fishing opportunity: YES limit=${:.4}, NO limit=${:.4}, Expected profit={:.2}%",
                 yes_limit, no_limit, profit_pct);
        return (true, yes_limit, no_limit, profit_pct);
    }
    println!("Fishing opportunity rejected: Expected profit={:.2}% < minimum {:.2}%",
             profit_pct, min_profit_pct);
    (false, yes_limit, no_limit, profit_pct)
}
pub fn calculate_minimum_order_size(price: f64, min_usd_value: f64) -> f64 {
    if price <= 0.0 {
        return 0.0;
    }
    let min_size = min_usd_value / price;
    let min_size_rounded = min_size.ceil().max(1.0);
    let actual_value = min_size_rounded * price;
    if actual_value < min_usd_value {
        println!("Calculated size {:.1} @ ${:.4} = ${:.2} (below ${:.2} minimum)",
                 min_size_rounded, price, actual_value, min_usd_value);
    } else {
        println!("Order size: {:.1} shares @ ${:.4} = ${:.2} (meets ${:.2} minimum)",
                 min_size_rounded, price, actual_value, min_usd_value);
    }
    min_size_rounded
}
pub fn should_redeem_filled_set(entry_cost: f64) -> bool {
    entry_cost < 0.995
}