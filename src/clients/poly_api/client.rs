use crate::config::settings::Settings;
use reqwest::Client;
pub struct PolyClient {
    pub client: Client,
    #[allow(dead_code)]
    pub api_key: String,
    #[allow(dead_code)]
    pub api_secret: String,
    #[allow(dead_code)]
    pub api_passphrase: String,
    pub base_url: String,
    pub cloudflare_cookie: Option<String>,
    pub flaresolverr_url: Option<String>,
    pub flaresolverr_proxy: Option<String>,
}
impl PolyClient {
    pub fn new(settings: &Settings) -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()
            .unwrap_or_else(|_| Client::new());
        PolyClient {
            client,
            api_key: settings.poly_api_key.clone(),
            api_secret: settings.poly_api_secret.clone(),
            api_passphrase: settings.poly_passphrase.clone(),
            base_url: "https://clob.polymarket.com".to_string(),
            cloudflare_cookie: settings.cloudflare_cookie.clone(),
            flaresolverr_url: settings.flaresolverr_url.clone(),
            flaresolverr_proxy: settings.flaresolverr_proxy.clone(),
        }
    }
}