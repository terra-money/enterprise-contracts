use crate::migration::migrate_to_v1_0_0;
use crate::state::ADMIN;
use common::cw::{Context, QueryContext};
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
};
use cw2::set_contract_version;
use enterprise_governance_api::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use poll_engine::execute::initialize_poll_engine;
use poll_engine::query::{
    query_poll, query_poll_status, query_poll_voter, query_poll_voters, query_polls,
    query_simulate_end_poll_status, query_total_votes, query_voter, query_voter_total_votes,
};
use poll_engine_api::api::{
    CastVoteParams, CreatePollParams, EndPollParams, PollStatus, UpdateVotesParams, VoteOutcome,
};
use poll_engine_api::error::PollError::Unauthorized;
use poll_engine_api::error::PollResult;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:enterprise-governance";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> PollResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let admin = deps.api.addr_validate(&msg.admin)?;
    ADMIN.save(deps.storage, &admin)?;

    let mut ctx = Context { deps, env, info };

    initialize_poll_engine(&mut ctx)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", admin.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> PollResult<Response> {
    let admin = ADMIN.load(deps.storage)?;

    if info.sender != admin {
        return Err(Unauthorized {});
    }

    let ctx = &mut Context { deps, env, info };
    match msg {
        ExecuteMsg::CreatePoll(params) => create_poll(ctx, params),
        ExecuteMsg::CastVote(params) => cast_vote(ctx, params),
        ExecuteMsg::UpdateVotes(params) => update_votes(ctx, params),
        ExecuteMsg::EndPoll(params) => end_poll(ctx, params),
    }
}

fn create_poll(ctx: &mut Context, params: CreatePollParams) -> PollResult<Response> {
    let poll = poll_engine::execute::create_poll(ctx, params)?;

    Ok(Response::new()
        .add_attribute("proposal_id", poll.id.to_string())
        .add_attribute("poll_id", poll.id.to_string())
        .add_attribute("action", "create_poll"))
}

fn cast_vote(ctx: &mut Context, params: CastVoteParams) -> PollResult<Response> {
    poll_engine::execute::cast_vote(ctx, params)?;

    Ok(Response::new().add_attribute("action", "cast_vote"))
}

fn update_votes(ctx: &mut Context, params: UpdateVotesParams) -> PollResult<Response> {
    let qctx = QueryContext {
        deps: ctx.deps.as_ref(),
        env: ctx.env.clone(),
    };
    let votes = query_voter(&qctx, &params.voter, None, None)?;

    for vote in votes.votes {
        let qctx = QueryContext::from(ctx.deps.as_ref(), ctx.env.clone());
        let status = query_poll_status(&qctx, vote.poll_id)?;
        if let PollStatus::InProgress { ends_at } = status.status {
            if ends_at > ctx.env.block.time {
                poll_engine::execute::cast_vote(
                    ctx,
                    CastVoteParams {
                        poll_id: vote.poll_id.into(),
                        outcome: VoteOutcome::from(vote.outcome),
                        voter: params.voter.to_string(),
                        amount: params.new_amount,
                    },
                )?;
            }
        }
    }

    Ok(Response::new().add_attribute("action", "update_votes"))
}

fn end_poll(ctx: &mut Context, params: EndPollParams) -> PollResult<Response> {
    poll_engine::execute::end_poll(ctx, params)?;

    Ok(Response::new().add_attribute("action", "end_poll"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> PollResult<Response> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> PollResult<Binary> {
    let qctx = QueryContext { deps, env };

    let response = match msg {
        QueryMsg::Poll(params) => to_json_binary(&query_poll(&qctx, params)?)?,
        QueryMsg::Polls(params) => to_json_binary(&query_polls(&qctx, params)?)?,
        QueryMsg::PollStatus { poll_id } => to_json_binary(&query_poll_status(&qctx, poll_id)?)?,
        QueryMsg::SimulateEndPollStatus {
            poll_id,
            maximum_available_votes,
        } => to_json_binary(&query_simulate_end_poll_status(
            &qctx,
            poll_id,
            maximum_available_votes,
        )?)?,
        QueryMsg::PollVoter(params) => to_json_binary(&query_poll_voter(&qctx, params)?)?,
        QueryMsg::PollVoters(params) => to_json_binary(&query_poll_voters(&qctx, params)?)?,
        QueryMsg::Voter(params) => to_json_binary(&query_voter(
            &qctx,
            params.voter_addr,
            params.start_after,
            params.limit,
        )?)?,
        QueryMsg::TotalVotes(params) => to_json_binary(&query_total_votes(&qctx, params)?)?,
        QueryMsg::VoterTotalVotes(params) => {
            to_json_binary(&query_voter_total_votes(&qctx, params)?)?
        }
    };
    Ok(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(mut deps: DepsMut, _env: Env, msg: MigrateMsg) -> PollResult<Response> {
    migrate_to_v1_0_0(deps.branch(), msg)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("action", "migrate"))
}
