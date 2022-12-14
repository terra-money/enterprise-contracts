use std::collections::BTreeMap;

use cosmwasm_std::{to_binary, Addr, Decimal, Timestamp, Uint128, Uint64};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use strum_macros::Display;

use common::cw::Pagination;

use crate::error::*;
use crate::state::Poll;

/// Unique identifier for a poll.
pub type PollId = u64;

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// Supported voting schemes.
pub enum VotingScheme {
    CoinVoting,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, JsonSchema, Display)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum DefaultVoteOption {
    Yes = 0,
    No = 1,
    Abstain = 2,
    Veto = 3,
}

impl From<u8> for DefaultVoteOption {
    fn from(v: u8) -> DefaultVoteOption {
        match v {
            0u8 => DefaultVoteOption::Yes,
            1u8 => DefaultVoteOption::No,
            2u8 => DefaultVoteOption::Abstain,
            3u8 => DefaultVoteOption::Veto,
            _ => panic!("invalid vote option"),
        }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VoteOutcome {
    Default(DefaultVoteOption),
    Multichoice(u8),
}

impl From<VoteOutcome> for u8 {
    fn from(source: VoteOutcome) -> Self {
        match source {
            VoteOutcome::Default(opt) => opt as u8,
            VoteOutcome::Multichoice(outcome) => outcome,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PollType {
    Default,

    /// Sentiment voting.
    Multichoice {
        /// Threshold ratio, i.e. sum(most_voted) / total_votes.
        threshold: Decimal,
        /// Number of outcomes, 0-indexed.
        n_outcomes: u8,
        /// List of possible winning outcomes that will cause a poll's status to become "Rejected".
        /// Can for example be used to create a Yes/No poll.
        rejecting_outcomes: Vec<u8>,
    },
}

/// Unique identifier for a vote, (voter, poll_id, outcome).
pub type VoteUid = (Addr, PollId, u8);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
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
    pub fn new(poll_id: PollId, voter: Addr, outcome: u8, count: u128) -> Self {
        Vote {
            poll_id,
            voter,
            outcome,
            amount: count,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
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
    /// Type of the poll
    pub poll_type: PollType,
    /// The poll type, e.g. "CoinVoting"
    pub scheme: VotingScheme,
    /// End-time for poll.
    pub ends_at: Timestamp, // TODO: consider supporting Height as well as Timestamp
    /// Quorum to be reached for the poll to be valid.
    pub quorum: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// Params for casting a vote on a poll.
pub struct CastVoteParams {
    /// Unique identifier for the poll
    pub poll_id: Uint64,
    /// The outcome.
    pub outcome: VoteOutcome,
    /// Number of votes on the outcome.
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// Params for ending a poll.
pub struct EndPollParams {
    /// Unique identifier for the poll
    pub poll_id: Uint64,
    /// Maximum total votes that could be cast, used to determine whether quorum was reached.
    pub maximum_available_votes: Uint128,
    /// Whether ending a poll should error if the poll had already ended
    pub error_if_already_ended: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// Params for querying a poll's status.
pub struct PollStatusParams {
    /// Unique identifier for the poll
    pub poll_id: Uint64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// Explanation why a poll was rejected.
pub enum PollRejectionReason {
    QuorumNotReached,
    ThresholdNotReached,
    QuorumAndThresholdNotReached,
    IsRejectingOutcome,
    OutcomeDraw(u8, u8, Uint128),
}

#[derive(Serialize, Deserialize, Clone, Debug, Display, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
/// Status of a poll.
pub enum PollStatus {
    InProgress {
        ends_at: Timestamp,
    },
    Passed {
        outcome: u8,
        count: Uint128,
    },
    Rejected {
        outcome: Option<u8>,
        count: Option<Uint128>,
        reason: PollRejectionReason,
    },
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

#[derive(Serialize, Deserialize, Clone, Debug, Display, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// Params for querying a poll.
pub struct PollParams {
    /// ID of the poll to be queried.
    pub poll_id: PollId,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// Response model for querying a poll.
pub struct PollResponse {
    /// The poll.
    pub poll: Poll,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// Params for listing polls.
pub struct PollsParams {
    /// Optional [poll status](PollStatusFilter) to filter for.
    pub filter: Option<PollStatusFilter>,
    /// Optional [Pagination] arguments.
    pub pagination: Pagination<Uint64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// Response model for listing polls.
pub struct PollsResponse {
    /// The polls.
    pub polls: Vec<Poll>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// Params for querying the votes of a certain voter on a specific poll.
pub struct PollVoterParams {
    /// The specific poll's ID.
    pub poll_id: Uint64,
    /// The voter's address.
    pub voter_addr: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// Response model for querying the votes of a certain voter on a specific poll.
pub struct PollVoterResponse {
    /// The voter's vote on the specific poll.
    pub vote: Option<Vote>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// Params for querying all votes on a specific poll.
pub struct PollVotersParams {
    /// The specific poll's ID.
    pub poll_id: PollId,
    /// Optional [Pagination] arguments.
    pub pagination: Pagination<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// Response model for querying the votes of all voters on a specific poll.
pub struct PollVotersResponse {
    /// All votes on the specific poll.
    pub votes: Vec<Vote>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// Params for querying all votes of a voter on any poll.
pub struct VoterParams {
    /// The voter's address.
    pub voter_addr: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// Response model for querying all votes of a voter on any poll.
pub struct VoterResponse {
    /// The voter's votes on any poll.
    pub votes: Vec<Vote>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// Params for querying the max vote of a voter on any poll with a certain [poll status](PollStatusFilter).
pub struct MaxVoteParams {
    /// The voter's address.
    pub voter_addr: String,
    /// [Poll status](PollStatusFilter) to filter for.
    pub poll_status: PollStatusFilter,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// Response model for querying the max vote of a voter on any poll with a certain [poll status](PollStatusFilter).
pub struct MaxVoteResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The matching max vote, if any.
    pub max_vote: Option<Vote>,
}
