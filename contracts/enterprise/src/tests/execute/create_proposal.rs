use crate::contract::{execute, instantiate, query_proposals};
use crate::proposals::ProposalType::General;
use crate::tests::helpers::{
    create_proposal, create_stub_proposal, existing_nft_dao_membership,
    existing_token_dao_membership, instantiate_stub_dao, multisig_dao_membership_info_with_members,
    stake_nfts, stub_dao_gov_config, stub_dao_metadata, stub_token_info, CW20_ADDR, NFT_ADDR,
};
use crate::tests::querier::mock_querier::mock_dependencies;
use common::cw::testing::{mock_env, mock_info, mock_query_ctx};
use cosmwasm_std::{to_binary, Addr, Attribute, Timestamp, Uint128, Uint64};
use cw20::Cw20ReceiveMsg;
use cw_asset::AssetInfo;
use cw_utils::Expiration;
use enterprise_protocol::api::ModifyValue::{Change, NoChange};
use enterprise_protocol::api::ProposalAction::{
    ExecuteMsgs, ModifyMultisigMembership, UpdateAssetWhitelist, UpdateNftWhitelist, UpgradeDao,
};
use enterprise_protocol::api::ProposalActionType::UpdateCouncil;
use enterprise_protocol::api::{
    CreateProposalMsg, DaoCouncil, DaoGovConfig, ExecuteMsgsMsg, ModifyMultisigMembershipMsg,
    Proposal, ProposalAction, ProposalResponse, ProposalStatus, ProposalsParams,
    UpdateAssetWhitelistMsg, UpdateCouncilMsg, UpdateGovConfigMsg, UpdateNftWhitelistMsg,
    UpgradeDaoMsg,
};
use enterprise_protocol::error::DaoError::{
    InsufficientProposalDeposit, InvalidEnterpriseCodeId, NotNftOwner,
    UnsupportedCouncilProposalAction, UnsupportedOperationForDaoType,
};
use enterprise_protocol::error::{DaoError, DaoResult};
use enterprise_protocol::msg::ExecuteMsg::Receive;
use enterprise_protocol::msg::{Cw20HookMsg, InstantiateMsg};
use DaoError::{InvalidCosmosMessage, NotMultisigMember};
use ProposalAction::UpdateGovConfig;

#[test]
fn create_proposal_token_dao() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    env.contract.address = Addr::unchecked("dao_addr");
    let current_time = Timestamp::from_seconds(12);
    env.block.time = current_time;
    let info = mock_info("sender", &[]);

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        existing_token_dao_membership(CW20_ADDR),
        None,
        None,
    )?;

    let proposal_actions = vec![UpdateGovConfig(UpdateGovConfigMsg {
        quorum: NoChange,
        threshold: NoChange,
        veto_threshold: NoChange,
        voting_duration: Change(Uint64::from(20u8)),
        unlocking_period: NoChange,
        minimum_deposit: NoChange,
    })];

    let response = create_proposal(
        deps.as_mut(),
        &env,
        &info,
        Some("Proposal title"),
        Some("Description"),
        proposal_actions.clone(),
    )?;

    assert_eq!(
        response.attributes,
        vec![
            Attribute::new("action", "create_proposal"),
            Attribute::new("dao_address", "dao_addr"),
            Attribute::new("proposal_id", "1"),
        ]
    );

    let proposals = query_proposals(
        mock_query_ctx(deps.as_ref(), &env),
        ProposalsParams {
            filter: None,
            start_after: None,
            limit: None,
        },
        General,
    )?;

    assert_eq!(
        proposals.proposals,
        vec![ProposalResponse {
            proposal: Proposal {
                id: 1,
                title: "Proposal title".to_string(),
                description: "Description".to_string(),
                status: ProposalStatus::InProgress,
                started_at: current_time,
                expires: Expiration::AtTime(
                    env.block
                        .time
                        .plus_seconds(stub_dao_gov_config().vote_duration)
                ),
                proposal_actions
            },
            results: Default::default(),
            total_votes_available: Default::default(),
        }]
    );

    Ok(())
}

#[test]
fn create_proposal_nft_dao() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    env.contract.address = Addr::unchecked("dao_addr");
    let current_time = Timestamp::from_seconds(12);
    env.block.time = current_time;
    let info = mock_info("sender", &[]);

    deps.querier.with_num_tokens(&[(NFT_ADDR, 1000u64)]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        existing_nft_dao_membership(NFT_ADDR),
        None,
        None,
    )?;

    let proposal_actions = vec![UpdateGovConfig(UpdateGovConfigMsg {
        quorum: NoChange,
        threshold: NoChange,
        veto_threshold: NoChange,
        voting_duration: Change(Uint64::from(20u8)),
        unlocking_period: NoChange,
        minimum_deposit: NoChange,
    })];

    stake_nfts(&mut deps.as_mut(), &env, NFT_ADDR, "user", vec!["token1"])?;

    let response = create_proposal(
        deps.as_mut(),
        &env,
        &mock_info("user", &vec![]),
        Some("Proposal title"),
        Some("Description"),
        proposal_actions.clone(),
    )?;

    assert_eq!(
        response.attributes,
        vec![
            Attribute::new("action", "create_proposal"),
            Attribute::new("dao_address", "dao_addr"),
            Attribute::new("proposal_id", "1"),
        ]
    );

    let proposals = query_proposals(
        mock_query_ctx(deps.as_ref(), &env),
        ProposalsParams {
            filter: None,
            start_after: None,
            limit: None,
        },
        General,
    )?;

    assert_eq!(
        proposals.proposals,
        vec![ProposalResponse {
            proposal: Proposal {
                id: 1,
                title: "Proposal title".to_string(),
                description: "Description".to_string(),
                status: ProposalStatus::InProgress,
                started_at: current_time,
                expires: Expiration::AtTime(
                    env.block
                        .time
                        .plus_seconds(stub_dao_gov_config().vote_duration)
                ),
                proposal_actions
            },
            results: Default::default(),
            total_votes_available: Uint128::one(),
        }]
    );

    Ok(())
}

#[test]
fn create_proposal_with_no_token_deposit_when_minimum_deposit_is_specified_fails() -> DaoResult<()>
{
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    env.contract.address = Addr::unchecked("dao_addr");
    let current_time = Timestamp::from_seconds(12);
    env.block.time = current_time;
    let info = mock_info("sender", &[]);

    let dao_gov_config = DaoGovConfig {
        minimum_deposit: Some(1u128.into()),
        ..stub_dao_gov_config()
    };

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        existing_token_dao_membership(CW20_ADDR),
        Some(dao_gov_config.clone()),
        None,
    )?;

    let result = create_stub_proposal(deps.as_mut(), &env, &info);

    assert_eq!(
        result,
        Err(InsufficientProposalDeposit {
            required_amount: 1u128.into()
        })
    );

    Ok(())
}

#[test]
fn create_proposal_with_insufficient_token_deposit_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    env.contract.address = Addr::unchecked("dao_addr");
    let current_time = Timestamp::from_seconds(12);
    env.block.time = current_time;
    let info = mock_info("sender", &[]);

    let dao_gov_config = DaoGovConfig {
        minimum_deposit: Some(2u128.into()),
        ..stub_dao_gov_config()
    };

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        existing_token_dao_membership(CW20_ADDR),
        Some(dao_gov_config.clone()),
        None,
    )?;

    let create_proposal_msg = CreateProposalMsg {
        title: "Proposal title".to_string(),
        description: Some("Description".to_string()),
        proposal_actions: vec![],
    };
    let result = execute(
        deps.as_mut(),
        env.clone(),
        mock_info(CW20_ADDR, &vec![]),
        Receive(Cw20ReceiveMsg {
            sender: "user".to_string(),
            amount: 1u128.into(),
            msg: to_binary(&Cw20HookMsg::CreateProposal(create_proposal_msg))?,
        }),
    );

    assert_eq!(
        result,
        Err(InsufficientProposalDeposit {
            required_amount: 2u128.into()
        })
    );

    Ok(())
}

#[test]
fn create_proposal_with_sufficient_token_deposit_succeeds() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    env.contract.address = Addr::unchecked("dao_addr");
    let current_time = Timestamp::from_seconds(12);
    env.block.time = current_time;
    let info = mock_info("sender", &[]);

    let dao_gov_config = DaoGovConfig {
        minimum_deposit: Some(2u128.into()),
        ..stub_dao_gov_config()
    };

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        existing_token_dao_membership(CW20_ADDR),
        Some(dao_gov_config.clone()),
        None,
    )?;

    let create_proposal_msg = CreateProposalMsg {
        title: "Proposal title".to_string(),
        description: Some("Description".to_string()),
        proposal_actions: vec![],
    };
    let result = execute(
        deps.as_mut(),
        env.clone(),
        mock_info(CW20_ADDR, &vec![]),
        Receive(Cw20ReceiveMsg {
            sender: "user".to_string(),
            amount: 3u128.into(),
            msg: to_binary(&Cw20HookMsg::CreateProposal(create_proposal_msg))?,
        }),
    );

    assert!(result.is_ok());

    Ok(())
}

#[test]
fn create_proposal_with_duplicate_add_whitelist_assets_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    env.contract.address = Addr::unchecked("dao_addr");
    let current_time = Timestamp::from_seconds(12);
    env.block.time = current_time;
    let info = mock_info("sender", &[]);

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        existing_token_dao_membership(CW20_ADDR),
        None,
        None,
    )?;

    let result = create_proposal(
        deps.as_mut(),
        &env,
        &info,
        None,
        None,
        vec![UpdateAssetWhitelist(UpdateAssetWhitelistMsg {
            add: vec![
                AssetInfo::cw20(Addr::unchecked("token")),
                AssetInfo::cw20(Addr::unchecked("token")),
            ],
            remove: vec![],
        })],
    );

    assert_eq!(result, Err(DaoError::DuplicateAssetFound));

    Ok(())
}

#[test]
fn create_proposal_with_duplicate_remove_whitelist_assets_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    env.contract.address = Addr::unchecked("dao_addr");
    let current_time = Timestamp::from_seconds(12);
    env.block.time = current_time;
    let info = mock_info("sender", &[]);

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        existing_token_dao_membership(CW20_ADDR),
        None,
        None,
    )?;

    let result = create_proposal(
        deps.as_mut(),
        &env,
        &info,
        None,
        None,
        vec![UpdateAssetWhitelist(UpdateAssetWhitelistMsg {
            add: vec![],
            remove: vec![
                AssetInfo::cw20(Addr::unchecked("token")),
                AssetInfo::cw20(Addr::unchecked("token")),
            ],
        })],
    );

    assert_eq!(result, Err(DaoError::DuplicateAssetFound));

    Ok(())
}

#[test]
fn create_proposal_with_duplicate_add_whitelist_nft_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    env.contract.address = Addr::unchecked("dao_addr");
    let current_time = Timestamp::from_seconds(12);
    env.block.time = current_time;
    let info = mock_info("sender", &[]);

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        existing_token_dao_membership(CW20_ADDR),
        None,
        None,
    )?;

    let result = create_proposal(
        deps.as_mut(),
        &env,
        &info,
        None,
        None,
        vec![UpdateNftWhitelist(UpdateNftWhitelistMsg {
            add: vec![Addr::unchecked("nft"), Addr::unchecked("nft")],
            remove: vec![],
        })],
    );

    assert_eq!(result, Err(DaoError::DuplicateNftFound));

    Ok(())
}

#[test]
fn create_proposal_with_duplicate_remove_whitelist_nft_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    env.contract.address = Addr::unchecked("dao_addr");
    let current_time = Timestamp::from_seconds(12);
    env.block.time = current_time;
    let info = mock_info("sender", &[]);

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        existing_token_dao_membership(CW20_ADDR),
        None,
        None,
    )?;

    let result = create_proposal(
        deps.as_mut(),
        &env,
        &info,
        None,
        None,
        vec![UpdateNftWhitelist(UpdateNftWhitelistMsg {
            add: vec![],
            remove: vec![Addr::unchecked("nft"), Addr::unchecked("nft")],
        })],
    );

    assert_eq!(result, Err(DaoError::DuplicateNftFound));

    Ok(())
}

#[test]
fn create_proposal_with_invalid_execute_msg_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    env.contract.address = Addr::unchecked("dao_addr");
    let current_time = Timestamp::from_seconds(12);
    env.block.time = current_time;
    let info = mock_info("sender", &[]);

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        existing_token_dao_membership(CW20_ADDR),
        None,
        None,
    )?;

    let result = create_proposal(
        deps.as_mut(),
        &env,
        &info,
        None,
        None,
        vec![ExecuteMsgs(ExecuteMsgsMsg {
            action_type: "random".to_string(),
            msgs: vec!["random_message".to_string()],
        })],
    );

    assert_eq!(result, Err(InvalidCosmosMessage));

    Ok(())
}

#[test]
fn create_proposal_with_invalid_upgrade_dao_version_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    env.contract.address = Addr::unchecked("dao_addr");
    let current_time = Timestamp::from_seconds(12);
    env.block.time = current_time;
    let info = mock_info("sender", &[]);

    let enterprise_factory_contract = "enterprise_factory_contract";

    deps.querier
        .with_enterprise_code_ids(&[(enterprise_factory_contract, &[1u64, 3u64])]);

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            dao_metadata: stub_dao_metadata(),
            dao_gov_config: stub_dao_gov_config(),
            dao_council: None,
            dao_membership_info: existing_token_dao_membership(CW20_ADDR),
            enterprise_factory_contract: enterprise_factory_contract.to_string(),
            asset_whitelist: None,
            nft_whitelist: None,
        },
    )?;

    let result = create_proposal(
        deps.as_mut(),
        &env,
        &info,
        None,
        None,
        vec![UpgradeDao(UpgradeDaoMsg {
            new_dao_code_id: 2u64,
            migrate_msg: to_binary("{}")?,
        })],
    );

    assert_eq!(result, Err(InvalidEnterpriseCodeId { code_id: 2u64 }));

    Ok(())
}

#[test]
fn create_modify_multisig_members_proposal_for_token_dao_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        existing_token_dao_membership(CW20_ADDR),
        None,
        None,
    )?;

    let result = create_proposal(
        deps.as_mut(),
        &env,
        &info,
        None,
        None,
        vec![ModifyMultisigMembership(ModifyMultisigMembershipMsg {
            edit_members: vec![],
        })],
    );

    assert_eq!(
        result,
        Err(UnsupportedOperationForDaoType {
            dao_type: "Token".to_string()
        })
    );

    Ok(())
}

#[test]
fn create_modify_multisig_members_proposal_for_nft_dao_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    deps.querier.with_num_tokens(&[(NFT_ADDR, 100u64)]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        existing_nft_dao_membership(NFT_ADDR),
        None,
        None,
    )?;

    let result = create_proposal(
        deps.as_mut(),
        &env,
        &info,
        None,
        None,
        vec![ModifyMultisigMembership(ModifyMultisigMembershipMsg {
            edit_members: vec![],
        })],
    );

    assert_eq!(
        result,
        Err(UnsupportedOperationForDaoType {
            dao_type: "Nft".to_string()
        })
    );

    Ok(())
}

#[test]
fn create_proposal_by_non_nft_holder_or_staker_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    env.contract.address = Addr::unchecked("dao_addr");
    let current_time = Timestamp::from_seconds(12);
    env.block.time = current_time;
    let info = mock_info("sender", &[]);

    deps.querier.with_num_tokens(&[(NFT_ADDR, 1u64)]);
    deps.querier
        .with_nft_holders(&[(NFT_ADDR, &[("holder", &["1", "2"])])]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        existing_nft_dao_membership(NFT_ADDR),
        None,
        None,
    )?;

    let result = create_stub_proposal(deps.as_mut(), &env, &info);

    assert_eq!(result, Err(NotNftOwner {}));

    Ok(())
}

#[test]
fn create_proposal_by_non_member_in_multisig_dao_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        multisig_dao_membership_info_with_members(&[("member", 100u64)]),
        None,
        None,
    )?;

    let result = create_proposal(
        deps.as_mut(),
        &env,
        &mock_info("non_member", &vec![]),
        None,
        None,
        vec![ModifyMultisigMembership(ModifyMultisigMembershipMsg {
            edit_members: vec![],
        })],
    );

    assert_eq!(result, Err(NotMultisigMember {}));

    Ok(())
}

#[test]
fn create_proposal_to_update_council_with_non_allowed_types_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        multisig_dao_membership_info_with_members(&[("member", 100u64)]),
        None,
        None,
    )?;

    let result = create_proposal(
        deps.as_mut(),
        &env,
        &mock_info("non_member", &vec![]),
        None,
        None,
        vec![ProposalAction::UpdateCouncil(UpdateCouncilMsg {
            dao_council: Some(DaoCouncil {
                members: vec!["member".to_string()],
                allowed_proposal_action_types: Some(vec![UpdateCouncil]),
            }),
        })],
    );

    assert_eq!(
        result,
        Err(UnsupportedCouncilProposalAction {
            action: UpdateCouncil
        })
    );

    Ok(())
}
