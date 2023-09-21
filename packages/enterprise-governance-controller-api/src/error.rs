use crate::api::ProposalActionType;
use cosmwasm_std::{StdError, Uint128};
use cw_utils::ParseReplyError;
use enterprise_protocol::error::DaoError;
use poll_engine_api::error::PollError;
use serde_json_wasm::ser::Error;
use thiserror::Error;

pub type GovernanceControllerResult<T> = Result<T, GovernanceControllerError>;

#[derive(Error, Debug, PartialEq)]
pub enum GovernanceControllerError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Poll(#[from] PollError),

    #[error("{0}")]
    Dao(#[from] DaoError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("The user has not signed the DAO's attestation and is not allowed to use most of DAO's functions")]
    RestrictedUser,

    #[error("The DAO does not have a council specified")]
    NoDaoCouncil,

    #[error("Proposal action {action} is not supported in council proposals")]
    UnsupportedCouncilProposalAction { action: ProposalActionType },

    #[error("Proposal exceeds maximum amount of proposal actions, which is {maximum}")]
    MaximumProposalActionsExceeded { maximum: u8 },

    #[error("Council members must be unique, however {member} was duplicated")]
    DuplicateCouncilMember { member: String },

    #[error("{code_id} is not a valid Enterprise code ID")]
    InvalidEnterpriseCodeId { code_id: u64 },

    #[error("Attempting to edit a member's weight multiple times")]
    DuplicateMultisigMemberWeightEdit,

    #[error("Zero-duration voting is not allowed")]
    ZeroVoteDuration,

    #[error("To create a proposal, a deposit amount of at least {required_amount} is required")]
    InsufficientProposalDeposit { required_amount: Uint128 },

    #[error("Invalid deposit type")]
    InvalidDepositType,

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

    #[error("This user has no voting power to create a proposal")]
    NoVotingPower,

    #[error("An asset is added or removed multiple times")]
    DuplicateAssetFound,

    #[error("An asset is present in both add and remove lists")]
    AssetPresentInBothAddAndRemove,

    #[error("CW1155 assets are not yet supported for this operation")]
    UnsupportedCw1155Asset,

    #[error("An NFT is added or removed multiple times")]
    DuplicateNftFound,

    #[error("An NFT token is being deposited multiple times")]
    DuplicateNftDeposit,

    #[error("An NFT is present in both add and remove lists")]
    NftPresentInBothAddAndRemove,

    #[error("Error parsing message into Cosmos message")]
    InvalidCosmosMessage,

    #[error("This operation is not a supported for {dao_type} DAOs")]
    UnsupportedOperationForDaoType { dao_type: String },

    #[error("No cross chain deployment has been deployed for the given chain ID")]
    NoCrossChainDeploymentForGivenChainId,

    #[error("Custom Error val: {val}")]
    CustomError { val: String },

    #[error("Invalid argument: {msg}")]
    InvalidArgument { msg: String },
}

impl From<serde_json_wasm::ser::Error> for GovernanceControllerError {
    fn from(value: Error) -> Self {
        GovernanceControllerError::Std(StdError::generic_err(value.to_string()))
    }
}

impl From<ParseReplyError> for GovernanceControllerError {
    fn from(value: ParseReplyError) -> Self {
        GovernanceControllerError::Std(StdError::generic_err(value.to_string()))
    }
}

impl GovernanceControllerError {
    /// Converts this GovernanceControllerError into a StdError.
    pub fn std_err(&self) -> StdError {
        StdError::generic_err(format!("{:?}", self))
    }
}
