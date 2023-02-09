use crate::contract::{execute, instantiate, query_proposal};
use crate::proposals::ProposalType::Council;
use crate::tests::helpers::{
    create_council_proposal, existing_token_dao_membership, instantiate_stub_dao,
    stub_dao_metadata, stub_enterprise_factory_contract, stub_token_info, vote_on_council_proposal,
    CW20_ADDR, DAO_ADDR, ENTERPRISE_GOVERNANCE_CODE_ID, FUNDS_DISTRIBUTOR_CODE_ID,
};
use crate::tests::querier::mock_querier::mock_dependencies;
use common::cw::testing::{mock_env, mock_info, mock_query_ctx};
use cosmwasm_std::{to_binary, Addr, Attribute, Decimal, SubMsg, Timestamp, WasmMsg};
use cw_utils::Duration;
use enterprise_protocol::api::ProposalAction::UpgradeDao;
use enterprise_protocol::api::{
    DaoCouncilSpec, DaoGovConfig, ExecuteProposalMsg, ProposalParams, UpgradeDaoMsg,
};
use enterprise_protocol::error::DaoResult;
use enterprise_protocol::msg::ExecuteMsg::ExecuteCouncilProposal;
use enterprise_protocol::msg::{InstantiateMsg, MigrateMsg};
use poll_engine_api::api::VoteOutcome::Yes;

#[test]
fn execute_proposal_with_outcome_yes_and_ended_executes_proposal_actions() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let info = mock_info("sender", &[]);

    env.contract.address = Addr::unchecked(DAO_ADDR);
    env.block.time = Timestamp::from_seconds(12000);
    let dao_gov_config = DaoGovConfig {
        vote_duration: 1000,
        quorum: Decimal::from_ratio(1u8, 10u8),
        threshold: Decimal::from_ratio(2u8, 10u8),
        unlocking_period: Duration::Time(1000),
        minimum_deposit: None,
        veto_threshold: None,
        allow_early_proposal_execution: false,
    };

    let enterprise_factory_contract = stub_enterprise_factory_contract();
    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);
    deps.querier
        .with_enterprise_code_ids(&[(&enterprise_factory_contract, &[7u64])]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            enterprise_governance_code_id: ENTERPRISE_GOVERNANCE_CODE_ID,
            funds_distributor_code_id: FUNDS_DISTRIBUTOR_CODE_ID,
            dao_metadata: stub_dao_metadata(),
            dao_gov_config: dao_gov_config.clone(),
            dao_council: Some(DaoCouncilSpec {
                members: vec!["council_member1".to_string(), "council_member2".to_string()],
                quorum: Decimal::percent(75),
                threshold: Decimal::percent(50),
                allowed_proposal_action_types: None,
            }),
            dao_membership_info: existing_token_dao_membership(CW20_ADDR),
            enterprise_factory_contract,
            asset_whitelist: None,
            nft_whitelist: None,
        },
    )?;

    let migrate_msg = to_binary(&MigrateMsg {})?;

    let proposal_actions = vec![UpgradeDao(UpgradeDaoMsg {
        new_dao_code_id: 7,
        migrate_msg: migrate_msg.clone(),
    })];

    let response = create_council_proposal(
        deps.as_mut(),
        &env,
        &mock_info("council_member1", &vec![]),
        None,
        None,
        proposal_actions.clone(),
    )?;

    assert_eq!(
        response.attributes,
        vec![
            Attribute::new("action", "create_council_proposal"),
            Attribute::new("dao_address", DAO_ADDR),
            Attribute::new("proposal_id", "1"),
        ]
    );

    vote_on_council_proposal(deps.as_mut(), &env, "council_member1", 1, Yes)?;
    vote_on_council_proposal(deps.as_mut(), &env, "council_member2", 1, Yes)?;

    env.block.time = env.block.time.plus_seconds(1000);

    let response = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteCouncilProposal(ExecuteProposalMsg { proposal_id: 1 }),
    )?;

    assert_eq!(
        response.messages,
        vec![SubMsg::new(WasmMsg::Migrate {
            contract_addr: DAO_ADDR.to_string(),
            new_code_id: 7,
            msg: migrate_msg,
        }),]
    );

    // ensure proposal actions were not removed after execution
    let proposal = query_proposal(
        mock_query_ctx(deps.as_ref(), &env),
        ProposalParams { proposal_id: 1 },
        Council,
    )?;
    assert_eq!(proposal.proposal.proposal_actions, proposal_actions);

    Ok(())
}
