use cosmwasm_std::StdError;
use thiserror::Error;

pub type StakingResult<T> = Result<T, StakingError>;

#[derive(Error, Debug, PartialEq)]
pub enum StakingError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Cannot initialize stakes - existing stakes already present")]
    StakesAlreadyInitialized,

    #[error("Initial stakes amount received does not match the sum of initial user stakes")]
    IncorrectStakesInitializationAmount,

    #[error("Received amount different from the total amount of claims")]
    IncorrectClaimsAmountReceived,

    #[error("Insufficient staked amount")]
    InsufficientStake,
}

impl StakingError {
    /// Converts this TokenStakingError into a StdError.
    pub fn std_err(&self) -> StdError {
        StdError::generic_err(format!("{:?}", self))
    }
}
