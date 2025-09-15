use alloy::sol_types::SolValue;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    Deps, DepsMut, Empty, Env, HexBinary, MessageInfo, Order, QueryResponse, Response, StdResult,
    ensure_eq, to_json_binary,
};
use cw_storage_plus::Bound;
use cw2::set_contract_version;

use crate::{
    CONTRACT_NAME, CONTRACT_VERSION, bind,
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, VerificationListResponse, VerificationResponse},
    state::{HYPERLANE_VERIFICATION, MAILBOX, UserData, VERIFICATIONS, VerifiedIdentity},
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let hpl_mailbox = deps.api.addr_validate(&msg.mailbox)?;
    let owner = deps.api.addr_validate(&msg.owner)?;

    hpl_ownable::initialize(deps.storage, &owner)?;
    MAILBOX.save(deps.storage, &hpl_mailbox)?;
    if let Some(hpl_verification_address) = msg.hyperlane_verification {
        HYPERLANE_VERIFICATION.save(deps.storage, &hpl_verification_address)?;
    }
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Ownable(msg) => hpl_ownable::handle(deps, env, info, msg).map_err(Into::into),
        ExecuteMsg::SetHyperlaneVerification(hpl_verification_address) => {
            ensure_eq!(
                hpl_ownable::get_owner(deps.storage)?,
                &info.sender,
                ContractError::Unauthorized {}
            );
            HYPERLANE_VERIFICATION.save(deps.storage, &hpl_verification_address)?;
            Ok(Response::new().add_attribute("action", "set_hyperlane_verification"))
        }
        ExecuteMsg::Handle(msg) => {
            // Ensure mailbox is sender on this chain
            let mailbox = MAILBOX.load(deps.storage)?;
            ensure_eq!(
                info.sender,
                mailbox,
                ContractError::NotMailbox {
                    mailbox: mailbox.into_string()
                }
            );

            // Ensure we got this letter from the saved hpl_verification
            let hpl_verification = HYPERLANE_VERIFICATION
                .may_load(deps.storage)?
                .unwrap_or_default();
            ensure_eq!(
                msg.sender,
                hpl_verification,
                ContractError::NotHyperlaneVerification {
                    hpl_verification,
                    sender: msg.sender
                }
            );

            let verification_result =
                bind::VerificationResult::abi_decode_params(msg.body.as_slice(), true)
                    .map_err(|_| ContractError::SelfDecodeFailure { body: msg.body })?;

            let verification = VerifiedIdentity::from(verification_result.output);

            let cosmos_address = crate::utils::key_to_addr(
                crate::state::CHAIN_BECH_PREFIX,
                &verification.user_identifier.to_be_bytes(),
            )?;
            let cosmos_address = deps.api.addr_validate(&cosmos_address)?;

            let user_data = verification_result
                .userDataPayload
                .split_last_chunk::<32>()
                .and_then(|(action, config_id)| {
                    if action.len() != 1 {
                        None
                    } else {
                        Some(UserData {
                            action: action[0],
                            config_id: cosmwasm_std::Uint256::from_be_bytes(*config_id),
                        })
                    }
                })
                .ok_or(ContractError::InvalidUserPayload {
                    payload: HexBinary::from(verification_result.userDataPayload.to_vec()),
                })?;

            VERIFICATIONS.save(deps.storage, &cosmos_address, &(verification, user_data))?;

            Ok(Response::new().add_attributes(vec![
                ("action", "verify"),
                ("cosmos_address", cosmos_address.as_str()),
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
        QueryMsg::VerificationList { start_after, limit } => {
            let limit = limit.unwrap_or(10) as usize;

            let start_after_verified = start_after
                .map(|human| deps.api.addr_validate(&human))
                .transpose()?;
            let start_bound = start_after_verified.as_ref().map(Bound::exclusive);

            let verifications = VERIFICATIONS
                .range(deps.storage, start_bound, None, Order::Ascending)
                .take(limit)
                .collect::<StdResult<Vec<_>>>()?;
            to_json_binary(&VerificationListResponse { verifications })
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

#[cfg(test)]
mod test {
    use alloy::primitives::U256;

    use crate::bind::{self, VerificationResult};

    use super::*;

    #[test]
    fn encode_bytes() {
        let output = bind::GenericDiscloseOutputV2 {
            attestationId: [1u8; 32].into(),
            userIdentifier: U256::from(2u128),
            nullifier: U256::from(3u128),
            forbiddenCountriesListPacked: [
                U256::from(4u128),
                U256::from(5u128),
                U256::from(6u128),
                U256::from(7u128),
            ],
            issuingState: "foo".into(),
            name: vec!["bar".into()],
            idNumber: "tar".into(),
            nationality: "zoo".into(),
            dateOfBirth: "gavk".into(),
            gender: "who".into(),
            expiryDate: "yesterday".into(),
            olderThan: U256::from(8u128),
            ofac: [true, false, true],
        };
        let d: VerificationResult = VerificationResult {
            output,
            userDataPayload: vec![1u8].into(),
        };
        let encoded = d.abi_encode_params();
        let hx = hex::encode(encoded);
        println!("{hx}");
    }

    const OUT: &[u8] = &alloy::hex!(
        "0x000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000004e00000000000000000000000000000000000000000000000000000000000000001d720f874691be72d19da5f73bcae697a9437d3ce53a5da6e69f00e468cdd9f372d72117c065d36066607a568e3eb6774d0697c1cc2bdf86ec6f4e803f7df26c4000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002400000000000000000000000000000000000000000000000000000000000000280000000000000000000000000000000000000000000000000000000000000036000000000000000000000000000000000000000000000000000000000000003a000000000000000000000000000000000000000000000000000000000000003e00000000000000000000000000000000000000000000000000000000000000420000000000000000000000000000000000000000000000000000000000000046000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000027000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000800002d00002d000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000800002d00002d0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002101000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
    );

    #[test]
    fn check_out_from_encoder() {
        let decoded = VerificationResult::abi_decode_params(OUT, true).unwrap();
        let addr = crate::utils::key_to_addr(
            crate::state::CHAIN_BECH_PREFIX,
            &decoded.output.userIdentifier.to_be_bytes::<32>(),
        )
        .unwrap();
        dbg!(addr);

        let (action, config_id) = decoded
            .userDataPayload
            .split_last_chunk::<32>()
            .expect(&format!("Userdata invalid length"));
        assert_eq!(action.len(), 1);

        let user_data = UserData {
            action: action[0],
            config_id: cosmwasm_std::Uint256::from_be_bytes(*config_id),
        };
        dbg!(user_data);
    }
}
