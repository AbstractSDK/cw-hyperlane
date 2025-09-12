use crate::error::ContractError;
use sha2::{Digest, Sha256};

pub fn sha256(msg: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(msg);
    hasher.finalize().to_vec()
}

pub fn key_to_addr(prefix: &str, address_bytes: &[u8]) -> Result<String, ContractError> {
    let hrp_result = bech32::Hrp::parse(prefix);
    if let Ok(hrp) = hrp_result {
        if let Ok(acc) = bech32::encode::<bech32::Bech32>(hrp, address_bytes) {
            return Ok(acc);
        }
    }
    Err(ContractError::Bech32AddressParseFailed {})
}
