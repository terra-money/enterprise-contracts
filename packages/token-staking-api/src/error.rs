use crate::error::TokenStakingError::Std;
use cosmwasm_std::{OverflowError, StdError};
use membership_common_api::error::MembershipError;
use thiserror::Error;

pub type TokenStakingResult<T> = Result<T, TokenStakingError>;

#[derive(Error, Debug, PartialEq)]
pub enum TokenStakingError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Common(#[from] MembershipError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Received amount different from the total amount of added stakes")]
    IncorrectStakesAmountReceived,

    #[error("Received amount different from the total amount of claims")]
    IncorrectClaimsAmountReceived,

    #[error("Insufficient staked amount")]
    InsufficientStake,
}

impl From<OverflowError> for TokenStakingError {
    fn from(e: OverflowError) -> Self {
        Std(StdError::generic_err(e.to_string()))
    }
}

impl TokenStakingError {
    /// Converts this TokenStakingError into a StdError.
    pub fn std_err(&self) -> StdError {
        StdError::generic_err(format!("{:?}", self))
    }
}
