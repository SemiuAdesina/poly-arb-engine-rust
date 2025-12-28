use anyhow::Result;
use crate::types::market::Market;
use serde_json::Value;
pub fn parse_market(data: &Value) -> Result<Market> {
    let verbose = std::env::var("LOG_VERBOSE").is_ok();
    let condition_id = data.get("conditionId")
        .or_else(|| data.get("condition_id"))
        .or_else(|| data.get("id"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing conditionId field. Available keys: {:?}",
            data.as_object().map(|o| o.keys().collect::<Vec<_>>())))?;
    let question = data.get("question")
        .or_else(|| data.get("title"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let (token_id_yes, token_id_no) = parse_token_ids(data, verbose);
    let end_date_iso = data.get("endDate")
        .or_else(|| data.get("end_date"))
        .or_else(|| data.get("endDateISO"))
        .or_else(|| data.get("endDateIso"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let enable_orderbook = data.get("enableOrderBook")
        .and_then(|v| {
            if let Some(b) = v.as_bool() {
                Some(b)
            } else if let Some(s) = v.as_str() {
                Some(s.to_lowercase() == "true")
            } else {
                None
            }
        })
        .unwrap_or(true);
    Ok(Market {
        condition_id: condition_id.to_string(),
        question,
        token_id_yes,
        token_id_no,
        end_date_iso,
        enable_orderbook,
    })
}
fn parse_token_ids(data: &Value, verbose: bool) -> (String, String) {
    let clob_token_value = data.get("clobTokenIds");
    if let Some(clob_val) = clob_token_value {
        if let Some(clob_str) = clob_val.as_str() {
            if verbose { eprintln!("clobTokenIds is a JSON string: {}", clob_str); }
            match serde_json::from_str::<Vec<String>>(clob_str) {
                Ok(clob_tokens) => {
                    if verbose { eprintln!("Parsed clobTokenIds array with {} items", clob_tokens.len()); }
                    let yes_token = clob_tokens.get(0).map(|s| s.as_str()).unwrap_or("");
                    let no_token = clob_tokens.get(1).map(|s| s.as_str()).unwrap_or("");
                    if verbose { eprintln!("Parsed clobTokenIds - YES: '{}', NO: '{}'", yes_token, no_token); }
                    if !yes_token.is_empty() && !no_token.is_empty() {
                        return (yes_token.to_string(), no_token.to_string());
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse clobTokenIds JSON string: {}", e);
                }
            }
        } else if let Some(clob_tokens) = clob_val.as_array() {
            if verbose { eprintln!("Found 'clobTokenIds' as direct array with {} items", clob_tokens.len()); }
            let yes_token = clob_tokens.get(0)
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let no_token = clob_tokens.get(1)
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if verbose { eprintln!("Parsed clobTokenIds - YES: '{}', NO: '{}'", yes_token, no_token); }
            if !yes_token.is_empty() && !no_token.is_empty() {
                return (yes_token.to_string(), no_token.to_string());
            }
        }
    }
    if let Some(tokens) = data.get("tokens").and_then(|t| t.as_array()) {
        eprintln!("Found 'tokens' array with {} items", tokens.len());
        let yes_token = tokens.get(0)
            .and_then(|t| t.get("tokenId").or_else(|| t.get("token_id")).or_else(|| t.get("id"))
                .and_then(|v| v.as_str()))
            .unwrap_or("");
        let no_token = tokens.get(1)
            .and_then(|t| t.get("tokenId").or_else(|| t.get("token_id")).or_else(|| t.get("id"))
                .and_then(|v| v.as_str()))
            .unwrap_or("");
        eprintln!("Parsed tokens - YES: '{}', NO: '{}'", yes_token, no_token);
        return (yes_token.to_string(), no_token.to_string());
    }
    if let Some(outcomes) = data.get("outcomes").and_then(|o| o.as_array()) {
        eprintln!("Found 'outcomes' array with {} items", outcomes.len());
        let yes_token = outcomes.get(0)
            .and_then(|o| {
                o.get("tokenId").or_else(|| o.get("token_id"))
                    .or_else(|| o.get("tokenAddress"))
                    .or_else(|| o.get("id"))
                    .and_then(|v| v.as_str())
            })
            .unwrap_or("");
        let no_token = outcomes.get(1)
            .and_then(|o| {
                o.get("tokenId").or_else(|| o.get("token_id"))
                    .or_else(|| o.get("tokenAddress"))
                    .or_else(|| o.get("id"))
                    .and_then(|v| v.as_str())
            })
            .unwrap_or("");
        eprintln!("Parsed outcomes - YES: '{}', NO: '{}'", yes_token, no_token);
        return (yes_token.to_string(), no_token.to_string());
    }
    if verbose { eprintln!("No 'clobTokenIds', 'tokens', or 'outcomes' array found in JSON"); }
    ("".to_string(), "".to_string())
}