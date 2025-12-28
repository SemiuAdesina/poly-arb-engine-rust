use anyhow::Result;
use ethers::types::Address;
use ethers::utils::keccak256;
use std::str::FromStr;
const POLYMARKET_DOMAIN_NAME: &str = "Polymarket CTF Exchange";
const POLYMARKET_DOMAIN_VERSION: &str = "1";
const POLYMARKET_CHAIN_ID: u64 = 137;
const POLYMARKET_EXCHANGE_ADDRESS: &str = "0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E"; // Polymarket CTF Exchange contract
pub fn compute_domain_separator() -> Result<[u8; 32]> {
    let domain_type_hash = keccak256(b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)");
    let name_hash = keccak256(POLYMARKET_DOMAIN_NAME.as_bytes());
    let version_hash = keccak256(POLYMARKET_DOMAIN_VERSION.as_bytes());
    let verifying_contract = Address::from_str(POLYMARKET_EXCHANGE_ADDRESS)
        .map_err(|e| anyhow::anyhow!("Invalid exchange address: {}", e))?;
    let mut data = Vec::new();
    data.extend_from_slice(&domain_type_hash);
    data.extend_from_slice(&name_hash);
    data.extend_from_slice(&version_hash);
    use ethers::types::U256;
    let mut chain_id_bytes = [0u8; 32];
    U256::from(POLYMARKET_CHAIN_ID).to_big_endian(&mut chain_id_bytes);
    data.extend_from_slice(&chain_id_bytes);
    let mut contract_bytes = [0u8; 32];
    contract_bytes[12..].copy_from_slice(verifying_contract.as_bytes());
    data.extend_from_slice(&contract_bytes);
    Ok(keccak256(data))
}