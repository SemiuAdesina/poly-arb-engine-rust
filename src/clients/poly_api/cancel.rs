use anyhow::{Result, Context};
use super::client::PolyClient;
use super::auth::sign_request;
use super::flaresolverr::FlareSolverrClient;
impl PolyClient {
    pub async fn cancel_order(
        &self,
        order_id: &str,
        signer: &crate::execution::signer::OrderSigner,
    ) -> Result<()> {
        println!("--> Cancelling order: {}", order_id);
        let timestamp = chrono::Utc::now().timestamp();
        let path = format!("/order/{}", order_id);
        let request_body = "";
        let request_signature = sign_request(&self.api_secret, timestamp, "DELETE", &path, request_body)
            .context("Failed to sign cancel request")?;
        let url = format!("{}/order/{}", self.base_url, order_id);
        let address = format!("{:#x}", signer.address()).to_lowercase();
        let mut request_builder = self.client
            .delete(&url)
            .header("x-api-key", &self.api_key)
            .header("POLY_API_KEY", &self.api_key)
            .header("POLY_SIGNATURE", &request_signature)
            .header("POLY_TIMESTAMP", timestamp.to_string())
            .header("POLY_ADDRESS", &address)
            .header("POLY_PASSPHRASE", &self.api_passphrase)
            .header("Accept", "application/json");
        if let Some(ref cookie) = self.cloudflare_cookie {
            request_builder = request_builder.header("Cookie", cookie.as_str());
        }
        let (status, response_text) = if let Some(ref flaresolverr_url) = self.flaresolverr_url {
            println!("Routing cancel request through FlareSolverr...");
            <PolyClient as FlareSolverrClient>::send_delete_via_flaresolverr(self, flaresolverr_url, &url, &request_signature, timestamp, &address).await?
        } else {
            let response = request_builder
                .send()
                .await
                .context("Failed to send cancel request")?;
            let status = response.status();
            let response_text = response.text().await
                .context("Failed to read cancel response")?;
            (status, response_text)
        };
        println!("Cancel response status: {}", status);
        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "Cancel order failed. Status: {}, Response: {}",
                status,
                response_text
            ));
        }
        println!("Order cancelled successfully");
        Ok(())
    }
}