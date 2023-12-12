use crate::error::EnterpriseTreasuryError::Std;
use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;

pub type EnterpriseTreasuryResult<T> = Result<T, EnterpriseTreasuryError>;

#[derive(Error, Debug, PartialEq)]
pub enum EnterpriseTreasuryError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Error parsing message into Cosmos message")]
    InvalidCosmosMessage,

    #[error("Cannot perform that operation in the current migration stage")]
    InvalidMigrationStage,
}

impl From<OverflowError> for EnterpriseTreasuryError {
    fn from(value: OverflowError) -> Self {
        Std(StdError::generic_err(value.to_string()))
    }
}

impl EnterpriseTreasuryError {
    /// Converts this EnterpriseTreasuryError into a StdError.
    pub fn std_err(&self) -> StdError {
        StdError::generic_err(format!("{:?}", self))
    }
}
