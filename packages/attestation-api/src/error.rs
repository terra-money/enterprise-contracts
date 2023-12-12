use cosmwasm_std::StdError;
use thiserror::Error;

pub type AttestationResult<T> = Result<T, AttestationError>;

#[derive(Error, Debug, PartialEq)]
pub enum AttestationError {
    #[error("{0}")]
    Std(#[from] StdError),
}

impl AttestationError {
    /// Converts this AttestationError into a StdError.
    pub fn std_err(&self) -> StdError {
        StdError::generic_err(format!("{:?}", self))
    }
}
