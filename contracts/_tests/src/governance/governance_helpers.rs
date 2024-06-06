use crate::helpers::facade_helpers::facade;
use crate::traits::ImplApp;
use cosmwasm_std::{coins, Addr, Uint128};
use cw_asset::AssetUnchecked;
use cw_multi_test::{App, AppResponse, Executor};
use enterprise_facade_api::api::ProposalId;
use enterprise_governance_controller_api::api::ProposalAction::{
    DistributeFunds, UpdateMinimumWeightForRewards, UpdateNumberProposalsTracked,
};
use enterprise_governance_controller_api::api::{
    CastVoteMsg, CreateProposalMsg, DistributeFundsMsg, ExecuteProposalMsg, ProposalAction,
    ProposalParams, ProposalResponse, UpdateMinimumWeightForRewardsMsg,
    UpdateNumberProposalsTrackedMsg,
};
use enterprise_governance_controller_api::msg::ExecuteMsg::{
    CastVote, CreateProposal, ExecuteProposal,
};
use enterprise_treasury_api::api::ConfigResponse;
use funds_distributor_api::api::DistributionType;
use poll_engine_api::api::VoteOutcome;

pub fn create_proposal(
    app: &mut App,
    proposer: &str,
    dao: Addr,
    proposal_actions: Vec<ProposalAction>,
) -> anyhow::Result<()> {
    app.execute_contract(
        Addr::unchecked(proposer),
        facade(&app, dao).gov_controller_addr(),
        &CreateProposal(CreateProposalMsg {
            title: "sth".to_string(),
            description: None,
            proposal_actions,
            deposit_owner: None,
        }),
        &vec![],
    )?;
    Ok(())
}

pub fn cast_vote(
    app: &mut App,
    dao: Addr,
    voter: &str,
    proposal_id: ProposalId,
    outcome: VoteOutcome,
) -> anyhow::Result<AppResponse> {
    app.execute_contract(
        Addr::unchecked(voter),
        facade(&app, dao).gov_controller_addr(),
        &CastVote(CastVoteMsg {
            proposal_id,
            outcome,
        }),
        &vec![],
    )
}

pub fn execute_proposal(
    app: &mut App,
    executor: &str,
    dao: Addr,
    proposal_id: ProposalId,
) -> anyhow::Result<AppResponse> {
    app.execute_contract(
        Addr::unchecked(executor),
        facade(&app, dao).gov_controller_addr(),
        &ExecuteProposal(ExecuteProposalMsg { proposal_id }),
        &vec![],
    )
}

pub fn query_proposal(
    app: &App,
    dao: Addr,
    proposal_id: ProposalId,
) -> anyhow::Result<ProposalResponse> {
    // TODO: extract a helper for getting this contract
    let treasury_config: ConfigResponse = app.wrap().query_wasm_smart(
        dao.to_string(),
        &enterprise_treasury_api::msg::QueryMsg::Config {},
    )?;
    let gov_controller = treasury_config.admin;

    let response = app.wrap().query_wasm_smart(
        gov_controller.to_string(),
        &enterprise_governance_controller_api::msg::QueryMsg::Proposal(ProposalParams {
            proposal_id,
        }),
    )?;
    Ok(response)
}

// TODO: move to MembershipContract trait somehow
pub fn stake_denom(
    app: &mut App,
    staker: &str,
    dao: Addr,
    denom: &str,
    amount: u8,
) -> anyhow::Result<()> {
    app.mint_native(vec![(staker, coins(amount.into(), denom))]);
    app.execute_contract(
        Addr::unchecked(staker),
        facade(&app, dao).membership_addr(),
        &denom_staking_api::msg::ExecuteMsg::Stake { user: None },
        &coins(amount.into(), denom),
    )?;
    Ok(())
}

pub fn update_number_proposals_tracked(n: u8) -> ProposalAction {
    UpdateNumberProposalsTracked(UpdateNumberProposalsTrackedMsg {
        number_proposals_tracked: n,
    })
}

pub fn update_minimum_weight_for_rewards(minimum_weight_for_rewards: u8) -> ProposalAction {
    UpdateMinimumWeightForRewards(UpdateMinimumWeightForRewardsMsg {
        minimum_weight_for_rewards: Uint128::from(minimum_weight_for_rewards),
    })
}

pub fn distribute_native_funds_action(
    denom: &str,
    amount: u128,
    distribution_type: DistributionType,
) -> ProposalAction {
    DistributeFunds(DistributeFundsMsg {
        funds: vec![AssetUnchecked::native(denom, Uint128::from(amount))],
        distribution_type: Some(distribution_type),
    })
}
