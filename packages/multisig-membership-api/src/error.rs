use cosmwasm_std::StdError;
use membership_common_api::error::MembershipError;
use thiserror::Error;

pub type MultisigMembershipResult<T> = Result<T, MultisigMembershipError>;

#[derive(Error, Debug, PartialEq)]
pub enum MultisigMembershipError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Common(#[from] MembershipError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("A user appears multiple times in the weights array")]
    DuplicateUserWeightFound,
}

impl MultisigMembershipError {
    /// Converts this MultisigMembershipError into a StdError.
    pub fn std_err(&self) -> StdError {
        StdError::generic_err(format!("{:?}", self))
    }
}
