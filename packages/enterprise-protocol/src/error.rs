use cosmwasm_std::StdError;
use enterprise_versioning_api::api::Version;
use thiserror::Error;

pub type DaoResult<T> = Result<T, DaoError>;

#[derive(Error, Debug, PartialEq)]
pub enum DaoError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Initialization has already happened")]
    AlreadyInitialized,

    #[error("Supplied existing token is not a valid CW20 contract")]
    InvalidExistingTokenContract,

    #[error("Supplied existing NFT is not a valid CW721 contract")]
    InvalidExistingNftContract,

    #[error("Supplied existing multisig is not a valid CW3 contract")]
    InvalidExistingMultisigContract,

    #[error("Zero-weighted members are not allowed upon DAO creation")]
    ZeroInitialWeightMember,

    #[error("Attempting to create a token DAO with no holders and no way to mint tokens")]
    TokenDaoWithNoBalancesOrMint,

    #[error("Attempting to create a multisig DAO without initial members")]
    MultisigDaoWithNoInitialMembers,

    #[error("Zero initial DAO balance is not allowed upon DAO creation")]
    ZeroInitialDaoBalance,

    #[error("Attempting to migrate from {current} to a lower version ({target})")]
    MigratingToLowerVersion { current: Version, target: Version },

    #[error("Supplied migrate msg array contains a duplicate migrate msg for version {version}")]
    DuplicateVersionMigrateMsgFound { version: Version },

    #[error("Duplicate whitelist asset has been supplied")]
    DuplicateWhitelistAssetFound,

    #[error("Proposal voting duration cannot be longer than unstaking duration")]
    VoteDurationLongerThanUnstaking,
}

impl DaoError {
    /// Converts this DaoError into a StdError.
    pub fn std_err(&self) -> StdError {
        StdError::generic_err(format!("{:?}", self))
    }
}
