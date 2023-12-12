use cosmwasm_std::StdError;
use thiserror::Error;

pub type MembershipResult<T> = Result<T, MembershipError>;

#[derive(Error, Debug, PartialEq)]
pub enum MembershipError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("User did not sign the attestation, and they're restricted from this functionality")]
    RestrictedUser,
}

impl MembershipError {
    /// Converts this MembershipError into a StdError.
    pub fn std_err(&self) -> StdError {
        StdError::generic_err(format!("{:?}", self))
    }
}
