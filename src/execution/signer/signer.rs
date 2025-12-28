use anyhow::{Result, Context};
use crate::config::settings::Settings;
use ethers::signers::{Signer, Wallet};
use ethers::types::{Address, U256};
use ethers::utils::keccak256;
use std::str::FromStr;
use super::domain_separator;
use super::order_hash;
pub struct OrderSigner {
    wallet: Wallet<ethers::core::k256::ecdsa::SigningKey>,
    address: Address,
}
impl OrderSigner {
    pub fn new(settings: &Settings) -> Result<Self> {
        let private_key_clean = settings.private_key.trim_start_matches("0x");
        let wallet = Wallet::from_str(private_key_clean)
            .context("Failed to parse private key for signing")?;
        let address = wallet.address();
        Ok(OrderSigner {
            wallet,
            address,
        })
    }
    pub fn signing_address(&self) -> Address {
        self.address
    }
    pub fn sign_order_with_amounts(
        &self,
        token_id: &str,
        maker_amount: u128,
        taker_amount: u128,
        is_buy: bool,
        expiration: i64,
        fee_rate: u64,
        salt: Option<u64>,
        nonce: Option<u64>,
    ) -> Result<String> {
        println!("--> Signing order with EIP-712 (makerAmount={}, takerAmount={}, is_buy={})...",
                 maker_amount, taker_amount, is_buy);
        let nonce_value = nonce.ok_or_else(|| anyhow::anyhow!("Nonce must be provided (timestamp-based)"))?;
        let token_id_address = match Address::from_str(token_id) {
            Ok(addr) => addr,
            Err(_) => {
                let token_id_u256 = U256::from_dec_str(token_id)
                    .context("Token ID is neither a valid address nor a valid decimal number")?;
                let mut token_bytes = [0u8; 32];
                token_id_u256.to_big_endian(&mut token_bytes);
                let mut addr_bytes = [0u8; 20];
                addr_bytes.copy_from_slice(&token_bytes[12..]);
                Address::from(addr_bytes)
            }
        };
        let salt_value = salt.unwrap_or_else(|| chrono::Utc::now().timestamp_millis() as u64);
        let salt_u256 = U256::from(salt_value);
        let maker_amount_u256 = U256::from(maker_amount);
        let taker_amount_u256 = U256::from(taker_amount);
        let domain_separator = domain_separator::compute_domain_separator()?;
        let order_hash = order_hash::compute_order_hash(
            salt_u256,
            self.address,
            self.address,
            Address::zero(),
            token_id_address,
            maker_amount_u256,
            taker_amount_u256,
            U256::from(expiration as u64),
            U256::from(nonce_value),
            U256::from(fee_rate),
            is_buy,
        )?;
        // EIP-712 Typed Data Hash: keccak256(0x1901 || domain_separator || order_hash)
        let mut data = vec![0x19, 0x01];
        data.extend_from_slice(&domain_separator);
        data.extend_from_slice(&order_hash);
        let typed_data_hash = keccak256(data);
        use ethers::types::H256;
        let hash_h256 = H256::from(typed_data_hash);
        let signature = self.wallet.sign_hash(hash_h256)
            .context("Failed to sign order hash")?;
        let sig_bytes = signature.to_vec();
        let sig_hex = format!("0x{}", hex::encode(sig_bytes));
        println!("Order signed successfully with EIP-712");
        Ok(sig_hex)
    }
    #[allow(dead_code)]
    pub fn address(&self) -> Address {
        self.address
    }
}