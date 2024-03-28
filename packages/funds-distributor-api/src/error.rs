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

    #[error("Cannot distribute - total weight of all users is 0")]
    ZeroTotalWeight,

    #[error("Duplicate initial user weight found")]
    DuplicateInitialWeight,

    #[error("Attempting to distribute an asset that is not whitelisted")]
    DistributingNonWhitelistedAsset,
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
