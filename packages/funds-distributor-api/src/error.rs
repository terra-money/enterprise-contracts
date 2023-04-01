use cosmwasm_std::StdError;
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
}

impl DistributorError {
    /// Converts this DistributorError into a StdError.
    pub fn std_err(&self) -> StdError {
        StdError::generic_err(format!("{:?}", self))
    }
}
