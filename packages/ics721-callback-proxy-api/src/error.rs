use cosmwasm_std::StdError;
use thiserror::Error;

pub type Ics721CallbackProxyResult<T> = Result<T, Ics721CallbackProxyError>;

#[derive(Error, Debug, PartialEq)]
pub enum Ics721CallbackProxyError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,
}

impl Ics721CallbackProxyError {
    /// Converts this Ics721CallbackProxyError into a StdError.
    pub fn std_err(&self) -> StdError {
        StdError::generic_err(format!("{:?}", self))
    }
}
