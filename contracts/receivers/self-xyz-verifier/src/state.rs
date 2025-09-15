use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint256};
use cw_storage_plus::{Item, Map};

use crate::{
    bind,
    utils::{sanitize_self_xyz_date, sanitize_self_xyz_string},
};

pub const MAILBOX: Item<Addr> = Item::new("hpl-mailbox");
pub const HYPERLANE_VERIFICATION: Item<cosmwasm_std::HexBinary> = Item::new("hpl-verification");
// Map<cosmos_address, (Identity, UserData)>
pub const VERIFICATIONS: Map<&Addr, (VerifiedIdentity, UserData)> = Map::new("verifications");
// Make it Item, if need to deploy to other chains
pub const CHAIN_BECH_PREFIX: &str = "xion";

#[cw_serde]
pub struct UserData {
    pub action: u8,
    pub config_id: Uint256,
}

#[cw_serde]
pub enum AttestationId {
    ElectronicPassport,
    EuIdCard,
    Unknown(Uint256),
}

#[cw_serde]
pub struct OfacResult {
    passport: bool,
    name_and_dob: bool,
    name_and_yob: bool,
}

// Cosmwasm format for https://docs.self.xyz/verification-in-the-identityverificationhub#genericdiscloseoutputv2-verification-result
#[cw_serde]
pub struct VerifiedIdentity {
    /// E_PASSPORT or EU_ID_CARD
    pub attestation_id: AttestationId,

    /// User's unique identifier
    pub user_identifier: Uint256,

    /// Anti-replay nullifier
    pub nullifier: Uint256,

    /// Forbidden countries used, packed into four 256-bit integers
    pub forbidden_countries_list_packed: Vec<Uint256>,

    // --- Disclosed identity information ---
    /// Document issuing country
    pub issuing_state: Option<String>,

    /// [first, middle, last] names
    pub name: Vec<String>,

    /// Passport/ID number
    pub id_number: Option<String>,

    /// User's nationality
    pub nationality: Option<String>,

    /// Birth date (e.g., "DD-MM-YY")
    pub date_of_birth: Option<String>,

    /// User's gender
    pub gender: Option<String>,

    /// Document expiry date (e.g., "DD-MM-YY")
    pub expiry_date: Option<String>,

    // --- Verification results ---
    /// Verified minimum age
    pub older_than: Uint256,

    /// OFAC results for [passport, name+dob, name+yob]
    pub ofac: OfacResult,
}

impl From<bind::GenericDiscloseOutputV2> for VerifiedIdentity {
    fn from(value: bind::GenericDiscloseOutputV2) -> Self {
        // Attestation id in human-readable format
        let attestation_id = {
            let raw = Uint256::from_be_bytes(value.attestationId.0);
            if raw == Uint256::one() {
                AttestationId::ElectronicPassport
            } else if raw == Uint256::from(2u128) {
                AttestationId::EuIdCard
            } else {
                AttestationId::Unknown(raw)
            }
        };

        // Forbidden countries sanitized
        let forbidden_countries_list_packed = value
            .forbiddenCountriesListPacked
            .iter()
            .filter_map(|fc| (!fc.is_zero()).then_some(Uint256::from_be_bytes(fc.to_be_bytes())))
            .collect();

        let name = value
            .name
            .into_iter()
            .filter_map(|name_part| sanitize_self_xyz_string(name_part))
            .collect();
        Self {
            attestation_id,
            user_identifier: Uint256::from_be_bytes(value.userIdentifier.to_be_bytes()),
            nullifier: Uint256::from_be_bytes(value.nullifier.to_be_bytes()),
            forbidden_countries_list_packed,
            issuing_state: sanitize_self_xyz_string(value.issuingState),
            name,
            id_number: sanitize_self_xyz_string(value.idNumber),
            nationality: sanitize_self_xyz_string(value.nationality),
            date_of_birth: sanitize_self_xyz_date(value.dateOfBirth),
            gender: sanitize_self_xyz_string(value.gender),
            expiry_date: sanitize_self_xyz_date(value.expiryDate),
            older_than: Uint256::from_be_bytes(value.olderThan.to_be_bytes()),
            ofac: OfacResult {
                passport: value.ofac[0],
                name_and_dob: value.ofac[1],
                name_and_yob: value.ofac[2],
            },
        }
    }
}
