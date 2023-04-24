use cosmwasm_std::StdError;
use thiserror::Error;

pub type NftStakingResult<T> = Result<T, NftStakingError>;

#[derive(Error, Debug, PartialEq)]
pub enum NftStakingError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,
}

impl NftStakingError {
    /// Converts this NftStakingError into a StdError.
    pub fn std_err(&self) -> StdError {
        StdError::generic_err(format!("{:?}", self))
    }
}
