use crate::error::ContractError;
use sha2::{Digest, Sha256};

pub fn sha256(msg: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(msg);
    hasher.finalize().to_vec()
}

pub fn key_to_addr(prefix: &str, address_bytes: &[u8]) -> Result<String, ContractError> {
    if let Ok(hrp) = bech32::Hrp::parse(prefix) {
        if let Ok(acc) = bech32::encode::<bech32::Bech32>(hrp, address_bytes) {
            return Ok(acc);
        }
    }
    Err(ContractError::Bech32AddressParseFailed {})
}

pub fn sanitize_self_xyz_string(mut self_str: String) -> Option<String> {
    self_str.retain(|c| c != '\u{0000}');
    if self_str.is_empty() {
        None
    } else {
        Some(self_str)
    }
}

pub fn sanitize_self_xyz_date(self_str: String) -> Option<String> {
    sanitize_self_xyz_string(self_str).and_then(|sanitized| {
        if sanitized == "--" {
            None
        } else {
            Some(sanitized)
        }
    })
}
