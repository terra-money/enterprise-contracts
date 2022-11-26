use cosmwasm_std::Uint64;

use common::cw::Context;

use crate::api::{CastVoteParams, CreatePollParams, EndPollParams, Vote};
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

    let outcome = outcome.into();

    let new = Vote {
        poll_id: poll_id.u64(),
        voter: voter.clone(),
        outcome,
        amount: amount.u128(),
    };
    // 3. load potential old voting data
    let key = (voter, poll_id.u64());
    votes()
        .update(ctx.deps.storage, key, |old| match old {
            // 5a. if old voting data for same outcome exists, subtract before adding new one to the results
            Some(old) => {
                poll.decrease_results(old.outcome, old.amount);
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
    }: EndPollParams,
) -> PollResult<()> {
    let now = ctx.env.block.time;
    let mut poll = polls()
        .may_load(ctx.deps.storage, poll_id.into())?
        .ok_or(PollNotFound { poll_id })?;

    validate_voting_period_ended(now, poll.ends_at)?;
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

    use crate::api::{
        CastVoteParams, CreatePollParams, EndPollParams, PollRejectionReason, PollStatus,
        PollStatusFilter, PollType, VoteOutcome, VotingScheme,
    };
    use crate::error::PollError;
    use crate::error::PollError::PollAlreadyEnded;
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
        let params = CreatePollParams {
            proposer: "proposer".to_string(),
            deposit_amount: Uint128::new(1000000),
            label: "some poll".to_string(),
            description: "some description".to_string(),
            poll_type: PollType::Multichoice {
                threshold: Decimal::percent(50),
                n_outcomes: 2,
                rejecting_outcomes: vec![1],
            },
            scheme: VotingScheme::CoinVoting,
            ends_at: ends_at.clone(),
            quorum: quorum.clone(),
        };

        create_poll(&mut ctx, params).unwrap();

        let poll = polls().load(ctx.deps.storage, 1).unwrap();
        assert_eq!(
            PollType::Multichoice {
                n_outcomes: 2,
                rejecting_outcomes: vec![1],
                threshold: Decimal::percent(50),
            },
            poll.poll_type
        );
        assert_eq!(PollStatus::InProgress { ends_at }, poll.status);
        assert_eq!(current_time, poll.started_at);
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
            outcome: VoteOutcome::Multichoice(1),
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
            outcome: VoteOutcome::Multichoice(1),
            amount: Uint128::new(4),
        };
        let _ = cast_vote(&mut ctx, params).unwrap();

        // then again
        ctx.env.block.time = Timestamp::from_nanos(2);
        let params = CastVoteParams {
            poll_id: poll.id.into(),
            outcome: VoteOutcome::Multichoice(1),
            amount: Uint128::new(9),
        };
        let _ = cast_vote(&mut ctx, params).unwrap();

        let poll = polls().load(ctx.deps.storage, poll.id).unwrap();
        assert_eq!(&9, poll.results.get(&1).unwrap());
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
            maximum_available_votes: Uint128::from(10u8),
            error_if_already_ended: true,
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
            outcome: Some(1),
            count: Some(Uint128::new(20)),
            reason: PollRejectionReason::IsRejectingOutcome,
        };
        poll.results = BTreeMap::from([(0, 10)]);
        polls().save(ctx.deps.storage, poll.id, &poll).unwrap();

        ctx.env.block.time = Timestamp::from_nanos(3);
        let params = EndPollParams {
            poll_id: poll.id.into(),
            maximum_available_votes: Uint128::from(1u8),
            error_if_already_ended: true,
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
            outcome: VoteOutcome::Multichoice(1),
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
            outcome: Some(1),
            count: Some(Uint128::new(20)),
            reason: PollRejectionReason::IsRejectingOutcome,
        };
        poll.results = BTreeMap::from([(0, 10)]);
        polls().save(ctx.deps.storage, poll.id, &poll).unwrap();

        ctx.env.block.time = Timestamp::from_nanos(3);
        let params = EndPollParams {
            poll_id: poll.id.into(),
            maximum_available_votes: Uint128::from(1u8),
            error_if_already_ended: false,
        };
        let res = end_poll(&mut ctx, params);

        assert!(res.is_ok());
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
        poll.poll_type = PollType::Multichoice {
            threshold: Default::default(),
            n_outcomes: 4,
            rejecting_outcomes: vec![1],
        };
        poll.results = BTreeMap::from([(0, 10)]);
        polls().save(ctx.deps.storage, poll.id, &poll).unwrap();

        ctx.env.block.time = Timestamp::from_nanos(2);
        let params = CastVoteParams {
            poll_id: poll.id.into(),
            outcome: VoteOutcome::Multichoice(1),
            amount: Uint128::new(10),
        };
        let params2 = CastVoteParams {
            poll_id: poll.id.into(),
            outcome: VoteOutcome::Multichoice(1),
            amount: Uint128::new(121),
        };
        let params3 = CastVoteParams {
            poll_id: poll.id.into(),
            outcome: VoteOutcome::Multichoice(2),
            amount: Uint128::new(131),
        };
        let params4 = CastVoteParams {
            poll_id: poll.id.into(),
            outcome: VoteOutcome::Multichoice(3),
            amount: Uint128::new(110),
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
        let expected_results = BTreeMap::from([(0, 10), (1, 131), (2, 131), (3, 110)]);
        assert_eq!(expected_results, poll.results);

        ctx.env.block.time = Timestamp::from_nanos(3);
        let end_params = EndPollParams {
            poll_id: poll.id.into(),
            maximum_available_votes: Uint128::from(372u16),
            error_if_already_ended: true,
        };
        let _ = end_poll(&mut ctx, end_params).unwrap();
        let poll = polls().load(ctx.deps.storage, poll.id).unwrap();

        assert_eq!(
            PollStatus::Rejected {
                outcome: None,
                count: None,
                reason: PollRejectionReason::OutcomeDraw(1, 2, Uint128::new(131)),
            },
            poll.status
        );
    }
}
