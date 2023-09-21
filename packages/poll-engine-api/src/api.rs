use cosmwasm_schema::cw_serde;
use std::collections::BTreeMap;

use cosmwasm_std::{to_binary, Addr, Decimal, Timestamp, Uint128, Uint64};
use serde_with::serde_as;
use strum_macros::Display;

use common::cw::Pagination;

use crate::error::*;

/// Unique identifier for a poll.
pub type PollId = u64;

#[serde_as]
#[cw_serde]
/// A poll.
pub struct Poll {
    /// Unique identifier for the poll.
    pub id: PollId,
    /// Proposer address.
    pub proposer: Addr,
    /// Poll deposit amount.
    pub deposit_amount: u128,
    /// User-defined label for the poll.
    pub label: String,
    /// User-defined description for the poll.
    pub description: String,
    /// Voting scheme of the poll, e.g. "CoinVoting".
    pub scheme: VotingScheme,
    /// Status of the poll.
    pub status: PollStatus,
    /// Start-time of poll.
    pub started_at: Timestamp,
    /// End-time of poll.
    pub ends_at: Timestamp,
    /// Quorum to be reached for the poll to be valid.
    /// Calculated as (total votes) / (total available votes).
    pub quorum: Decimal,
    /// Threshold ratio for a vote option to be the winning one.
    /// Calculated as (votes for certain option) / (total available votes - abstaining votes).
    pub threshold: Decimal,
    /// Optional separate threshold ratio for a veto option to be the winning one.
    /// Calculated as (veto votes) / (total available votes - abstaining votes).
    /// If None, regular threshold will be used for veto option.
    pub veto_threshold: Option<Decimal>,

    #[schemars(with = "Vec<(u8, Uint128)>")]
    #[serde_as(as = "Vec<(_, _)>")]
    /// Total vote-count (value) for each outcome (key).
    pub results: BTreeMap<u8, u128>,
}

#[derive(Copy)]
#[cw_serde]
/// Supported voting schemes.
pub enum VotingScheme {
    CoinVoting,
}

#[derive(Copy, Display)]
#[cw_serde]
#[repr(u8)]
// TODO: rename to VoteOption?
pub enum VoteOutcome {
    Yes = 0,
    No = 1,
    Abstain = 2,
    Veto = 3,
}

impl From<u8> for VoteOutcome {
    fn from(v: u8) -> VoteOutcome {
        match v {
            0u8 => VoteOutcome::Yes,
            1u8 => VoteOutcome::No,
            2u8 => VoteOutcome::Abstain,
            3u8 => VoteOutcome::Veto,
            _ => panic!("invalid vote option"),
        }
    }
}

/// Unique identifier for a vote, (voter, poll_id, outcome).
pub type VoteUid = (Addr, PollId, u8);

#[cw_serde]
/// A poll vote.
pub struct Vote {
    /// Unique identifier for the poll.
    pub poll_id: PollId,
    /// Voter address.
    pub voter: Addr,
    /// The outcome, 0-indexed.
    pub outcome: u8,
    /// Number of votes on the outcome.
    pub amount: u128,
}

impl Vote {
    pub fn new(poll_id: PollId, voter: Addr, outcome: VoteOutcome, count: u128) -> Self {
        Vote {
            poll_id,
            voter,
            outcome: outcome as u8,
            amount: count,
        }
    }
}

#[cw_serde]
/// Params for creating a new poll.
pub struct CreatePollParams {
    /// Proposer address.
    pub proposer: String,
    /// Poll deposit amount.
    pub deposit_amount: Uint128,
    /// User-defined label for the poll.
    pub label: String,
    /// User-defined label for the poll.
    pub description: String,
    /// The poll type, e.g. "CoinVoting"
    pub scheme: VotingScheme,
    /// End-time for poll.
    pub ends_at: Timestamp, // TODO: consider supporting Height as well as Timestamp
    /// Quorum to be reached for the poll to be valid.
    pub quorum: Decimal,
    /// Threshold ratio for a vote option to be the winning one.
    /// Calculated as (votes for certain option) / (total available votes - abstaining votes).
    pub threshold: Decimal,
    /// Optional separate threshold ratio for a veto option to be the winning one.
    /// Calculated as (veto votes) / (total available votes - abstaining votes).
    /// If None, regular threshold will be used for veto option.
    pub veto_threshold: Option<Decimal>,
}

#[cw_serde]
/// Params for casting a vote on a poll.
pub struct CastVoteParams {
    /// Unique identifier for the poll
    pub poll_id: Uint64,
    /// The outcome.
    pub outcome: VoteOutcome,
    /// Address of the voter casting the vote.
    pub voter: String,
    /// Number of votes on the outcome.
    pub amount: Uint128,
}

#[cw_serde]
/// Params for updating user's vote amount on all active polls they voted on.
pub struct UpdateVotesParams {
    /// Address of the voter casting the vote.
    pub voter: String,
    /// New number of user's available votes.
    pub new_amount: Uint128,
}

#[cw_serde]
/// Params for ending a poll.
pub struct EndPollParams {
    /// Unique identifier for the poll
    pub poll_id: Uint64,
    /// Maximum total votes that could be cast, used to determine whether quorum was reached.
    pub maximum_available_votes: Uint128,
    /// Whether ending the poll should error if the poll had already ended
    pub error_if_already_ended: bool,
    /// Whether ending the poll should be allowed before its expiration is reached
    pub allow_early_ending: bool,
}

#[cw_serde]
/// Params for querying a poll's status.
pub struct PollStatusParams {
    /// Unique identifier for the poll
    pub poll_id: Uint64,
}

#[cw_serde]
/// Explanation why a poll was rejected.
pub enum PollRejectionReason {
    QuorumNotReached,
    ThresholdNotReached,
    QuorumAndThresholdNotReached,
    IsRejectingOutcome,
    IsVetoOutcome,
    OutcomeDraw(u8, u8, Uint128),
}

#[derive(Display)]
#[cw_serde]
/// Status of a poll.
pub enum PollStatus {
    InProgress { ends_at: Timestamp },
    Passed { outcome: u8, count: Uint128 },
    Rejected { reason: PollRejectionReason },
}

impl PollStatus {
    /// Turns a PollStatus into a PollStatusFilter, i.e. without any containing properties.
    pub fn to_filter(&self) -> PollStatusFilter {
        match self {
            PollStatus::InProgress { .. } => PollStatusFilter::InProgress,
            PollStatus::Passed { .. } => PollStatusFilter::Passed,
            PollStatus::Rejected { .. } => PollStatusFilter::Rejected,
        }
    }
}

#[derive(Display)]
#[cw_serde]
/// Like PollStatus, but used for query filtering on a poll's status.
pub enum PollStatusFilter {
    InProgress,
    Passed,
    Rejected,
}

impl PollStatusFilter {
    pub fn to_vec(&self) -> PollResult<Vec<u8>> {
        to_binary(&self).map(|b| b.to_vec()).map_err(PollError::Std)
    }
}

#[serde_as]
#[cw_serde]
/// Response model for querying a poll's status.
pub struct PollStatusResponse {
    /// Status of the poll.
    pub status: PollStatus,
    /// Poll end time.
    pub ends_at: Timestamp,

    #[schemars(with = "Vec<(u8, Uint128)>")]
    #[serde_as(as = "Vec<(_, _)>")]
    /// Total vote-count (value) for each outcome (key).
    pub results: BTreeMap<u8, u128>,
}

#[cw_serde]
/// Params for querying a poll.
pub struct PollParams {
    /// ID of the poll to be queried.
    pub poll_id: PollId,
}

#[cw_serde]
/// Response model for querying a poll.
pub struct PollResponse {
    /// The poll.
    pub poll: Poll,
}

#[cw_serde]
/// Params for listing polls.
pub struct PollsParams {
    /// Optional [poll status](PollStatusFilter) to filter for.
    pub filter: Option<PollStatusFilter>,
    /// Optional [Pagination] arguments.
    pub pagination: Pagination<Uint64>,
}

#[cw_serde]
/// Response model for listing polls.
pub struct PollsResponse {
    /// The polls.
    pub polls: Vec<Poll>,
}

#[cw_serde]
/// Params for querying the votes of a certain voter on a specific poll.
pub struct PollVoterParams {
    /// The specific poll's ID.
    pub poll_id: Uint64,
    /// The voter's address.
    pub voter_addr: String,
}

#[cw_serde]
/// Response model for querying the votes of a certain voter on a specific poll.
pub struct PollVoterResponse {
    /// The voter's vote on the specific poll.
    pub vote: Option<Vote>,
}

#[cw_serde]
/// Params for querying all votes on a specific poll.
pub struct PollVotersParams {
    /// The specific poll's ID.
    pub poll_id: PollId,
    /// Optional [Pagination] arguments.
    pub pagination: Pagination<String>,
}

#[cw_serde]
/// Response model for querying the votes of all voters on a specific poll.
pub struct PollVotersResponse {
    /// All votes on the specific poll.
    pub votes: Vec<Vote>,
}

#[cw_serde]
/// Params for querying all votes of a voter on any poll.
pub struct VoterParams {
    /// The voter's address.
    pub voter_addr: String,
    pub start_after: Option<PollId>,
    pub limit: Option<u64>,
}

#[cw_serde]
/// Response model for querying all votes of a voter on any poll.
pub struct VoterResponse {
    /// The voter's votes on any poll.
    pub votes: Vec<Vote>,
}
