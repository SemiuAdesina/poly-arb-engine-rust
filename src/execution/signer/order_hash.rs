use anyhow::Result;
use ethers::types::{Address, U256};
use ethers::utils::keccak256;
pub fn compute_order_hash(
    salt: U256,
    maker: Address,
    signer: Address,
    taker: Address,
    token_id: Address,
    maker_amount: U256,
    taker_amount: U256,
    expiration: U256,
    nonce: U256,
    fee_rate: U256,
    side: bool,
) -> Result<[u8; 32]> {
    let order_type_hash = keccak256(b"Order(uint256 salt,address maker,address signer,address taker,address tokenId,uint256 makerAmount,uint256 takerAmount,uint256 expiration,uint256 nonce,uint256 feeRate,uint8 side)");
    let mut data = Vec::new();
    data.extend_from_slice(&order_type_hash);
    let mut salt_bytes = [0u8; 32];
    salt.to_big_endian(&mut salt_bytes);
    data.extend_from_slice(&salt_bytes);
    let mut maker_bytes = [0u8; 32];
    maker_bytes[12..].copy_from_slice(maker.as_bytes());
    data.extend_from_slice(&maker_bytes);
    let mut signer_bytes = [0u8; 32];
    signer_bytes[12..].copy_from_slice(signer.as_bytes());
    data.extend_from_slice(&signer_bytes);
    let mut taker_bytes = [0u8; 32];
    taker_bytes[12..].copy_from_slice(taker.as_bytes());
    data.extend_from_slice(&taker_bytes);
    let mut token_bytes = [0u8; 32];
    token_bytes[12..].copy_from_slice(token_id.as_bytes());
    data.extend_from_slice(&token_bytes);
    let mut maker_amount_bytes = [0u8; 32];
    maker_amount.to_big_endian(&mut maker_amount_bytes);
    data.extend_from_slice(&maker_amount_bytes);
    let mut taker_amount_bytes = [0u8; 32];
    taker_amount.to_big_endian(&mut taker_amount_bytes);
    data.extend_from_slice(&taker_amount_bytes);
    let mut expiration_bytes = [0u8; 32];
    expiration.to_big_endian(&mut expiration_bytes);
    data.extend_from_slice(&expiration_bytes);
    let mut nonce_bytes = [0u8; 32];
    nonce.to_big_endian(&mut nonce_bytes);
    data.extend_from_slice(&nonce_bytes);
    let mut fee_rate_bytes = [0u8; 32];
    fee_rate.to_big_endian(&mut fee_rate_bytes);
    data.extend_from_slice(&fee_rate_bytes);
    let mut side_bytes = [0u8; 32];
    side_bytes[31] = if side { 0 } else { 1 };
    data.extend_from_slice(&side_bytes);
    Ok(keccak256(data))
}