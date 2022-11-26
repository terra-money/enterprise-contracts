use cosmwasm_std::Order;
use cw_storage_plus::Bound;
use itertools::Itertools;

use common::cw::{Pagination, QueryContext, RangeArgs};

use crate::api::{
    MaxVoteParams, MaxVoteResponse, PollId, PollParams, PollResponse, PollStatusResponse,
    PollVoterParams, PollVoterResponse, PollVotersParams, PollVotersResponse, PollsParams,
    PollsResponse, VoterResponse,
};
use crate::error::*;
use crate::state::{polls, votes, PollStorage, VoteStorage};

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

pub fn query_voter(qctx: &QueryContext, voter_addr: impl AsRef<str>) -> PollResult<VoterResponse> {
    let voter = qctx.deps.api.addr_validate(voter_addr.as_ref())?;

    let votes = votes()
        .prefix(voter)
        .range(qctx.deps.storage, None, None, Order::Ascending)
        .flatten()
        .map(|(_, vote)| vote)
        .collect_vec();

    Ok(VoterResponse { votes })
}

pub fn query_max_vote(
    qctx: &QueryContext,
    MaxVoteParams {
        voter_addr,
        poll_status,
    }: MaxVoteParams,
) -> PollResult<MaxVoteResponse> {
    let voter = qctx.deps.api.addr_validate(&voter_addr)?;

    let max_vote = votes().max_vote(
        qctx.deps.storage,
        voter,
        poll_status,
        RangeArgs::default(),
        RangeArgs::default(),
    )?;

    Ok(MaxVoteResponse { max_vote })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use cosmwasm_std::{testing::mock_dependencies, Timestamp};

    use crate::api::{PollStatus, PollStatusResponse};
    use common::cw::testing::mock_ctx;

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
