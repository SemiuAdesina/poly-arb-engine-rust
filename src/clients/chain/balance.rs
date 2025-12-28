use anyhow::{Result, Context};
use crate::config::settings::Settings;
use ethers::{
    providers::{Provider, Http},
    types::{Address, U256, Bytes},
    prelude::*,
};
use std::str::FromStr;
use std::sync::Arc;
const USDC_ADDRESS: &str = "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359";
pub struct BalanceInfo {
    pub usdc: f64,
    pub pol: f64,
}
pub async fn check_balance(settings: &Settings) -> Result<BalanceInfo> {
    println!("--> Checking wallet balances on Polygon...");
    let provider = Provider::<Http>::try_from(&settings.rpc_url)
        .context("Failed to create provider")?;
    let provider = Arc::new(provider);
    let private_key_clean = settings.private_key.trim_start_matches("0x");
    let wallet = ethers::signers::Wallet::from_str(private_key_clean)
        .context("Failed to parse private key")?;
    let address = wallet.address();
    println!("Wallet address: {:?}", address);
    let pol_balance = provider.get_balance(address, None).await
        .context("Failed to fetch POL balance")?;
    let pol_balance_str = ethers::utils::format_units(pol_balance, 18)
        .map_err(|e| anyhow::anyhow!("Failed to format POL balance: {}", e))?;
    let pol_balance_f64 = pol_balance_str.parse::<f64>()
        .unwrap_or(0.0);
    let usdc_address = Address::from_str(USDC_ADDRESS)
        .context("Invalid USDC address")?;
    // ERC20 balanceOf(address) function signature: 0x70a08231
    let balance_selector = [0x70, 0xa0, 0x82, 0x31];
    let mut data = balance_selector.to_vec();
    let address_bytes = {
        let mut bytes = [0u8; 32];
        bytes[12..].copy_from_slice(&address.as_bytes());
        bytes
    };
    data.extend_from_slice(&address_bytes);
    let tx = ethers::types::TransactionRequest::new()
        .to(usdc_address)
        .data(Bytes::from(data));
    use ethers::types::transaction::eip2718::TypedTransaction;
    let typed_tx = TypedTransaction::Legacy(tx);
    let call_result = provider.call(&typed_tx, None).await;
    let usdc_balance_f64 = match call_result {
        Ok(return_data) => {
            if return_data.len() >= 32 {
                let balance = U256::from_big_endian(&return_data[0..32]);
                match ethers::utils::format_units(balance, 6) {
                    Ok(units_str) => units_str.parse::<f64>().unwrap_or(0.0),
                    Err(_) => {
                        println!("Warning: Failed to format USDC balance");
                        0.0
                    }
                }
            } else {
                0.0
            }
        }
        Err(_) => {
            println!("Warning: Could not fetch USDC balance, using 0.0");
            0.0
        }
    };
    println!("USDC: ${:.2}", usdc_balance_f64);
    println!("POL: {:.4}", pol_balance_f64);
    Ok(BalanceInfo {
        usdc: usdc_balance_f64,
        pol: pol_balance_f64,
    })
}