use alloy::sol_types::SolValue;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    CanonicalAddr, Deps, DepsMut, Empty, Env, MessageInfo, QueryResponse, Response, StdResult,
    ensure_eq, to_json_binary,
};
use cw2::set_contract_version;
use hex;
use hpl_interface::core::ExpectedHandleMsg;

use crate::{
    CONTRACT_NAME, CONTRACT_VERSION,
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, VerificationResponse},
    state::{GenericDiscloseOutputV2, MAILBOX, VERIFICATIONS},
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

            type DecodedTuple = (
                crate::bind::GenericDiscloseOutputV2,
                alloy::primitives::Bytes,
            );
            let (disclose, cosmos_address_bytes) = DecodedTuple::abi_decode(&msg.body, true)
                .map_err(|_| ContractError::SelfDecodeFailure {})?;
            // TODO:
            // 1. msg.sender should be saved address of an evm contract, configured at instantiation

            let canon_addr = CanonicalAddr::from(cosmos_address_bytes.as_ref());
            let cosmos_address = deps.api.addr_humanize(&canon_addr)?;

            let verification = GenericDiscloseOutputV2::from(disclose);

            VERIFICATIONS.save(deps.storage, &cosmos_address, &verification)?;

            Ok(Response::new().add_attributes(vec![
                ("action", "verify"),
                ("cosmos_address", cosmos_address.as_str()),
                (
                    "user_identifier",
                    hex::encode(&verification.user_identifier.to_be_bytes()).as_str(),
                ),
            ]))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Verification { address } => {
            let address = deps.api.addr_validate(&address)?;
            let verification = VERIFICATIONS.may_load(deps.storage, &address)?;
            to_json_binary(&VerificationResponse { verification })
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
