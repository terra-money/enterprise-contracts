use cosmwasm_std::StdError;
use thiserror::Error;

pub type EnterpriseVersioningResult<T> = Result<T, EnterpriseVersioningError>;

#[derive(Error, Debug, PartialEq)]
pub enum EnterpriseVersioningError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,
}

impl EnterpriseVersioningError {
    /// Converts this EnterpriseVersioningError into a StdError.
    pub fn std_err(&self) -> StdError {
        StdError::generic_err(format!("{:?}", self))
    }
}
