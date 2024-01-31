use crate::factory_helpers::{
    create_dao, default_create_dao_msg, default_dao_council, default_gov_config,
};
use crate::helpers::startup_with_versioning;
use cosmwasm_std::Decimal;
use enterprise_factory_api::api::CreateDaoMsg;
use enterprise_governance_controller_api::api::ProposalActionType::{
    AddAttestation, DistributeFunds, ExecuteEnterpriseMsgs, ExecuteMsgs, ExecuteTreasuryMsgs,
    ModifyMultisigMembership, RequestFundingFromDao, UpdateCouncil, UpdateGovConfig,
    UpdateMinimumWeightForRewards,
};
use enterprise_governance_controller_api::api::{DaoCouncilSpec, GovConfig, ProposalActionType};

#[test]
fn quorum_zero_not_allowed() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        gov_config: GovConfig {
            quorum: Decimal::zero(),
            ..default_gov_config()
        },
        ..default_create_dao_msg()
    };

    let result = create_dao(&mut app, msg);

    assert!(result.is_err());

    Ok(())
}

#[test]
fn quorum_over_one_not_allowed() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        gov_config: GovConfig {
            quorum: Decimal::percent(101),
            ..default_gov_config()
        },
        ..default_create_dao_msg()
    };

    let result = create_dao(&mut app, msg);

    assert!(result.is_err());

    Ok(())
}

#[test]
fn threshold_zero_not_allowed() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        gov_config: GovConfig {
            threshold: Decimal::zero(),
            ..default_gov_config()
        },
        ..default_create_dao_msg()
    };

    let result = create_dao(&mut app, msg);

    assert!(result.is_err());

    Ok(())
}

#[test]
fn threshold_over_one_not_allowed() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        gov_config: GovConfig {
            threshold: Decimal::percent(101),
            ..default_gov_config()
        },
        ..default_create_dao_msg()
    };

    let result = create_dao(&mut app, msg);

    assert!(result.is_err());

    Ok(())
}

#[test]
fn veto_threshold_zero_not_allowed() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        gov_config: GovConfig {
            veto_threshold: Some(Decimal::zero()),
            ..default_gov_config()
        },
        ..default_create_dao_msg()
    };

    let result = create_dao(&mut app, msg);

    assert!(result.is_err());

    Ok(())
}

#[test]
fn veto_threshold_over_one_not_allowed() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        gov_config: GovConfig {
            veto_threshold: Some(Decimal::percent(101)),
            ..default_gov_config()
        },
        ..default_create_dao_msg()
    };

    let result = create_dao(&mut app, msg);

    assert!(result.is_err());

    Ok(())
}

#[test]
fn council_quorum_zero_not_allowed() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        dao_council: Some(DaoCouncilSpec {
            quorum: Decimal::zero(),
            ..default_dao_council()
        }),
        ..default_create_dao_msg()
    };

    let result = create_dao(&mut app, msg);

    assert!(result.is_err());

    Ok(())
}

#[test]
fn council_quorum_over_one_not_allowed() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        dao_council: Some(DaoCouncilSpec {
            quorum: Decimal::percent(101),
            ..default_dao_council()
        }),
        ..default_create_dao_msg()
    };

    let result = create_dao(&mut app, msg);

    assert!(result.is_err());

    Ok(())
}

#[test]
fn council_threshold_zero_not_allowed() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        dao_council: Some(DaoCouncilSpec {
            threshold: Decimal::zero(),
            ..default_dao_council()
        }),
        ..default_create_dao_msg()
    };

    let result = create_dao(&mut app, msg);

    assert!(result.is_err());

    Ok(())
}

#[test]
fn council_threshold_over_one_not_allowed() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        dao_council: Some(DaoCouncilSpec {
            threshold: Decimal::percent(101),
            ..default_dao_council()
        }),
        ..default_create_dao_msg()
    };

    let result = create_dao(&mut app, msg);

    assert!(result.is_err());

    Ok(())
}

#[test]
fn dangerous_council_actions_not_allowed() -> anyhow::Result<()> {
    let dangerous_actions = vec![
        UpdateGovConfig,
        UpdateCouncil,
        RequestFundingFromDao,
        ExecuteMsgs,
        ExecuteTreasuryMsgs,
        ExecuteEnterpriseMsgs,
        ModifyMultisigMembership,
        DistributeFunds,
        UpdateMinimumWeightForRewards,
        AddAttestation,
    ];

    for action in dangerous_actions {
        assert_council_action_not_allowed(action)?;
    }

    Ok(())
}

fn assert_council_action_not_allowed(action: ProposalActionType) -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        dao_council: Some(DaoCouncilSpec {
            allowed_proposal_action_types: Some(vec![action.clone()]),
            ..default_dao_council()
        }),
        ..default_create_dao_msg()
    };

    let result = create_dao(&mut app, msg);

    assert!(
        result.is_err(),
        "{action} was allowed, when expected not allowed"
    );

    Ok(())
}
