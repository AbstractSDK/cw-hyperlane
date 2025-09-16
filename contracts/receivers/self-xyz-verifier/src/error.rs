use cosmwasm_std::{HexBinary, StdError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Sender not a Mailbox({mailbox})")]
    NotMailbox { mailbox: String },

    #[error("Hyperlane sender not a HyperlaneVerification({hpl_verification}), sender: {sender}")]
    NotHyperlaneVerification {
        hpl_verification: HexBinary,
        sender: HexBinary,
    },

    #[error("Failed to decode Self verification: {body}")]
    SelfDecodeFailure { body: HexBinary },

    #[error("bech32 address parsing failed")]
    Bech32AddressParseFailed {},

    #[error("Payload parse failed: {payload}, expected format '[actionId(1B) | configId(32B)]'")]
    InvalidUserPayload { payload: HexBinary },

    #[error(
        "Invalid length of the HyperlaneVerification address, expected: 32 bytes long, got: {hyperlane_len}. Fill start with zeroes if needed"
    )]
    InvalidHyperlaneLen { hyperlane_len: u32 },
}
