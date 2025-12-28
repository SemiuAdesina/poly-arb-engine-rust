use crate::types::orderbook::PriceLevel;
use serde_json::Value;
pub fn parse_price_levels(array: &[Value]) -> Vec<PriceLevel> {
    let mut levels = Vec::new();
    for item in array {
        if let Some(obj) = item.as_object() {
            if let (Some(price_val), Some(size_val)) = (obj.get("price"), obj.get("size")) {
                let price_str = extract_string_from_value(price_val);
                let size_str = extract_string_from_value(size_val);
                levels.push(PriceLevel {
                    price: price_str,
                    size: size_str,
                });
            }
        } else if let Some(arr) = item.as_array() {
            if arr.len() >= 2 {
                let price_str = extract_string_from_value(&arr[0]);
                let size_str = extract_string_from_value(&arr[1]);
                levels.push(PriceLevel {
                    price: price_str,
                    size: size_str,
                });
            }
        }
    }
    levels
}
fn extract_string_from_value(val: &Value) -> String {
    val.as_str()
        .map(|s| s.to_string())
        .or_else(|| val.as_f64().map(|n| n.to_string()))
        .or_else(|| val.as_u64().map(|n| n.to_string()))
        .or_else(|| val.as_i64().map(|n| n.to_string()))
        .unwrap_or_else(|| val.to_string())
}