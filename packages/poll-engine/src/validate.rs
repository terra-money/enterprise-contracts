use std::string::ToString;

use cosmwasm_std::{Decimal, Timestamp};

use common::cw::Context;

use crate::api::{CreatePollParams, PollStatus, PollType};
use crate::error::PollError::{OutsideVotingPeriod, PollAlreadyEnded, WithinVotingPeriod};
use crate::error::*;
use crate::state::Poll;

pub fn validate_create_poll(ctx: &mut Context, params: &CreatePollParams) -> PollResult<()> {
    let now = ctx.env.block.time;

    match params {
        // if now >= ends_at
        CreatePollParams { ends_at, .. } if now.ge(ends_at) => Err(PollError::InvalidArgument {
            msg: format!("Invalid end time, must be {} < {} (now)", ends_at, now),
        }),

        // if not 0 <= threshold <= 1
        CreatePollParams {
            poll_type: PollType::Multichoice { threshold, .. },
            ..
        } if !(Decimal::zero().le(threshold) && Decimal::one().ge(threshold)) => {
            Err(PollError::InvalidArgument {
                msg: format!(
                    "Invalid threshold {}, must be 0 <= threshold <= 1",
                    threshold
                ),
            })
        }

        CreatePollParams {
            poll_type:
                PollType::Multichoice {
                    rejecting_outcomes,
                    n_outcomes,
                    ..
                },
            ..
        } if rejecting_outcomes.len() >= (*n_outcomes) as usize => {
            Err(PollError::InvalidArgument {
                msg: format!(
                    "Invalid rejecting outcomes count {}, must be count < {} (n_outcomes)",
                    rejecting_outcomes.len(),
                    n_outcomes,
                ),
            })
        }

        _ => Ok(()),
    }
}

pub fn validate_within_voting_period(
    now: Timestamp,
    voting_period: (Timestamp, Timestamp),
) -> PollResult<()> {
    let (start, end) = voting_period;

    // must be start <= now < end
    if !(start <= now && now < end) {
        Err(OutsideVotingPeriod { voting_period, now })
    } else {
        Ok(())
    }
}

pub fn validate_voting_period_ended(now: Timestamp, ends_at: Timestamp) -> PollResult<()> {
    // must be now < ends_at
    if now < ends_at {
        Err(WithinVotingPeriod { now, ends_at })
    } else {
        Ok(())
    }
}

pub fn validate_not_already_ended(poll: &Poll) -> PollResult<()> {
    match &poll.status {
        PollStatus::InProgress { .. } => Ok(()),
        status => Err(PollAlreadyEnded {
            poll_id: poll.id.into(),
            status: status.to_string(),
        }),
    }
}
