use std::string::ToString;

use cosmwasm_std::{Decimal, Timestamp};

use common::cw::Context;

use poll_engine_api::api::{CreatePollParams, Poll, PollStatus};
use poll_engine_api::error::PollError::{
    OutsideVotingPeriod, PollAlreadyEnded, WithinVotingPeriod,
};
use poll_engine_api::error::*;

pub fn validate_create_poll(ctx: &mut Context, params: &CreatePollParams) -> PollResult<()> {
    let now = ctx.env.block.time;

    // if not 0 <= threshold <= 1
    if !(Decimal::zero().le(&params.threshold) && Decimal::one().ge(&params.threshold)) {
        return Err(PollError::InvalidArgument {
            msg: format!(
                "Invalid threshold {}, must be 0 <= threshold <= 1",
                params.threshold,
            ),
        });
    }

    match params {
        // if now >= ends_at
        CreatePollParams { ends_at, .. } if now.ge(ends_at) => Err(PollError::InvalidArgument {
            msg: format!("Invalid end time, must be {} > {} (now)", ends_at, now),
        }),

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
