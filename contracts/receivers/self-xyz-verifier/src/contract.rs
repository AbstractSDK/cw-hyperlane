use alloy::sol_types::SolValue;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    Addr, CanonicalAddr, Deps, DepsMut, Empty, Env, MessageInfo, QueryResponse, Response,
    StdResult, Uint256, ensure_eq, to_json_binary,
};
use cw2::set_contract_version;
use hpl_interface::core::ExpectedHandleMsg;

use crate::{
    CONTRACT_NAME, CONTRACT_VERSION,
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, VerificationResponse},
    state::{GenericDiscloseOutputV2, MAILBOX, UserData, VERIFICATIONS},
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

type DecodedTuple = (
    crate::bind::GenericDiscloseOutputV2,
    alloy::primitives::Bytes,
);

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
            let (disclose, user_data) = DecodedTuple::abi_decode_params(msg.body.as_slice(), true)
                .map_err(|_| ContractError::SelfDecodeFailure { body: msg.body })?;
            // TODO:
            // 1. msg.sender should be saved address of an evm contract, configured at instantiation

            let verification = GenericDiscloseOutputV2::from(disclose);

            let cosmos_address = crate::utils::key_to_addr(
                crate::state::CHAIN_BECH_PREFIX,
                &verification.user_identifier.to_be_bytes(),
            )?;
            let cosmos_address = deps.api.addr_validate(&cosmos_address)?;

            let (action, config_id) = user_data
                .split_last_chunk::<32>()
                .expect(&format!("Userdata invalid length, {user_data}"));
            assert_eq!(action.len(), 1);

            let user_data = UserData {
                action: action[0],
                config_id: cosmwasm_std::Uint256::from_be_bytes(*config_id),
            };

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
        QueryMsg::IsmSpecifier(
            hpl_interface::ism::IsmSpecifierQueryMsg::InterchainSecurityModule(),
        ) => to_json_binary(&hpl_interface::ism::InterchainSecurityModuleResponse { ism: None }),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: Empty) -> StdResult<Response> {
    #[cosmwasm_schema::cw_serde]
    pub struct UserDataV1 {
        pub action: u8,
        pub config_id: [u8; 32],
    }
    const VERIFICATIONS_V1: cw_storage_plus::Map<
        &cosmwasm_std::Addr,
        (GenericDiscloseOutputV2, UserDataV1),
    > = cw_storage_plus::Map::new("verifications");

    for (cosmos_addr, (disclose, user_data)) in VERIFICATIONS_V1
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .flatten()
        .collect::<Vec<(Addr, (GenericDiscloseOutputV2, UserDataV1))>>()
    {
        VERIFICATIONS.save(
            deps.storage,
            &cosmos_addr,
            &(
                disclose,
                UserData {
                    action: user_data.action,
                    config_id: Uint256::from_be_bytes(user_data.config_id),
                },
            ),
        )?;
    }

    hpl_utils::migrate(deps.storage, CONTRACT_NAME, CONTRACT_VERSION).unwrap();
    Ok(Response::default())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn encode_bytes() {
        let output = crate::bind::GenericDiscloseOutputV2 {
            attestationId: [1u8; 32].into(),
            userIdentifier: alloy::primitives::U256::from(2u128),
            nullifier: alloy::primitives::U256::from(3u128),
            forbiddenCountriesListPacked: [
                alloy::primitives::U256::from(4u128),
                alloy::primitives::U256::from(5u128),
                alloy::primitives::U256::from(6u128),
                alloy::primitives::U256::from(7u128),
            ],
            issuingState: "foo".into(),
            name: vec!["bar".into()],
            idNumber: "tar".into(),
            nationality: "zoo".into(),
            dateOfBirth: "gavk".into(),
            gender: "who".into(),
            expiryDate: "yesterday".into(),
            olderThan: alloy::primitives::U256::from(8u128),
            ofac: [true, false, true],
        };
        let d: DecodedTuple = (output, vec![1u8].into());
        let encoded = d.abi_encode_params();
        let hx = hex::encode(encoded);
        println!("{hx}");
    }

    const OUT: &[u8] = &alloy::hex!(
        "0x000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000004e00000000000000000000000000000000000000000000000000000000000000001d720f874691be72d19da5f73bcae697a9437d3ce53a5da6e69f00e468cdd9f372d72117c065d36066607a568e3eb6774d0697c1cc2bdf86ec6f4e803f7df26c4000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002400000000000000000000000000000000000000000000000000000000000000280000000000000000000000000000000000000000000000000000000000000036000000000000000000000000000000000000000000000000000000000000003a000000000000000000000000000000000000000000000000000000000000003e00000000000000000000000000000000000000000000000000000000000000420000000000000000000000000000000000000000000000000000000000000046000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000027000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000800002d00002d000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000800002d00002d0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002101000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
    );

    #[test]
    fn check_out_from_encoder() {
        let decoded = DecodedTuple::abi_decode_params(OUT, true).unwrap();
        let addr = crate::utils::key_to_addr(
            crate::state::CHAIN_BECH_PREFIX,
            &decoded.0.userIdentifier.to_be_bytes::<32>(),
        )
        .unwrap();
        dbg!(addr);

        let (action, config_id) = decoded
            .1
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
