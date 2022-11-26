use cosmwasm_std::StdError;
use thiserror::Error;

pub type StakingResult<T> = Result<T, StakingError>;

#[derive(Error, Debug, PartialEq)]
pub enum StakingError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("No assets have been staked")]
    NoAssetsStaked,

    #[error("Asset cannot be staked or unstaked - does not match DAO's governance asset")]
    InvalidStakingAsset,

    #[error("Insufficient staked assets to perform the unstaking")]
    InsufficientStakedAssets,

    #[error("No assets are currently claimable")]
    NothingToClaim,

    #[error("Invalid argument: {msg}")]
    CustomError { msg: String },
}
