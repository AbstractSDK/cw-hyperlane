use cosmwasm_schema::{QueryResponses, cw_serde};
use cosmwasm_std::{Addr, Binary};
use hpl_interface::core::ExpectedHandleMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub mailbox: String,
}

pub type ExecuteMsg = ExpectedHandleMsg;

#[cw_serde]
pub struct VerificationMsg {
    pub evm_address: Binary,
    pub cosmos_address: String,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(VerificationResponse)]
    Verification { evm_address: Binary },

    // To fulfill hpl-interface ism-specifier query
    #[returns(hpl_interface::ism::InterchainSecurityModuleResponse)]
    IsmSpecifier(hpl_interface::ism::IsmSpecifierQueryMsg),
}

#[cw_serde]
pub struct VerificationResponse {
    pub address: Option<Addr>,
}
