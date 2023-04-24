use crate::api::ProposalActionType;
use cosmwasm_std::{StdError, Uint128};
use poll_engine_api::error::PollError;
use thiserror::Error;

pub type DaoResult<T> = Result<T, DaoError>;

#[derive(Error, Debug, PartialEq)]
pub enum DaoError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Poll(#[from] PollError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("The DAO does not have a council specified")]
    NoDaoCouncil,

    #[error("Proposal action {action} is not supported in council proposals")]
    UnsupportedCouncilProposalAction { action: ProposalActionType },

    #[error("Council members must be unique, however {member} was duplicated")]
    DuplicateCouncilMember { member: String },

    #[error("{code_id} is not a valid Enterprise code ID")]
    InvalidEnterpriseCodeId { code_id: u64 },

    #[error("Supplied existing token is not a valid CW20 contract")]
    InvalidExistingTokenContract,

    #[error("Supplied existing NFT is not a valid CW721 contract")]
    InvalidExistingNftContract,

    #[error("Supplied existing multisig is not a valid CW3 contract")]
    InvalidExistingMultisigContract,

    #[error("Zero-weighted members are not allowed upon DAO creation")]
    ZeroInitialWeightMember,

    #[error("Zero initial DAO balance is not allowed upon DAO creation")]
    ZeroInitialDaoBalance,

    #[error("Duplicate multisig members are not allowed upon DAO creation")]
    DuplicateMultisigMember,

    #[error("Attempting to edit a member's weight multiple times")]
    DuplicateMultisigMemberWeightEdit,

    #[error("Zero-duration voting is not allowed")]
    ZeroVoteDuration,

    #[error("Proposal voting duration cannot be longer than unstaking duration")]
    VoteDurationLongerThanUnstaking,

    #[error("Requiring a minimum deposit for proposals is not allowed for this DAO type")]
    MinimumDepositNotAllowed,

    #[error("The given proposal was not found in this DAO")]
    NoSuchProposal,

    #[error("Proposal is of another type")]
    WrongProposalType,

    #[error("The given proposal has already been executed")]
    ProposalAlreadyExecuted,

    #[error("No votes are available")]
    NoVotesAvailable,

    #[error("Asset cannot be staked or unstaked - does not match DAO's governance asset")]
    InvalidStakingAsset,

    #[error("Insufficient staked assets to perform the unstaking")]
    InsufficientStakedAssets,

    #[error("To create a proposal, a deposit amount of at least {required_amount} is required")]
    InsufficientProposalDeposit { required_amount: Uint128 },

    #[error("No NFT token with ID {token_id} has been staked by this user")]
    NoNftTokenStaked { token_id: String },

    #[error("This user does not own nor stake DAO's NFT")]
    NotNftOwner {},

    #[error("This user is not a member of the DAO's multisig")]
    NotMultisigMember {},

    #[error("NFT token with ID {token_id} has already been staked")]
    NftTokenAlreadyStaked { token_id: String },

    #[error("No assets are currently claimable")]
    NothingToClaim,

    #[error("An asset is added or removed multiple times")]
    DuplicateAssetFound,

    #[error("An asset is present in both add and remove lists")]
    AssetPresentInBothAddAndRemove,

    #[error("An NFT is added or removed multiple times")]
    DuplicateNftFound,

    #[error("An NFT is present in both add and remove lists")]
    NftPresentInBothAddAndRemove,

    #[error("Error parsing message into Cosmos message")]
    InvalidCosmosMessage,

    #[error("This operation is not a supported for {dao_type} DAOs")]
    UnsupportedOperationForDaoType { dao_type: String },

    #[error("Custom Error val: {val}")]
    CustomError { val: String },

    #[error("Invalid argument: {msg}")]
    InvalidArgument { msg: String },
}

impl DaoError {
    /// Converts this DaoError into a StdError.
    pub fn std_err(&self) -> StdError {
        StdError::generic_err(format!("{:?}", self))
    }
}
