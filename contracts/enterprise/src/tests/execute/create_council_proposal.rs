use crate::contract::{execute, instantiate, query_proposal, query_proposals};
use crate::proposals::ProposalType;
use crate::proposals::ProposalType::General;
use crate::tests::helpers::{
    existing_token_dao_membership, stub_dao_gov_config, stub_dao_metadata,
    stub_enterprise_factory_contract, stub_token_info, CW20_ADDR, ENTERPRISE_GOVERNANCE_CODE_ID,
};
use crate::tests::querier::mock_querier::mock_dependencies;
use common::cw::testing::{mock_env, mock_info, mock_query_ctx};
use cosmwasm_std::{to_binary, Timestamp, Uint128};
use enterprise_protocol::api::ProposalAction::UpgradeDao;
use enterprise_protocol::api::ProposalActionType::UpdateMetadata;
use enterprise_protocol::api::{
    CreateProposalMsg, DaoCouncilSpec, ProposalActionType, ProposalParams, ProposalsParams,
    UpgradeDaoMsg,
};
use enterprise_protocol::error::DaoError::{
    NoDaoCouncil, Unauthorized, UnsupportedCouncilProposalAction,
};
use enterprise_protocol::error::DaoResult;
use enterprise_protocol::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg};
use ProposalType::Council;

// TODO: replace instantiates with the function from helpers.rs

#[test]
fn create_council_proposal_with_no_council_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let current_time = Timestamp::from_seconds(12);
    env.block.time = current_time;
    let info = mock_info("sender", &[]);

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            enterprise_governance_code_id: ENTERPRISE_GOVERNANCE_CODE_ID,
            dao_metadata: stub_dao_metadata(),
            dao_gov_config: stub_dao_gov_config(),
            dao_council: None,
            dao_membership_info: existing_token_dao_membership(CW20_ADDR),
            enterprise_factory_contract: stub_enterprise_factory_contract(),
            asset_whitelist: None,
            nft_whitelist: None,
        },
    )?;

    let create_proposal_msg = CreateProposalMsg {
        title: "Proposal title".to_string(),
        description: Some("Description".to_string()),
        proposal_actions: vec![],
    };
    let result = execute(
        deps.as_mut(),
        env.clone(),
        mock_info("user", &vec![]),
        ExecuteMsg::CreateCouncilProposal(create_proposal_msg),
    );

    assert_eq!(result, Err(NoDaoCouncil));

    Ok(())
}

#[test]
fn create_council_proposal_by_non_council_member_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let current_time = Timestamp::from_seconds(12);
    env.block.time = current_time;
    let info = mock_info("sender", &[]);

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            enterprise_governance_code_id: ENTERPRISE_GOVERNANCE_CODE_ID,
            dao_metadata: stub_dao_metadata(),
            dao_gov_config: stub_dao_gov_config(),
            dao_council: Some(DaoCouncilSpec {
                members: vec!["council_member".to_string()],
                allowed_proposal_action_types: None,
            }),
            dao_membership_info: existing_token_dao_membership(CW20_ADDR),
            enterprise_factory_contract: stub_enterprise_factory_contract(),
            asset_whitelist: None,
            nft_whitelist: None,
        },
    )?;

    let create_proposal_msg = CreateProposalMsg {
        title: "Proposal title".to_string(),
        description: Some("Description".to_string()),
        proposal_actions: vec![],
    };
    let result = execute(
        deps.as_mut(),
        env.clone(),
        mock_info("non_council_member", &vec![]),
        ExecuteMsg::CreateCouncilProposal(create_proposal_msg),
    );

    assert_eq!(result, Err(Unauthorized));

    Ok(())
}

#[test]
fn create_council_proposal_allows_upgrade_dao_by_default() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let current_time = Timestamp::from_seconds(12);
    env.block.time = current_time;
    let info = mock_info("sender", &[]);

    let enterprise_factory_contract = "enterprise_factory_contract";

    deps.querier
        .with_enterprise_code_ids(&[(enterprise_factory_contract, &[10u64])]);

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            enterprise_governance_code_id: ENTERPRISE_GOVERNANCE_CODE_ID,
            dao_metadata: stub_dao_metadata(),
            dao_gov_config: stub_dao_gov_config(),
            dao_council: Some(DaoCouncilSpec {
                members: vec!["council_member".to_string()],
                allowed_proposal_action_types: None,
            }),
            dao_membership_info: existing_token_dao_membership(CW20_ADDR),
            enterprise_factory_contract: enterprise_factory_contract.to_string(),
            asset_whitelist: None,
            nft_whitelist: None,
        },
    )?;

    let create_proposal_msg = CreateProposalMsg {
        title: "Proposal title".to_string(),
        description: Some("Description".to_string()),
        proposal_actions: vec![UpgradeDao(UpgradeDaoMsg {
            new_dao_code_id: 10,
            migrate_msg: to_binary(&MigrateMsg {})?,
        })],
    };
    execute(
        deps.as_mut(),
        env.clone(),
        mock_info("council_member", &vec![]),
        ExecuteMsg::CreateCouncilProposal(create_proposal_msg),
    )?;

    Ok(())
}

#[test]
fn create_council_proposal_with_not_allowed_proposal_action_type_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let current_time = Timestamp::from_seconds(12);
    env.block.time = current_time;
    let info = mock_info("sender", &[]);

    let enterprise_factory_contract = "enterprise_factory_contract";

    deps.querier
        .with_enterprise_code_ids(&[(enterprise_factory_contract, &[10u64])]);

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            enterprise_governance_code_id: ENTERPRISE_GOVERNANCE_CODE_ID,
            dao_metadata: stub_dao_metadata(),
            dao_gov_config: stub_dao_gov_config(),
            dao_council: Some(DaoCouncilSpec {
                members: vec!["council_member".to_string()],
                allowed_proposal_action_types: Some(vec![UpdateMetadata]),
            }),
            dao_membership_info: existing_token_dao_membership(CW20_ADDR),
            enterprise_factory_contract: enterprise_factory_contract.to_string(),
            asset_whitelist: None,
            nft_whitelist: None,
        },
    )?;

    let create_proposal_msg = CreateProposalMsg {
        title: "Proposal title".to_string(),
        description: Some("Description".to_string()),
        proposal_actions: vec![UpgradeDao(UpgradeDaoMsg {
            new_dao_code_id: 10,
            migrate_msg: to_binary(&MigrateMsg {})?,
        })],
    };
    let result = execute(
        deps.as_mut(),
        env.clone(),
        mock_info("council_member", &vec![]),
        ExecuteMsg::CreateCouncilProposal(create_proposal_msg),
    );

    assert_eq!(
        result,
        Err(UnsupportedCouncilProposalAction {
            action: ProposalActionType::UpgradeDao
        })
    );

    Ok(())
}

#[test]
fn create_council_proposal_shows_up_in_query() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let current_time = Timestamp::from_seconds(12);
    env.block.time = current_time;
    let info = mock_info("sender", &[]);

    let enterprise_factory_contract = "enterprise_factory_contract";

    deps.querier
        .with_enterprise_code_ids(&[(enterprise_factory_contract, &[10u64])]);

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            enterprise_governance_code_id: ENTERPRISE_GOVERNANCE_CODE_ID,
            dao_metadata: stub_dao_metadata(),
            dao_gov_config: stub_dao_gov_config(),
            dao_council: Some(DaoCouncilSpec {
                members: vec![
                    "council_member1".to_string(),
                    "council_member2".to_string(),
                    "council_member3".to_string(),
                ],
                allowed_proposal_action_types: None,
            }),
            dao_membership_info: existing_token_dao_membership(CW20_ADDR),
            enterprise_factory_contract: enterprise_factory_contract.to_string(),
            asset_whitelist: None,
            nft_whitelist: None,
        },
    )?;

    let create_proposal_msg = CreateProposalMsg {
        title: "Proposal title".to_string(),
        description: Some("Description".to_string()),
        proposal_actions: vec![UpgradeDao(UpgradeDaoMsg {
            new_dao_code_id: 10,
            migrate_msg: to_binary(&MigrateMsg {})?,
        })],
    };
    execute(
        deps.as_mut(),
        env.clone(),
        mock_info("council_member1", &vec![]),
        ExecuteMsg::CreateCouncilProposal(create_proposal_msg),
    )?;

    let proposal = query_proposal(
        mock_query_ctx(deps.as_ref(), &env),
        ProposalParams { proposal_id: 1 },
        Council,
    )?;

    assert_eq!(proposal.proposal.id, 1u64);
    assert_eq!(proposal.total_votes_available, Uint128::from(3u8));

    assert!(query_proposals(
        mock_query_ctx(deps.as_ref(), &env),
        ProposalsParams {
            filter: None,
            start_after: None,
            limit: None,
        },
        General,
    )?
    .proposals
    .is_empty());

    Ok(())
}
