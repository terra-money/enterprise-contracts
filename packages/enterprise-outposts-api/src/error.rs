use cosmwasm_std::StdError;
use thiserror::Error;

pub type EnterpriseOutpostsResult<T> = Result<T, EnterpriseOutpostsError>;

#[derive(Error, Debug, PartialEq)]
pub enum EnterpriseOutpostsError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Trying to add a proxy for a chain ID that already has a DAO-owned proxy deployed")]
    ProxyAlreadyExistsForChainId,

    #[error("Trying to add a treasury for a chain ID that already has a treasury deployed")]
    TreasuryAlreadyExistsForChainId,
}

impl EnterpriseOutpostsError {
    /// Converts this EnterpriseOutpostsError into a StdError.
    pub fn std_err(&self) -> StdError {
        StdError::generic_err(format!("{:?}", self))
    }
}
