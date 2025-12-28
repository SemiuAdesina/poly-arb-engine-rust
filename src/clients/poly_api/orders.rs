use anyhow::Result;
use super::client::PolyClient;
use super::types::{OutcomeSide, OrderSide};
use super::flaresolverr::FlareSolverrClient;
impl PolyClient {
    pub async fn place_order(
        &self,
        market_id: &str,
        side: &str,
        size: f64,
        price: f64,
        signer: &crate::execution::signer::OrderSigner,
    ) -> Result<String> {
        println!("--> Placing {} order: {} shares @ ${:.4} on market {}", side, size, price, market_id);
        println!("Fetching market details for condition ID: {}", market_id);
        let market = self.get_market_by_condition(market_id).await
            .map_err(|e| {
                eprintln!("Error fetching market details: {}", e);
                anyhow::anyhow!("Failed to fetch market details: {}", e)
            })?;
        println!("Market details fetched successfully");
        println!("Market question: {}", market.question);
        println!("Token ID YES: {}", if market.token_id_yes.is_empty() { "EMPTY" } else { &market.token_id_yes });
        println!("Token ID NO: {}", if market.token_id_no.is_empty() { "EMPTY" } else { &market.token_id_no });
        let outcome = OutcomeSide::from_str_lower(side)
            .ok_or_else(|| anyhow::anyhow!("Invalid side: {}. Must be 'YES' or 'NO'", side))?;
        let token_id = match outcome {
            OutcomeSide::Yes => &market.token_id_yes,
            OutcomeSide::No => &market.token_id_no,
        };
        if token_id.is_empty() {
            return Err(anyhow::anyhow!("Token ID not found for side: {}. YES token: '{}', NO token: '{}'", side, market.token_id_yes, market.token_id_no));
        }
        println!("Token ID: {}", token_id);
        self.place_order_with_token_id(market_id, token_id.to_string(), outcome, size, price, OrderSide::Buy, signer).await
    }
    pub async fn place_order_with_token_id(
        &self,
        market_id: &str,
        token_id: String,
        outcome: OutcomeSide,
        size: f64,
        price: f64,
        order_side: OrderSide,
        signer: &crate::execution::signer::OrderSigner,
    ) -> Result<String> {
        println!("--> Placing {} order (order_side={}): {} shares @ ${:.4} on market {}",
                 outcome.as_str(), if order_side.is_buy() { "BUY" } else { "SELL" }, size, price, market_id);
        println!("Token ID: {}", token_id);
        let (order_body, request_signature, timestamp, address) =
            super::order_builder::build_order_payload(
                self, &token_id, size, price, order_side, signer
            ).await?;
        let dry_run = std::env::var("DRY_RUN")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);
        if dry_run {
            println!("DRY_RUN mode enabled - skipping actual order placement");
            println!("📋 Would place order: {} {} shares @ ${:.4}", outcome.as_str(), size, price);
            return Ok("DRY_RUN_ORDER_ID".to_string());
        }
        <PolyClient as FlareSolverrClient>::ensure_flaresolverr(self).await?;
        let url = format!("{}/order", self.base_url);
        let (status, response_text) = if let Some(ref flaresolverr_url) = self.flaresolverr_url {
            println!("Routing request through FlareSolverr (may take 10-30 seconds)...");
            if let Some(ref proxy) = self.flaresolverr_proxy {
                println!("Using residential proxy: {}", proxy);
            }
            <PolyClient as FlareSolverrClient>::send_post_via_flaresolverr(self, flaresolverr_url, &url, &order_body, &request_signature, timestamp, &address).await?
        } else {
            return Err(anyhow::anyhow!(
                "FlareSolverr not configured. Direct connection will be blocked by Cloudflare.\n\
Set FLARESOLVERR_URL=http://127.0.0.1:8191 (or http://127.0.0.1:8191/v1) in .env\n\
Then run: docker run -d -p 8191:8191 ghcr.io/flaresolverr/flaresolverr:latest"
            ));
        };
        super::order_response::handle_order_response(status, response_text)
    }
}