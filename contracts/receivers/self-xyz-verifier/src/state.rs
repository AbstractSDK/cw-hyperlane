use cosmwasm_std::{Addr, HexBinary, Uint256};
use cw_storage_plus::{Item, Map};

pub const MAILBOX: Item<Addr> = Item::new("hpl-mailbox");
// Map<evm_address_bytes, cosmos_address>
pub const VERIFICATIONS: Map<&Addr, GenericDiscloseOutputV2> = Map::new("verifications");

// Cosmwasm format for https://docs.self.xyz/verification-in-the-identityverificationhub#genericdiscloseoutputv2-verification-result
#[cosmwasm_schema::cw_serde]
pub struct GenericDiscloseOutputV2 {
    /// E_PASSPORT or EU_ID_CARD
    pub attestation_id: HexBinary,

    /// User's unique identifier
    pub user_identifier: Uint256,

    /// Anti-replay nullifier
    pub nullifier: Uint256,

    /// Forbidden countries used, packed into four 256-bit integers
    pub forbidden_countries_list_packed: [Uint256; 4],

    // --- Disclosed identity information ---
    /// Document issuing country
    pub issuing_state: String,

    /// [first, middle, last] names
    pub name: Vec<String>,

    /// Passport/ID number
    pub id_number: String,

    /// User's nationality
    pub nationality: String,

    /// Birth date (e.g., "DD-MM-YY")
    pub date_of_birth: String,

    /// User's gender
    pub gender: String,

    /// Document expiry date
    pub expiry_date: String,

    // --- Verification results ---
    /// Verified minimum age
    pub older_than: Uint256,

    /// OFAC results for [passport, name+dob, name+yob]
    pub ofac: [bool; 3],
}

impl From<crate::bind::GenericDiscloseOutputV2> for GenericDiscloseOutputV2 {
    fn from(value: crate::bind::GenericDiscloseOutputV2) -> Self {
        Self {
            attestation_id: HexBinary::from(value.attestationId.to_vec()),
            user_identifier: Uint256::from_be_bytes(value.userIdentifier.to_be_bytes()),
            nullifier: Uint256::from_be_bytes(value.nullifier.to_be_bytes()),
            forbidden_countries_list_packed: value
                .forbiddenCountriesListPacked
                .map(|fc| Uint256::from_be_bytes(fc.to_be_bytes())),
            issuing_state: value.issuingState,
            name: value.name,
            id_number: value.idNumber,
            nationality: value.nationality,
            date_of_birth: value.dateOfBirth,
            gender: value.gender,
            expiry_date: value.expiryDate,
            older_than: Uint256::from_be_bytes(value.olderThan.to_be_bytes()),
            ofac: value.ofac,
        }
    }
}
