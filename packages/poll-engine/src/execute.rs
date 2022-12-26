use cosmwasm_std::Uint64;

use common::cw::Context;

use crate::api::{CastVoteParams, CreatePollParams, EndPollParams, Vote, VoteOutcome};
use crate::error::PollError::PollNotFound;
use crate::error::*;
use crate::state::{polls, votes, GovState, Poll, PollStorage, GOV_STATE};
use crate::validate::{
    validate_create_poll, validate_not_already_ended, validate_voting_period_ended,
    validate_within_voting_period,
};

/// Initializes library state.
pub fn initialize_poll_engine(ctx: &mut Context) -> PollResult<()> {
    GOV_STATE.save(ctx.deps.storage, &GovState { poll_count: 0 })?;

    Ok(())
}

/// Creates a poll.
pub fn create_poll(ctx: &mut Context, params: CreatePollParams) -> PollResult<Poll> {
    validate_create_poll(ctx, &params)?;

    let poll = Poll::from(&mut ctx.deps, &ctx.env, params.clone())?;
    polls().save_poll(ctx.deps.storage, poll.clone())?;

    Ok(poll)
}

/// Casts a vote on a poll.
pub fn cast_vote(
    ctx: &mut Context,
    CastVoteParams {
        poll_id,
        outcome,
        amount,
    }: CastVoteParams,
) -> PollResult<()> {
    let state = GOV_STATE.load(ctx.deps.storage)?;

    if poll_id == Uint64::zero() || poll_id.u64() > state.poll_count {
        return Err(PollNotFound { poll_id });
    }

    let poll_key = poll_id.u64();
    let voter = ctx.info.sender.clone();

    // 1. load existing poll
    let mut poll = polls()
        .may_load(ctx.deps.storage, poll_key)?
        .ok_or(PollNotFound { poll_id })?;

    // 2. validate
    validate_not_already_ended(&poll)?;
    validate_within_voting_period(ctx.env.block.time, (poll.started_at, poll.ends_at))?;

    let new = Vote {
        poll_id: poll_id.u64(),
        voter: voter.clone(),
        outcome: outcome as u8,
        amount: amount.u128(),
    };
    // 3. load potential old voting data
    let key = (voter, poll_id.u64());
    votes()
        .update(ctx.deps.storage, key, |old| match old {
            // 5a. if old voting data for same outcome exists, subtract before adding new one to the results
            Some(old) => {
                poll.decrease_results(VoteOutcome::from(old.outcome), old.amount);
                poll.increase_results(outcome, new.amount)
                    .map_err(|e| e.std_err())?;
                // 6. also save vote in the voting storage
                Ok(new)
            }
            // 5b. if not, just add new one to the results
            None => {
                poll.increase_results(outcome, new.amount)
                    .map_err(|e| e.std_err())?;
                // 6. also save vote in the voting storage
                Ok(new)
            }
        })
        .map_err(PollError::Std)?;

    // 7. ...and poll storage
    polls().save(ctx.deps.storage, poll_key, &poll)?;

    Ok(())
}

/// Ends a poll. Must be outside of the voting period.
pub fn end_poll(
    ctx: &mut Context,
    EndPollParams {
        poll_id,
        maximum_available_votes,
        error_if_already_ended,
        allow_early_ending,
    }: EndPollParams,
) -> PollResult<()> {
    let now = ctx.env.block.time;
    let mut poll = polls()
        .may_load(ctx.deps.storage, poll_id.into())?
        .ok_or(PollNotFound { poll_id })?;

    if !allow_early_ending {
        validate_voting_period_ended(now, poll.ends_at)?;
    }

    if error_if_already_ended {
        validate_not_already_ended(&poll)?;
    }

    poll.status = poll.final_status(maximum_available_votes)?;
    polls().save(ctx.deps.storage, poll_id.into(), &poll)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use cosmwasm_std::testing::mock_dependencies;
    use cosmwasm_std::{Addr, Decimal, Timestamp, Uint128};

    use common::cw::testing::mock_ctx;
    use PollRejectionReason::OutcomeDraw;
    use VoteOutcome::{No, Veto};

    use crate::api::VoteOutcome::{Abstain, Yes};
    use crate::api::{
        CastVoteParams, CreatePollParams, EndPollParams, PollRejectionReason, PollStatus,
        PollStatusFilter, VoteOutcome, VotingScheme,
    };
    use crate::error::PollError;
    use crate::error::PollError::{PollAlreadyEnded, WithinVotingPeriod};
    use crate::execute::{cast_vote, create_poll, end_poll, initialize_poll_engine};
    use crate::helpers::mock_poll;
    use crate::query::query_poll_status;
    use crate::state::{polls, GovState, Poll, GOV_STATE};

    #[test]
    fn initialize_sets_default_gov_state() {
        let mut deps = mock_dependencies();
        let mut ctx = mock_ctx(deps.as_mut());

        initialize_poll_engine(&mut ctx).unwrap();

        let gov_state = GOV_STATE.load(ctx.deps.storage).unwrap();

        assert_eq!(gov_state, GovState { poll_count: 0 });
    }

    #[test]
    fn creates_poll() {
        let mut deps = mock_dependencies();
        let mut ctx = mock_ctx(deps.as_mut());
        let current_time = Timestamp::from_nanos(15);
        ctx.env.block.time = current_time.clone();
        let state = GovState::default();
        GOV_STATE.save(ctx.deps.storage, &state).unwrap();

        let ends_at = Timestamp::from_nanos(1015u64);
        let quorum = Decimal::from_ratio(3u8, 10u8);
        let threshold = Decimal::percent(50);
        let params = CreatePollParams {
            proposer: "proposer".to_string(),
            deposit_amount: Uint128::new(1000000),
            label: "some poll".to_string(),
            description: "some description".to_string(),
            scheme: VotingScheme::CoinVoting,
            ends_at: ends_at.clone(),
            quorum: quorum.clone(),
            threshold: threshold.clone(),
            veto_threshold: None,
        };

        create_poll(&mut ctx, params).unwrap();

        let poll = polls().load(ctx.deps.storage, 1).unwrap();
        assert_eq!(PollStatus::InProgress { ends_at }, poll.status);
        assert_eq!(current_time, poll.started_at);
        assert_eq!(quorum, poll.quorum);
        assert_eq!(threshold, poll.threshold);
    }

    #[test]
    fn casts_vote() {
        let mut deps = mock_dependencies();
        let mut ctx = mock_ctx(deps.as_mut());
        let state = GovState::default();
        GOV_STATE.save(ctx.deps.storage, &state).unwrap();
        ctx.env.block.time = Timestamp::from_nanos(0);

        let mut poll = mock_poll(ctx.deps.storage);
        poll.results = BTreeMap::from([(0, 10)]);
        poll.ends_at = ctx.env.block.time.plus_seconds(1000u64);
        polls().save(ctx.deps.storage, poll.id, &poll).unwrap();

        let params = CastVoteParams {
            poll_id: poll.id.into(),
            outcome: No,
            amount: Uint128::new(10),
        };
        cast_vote(&mut ctx, params).unwrap();

        let poll = polls().load(ctx.deps.storage, poll.id).unwrap();
        assert_eq!(&10, poll.results.get(&1).unwrap());
        assert_eq!(
            PollStatus::InProgress {
                ends_at: Timestamp::from_nanos(3)
            },
            poll.status
        );
    }

    #[test]
    fn casts_new_vote() {
        let mut deps = mock_dependencies();
        let mut ctx = mock_ctx(deps.as_mut());
        let state = GovState::default();
        GOV_STATE.save(ctx.deps.storage, &state).unwrap();

        let mut poll = mock_poll(ctx.deps.storage);
        poll.results = BTreeMap::from([(0, 10)]);
        polls().save(ctx.deps.storage, poll.id, &poll).unwrap();

        // cast a vote
        ctx.env.block.time = Timestamp::from_nanos(2);
        let params = CastVoteParams {
            poll_id: poll.id.into(),
            outcome: No,
            amount: Uint128::new(4),
        };
        cast_vote(&mut ctx, params).unwrap();

        // then again
        ctx.env.block.time = Timestamp::from_nanos(2);
        let params = CastVoteParams {
            poll_id: poll.id.into(),
            outcome: No,
            amount: Uint128::new(9),
        };
        cast_vote(&mut ctx, params).unwrap();

        let poll = polls().load(ctx.deps.storage, poll.id).unwrap();
        assert_eq!(&9, poll.results.get(&(No as u8)).unwrap());
    }

    #[test]
    fn ends_active_poll() {
        let mut deps = mock_dependencies();
        let mut ctx = mock_ctx(deps.as_mut());
        GOV_STATE
            .save(ctx.deps.storage, &GovState::default())
            .unwrap();

        let mut poll = mock_poll(ctx.deps.storage);
        poll.results = BTreeMap::from([(0, 10)]);
        poll.deposit_amount = 10;
        polls().save(ctx.deps.storage, poll.id, &poll).unwrap();

        ctx.env.block.time = Timestamp::from_nanos(3);
        let params = EndPollParams {
            poll_id: poll.id.into(),
            maximum_available_votes: 10u8.into(),
            error_if_already_ended: true,
            allow_early_ending: false,
        };
        end_poll(&mut ctx, params).unwrap();

        let res = query_poll_status(&ctx.to_query(), poll.id).unwrap();

        assert_eq!(
            PollStatus::Passed {
                outcome: 0,
                count: Uint128::new(10),
            },
            res.status
        );
        assert_eq!(poll.results, res.results);
        assert_eq!(poll.ends_at, res.ends_at);
    }

    #[test]
    fn cannot_end_already_ended_poll() {
        let mut deps = mock_dependencies();
        let mut ctx = mock_ctx(deps.as_mut());
        let state = GovState::default();
        GOV_STATE.save(ctx.deps.storage, &state).unwrap();

        let mut poll = mock_poll(ctx.deps.storage);
        poll.status = PollStatus::Rejected {
            reason: PollRejectionReason::IsRejectingOutcome,
        };
        poll.results = BTreeMap::from([(0, 10)]);
        polls().save(ctx.deps.storage, poll.id, &poll).unwrap();

        ctx.env.block.time = Timestamp::from_nanos(3);
        let params = EndPollParams {
            poll_id: poll.id.into(),
            maximum_available_votes: Uint128::one(),
            error_if_already_ended: true,
            allow_early_ending: false,
        };
        let res = end_poll(&mut ctx, params);

        assert!(res.is_err());
        assert_eq!(
            PollAlreadyEnded {
                poll_id: poll.id.into(),
                status: PollStatusFilter::Rejected.to_string(),
            },
            res.unwrap_err()
        );
    }

    #[test]
    fn cannot_vote_on_expired_poll() {
        let mut deps = mock_dependencies();
        let mut ctx = mock_ctx(deps.as_mut());
        let state = GovState::default();
        GOV_STATE.save(ctx.deps.storage, &state).unwrap();

        let mut poll = mock_poll(ctx.deps.storage);
        poll.results = BTreeMap::from([(0, 10)]);
        polls().save(ctx.deps.storage, poll.id, &poll).unwrap();

        // cast a vote
        ctx.env.block.time = Timestamp::from_nanos(4);
        let params = CastVoteParams {
            poll_id: poll.id.into(),
            outcome: No,
            amount: Uint128::new(4),
        };
        let result = cast_vote(&mut ctx, params);

        assert_eq!(
            result,
            Err(PollError::OutsideVotingPeriod {
                voting_period: (Timestamp::default(), Timestamp::from_nanos(3)),
                now: Timestamp::from_nanos(4)
            })
        );
    }

    #[test]
    fn can_end_already_ended_poll_with_error_flag_set_to_false() {
        let mut deps = mock_dependencies();
        let mut ctx = mock_ctx(deps.as_mut());
        let state = GovState { poll_count: 0 };
        GOV_STATE.save(ctx.deps.storage, &state).unwrap();

        let mut poll = Poll {
            deposit_amount: 10u128,
            ..mock_poll(ctx.deps.storage)
        };
        poll.status = PollStatus::Rejected {
            reason: PollRejectionReason::IsRejectingOutcome,
        };
        poll.results = BTreeMap::from([(0, 10)]);
        polls().save(ctx.deps.storage, poll.id, &poll).unwrap();

        ctx.env.block.time = Timestamp::from_nanos(3);
        let params = EndPollParams {
            poll_id: poll.id.into(),
            maximum_available_votes: Uint128::one(),
            error_if_already_ended: false,
            allow_early_ending: false,
        };
        let res = end_poll(&mut ctx, params);

        assert!(res.is_ok());
    }

    #[test]
    fn cannot_end_poll_before_expiration() {
        let mut deps = mock_dependencies();
        let mut ctx = mock_ctx(deps.as_mut());
        GOV_STATE
            .save(ctx.deps.storage, &GovState::default())
            .unwrap();

        let mut poll = mock_poll(ctx.deps.storage);
        poll.results = BTreeMap::from([(0, 10)]);
        poll.deposit_amount = 10;
        poll.ends_at = Timestamp::from_nanos(3);
        polls().save(ctx.deps.storage, poll.id, &poll).unwrap();

        ctx.env.block.time = Timestamp::from_nanos(2);
        let params = EndPollParams {
            poll_id: poll.id.into(),
            maximum_available_votes: 10u8.into(),
            error_if_already_ended: true,
            allow_early_ending: false,
        };
        let res = end_poll(&mut ctx, params);

        assert!(res.is_err());
        assert_eq!(
            WithinVotingPeriod {
                now: Timestamp::from_nanos(2),
                ends_at: Timestamp::from_nanos(3),
            },
            res.unwrap_err()
        );
    }

    #[test]
    fn can_end_poll_before_expiration_with_allow_early_ending_flag_set_to_true() {
        let mut deps = mock_dependencies();
        let mut ctx = mock_ctx(deps.as_mut());
        GOV_STATE
            .save(ctx.deps.storage, &GovState::default())
            .unwrap();

        let mut poll = mock_poll(ctx.deps.storage);
        poll.results = BTreeMap::from([(0, 10)]);
        poll.deposit_amount = 10;
        poll.ends_at = Timestamp::from_nanos(3);
        polls().save(ctx.deps.storage, poll.id, &poll).unwrap();

        ctx.env.block.time = Timestamp::from_nanos(2);
        let params = EndPollParams {
            poll_id: poll.id.into(),
            maximum_available_votes: 10u8.into(),
            error_if_already_ended: true,
            allow_early_ending: true,
        };
        end_poll(&mut ctx, params).unwrap();

        let res = query_poll_status(&ctx.to_query(), poll.id).unwrap();

        assert_eq!(
            PollStatus::Passed {
                outcome: 0,
                count: Uint128::new(10),
            },
            res.status
        );
        assert_eq!(poll.results, res.results);
        assert_eq!(poll.ends_at, res.ends_at);
    }

    #[test]
    fn equal_max_outcomes_ends_in_draw() {
        let mut deps = mock_dependencies();
        let mut ctx = mock_ctx(deps.as_mut());
        GOV_STATE
            .save(ctx.deps.storage, &GovState::default())
            .unwrap();

        let mut poll = mock_poll(ctx.deps.storage);
        poll.deposit_amount = 10;
        poll.threshold = Decimal::percent(15);
        poll.results = BTreeMap::from([(0, 131)]);
        polls().save(ctx.deps.storage, poll.id, &poll).unwrap();

        ctx.env.block.time = Timestamp::from_nanos(2);
        let params = CastVoteParams {
            poll_id: poll.id.into(),
            outcome: No,
            amount: Uint128::new(10),
        };
        let params2 = CastVoteParams {
            poll_id: poll.id.into(),
            outcome: No,
            amount: Uint128::new(121),
        };
        let params3 = CastVoteParams {
            poll_id: poll.id.into(),
            outcome: Abstain,
            amount: Uint128::new(110),
        };
        let params4 = CastVoteParams {
            poll_id: poll.id.into(),
            outcome: Veto,
            amount: Uint128::new(10),
        };

        ctx.info.sender = Addr::unchecked("voter_1");
        let _ = cast_vote(&mut ctx, params).unwrap();
        ctx.info.sender = Addr::unchecked("voter_2");
        let _ = cast_vote(&mut ctx, params2).unwrap();
        ctx.info.sender = Addr::unchecked("voter_3");
        let _ = cast_vote(&mut ctx, params3).unwrap();
        ctx.info.sender = Addr::unchecked("voter_4");
        let _ = cast_vote(&mut ctx, params4).unwrap();
        let poll = polls().load(ctx.deps.storage, poll.id).unwrap();

        // Expected second vote on choice 1 to override first vote
        let expected_results = BTreeMap::from([(0, 131), (1, 131), (2, 110), (3, 10)]);
        assert_eq!(expected_results, poll.results);

        ctx.env.block.time = Timestamp::from_nanos(3);
        let end_params = EndPollParams {
            poll_id: poll.id.into(),
            maximum_available_votes: Uint128::from(372u16),
            error_if_already_ended: true,
            allow_early_ending: false,
        };
        let _ = end_poll(&mut ctx, end_params).unwrap();
        let poll = polls().load(ctx.deps.storage, poll.id).unwrap();

        assert_eq!(
            PollStatus::Rejected {
                reason: OutcomeDraw(Yes as u8, No as u8, Uint128::new(131)),
            },
            poll.status
        );
    }
}
