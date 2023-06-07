use crate::api::ProposalActionType;
use cosmwasm_std::StdError;
use poll_engine_api::error::PollError;
use thiserror::Error;

pub type GovernanceControllerResult<T> = Result<T, GovernanceControllerError>;

#[derive(Error, Debug, PartialEq)]
pub enum GovernanceControllerError {
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

    #[error("This user is not a member of the DAO's multisig")]
    NotMultisigMember {},

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

impl GovernanceControllerError {
    /// Converts this GovernanceControllerError into a StdError.
    pub fn std_err(&self) -> StdError {
        StdError::generic_err(format!("{:?}", self))
    }
}
