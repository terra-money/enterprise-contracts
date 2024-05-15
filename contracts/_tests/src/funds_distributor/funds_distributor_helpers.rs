use crate::helpers::cw_multitest_helpers::USER1;
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
    UpdateMinimumWeightForRewardsMsg, UpdateNumberProposalsTrackedMsg,
};
use enterprise_governance_controller_api::msg::ExecuteMsg::{
    CastVote, CreateProposal, ExecuteProposal,
};
use funds_distributor_api::api::{ClaimRewardsMsg, DistributionType};
use funds_distributor_api::msg::ExecuteMsg::{ClaimRewards, DistributeNative};
use poll_engine_api::api::VoteOutcome;

// TODO: move to GovControllerContract trait somehow
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

// TODO: move to GovControllerContract trait somehow
pub fn cast_vote(
    app: &mut App,
    dao: Addr,
    voter: &str,
    proposal_id: ProposalId,
    outcome: VoteOutcome,
) -> anyhow::Result<()> {
    app.execute_contract(
        Addr::unchecked(voter),
        facade(&app, dao).gov_controller_addr(),
        &CastVote(CastVoteMsg {
            proposal_id,
            outcome,
        }),
        &vec![],
    )?;
    Ok(())
}

// TODO: move to GovControllerContract trait somehow
pub fn execute_proposal(
    app: &mut App,
    executor: &str,
    dao: Addr,
    proposal_id: ProposalId,
) -> anyhow::Result<()> {
    app.execute_contract(
        Addr::unchecked(executor),
        facade(&app, dao).gov_controller_addr(),
        &ExecuteProposal(ExecuteProposalMsg { proposal_id }),
        &vec![],
    )?;
    Ok(())
}

// TODO: move to gov controller helpers
pub fn update_number_proposals_tracked(n: u8) -> ProposalAction {
    UpdateNumberProposalsTracked(UpdateNumberProposalsTrackedMsg {
        number_proposals_tracked: n,
    })
}

// TODO: move to gov controller helpers
pub fn update_minimum_weight_for_rewards(minimum_weight_for_rewards: u8) -> ProposalAction {
    UpdateMinimumWeightForRewards(UpdateMinimumWeightForRewardsMsg {
        minimum_weight_for_rewards: Uint128::from(minimum_weight_for_rewards),
    })
}

// TODO: move to gov controller helpers
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

pub fn distribute_native_funds(
    app: &mut App,
    distributor: &str,
    denom: &str,
    amount: u128,
    distribution_type: DistributionType,
    dao: Addr,
) -> anyhow::Result<AppResponse> {
    app.mint_native(vec![(distributor, coins(amount, denom))]);
    app.execute_contract(
        Addr::unchecked(distributor),
        facade(&app, dao).funds_distributor().addr,
        &DistributeNative {
            distribution_type: Some(distribution_type),
        },
        &coins(amount, denom),
    )
}

pub fn claim_native_rewards(
    app: &mut App,
    user: &str,
    denom: &str,
    dao: Addr,
) -> anyhow::Result<AppResponse> {
    app.execute_contract(
        Addr::unchecked(user),
        facade(&app, dao).funds_distributor().addr,
        &ClaimRewards(ClaimRewardsMsg {
            user: USER1.to_string(),
            native_denoms: vec![denom.to_string()],
            cw20_assets: vec![],
        }),
        &vec![],
    )
}
