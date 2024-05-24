use cosmwasm_std::StdError;
use membership_common_api::error::MembershipError;
use thiserror::Error;

pub type NftStakingResult<T> = Result<T, NftStakingError>;

#[derive(Error, Debug, PartialEq)]
pub enum NftStakingError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Common(#[from] MembershipError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("ICS721 NFT still not transferred - NFT contract does not yet exist on local chain")]
    Ics721StillNotTransferred,

    #[error("NFT token with ID {token_id} has already been staked")]
    NftTokenAlreadyStaked { token_id: String },

    #[error("No NFT token with ID {token_id} has been staked by this user")]
    NoNftTokenStaked { token_id: String },
}

impl NftStakingError {
    /// Converts this NftStakingError into a StdError.
    pub fn std_err(&self) -> StdError {
        StdError::generic_err(format!("{:?}", self))
    }
}
