use cosmwasm_std::StdError;
use thiserror::Error;

pub type DistributorResult<T> = Result<T, DistributorError>;

#[derive(Error, Debug, PartialEq)]
pub enum DistributorError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Cannot distribute - there are no stakers currently")]
    NothingStaked,
}

impl DistributorError {
    /// Converts this DistributorError into a StdError.
    pub fn std_err(&self) -> StdError {
        StdError::generic_err(format!("{:?}", self))
    }
}
