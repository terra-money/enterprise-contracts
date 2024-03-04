use cosmwasm_std::{Order, Uint128};
use cw_storage_plus::Bound;
use itertools::Itertools;
use std::ops::Add;

use common::cw::{Pagination, QueryContext};

use crate::state::{polls, votes, PollHelpers, PollStorage, VoteStorage};
use poll_engine_api::api::{
    PollId, PollParams, PollResponse, PollStatusResponse, PollVoterParams, PollVoterResponse,
    PollVotersParams, PollVotersResponse, PollsParams, PollsResponse, TotalVotesParams,
    TotalVotesResponse, VoterResponse, VoterTotalVotesParams,
};
use poll_engine_api::error::*;

// TODO: tests
pub fn query_poll(
    qctx: &QueryContext,
    PollParams { poll_id }: PollParams,
) -> PollResult<PollResponse> {
    let poll = polls()
        .may_load(qctx.deps.storage, poll_id)?
        .ok_or(PollError::PollNotFound {
            poll_id: poll_id.into(),
        })?;

    Ok(PollResponse { poll })
}

pub fn query_polls(
    qctx: &QueryContext,
    PollsParams { filter, pagination }: PollsParams,
) -> PollResult<PollsResponse> {
    let Pagination {
        start_after,
        end_at,
        limit,
        order_by,
    } = pagination;
    let min = start_after.map(|v| Bound::exclusive(PollId::from(v)));
    let max = end_at.map(|v| Bound::inclusive(PollId::from(v)));
    let order = order_by
        .map(cosmwasm_std::Order::from)
        .unwrap_or(Order::Ascending);

    // apply poll status filter if provided
    let iter = match filter {
        Some(filter) => polls().idx.poll_status.prefix(filter.to_vec()?).range(
            qctx.deps.storage,
            min,
            max,
            order,
        ),
        None => polls().range(qctx.deps.storage, min, max, order),
    }
    .flatten()
    .map(|(_, poll)| poll);

    // apply pagination limit if provided
    let polls = match limit {
        Some(n) => iter.take(n as usize).collect_vec(),
        None => iter.collect_vec(),
    };

    Ok(PollsResponse { polls })
}

pub fn query_poll_status(
    qctx: &QueryContext,
    poll_id: impl Into<PollId>,
) -> PollResult<PollStatusResponse> {
    let poll = polls().load_poll(qctx.deps.storage, poll_id.into())?;

    Ok(PollStatusResponse {
        status: poll.status.clone(),
        ends_at: poll.ends_at,
        results: poll.results,
    })
}

// TODO: tests
pub fn query_simulate_end_poll_status(
    qctx: &QueryContext,
    poll_id: impl Into<PollId>,
    maximum_available_votes: Uint128,
) -> PollResult<PollStatusResponse> {
    let poll = polls().load_poll(qctx.deps.storage, poll_id.into())?;

    let simulated_final_status = poll.final_status(maximum_available_votes)?;

    Ok(PollStatusResponse {
        status: simulated_final_status,
        ends_at: poll.ends_at,
        results: poll.results,
    })
}

pub fn query_poll_voter(
    qctx: &QueryContext,
    PollVoterParams {
        poll_id,
        voter_addr,
    }: PollVoterParams,
) -> PollResult<PollVoterResponse> {
    let voter = qctx.deps.api.addr_validate(&voter_addr)?;

    let vote = votes().poll_voter(qctx.deps.storage, poll_id.into(), voter)?;

    Ok(PollVoterResponse { vote })
}

pub fn query_poll_voters(
    qctx: &QueryContext,
    PollVotersParams {
        poll_id,
        pagination,
    }: PollVotersParams,
) -> PollResult<PollVotersResponse> {
    let Pagination {
        start_after,
        end_at,
        limit,
        order_by,
    } = pagination;
    let min = start_after
        .map(|voter_addr| qctx.deps.api.addr_validate(&voter_addr))
        .transpose()?
        .map(|voter| Bound::exclusive((voter, poll_id)));
    let max = end_at
        .map(|voter_addr| qctx.deps.api.addr_validate(&voter_addr))
        .transpose()?
        .map(|voter| Bound::inclusive((voter, poll_id)));
    let order = order_by
        .map(cosmwasm_std::Order::from)
        .unwrap_or(Order::Ascending);

    let iter = votes()
        .idx
        .poll
        .prefix(poll_id)
        .range(qctx.deps.storage, min, max, order)
        .flatten()
        .map(|(_, vote)| vote);

    // apply pagination limit if provided
    let votes = match limit {
        Some(n) => iter.take(n as usize).collect_vec(),
        None => iter.collect_vec(),
    };

    Ok(PollVotersResponse { votes })
}

pub fn query_voter(
    qctx: &QueryContext,
    voter_addr: impl AsRef<str>,
    start_after: Option<PollId>,
    limit: Option<u64>,
) -> PollResult<VoterResponse> {
    let voter = qctx.deps.api.addr_validate(voter_addr.as_ref())?;

    let votes_iter = votes()
        .prefix(voter)
        .range(
            qctx.deps.storage,
            start_after.map(Bound::exclusive),
            None,
            Order::Ascending,
        )
        .flatten()
        .map(|(_, vote)| vote);

    // apply pagination limit if provided
    let votes = match limit {
        Some(n) => votes_iter.take(n as usize).collect_vec(),
        None => votes_iter.collect_vec(),
    };

    Ok(VoterResponse { votes })
}

// TODO: test
pub fn query_total_votes(
    qctx: &QueryContext,
    params: TotalVotesParams,
) -> PollResult<TotalVotesResponse> {
    let mut total_votes = Uint128::zero();

    for poll_id in params.poll_ids {
        // TODO: is it faster to query a bunch of them and then filter the ones we need? we can set lowest and highest as Bound, and then filter
        let poll_info = query_poll(qctx, PollParams { poll_id })?;

        total_votes = total_votes.add(Uint128::from(poll_info.poll.total_votes()));
        // TODO: checked add?
    }

    Ok(TotalVotesResponse { total_votes })
}

// TODO: test
pub fn query_voter_total_votes(
    qctx: &QueryContext,
    params: VoterTotalVotesParams,
) -> PollResult<TotalVotesResponse> {
    let voter = qctx.deps.api.addr_validate(&params.voter_addr)?;

    let mut total_votes = Uint128::zero();

    for poll_id in params.poll_ids {
        // TODO: is it faster to query a bunch of them and then filter the ones we need? we can set lowest and highest as Bound, and then filter
        let votes = votes()
            .may_load(qctx.deps.storage, (voter.clone(), poll_id))?
            .map(|it| it.amount)
            .unwrap_or_default();

        total_votes = total_votes.add(Uint128::from(votes)); // TODO: checked add?
    }

    Ok(TotalVotesResponse { total_votes })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use cosmwasm_std::{testing::mock_dependencies, Timestamp};

    use common::cw::testing::mock_ctx;
    use poll_engine_api::api::{PollStatus, PollStatusResponse};

    use crate::helpers::mock_poll;
    use crate::query::query_poll_status;
    use crate::state::{polls, GovState, GOV_STATE};

    #[test]
    fn status_in_progress_with_no_votes_when_within_voting_period() {
        let mut deps = mock_dependencies();
        let mut ctx = mock_ctx(deps.as_mut());
        ctx.env.block.time = Timestamp::from_nanos(2);
        let state = GovState::default();
        GOV_STATE.save(ctx.deps.storage, &state).unwrap();

        let ends_at = Timestamp::from_nanos(3);
        let mut poll = mock_poll(ctx.deps.storage);
        poll.ends_at = ends_at.clone();
        polls().save(ctx.deps.storage, poll.id, &poll).unwrap();

        let expected = PollStatusResponse {
            status: PollStatus::InProgress {
                ends_at: ends_at.clone(),
            },
            ends_at,
            results: BTreeMap::new(),
        };
        let actual = query_poll_status(&ctx.to_query(), poll.id).unwrap();

        assert_eq!(expected, actual);
    }
}
