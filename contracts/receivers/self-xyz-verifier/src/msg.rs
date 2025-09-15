use cosmwasm_schema::{QueryResponses, cw_serde};
use cosmwasm_std::{Addr, HexBinary};
use hpl_interface::{core::HandleMsg, ownable::OwnableMsg};

use crate::state::{UserData, VerifiedIdentity};

#[cw_serde]
pub struct InstantiateMsg {
    /// Address of the Mailbox
    pub mailbox: String,
    /// Address of the owner
    pub owner: String,
    /// Address of the HyperlaneVerification contract on the origin chain
    pub hyperlane_verification: Option<cosmwasm_std::HexBinary>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Ownable(OwnableMsg),
    Handle(HandleMsg),
    SetHyperlaneVerification(HexBinary),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(VerificationResponse)]
    Verification { address: String },

    #[returns(Vec<VerificationResponse>)]
    VerificationList {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    // To fulfill hpl-interface ism-specifier query
    #[returns(hpl_interface::ism::InterchainSecurityModuleResponse)]
    IsmSpecifier(hpl_interface::ism::IsmSpecifierQueryMsg),
}

#[cw_serde]
pub struct VerificationResponse {
    pub verification: Option<(VerifiedIdentity, UserData)>,
}

#[cw_serde]
pub struct VerificationListResponse {
    pub verifications: Vec<(Addr, (VerifiedIdentity, UserData))>,
}
