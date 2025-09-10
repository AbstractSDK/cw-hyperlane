#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    Deps, DepsMut, Empty, Env, MessageInfo, QueryResponse, Response, StdResult, ensure_eq,
    from_json, to_json_binary,
};
use cw2::set_contract_version;
use hex;
use hpl_interface::core::ExpectedHandleMsg;

use crate::{
    CONTRACT_NAME, CONTRACT_VERSION,
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, VerificationMsg, VerificationResponse},
    state::{MAILBOX, VERIFICATIONS},
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let hpl_mailbox = deps.api.addr_validate(&msg.mailbox)?;
    MAILBOX.save(deps.storage, &hpl_mailbox)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExpectedHandleMsg::Handle(msg) => {
            let mailbox = MAILBOX.load(deps.storage)?;
            ensure_eq!(
                info.sender,
                mailbox,
                ContractError::NotMailbox {
                    mailbox: mailbox.into_string()
                }
            );
            // TODO:
            // 1. body won't be json, need to use evm bind for the contract
            // 2. msg.sender should be saved address, can we trust it?
            let verification_msg: VerificationMsg = from_json(&msg.body)?;

            let cosmos_address = deps.api.addr_validate(&verification_msg.cosmos_address)?;
            let evm_address = verification_msg.evm_address;

            VERIFICATIONS.save(deps.storage, evm_address.as_slice(), &cosmos_address)?;

            Ok(Response::new().add_attributes(vec![
                ("action", "verify"),
                (
                    "evm_address",
                    &format!("0x{}", hex::encode(evm_address.as_slice())),
                ),
                ("cosmos_address", cosmos_address.as_str()),
            ]))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Verification { evm_address } => {
            let address = VERIFICATIONS.may_load(deps.storage, evm_address.as_slice())?;
            to_json_binary(&VerificationResponse { address })
        }
        QueryMsg::IsmSpecifier(
            hpl_interface::ism::IsmSpecifierQueryMsg::InterchainSecurityModule(),
        ) => to_json_binary(&hpl_interface::ism::InterchainSecurityModuleResponse { ism: None }),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: Empty) -> StdResult<Response> {
    hpl_utils::migrate(deps.storage, CONTRACT_NAME, CONTRACT_VERSION).unwrap();
    Ok(Response::default())
}
