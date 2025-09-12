use cosmwasm_std::{HexBinary, StdError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Sender not a Mailbox({mailbox})")]
    NotMailbox { mailbox: String },

    #[error("Failed to decode Self verification: {body}")]
    SelfDecodeFailure { body: HexBinary },

    #[error("bech32 address parsing failed")]
    Bech32AddressParseFailed {},
}
