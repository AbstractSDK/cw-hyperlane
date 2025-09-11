use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Sender not a Mailbox({mailbox})")]
    NotMailbox { mailbox: String },

    #[error("Failed to decode Self verification")]
    SelfDecodeFailure {},
}
