use crate::facade_v1::EnterpriseFacadeV1;
use crate::msg::InstantiateMsg;
use crate::state::ENTERPRISE_VERSIONING;
use common::cw::{Context, QueryContext};
use cosmwasm_std::{
    entry_point, to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
};
use cw2::set_contract_version;
use enterprise_facade_api::error::EnterpriseFacadeResult;
use enterprise_facade_api::msg::{ExecuteMsg, QueryMsg};
use enterprise_facade_common::facade::EnterpriseFacade;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:enterprise-facade-v1";
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
    _msg: ExecuteMsg,
) -> EnterpriseFacadeResult<Response> {
    let _ctx = &mut Context { deps, env, info };

    Ok(Response::new())
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
            let facade = get_facade(contract, qctx)?;
            to_json_binary(&facade.query_treasury_address()?)?
        }
        QueryMsg::DaoInfo { contract } => {
            let facade = get_facade(contract, qctx)?;
            to_json_binary(&facade.query_dao_info()?)?
        }
        QueryMsg::ComponentContracts { contract } => {
            let facade = get_facade(contract, qctx)?;
            to_json_binary(&facade.query_component_contracts()?)?
        }
        QueryMsg::MemberInfo { contract, msg } => {
            let facade = get_facade(contract, qctx)?;
            to_json_binary(&facade.query_member_info(msg)?)?
        }
        QueryMsg::Members { contract, msg } => {
            let facade = get_facade(contract, qctx)?;
            to_json_binary(&facade.query_members(msg)?)?
        }
        QueryMsg::ListMultisigMembers { contract, msg } => {
            let facade = get_facade(contract, qctx)?;
            to_json_binary(&facade.query_list_multisig_members(msg)?)?
        }
        QueryMsg::AssetWhitelist { contract, params } => {
            let facade = get_facade(contract, qctx)?;
            to_json_binary(&facade.query_asset_whitelist(params)?)?
        }
        QueryMsg::NftWhitelist { contract, params } => {
            let facade = get_facade(contract, qctx)?;
            to_json_binary(&facade.query_nft_whitelist(params)?)?
        }
        QueryMsg::NumberProposalsTracked { contract } => {
            let facade = get_facade(contract, qctx)?;
            to_json_binary(&facade.query_number_proposals_tracked()?)?
        }
        QueryMsg::Proposal { contract, params } => {
            let facade = get_facade(contract, qctx)?;
            to_json_binary(&facade.query_proposal(params)?)?
        }
        QueryMsg::Proposals { contract, params } => {
            let facade = get_facade(contract, qctx)?;
            to_json_binary(&facade.query_proposals(params)?)?
        }
        QueryMsg::ProposalStatus { contract, params } => {
            let facade = get_facade(contract, qctx)?;
            to_json_binary(&facade.query_proposal_status(params)?)?
        }
        QueryMsg::MemberVote { contract, params } => {
            let facade = get_facade(contract, qctx)?;
            to_json_binary(&facade.query_member_vote(params)?)?
        }
        QueryMsg::ProposalVotes { contract, params } => {
            let facade = get_facade(contract, qctx)?;
            to_json_binary(&facade.query_proposal_votes(params)?)?
        }
        QueryMsg::UserStake { contract, params } => {
            let facade = get_facade(contract, qctx)?;
            to_json_binary(&facade.query_user_stake(params)?)?
        }
        QueryMsg::TotalStakedAmount { contract } => {
            let facade = get_facade(contract, qctx)?;
            to_json_binary(&facade.query_total_staked_amount()?)?
        }
        QueryMsg::StakedNfts { contract, params } => {
            let facade = get_facade(contract, qctx)?;
            to_json_binary(&facade.query_staked_nfts(params)?)?
        }
        QueryMsg::Claims { contract, params } => {
            let facade = get_facade(contract, qctx)?;
            to_json_binary(&facade.query_claims(params)?)?
        }
        QueryMsg::ReleasableClaims { contract, params } => {
            let facade = get_facade(contract, qctx)?;
            to_json_binary(&facade.query_releasable_claims(params)?)?
        }
        QueryMsg::CrossChainTreasuries { contract, params } => {
            let facade = get_facade(contract, qctx)?;
            to_json_binary(&facade.query_cross_chain_treasuries(params)?)?
        }
        QueryMsg::HasIncompleteV2Migration { contract } => {
            let facade = get_facade(contract, qctx)?;
            to_json_binary(&facade.query_has_incomplete_v2_migration()?)?
        }
        QueryMsg::HasUnmovedStakesOrClaims { contract } => {
            let facade = get_facade(contract, qctx)?;
            to_json_binary(&facade.query_has_unmoved_stakes_or_claims()?)?
        }
        QueryMsg::V2MigrationStage { contract } => {
            let facade = get_facade(contract, qctx)?;
            to_json_binary(&facade.query_v2_migration_stage()?)?
        }
        QueryMsg::CreateProposalAdapted { contract, params } => {
            let facade = get_facade(contract, qctx)?;
            to_json_binary(&facade.adapt_create_proposal(params)?)?
        }
        QueryMsg::CreateProposalWithDenomDepositAdapted { contract, params } => {
            let facade = get_facade(contract, qctx)?;
            to_json_binary(&facade.adapt_create_proposal_with_denom_deposit(params)?)?
        }
        QueryMsg::CreateProposalWithTokenDepositAdapted { contract, params } => {
            let facade = get_facade(contract, qctx)?;
            to_json_binary(&facade.adapt_create_proposal_with_token_deposit(params)?)?
        }
        QueryMsg::CreateProposalWithNftDepositAdapted { contract, params } => {
            let facade = get_facade(contract, qctx)?;
            to_json_binary(&facade.adapt_create_proposal_with_nft_deposit(params)?)?
        }
        QueryMsg::CreateCouncilProposalAdapted { contract, params } => {
            let facade = get_facade(contract, qctx)?;
            to_json_binary(&facade.adapt_create_council_proposal(params)?)?
        }
        QueryMsg::CastVoteAdapted { contract, params } => {
            let facade = get_facade(contract, qctx)?;
            to_json_binary(&facade.adapt_cast_vote(params)?)?
        }
        QueryMsg::CastCouncilVoteAdapted { contract, params } => {
            let facade = get_facade(contract, qctx)?;
            to_json_binary(&facade.adapt_cast_council_vote(params)?)?
        }
        QueryMsg::ExecuteProposalAdapted { contract, params } => {
            let facade = get_facade(contract, qctx)?;
            to_json_binary(&facade.adapt_execute_proposal(params)?)?
        }
        QueryMsg::StakeAdapted { contract, params } => {
            let facade = get_facade(contract, qctx)?;
            to_json_binary(&facade.adapt_stake(params)?)?
        }
        QueryMsg::UnstakeAdapted { contract, params } => {
            let facade = get_facade(contract, qctx)?;
            to_json_binary(&facade.adapt_unstake(params)?)?
        }
        QueryMsg::ClaimAdapted { contract } => {
            let facade = get_facade(contract, qctx)?;
            to_json_binary(&facade.adapt_claim()?)?
        }
    };
    Ok(response)
}

fn get_facade(address: Addr, qctx: QueryContext) -> EnterpriseFacadeResult<EnterpriseFacadeV1> {
    Ok(EnterpriseFacadeV1 {
        enterprise_address: address,
        qctx,
    })
}
