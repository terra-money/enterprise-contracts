use crate::facade::get_facade;
use crate::state::ENTERPRISE_VERSIONING;
use common::cw::{Context, QueryContext};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
};
use cw2::set_contract_version;
use enterprise_facade_api::error::EnterpriseFacadeResult;
use enterprise_facade_api::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:enterprise-facade";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> EnterpriseFacadeResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    ENTERPRISE_VERSIONING.save(
        deps.storage,
        &deps.api.addr_validate(&msg.enterprise_versioning)?,
    )?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> EnterpriseFacadeResult<Response> {
    let ctx = &mut Context { deps, env, info };

    match msg {
        ExecuteMsg::ExecuteProposal { contract, msg } => {
            let facade = get_facade(ctx.deps.as_ref(), contract)?;

            facade.execute_proposal(ctx, msg)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> EnterpriseFacadeResult<Response> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> EnterpriseFacadeResult<Binary> {
    let qctx = QueryContext { deps, env };

    let response = match msg {
        QueryMsg::TreasuryAddress { contract } => {
            let facade = get_facade(deps, contract)?;
            to_binary(&facade.query_treasury_address(qctx)?)?
        }
        QueryMsg::DaoInfo { contract } => {
            let facade = get_facade(deps, contract)?;
            to_binary(&facade.query_dao_info(qctx)?)?
        }
        QueryMsg::MemberInfo { contract, msg } => {
            let facade = get_facade(deps, contract)?;
            to_binary(&facade.query_member_info(qctx, msg)?)?
        }
        QueryMsg::ListMultisigMembers { contract, msg } => {
            let facade = get_facade(deps, contract)?;
            to_binary(&facade.query_list_multisig_members(qctx, msg)?)?
        }
        QueryMsg::AssetWhitelist { contract, params } => {
            let facade = get_facade(deps, contract)?;
            to_binary(&facade.query_asset_whitelist(qctx, params)?)?
        }
        QueryMsg::NftWhitelist { contract, params } => {
            let facade = get_facade(deps, contract)?;
            to_binary(&facade.query_nft_whitelist(qctx, params)?)?
        }
        QueryMsg::Proposal { contract, params } => {
            let facade = get_facade(deps, contract)?;
            to_binary(&facade.query_proposal(qctx, params)?)?
        }
        QueryMsg::Proposals { contract, params } => {
            let facade = get_facade(deps, contract)?;
            to_binary(&facade.query_proposals(qctx, params)?)?
        }
        QueryMsg::ProposalStatus { contract, params } => {
            let facade = get_facade(deps, contract)?;
            to_binary(&facade.query_proposal_status(qctx, params)?)?
        }
        QueryMsg::MemberVote { contract, params } => {
            let facade = get_facade(deps, contract)?;
            to_binary(&facade.query_member_vote(qctx, params)?)?
        }
        QueryMsg::ProposalVotes { contract, params } => {
            let facade = get_facade(deps, contract)?;
            to_binary(&facade.query_proposal_votes(qctx, params)?)?
        }
        QueryMsg::UserStake { contract, params } => {
            let facade = get_facade(deps, contract)?;
            to_binary(&facade.query_user_stake(qctx, params)?)?
        }
        QueryMsg::TotalStakedAmount { contract } => {
            let facade = get_facade(deps, contract)?;
            to_binary(&facade.query_total_staked_amount(qctx)?)?
        }
        QueryMsg::StakedNfts { contract, params } => {
            let facade = get_facade(deps, contract)?;
            to_binary(&facade.query_staked_nfts(qctx, params)?)?
        }
        QueryMsg::Claims { contract, params } => {
            let facade = get_facade(deps, contract)?;
            to_binary(&facade.query_claims(qctx, params)?)?
        }
        QueryMsg::ReleasableClaims { contract, params } => {
            let facade = get_facade(deps, contract)?;
            to_binary(&facade.query_releasable_claims(qctx, params)?)?
        }
        QueryMsg::CrossChainTreasuries { contract, params } => {
            let facade = get_facade(deps, contract)?;
            to_binary(&facade.query_cross_chain_treasuries(qctx, params)?)?
        }
        QueryMsg::CreateProposalAdapted { contract, params } => {
            let facade = get_facade(deps, contract)?;
            to_binary(&facade.adapt_create_proposal(qctx, params)?)?
        }
        QueryMsg::CreateProposalWithDenomDepositAdapted { contract, params } => {
            let facade = get_facade(deps, contract)?;
            to_binary(&facade.adapt_create_proposal_with_denom_deposit(qctx, params)?)?
        }
        QueryMsg::CreateProposalWithTokenDepositAdapted { contract, params } => {
            let facade = get_facade(deps, contract)?;
            to_binary(&facade.adapt_create_proposal_with_token_deposit(qctx, params)?)?
        }
        QueryMsg::CreateProposalWithNftDepositAdapted { contract, params } => {
            let facade = get_facade(deps, contract)?;
            to_binary(&facade.adapt_create_proposal_with_nft_deposit(qctx, params)?)?
        }
        QueryMsg::CreateCouncilProposalAdapted { contract, params } => {
            let facade = get_facade(deps, contract)?;
            to_binary(&facade.adapt_create_council_proposal(qctx, params)?)?
        }
        QueryMsg::CastVoteAdapted { contract, params } => {
            let facade = get_facade(deps, contract)?;
            to_binary(&facade.adapt_cast_vote(qctx, params)?)?
        }
        QueryMsg::CastCouncilVoteAdapted { contract, params } => {
            let facade = get_facade(deps, contract)?;
            to_binary(&facade.adapt_cast_council_vote(qctx, params)?)?
        }
        QueryMsg::StakeAdapted { contract, params } => {
            let facade = get_facade(deps, contract)?;
            to_binary(&facade.adapt_stake(qctx, params)?)?
        }
        QueryMsg::UnstakeAdapted { contract, params } => {
            let facade = get_facade(deps, contract)?;
            to_binary(&facade.adapt_unstake(qctx, params)?)?
        }
        QueryMsg::ClaimAdapted { contract } => {
            let facade = get_facade(deps, contract)?;
            to_binary(&facade.adapt_claim(qctx)?)?
        }
    };
    Ok(response)
}