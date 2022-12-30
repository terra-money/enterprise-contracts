use crate::api::PollRejectionReason::{
    IsRejectingOutcome, IsVetoOutcome, OutcomeDraw, QuorumNotReached, ThresholdNotReached,
};
use crate::api::VoteOutcome::Veto;
use crate::api::{PollId, PollStatus, PollStatusFilter, VotingScheme};
use crate::error::PollResult;
use crate::state::{polls, Poll, PollStorage};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{to_binary, Addr, Decimal, StdResult, Storage, Timestamp, Uint128};
use cw_storage_plus::{Index, IndexList, IndexedMap, MultiIndex};
use serde_with::serde_as;
use std::collections::BTreeMap;

#[cw_serde]
#[serde_as]
struct PollV1 {
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
    /// Type of the poll
    pub poll_type: PollTypeV1,
    /// Voting scheme of the poll, e.g. "CoinVoting".
    pub scheme: VotingScheme,
    /// Status of the poll.
    pub status: PollStatusV1,
    /// Start-time of poll.
    pub started_at: Timestamp,
    /// End-time of poll.
    pub ends_at: Timestamp,
    /// Quorum to be reached for the poll to be valid
    pub quorum: Decimal,

    #[schemars(with = "Vec<(u8, Uint128)>")]
    #[serde_as(as = "Vec<(_, _)>")]
    /// Total vote-count (value) for each outcome (key).
    pub results: BTreeMap<u8, u128>,
}

#[cw_serde]
enum PollTypeV1 {
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

#[cw_serde]
enum PollStatusV1 {
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
        reason: PollRejectionReasonV1,
    },
}

impl PollStatusV1 {
    /// Turns a PollStatus into a PollStatusFilter, i.e. without any containing properties.
    pub fn to_filter(&self) -> PollStatusFilter {
        match self {
            PollStatusV1::InProgress { .. } => PollStatusFilter::InProgress,
            PollStatusV1::Passed { .. } => PollStatusFilter::Passed,
            PollStatusV1::Rejected { .. } => PollStatusFilter::Rejected,
        }
    }
}

#[cw_serde]
enum PollRejectionReasonV1 {
    QuorumNotReached,
    ThresholdNotReached,
    QuorumAndThresholdNotReached,
    IsRejectingOutcome,
    OutcomeDraw(u8, u8, Uint128),
}

struct PollIndices<'a> {
    /// pk(poll_id)

    /// ik(poll_status)
    pub poll_status: MultiIndex<'a, Vec<u8>, PollV1, PollId>,
}

impl<'a> IndexList<PollV1> for PollIndices<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<PollV1>> + '_> {
        let v: Vec<&dyn Index<PollV1>> = vec![&self.poll_status];
        Box::new(v.into_iter())
    }
}

fn polls_v1<'a>() -> IndexedMap<'a, PollId, PollV1, PollIndices<'a>> {
    let indices = PollIndices {
        poll_status: MultiIndex::new(
            |_, d| {
                to_binary(&d.status.clone().to_filter())
                    .expect("error serializing poll status")
                    .0
            },
            "POLLS",
            "POLLS__POLL_STATUS",
        ),
    };
    IndexedMap::new("POLLS", indices)
}

// TODO: test somehow?
pub fn migrate_v1_to_v2(store: &mut dyn Storage) -> PollResult<()> {
    polls_v1()
        .range(store, None, None, Ascending)
        .collect::<StdResult<Vec<(PollId, PollV1)>>>()?
        .into_iter()
        .try_for_each(|(id, poll_v1)| {
            let status = match poll_v1.status {
                PollStatusV1::InProgress { ends_at } => PollStatus::InProgress { ends_at },
                PollStatusV1::Passed { outcome, count } => PollStatus::Passed { outcome, count },
                PollStatusV1::Rejected {
                    reason, outcome, ..
                } => {
                    let new_reason = match reason {
                        PollRejectionReasonV1::QuorumNotReached => QuorumNotReached,
                        PollRejectionReasonV1::ThresholdNotReached => ThresholdNotReached,
                        PollRejectionReasonV1::QuorumAndThresholdNotReached => QuorumNotReached,
                        PollRejectionReasonV1::IsRejectingOutcome => {
                            if outcome == Some(Veto as u8) {
                                IsVetoOutcome
                            } else {
                                IsRejectingOutcome
                            }
                        }
                        PollRejectionReasonV1::OutcomeDraw(a, b, count) => OutcomeDraw(a, b, count),
                    };
                    PollStatus::Rejected { reason: new_reason }
                }
            };
            let threshold = match poll_v1.poll_type {
                PollTypeV1::Default => Decimal::percent(50),
                PollTypeV1::Multichoice { threshold, .. } => threshold,
            };
            let poll = Poll {
                id,
                proposer: poll_v1.proposer,
                deposit_amount: poll_v1.deposit_amount,
                label: poll_v1.label,
                description: poll_v1.description,
                scheme: poll_v1.scheme,
                status,
                started_at: poll_v1.started_at,
                ends_at: poll_v1.ends_at,
                quorum: poll_v1.quorum,
                threshold,
                veto_threshold: None,
                results: poll_v1.results,
            };

            let remove_result = polls_v1().remove(store, id);
            if let Err(e) = remove_result {
                Err(e.into())
            } else {
                polls().save_poll(store, poll)
            }
        })?;

    Ok(())
}
