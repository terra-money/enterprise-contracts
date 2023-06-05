use cosmwasm_std::StdError;
use thiserror::Error;

pub type MultisigMembershipResult<T> = Result<T, MultisigMembershipError>;

#[derive(Error, Debug, PartialEq)]
pub enum MultisigMembershipError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,
}

impl MultisigMembershipError {
    /// Converts this MultisigMembershipError into a StdError.
    pub fn std_err(&self) -> StdError {
        StdError::generic_err(format!("{:?}", self))
    }
}
