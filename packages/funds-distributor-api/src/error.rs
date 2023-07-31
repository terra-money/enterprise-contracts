use crate::error::DistributorError::Std;
use cosmwasm_std::{CheckedMultiplyRatioError, OverflowError, StdError};
use thiserror::Error;

pub type DistributorResult<T> = Result<T, DistributorError>;

#[derive(Error, Debug, PartialEq)]
pub enum DistributorError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error(
        "The user is restricted from receiving rewards, due to not signing the DAO's attestation"
    )]
    RestrictedUser,

    #[error("Cannot distribute - total weight of all users is 0")]
    ZeroTotalWeight,

    #[error("Duplicate initial user weight found")]
    DuplicateInitialWeight,
}

impl From<OverflowError> for DistributorError {
    fn from(e: OverflowError) -> Self {
        Std(StdError::generic_err(e.to_string()))
    }
}

impl From<CheckedMultiplyRatioError> for DistributorError {
    fn from(e: CheckedMultiplyRatioError) -> Self {
        Std(StdError::generic_err(e.to_string()))
    }
}

impl DistributorError {
    /// Converts this DistributorError into a StdError.
    pub fn std_err(&self) -> StdError {
        StdError::generic_err(format!("{:?}", self))
    }
}
