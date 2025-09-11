use cosmwasm_schema::{QueryResponses, cw_serde};
use cosmwasm_std::{Addr, Binary, HexBinary, Uint256};
use hpl_interface::core::ExpectedHandleMsg;

use crate::state::GenericDiscloseOutputV2;

#[cw_serde]
pub struct InstantiateMsg {
    pub mailbox: String,
}

pub type ExecuteMsg = ExpectedHandleMsg;

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(VerificationResponse)]
    Verification { address: String },

    // To fulfill hpl-interface ism-specifier query
    #[returns(hpl_interface::ism::InterchainSecurityModuleResponse)]
    IsmSpecifier(hpl_interface::ism::IsmSpecifierQueryMsg),
}

#[cw_serde]
pub struct VerificationResponse {
    pub verification: Option<GenericDiscloseOutputV2>,
}
