use cosmwasm_std::{StdError,Addr};
use thiserror::Error;


#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("{msg}")]
    GenericMsgErr {msg:String},

    #[error("Invalid address provided")]
    InvalidAddress {},

    #[error("Already registered")]
    ExistingUser {},

    #[error("Not the owner of the listing")]
    NotAnOwner {},

    #[error("Owner can't buy")]
    BuyFromSelf {},

    #[error("User with address {addr} has already rated.")]
    HasAlreadyRated {addr:Addr},

    #[error("Insufficient amount send.Expected {exp}, received: {rec} ")]
    InsufficientFunds {exp: u128, rec: u128},

    #[error("Invalid denom used for amount. Expected {exp}, received: {rec} ")]
    InvalidDenom {exp: String, rec: String},

    #[error("Name too short (length {length} min_length {min_length})")]
    NameTooShort { length: u64, min_length: u64 },

    #[error("Name too long (length {length} min_length {max_length})")]
    NameTooLong { length: u64, max_length: u64 },

    #[error("Invalid character(char {c}")]
    InvalidCharacter { c: char },

    #[error("Invalid rating. Must be between 1 and 5.")]
    InvalidRating {},
}
