use cosmwasm_std::StdError;
use membership_common_api::error::MembershipError;
use thiserror::Error;

pub type DenomStakingResult<T> = Result<T, DenomStakingError>;

#[derive(Error, Debug, PartialEq)]
pub enum DenomStakingError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Common(#[from] MembershipError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Insufficient staked amount")]
    InsufficientStake,

    #[error("Attempting to stake an incompatible asset")]
    InvalidStakingDenom,

    #[error("Attempting to stake multiple assets")]
    MultipleDenomsBeingStaked,
}

impl DenomStakingError {
    /// Converts this DenomStakingError into a StdError.
    pub fn std_err(&self) -> StdError {
        StdError::generic_err(format!("{:?}", self))
    }
}
