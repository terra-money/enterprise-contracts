use cosmwasm_std::StdError;
use thiserror::Error;

pub type EnterpriseTreasuryResult<T> = Result<T, EnterpriseTreasuryError>;

#[derive(Error, Debug, PartialEq)]
pub enum EnterpriseTreasuryError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,
}

impl EnterpriseTreasuryError {
    /// Converts this EnterpriseTreasuryError into a StdError.
    pub fn std_err(&self) -> StdError {
        StdError::generic_err(format!("{:?}", self))
    }
}
