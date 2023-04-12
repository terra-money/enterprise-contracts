use cosmwasm_std::{StdError, Timestamp, Uint64};
use thiserror::Error;

pub type PollResult<T> = Result<T, PollError>;

#[derive(Error, Debug, PartialEq)]
pub enum PollError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Poll {poll_id} already exists")]
    PollAlreadyExists { poll_id: Uint64 },

    #[error("Poll {poll_id} not found")]
    PollNotFound { poll_id: Uint64 },

    #[error("Poll {poll_id} is still in progress")]
    PollInProgress { poll_id: Uint64 },

    #[error("Poll {poll_id} already ended with status: {status}")]
    PollAlreadyEnded { poll_id: Uint64, status: String },

    #[error("Outside voting period: {}/{}, must be start < {now} (now) < end", voting_period.0, voting_period.1)]
    OutsideVotingPeriod {
        voting_period: (Timestamp, Timestamp),
        now: Timestamp,
    },

    #[error("Still within voting period; must be {now} (now) >= {ends_at} (ends_at)")]
    WithinVotingPeriod { now: Timestamp, ends_at: Timestamp },

    #[error("Quorum not reached, unable to end the poll early")]
    EndingEarlyQuorumNotReached {},

    #[error("Threshold not reached, unable to end the poll early")]
    EndingEarlyThresholdNotReached {},

    #[error("Invalid argument: {msg}")]
    InvalidArgument { msg: String },
}

impl PollError {
    /// Converts this DaoError into a StdError.
    pub fn std_err(&self) -> StdError {
        StdError::generic_err(format!("{:?}", self))
    }
}
