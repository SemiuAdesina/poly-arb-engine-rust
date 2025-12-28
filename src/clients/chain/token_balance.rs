use anyhow::{Result, Context};
use crate::config::settings::Settings;
use ethers::{
    providers::{Provider, Http},
    types::{Address, U256, Bytes},
    prelude::*,
};
use std::str::FromStr;
use std::sync::Arc;
pub async fn check_token_balance(
    settings: &Settings,
    token_id: &str,
) -> Result<f64> {
    let provider = Provider::<Http>::try_from(&settings.rpc_url)
        .context("Failed to create provider")?;
    let provider = Arc::new(provider);
    let private_key_clean = settings.private_key.trim_start_matches("0x");
    let wallet = ethers::signers::Wallet::from_str(private_key_clean)
        .context("Failed to parse private key")?;
    let address = wallet.address();
    // The conditional token contract address on Polygon is: 0x4D97DCd97eC945f40cF65F87097ACe5EA0476045
    let conditional_token_address = Address::from_str("0x4D97DCd97eC945f40cF65F87097ACe5EA0476045")
        .context("Invalid conditional token contract address")?;
    let token_id_u256 = U256::from_dec_str(token_id)
        .context(format!("Failed to parse token_id as number: {}", token_id))?;
    // ERC1155 balanceOf(address, uint256) function signature: 0x00fdd58e
    let balance_selector = [0x00, 0xfd, 0xd5, 0x8e];
    let mut data = balance_selector.to_vec();
    let address_bytes = {
        let mut bytes = [0u8; 32];
        bytes[12..].copy_from_slice(&address.as_bytes());
        bytes
    };
    data.extend_from_slice(&address_bytes);
    let token_id_bytes = {
        let mut bytes = [0u8; 32];
        token_id_u256.to_big_endian(&mut bytes);
        bytes
    };
    data.extend_from_slice(&token_id_bytes);
    let tx = ethers::types::TransactionRequest::new()
        .to(conditional_token_address)
        .data(Bytes::from(data));
    use ethers::types::transaction::eip2718::TypedTransaction;
    let typed_tx = TypedTransaction::Legacy(tx);
    let call_result = provider.call(&typed_tx, None).await;
    match call_result {
        Ok(return_data) => {
            if return_data.len() >= 32 {
                let balance = U256::from_big_endian(&return_data[0..32]);
                match ethers::utils::format_units(balance, 18) {
                    Ok(units_str) => {
                        let balance_f64 = units_str.parse::<f64>().unwrap_or(0.0);
                        Ok(balance_f64)
                    }
                    Err(e) => {
                        Err(anyhow::anyhow!("Failed to format token balance: {}", e))
                    }
                }
            } else {
                Ok(0.0)
            }
        }
        Err(e) => {
            Err(anyhow::anyhow!("Failed to call balanceOf: {}", e))
        }
    }
}