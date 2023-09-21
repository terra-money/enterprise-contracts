use cosmwasm_std::StdError;
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

    #[error("Cannot initialize stakes - existing stakes already present")]
    StakesAlreadyInitialized,

    #[error("A staker is duplicated in the stakers array")]
    DuplicateInitialStakerFound,

    #[error("Initial stakes amount received does not match the sum of initial user stakes")]
    IncorrectStakesInitializationAmount,

    #[error("Received amount different from the total amount of claims")]
    IncorrectClaimsAmountReceived,

    #[error("Insufficient staked amount")]
    InsufficientStake,
}

impl TokenStakingError {
    /// Converts this TokenStakingError into a StdError.
    pub fn std_err(&self) -> StdError {
        StdError::generic_err(format!("{:?}", self))
    }
}
