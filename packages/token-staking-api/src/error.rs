use cosmwasm_std::StdError;
use thiserror::Error;

pub type TokenStakingResult<T> = Result<T, TokenStakingError>;

#[derive(Error, Debug, PartialEq)]
pub enum TokenStakingError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Insufficient staked amount")]
    InsufficientStake,
}

impl TokenStakingError {
    /// Converts this TokenStakingError into a StdError.
    pub fn std_err(&self) -> StdError {
        StdError::generic_err(format!("{:?}", self))
    }
}
