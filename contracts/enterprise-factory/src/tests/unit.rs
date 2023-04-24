use crate::contract::{
    execute, instantiate, query_asset_whitelist, query_config, query_nft_whitelist, reply,
    ENTERPRISE_INSTANTIATE_ID,
};
use common::cw::testing::{mock_env, mock_info};
use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::{
    to_binary, Addr, Decimal, Reply, SubMsg, SubMsgResponse, SubMsgResult, Uint128, WasmMsg,
};
use cw20::{Cw20Coin, MinterResponse};
use cw_asset::AssetInfo;
use cw_utils::Duration;
use enterprise_factory_api::api::{Config, CreateDaoMembershipMsg, CreateDaoMsg};
use enterprise_factory_api::msg::{ExecuteMsg, InstantiateMsg};
use enterprise_protocol::api::DaoMembershipInfo::Existing;
use enterprise_protocol::api::DaoType::Token;
use enterprise_protocol::api::NewMembershipInfo::NewMultisig;
use enterprise_protocol::api::ProposalActionType::{
    RequestFundingFromDao, UpdateMetadata, UpgradeDao,
};
use enterprise_protocol::api::{
    DaoCouncilSpec, DaoGovConfig, DaoMembershipInfo, DaoMetadata, DaoSocialData,
    ExistingDaoMembershipMsg, Logo, MultisigMember, NewDaoMembershipMsg, NewMembershipInfo,
    NewMultisigMembershipInfo, NewNftMembershipInfo, NewTokenMembershipInfo, TokenMarketingInfo,
};
use enterprise_protocol::error::DaoResult;
use CreateDaoMembershipMsg::{ExistingMembership, NewMembership};
use DaoMembershipInfo::New;
use NewMembershipInfo::{NewNft, NewToken};

const ENTERPRISE_FACTORY_ADDR: &str = "enterprise_factory_addr";

const ENTERPRISE_CODE_ID: u64 = 201;
const ENTERPRISE_GOVERNANCE_CODE_ID: u64 = 202;
const FUNDS_DISTRIBUTOR_CODE_ID: u64 = 203;
const CW3_FIXED_MULTISIG_CODE_ID: u64 = 204;
const CW_20_CODE_ID: u64 = 205;
const CW_721_CODE_ID: u64 = 206;

const TOKEN_NAME: &str = "some_token";
const TOKEN_SYMBOL: &str = "SMBL";
const TOKEN_DECIMALS: u8 = 6;
const TOKEN_MARKETING_OWNER: &str = "marketing_owner";
const TOKEN_LOGO_URL: &str = "logo_url";
const TOKEN_PROJECT_NAME: &str = "project_name";
const TOKEN_PROJECT_DESCRIPTION: &str = "project_description";

const NFT_NAME: &str = "some_nft";
const NFT_SYMBOL: &str = "NFTSML";

const MINTER: &str = "minter";

#[test]
fn instantiate_stores_data() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    let asset_whitelist = vec![native_asset("luna"), cw20_asset("some_cw20_token")];

    let nft_whitelist = vec![nft_asset("nf1"), nft_asset("nft2")];

    instantiate(
        deps.as_mut(),
        env,
        info,
        InstantiateMsg {
            config: Config {
                enterprise_code_id: ENTERPRISE_CODE_ID,
                enterprise_governance_code_id: ENTERPRISE_GOVERNANCE_CODE_ID,
                funds_distributor_code_id: FUNDS_DISTRIBUTOR_CODE_ID,
                cw3_fixed_multisig_code_id: CW3_FIXED_MULTISIG_CODE_ID,
                cw20_code_id: CW_20_CODE_ID,
                cw721_code_id: CW_721_CODE_ID,
            },
            global_asset_whitelist: Some(asset_whitelist.clone()),
            global_nft_whitelist: Some(nft_whitelist.clone()),
        },
    )?;

    let config = query_config(deps.as_ref())?;
    assert_eq!(
        config.config,
        Config {
            enterprise_code_id: ENTERPRISE_CODE_ID,
            enterprise_governance_code_id: ENTERPRISE_GOVERNANCE_CODE_ID,
            funds_distributor_code_id: FUNDS_DISTRIBUTOR_CODE_ID,
            cw3_fixed_multisig_code_id: CW3_FIXED_MULTISIG_CODE_ID,
            cw20_code_id: CW_20_CODE_ID,
            cw721_code_id: CW_721_CODE_ID,
        }
    );

    let global_asset_whitelist = query_asset_whitelist(deps.as_ref())?;
    assert_eq!(global_asset_whitelist.assets, asset_whitelist);

    let global_nft_whitelist = query_nft_whitelist(deps.as_ref())?;
    assert_eq!(global_nft_whitelist.nfts, nft_whitelist);

    Ok(())
}

#[test]
fn create_token_dao_instantiates_proper_enterprise_contract() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    env.contract.address = Addr::unchecked(ENTERPRISE_FACTORY_ADDR);
    let info = mock_info("sender", &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            config: Config {
                enterprise_code_id: ENTERPRISE_CODE_ID,
                enterprise_governance_code_id: ENTERPRISE_GOVERNANCE_CODE_ID,
                funds_distributor_code_id: FUNDS_DISTRIBUTOR_CODE_ID,
                cw3_fixed_multisig_code_id: CW3_FIXED_MULTISIG_CODE_ID,
                cw20_code_id: CW_20_CODE_ID,
                cw721_code_id: CW_721_CODE_ID,
            },
            global_asset_whitelist: None,
            global_nft_whitelist: None,
        },
    )?;

    let initial_token_balances = vec![Cw20Coin {
        address: "my_address".to_string(),
        amount: 1234u128.into(),
    }];
    let token_mint = Some(MinterResponse {
        minter: MINTER.to_string(),
        cap: Some(123456789u128.into()),
    });
    let token_marketing_info = TokenMarketingInfo {
        project: Some(TOKEN_PROJECT_NAME.to_string()),
        description: Some(TOKEN_PROJECT_DESCRIPTION.to_string()),
        marketing_owner: Some(TOKEN_MARKETING_OWNER.to_string()),
        logo_url: Some(TOKEN_LOGO_URL.to_string()),
    };
    let membership_info = NewMembership(NewToken(Box::new(NewTokenMembershipInfo {
        token_name: TOKEN_NAME.to_string(),
        token_symbol: TOKEN_SYMBOL.to_string(),
        token_decimals: TOKEN_DECIMALS,
        initial_token_balances: initial_token_balances.clone(),
        initial_dao_balance: Some(456u128.into()),
        token_mint: token_mint.clone(),
        token_marketing: Some(token_marketing_info.clone()),
    })));
    let dao_metadata = anonymous_dao_metadata();
    let dao_gov_config = anonymous_dao_gov_config();
    let dao_council = anonymous_dao_council();
    let asset_whitelist = vec![
        native_asset("uluna"),
        cw20_asset("token1"),
        cw20_asset("token2"),
    ];
    let nft_whitelist = vec![nft_asset("nft1"), nft_asset("nft2")];
    let response = execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::CreateDao(CreateDaoMsg {
            dao_metadata: dao_metadata.clone(),
            dao_gov_config: dao_gov_config.clone(),
            dao_council: Some(dao_council.clone()),
            dao_membership: membership_info,
            asset_whitelist: Some(asset_whitelist.clone()),
            nft_whitelist: Some(nft_whitelist.clone()),
            minimum_weight_for_rewards: Some(Uint128::from(3u8)),
        }),
    )?;

    assert_eq!(
        response.messages,
        vec![SubMsg::reply_on_success(
            WasmMsg::Instantiate {
                admin: Some(ENTERPRISE_FACTORY_ADDR.to_string()),
                code_id: ENTERPRISE_CODE_ID,
                msg: to_binary(&enterprise_protocol::msg::InstantiateMsg {
                    enterprise_governance_code_id: ENTERPRISE_GOVERNANCE_CODE_ID,
                    funds_distributor_code_id: FUNDS_DISTRIBUTOR_CODE_ID,
                    dao_metadata,
                    dao_gov_config,
                    dao_council: Some(dao_council),
                    dao_membership_info: New(NewDaoMembershipMsg {
                        membership_contract_code_id: CW_20_CODE_ID,
                        membership_info: NewToken(Box::new(NewTokenMembershipInfo {
                            token_name: TOKEN_NAME.to_string(),
                            token_symbol: TOKEN_SYMBOL.to_string(),
                            token_decimals: TOKEN_DECIMALS,
                            initial_token_balances,
                            initial_dao_balance: Some(456u128.into()),
                            token_mint,
                            token_marketing: Some(token_marketing_info),
                        })),
                    }),
                    enterprise_factory_contract: ENTERPRISE_FACTORY_ADDR.to_string(),
                    asset_whitelist: Some(asset_whitelist),
                    nft_whitelist: Some(nft_whitelist),
                    minimum_weight_for_rewards: Some(Uint128::from(3u8)),
                })?,
                funds: vec![],
                label: "DAO name".to_string(),
            },
            ENTERPRISE_INSTANTIATE_ID,
        )]
    );

    Ok(())
}

#[test]
fn create_nft_dao_instantiates_proper_enterprise_contract() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    env.contract.address = Addr::unchecked(ENTERPRISE_FACTORY_ADDR);
    let info = mock_info("sender", &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            config: Config {
                enterprise_code_id: ENTERPRISE_CODE_ID,
                enterprise_governance_code_id: ENTERPRISE_GOVERNANCE_CODE_ID,
                funds_distributor_code_id: FUNDS_DISTRIBUTOR_CODE_ID,
                cw3_fixed_multisig_code_id: CW3_FIXED_MULTISIG_CODE_ID,
                cw20_code_id: CW_20_CODE_ID,
                cw721_code_id: CW_721_CODE_ID,
            },
            global_asset_whitelist: None,
            global_nft_whitelist: None,
        },
    )?;

    let dao_metadata = anonymous_dao_metadata();
    let dao_gov_config = anonymous_dao_gov_config();
    let dao_council = anonymous_dao_council();
    let membership_info = NewNft(NewNftMembershipInfo {
        nft_name: NFT_NAME.to_string(),
        nft_symbol: NFT_SYMBOL.to_string(),
        minter: Some(MINTER.to_string()),
    });
    let asset_whitelist = vec![
        native_asset("uluna"),
        cw20_asset("token1"),
        cw20_asset("token2"),
    ];
    let nft_whitelist = vec![nft_asset("nft1"), nft_asset("nft2")];
    let response = execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::CreateDao(CreateDaoMsg {
            dao_metadata: dao_metadata.clone(),
            dao_gov_config: dao_gov_config.clone(),
            dao_council: Some(dao_council.clone()),
            dao_membership: NewMembership(membership_info.clone()),
            asset_whitelist: Some(asset_whitelist.clone()),
            nft_whitelist: Some(nft_whitelist.clone()),
            minimum_weight_for_rewards: Some(Uint128::from(3u8)),
        }),
    )?;

    assert_eq!(
        response.messages,
        vec![SubMsg::reply_on_success(
            WasmMsg::Instantiate {
                admin: Some(ENTERPRISE_FACTORY_ADDR.to_string()),
                code_id: ENTERPRISE_CODE_ID,
                msg: to_binary(&enterprise_protocol::msg::InstantiateMsg {
                    enterprise_governance_code_id: ENTERPRISE_GOVERNANCE_CODE_ID,
                    funds_distributor_code_id: FUNDS_DISTRIBUTOR_CODE_ID,
                    dao_metadata,
                    dao_gov_config,
                    dao_council: Some(dao_council),
                    dao_membership_info: New(NewDaoMembershipMsg {
                        membership_contract_code_id: CW_721_CODE_ID,
                        membership_info,
                    }),
                    enterprise_factory_contract: ENTERPRISE_FACTORY_ADDR.to_string(),
                    asset_whitelist: Some(asset_whitelist),
                    nft_whitelist: Some(nft_whitelist),
                    minimum_weight_for_rewards: Some(Uint128::from(3u8)),
                })?,
                funds: vec![],
                label: "DAO name".to_string(),
            },
            ENTERPRISE_INSTANTIATE_ID,
        )]
    );

    Ok(())
}

#[test]
fn create_multisig_dao_instantiates_proper_enterprise_contract() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    env.contract.address = Addr::unchecked(ENTERPRISE_FACTORY_ADDR);
    let info = mock_info("sender", &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            config: Config {
                enterprise_code_id: ENTERPRISE_CODE_ID,
                enterprise_governance_code_id: ENTERPRISE_GOVERNANCE_CODE_ID,
                funds_distributor_code_id: FUNDS_DISTRIBUTOR_CODE_ID,
                cw3_fixed_multisig_code_id: CW3_FIXED_MULTISIG_CODE_ID,
                cw20_code_id: CW_20_CODE_ID,
                cw721_code_id: CW_721_CODE_ID,
            },
            global_asset_whitelist: None,
            global_nft_whitelist: None,
        },
    )?;

    let dao_metadata = anonymous_dao_metadata();
    let dao_gov_config = anonymous_dao_gov_config();
    let dao_council = anonymous_dao_council();
    let membership_info = NewMultisig(NewMultisigMembershipInfo {
        multisig_members: vec![
            MultisigMember {
                address: "member1".to_string(),
                weight: 200u64.into(),
            },
            MultisigMember {
                address: "member2".to_string(),
                weight: 400u64.into(),
            },
        ],
    });
    let asset_whitelist = vec![
        native_asset("uluna"),
        cw20_asset("token1"),
        cw20_asset("token2"),
    ];
    let nft_whitelist = vec![nft_asset("nft1"), nft_asset("nft2")];
    let response = execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::CreateDao(CreateDaoMsg {
            dao_metadata: dao_metadata.clone(),
            dao_gov_config: dao_gov_config.clone(),
            dao_council: Some(dao_council.clone()),
            dao_membership: NewMembership(membership_info.clone()),
            asset_whitelist: Some(asset_whitelist.clone()),
            nft_whitelist: Some(nft_whitelist.clone()),
            minimum_weight_for_rewards: Some(Uint128::from(3u8)),
        }),
    )?;

    assert_eq!(
        response.messages,
        vec![SubMsg::reply_on_success(
            WasmMsg::Instantiate {
                admin: Some(ENTERPRISE_FACTORY_ADDR.to_string()),
                code_id: ENTERPRISE_CODE_ID,
                msg: to_binary(&enterprise_protocol::msg::InstantiateMsg {
                    enterprise_governance_code_id: ENTERPRISE_GOVERNANCE_CODE_ID,
                    funds_distributor_code_id: FUNDS_DISTRIBUTOR_CODE_ID,
                    dao_metadata,
                    dao_gov_config,
                    dao_council: Some(dao_council),
                    dao_membership_info: New(NewDaoMembershipMsg {
                        membership_contract_code_id: CW3_FIXED_MULTISIG_CODE_ID,
                        membership_info,
                    }),
                    enterprise_factory_contract: ENTERPRISE_FACTORY_ADDR.to_string(),
                    asset_whitelist: Some(asset_whitelist),
                    nft_whitelist: Some(nft_whitelist),
                    minimum_weight_for_rewards: Some(Uint128::from(3u8)),
                })?,
                funds: vec![],
                label: "DAO name".to_string(),
            },
            ENTERPRISE_INSTANTIATE_ID,
        )]
    );

    Ok(())
}

#[test]
fn create_existing_membership_dao_instantiates_proper_enterprise_contract() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    env.contract.address = Addr::unchecked(ENTERPRISE_FACTORY_ADDR);
    let info = mock_info("sender", &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            config: Config {
                enterprise_code_id: ENTERPRISE_CODE_ID,
                enterprise_governance_code_id: ENTERPRISE_GOVERNANCE_CODE_ID,
                funds_distributor_code_id: FUNDS_DISTRIBUTOR_CODE_ID,
                cw3_fixed_multisig_code_id: CW3_FIXED_MULTISIG_CODE_ID,
                cw20_code_id: CW_20_CODE_ID,
                cw721_code_id: CW_721_CODE_ID,
            },
            global_asset_whitelist: None,
            global_nft_whitelist: None,
        },
    )?;

    let dao_metadata = anonymous_dao_metadata();
    let dao_council = anonymous_dao_council();
    let membership_info = ExistingMembership(ExistingDaoMembershipMsg {
        dao_type: Token,
        membership_contract_addr: "membership_addr".to_string(),
    });
    let asset_whitelist = vec![
        native_asset("uluna"),
        cw20_asset("token1"),
        cw20_asset("token2"),
    ];
    let nft_whitelist = vec![nft_asset("nft1"), nft_asset("nft2")];
    let response = execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::CreateDao(CreateDaoMsg {
            dao_metadata: dao_metadata.clone(),
            dao_gov_config: DaoGovConfig {
                quorum: Decimal::percent(10),
                threshold: Decimal::percent(20),
                veto_threshold: Some(Decimal::percent(33)),
                vote_duration: 1000,
                unlocking_period: Duration::Height(10),
                minimum_deposit: Some(713u128.into()),
                allow_early_proposal_execution: false,
            },
            dao_council: Some(dao_council.clone()),
            dao_membership: membership_info,
            asset_whitelist: Some(asset_whitelist.clone()),
            nft_whitelist: Some(nft_whitelist.clone()),
            minimum_weight_for_rewards: Some(Uint128::from(3u8)),
        }),
    )?;

    assert_eq!(
        response.messages,
        vec![SubMsg::reply_on_success(
            WasmMsg::Instantiate {
                admin: Some(ENTERPRISE_FACTORY_ADDR.to_string()),
                code_id: ENTERPRISE_CODE_ID,
                msg: to_binary(&enterprise_protocol::msg::InstantiateMsg {
                    enterprise_governance_code_id: ENTERPRISE_GOVERNANCE_CODE_ID,
                    funds_distributor_code_id: FUNDS_DISTRIBUTOR_CODE_ID,
                    dao_metadata,
                    dao_gov_config: DaoGovConfig {
                        quorum: Decimal::percent(10),
                        threshold: Decimal::percent(20),
                        veto_threshold: Some(Decimal::percent(33)),
                        vote_duration: 1000,
                        unlocking_period: Duration::Height(10),
                        minimum_deposit: Some(713u128.into()),
                        allow_early_proposal_execution: false
                    },
                    dao_council: Some(dao_council),
                    dao_membership_info: Existing(ExistingDaoMembershipMsg {
                        dao_type: Token,
                        membership_contract_addr: "membership_addr".to_string(),
                    }),
                    enterprise_factory_contract: ENTERPRISE_FACTORY_ADDR.to_string(),
                    asset_whitelist: Some(asset_whitelist),
                    nft_whitelist: Some(nft_whitelist),
                    minimum_weight_for_rewards: Some(Uint128::from(3u8)),
                })?,
                funds: vec![],
                label: "DAO name".to_string(),
            },
            ENTERPRISE_INSTANTIATE_ID,
        )]
    );

    Ok(())
}

#[test]
fn reply_with_unknown_reply_id_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            config: stub_config(),
            global_asset_whitelist: None,
            global_nft_whitelist: None,
        },
    )?;

    let result = reply(
        deps.as_mut(),
        env,
        Reply {
            id: 215,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![],
                data: None,
            }),
        },
    );

    assert!(result.is_err());

    Ok(())
}

fn native_asset(denom: impl Into<String>) -> AssetInfo {
    AssetInfo::native(denom)
}

fn cw20_asset(addr: impl Into<String>) -> AssetInfo {
    AssetInfo::cw20(Addr::unchecked(addr))
}

fn nft_asset(addr: impl Into<String>) -> Addr {
    Addr::unchecked(addr)
}

fn stub_config() -> Config {
    Config {
        enterprise_code_id: ENTERPRISE_CODE_ID,
        enterprise_governance_code_id: ENTERPRISE_GOVERNANCE_CODE_ID,
        funds_distributor_code_id: FUNDS_DISTRIBUTOR_CODE_ID,
        cw3_fixed_multisig_code_id: CW3_FIXED_MULTISIG_CODE_ID,
        cw20_code_id: CW_20_CODE_ID,
        cw721_code_id: CW_721_CODE_ID,
    }
}

fn anonymous_dao_metadata() -> DaoMetadata {
    DaoMetadata {
        name: "DAO name".to_string(),
        description: Some("DAO description".to_string()),
        logo: Logo::Url("logo_url".to_string()),
        socials: DaoSocialData {
            github_username: Some("github_url".to_string()),
            discord_username: Some("discord_url".to_string()),
            twitter_username: Some("twitter_url".to_string()),
            telegram_username: Some("telegram_url".to_string()),
        },
    }
}

fn anonymous_dao_gov_config() -> DaoGovConfig {
    DaoGovConfig {
        quorum: Decimal::percent(70),
        threshold: Decimal::percent(30),
        veto_threshold: Some(Decimal::percent(33)),
        vote_duration: 1000,
        unlocking_period: Duration::Height(10),
        minimum_deposit: Some(713u128.into()),
        allow_early_proposal_execution: false,
    }
}

fn anonymous_dao_council() -> DaoCouncilSpec {
    DaoCouncilSpec {
        members: vec![],
        quorum: Decimal::percent(75),
        threshold: Decimal::percent(50),
        allowed_proposal_action_types: Some(vec![
            UpdateMetadata,
            RequestFundingFromDao,
            UpgradeDao,
        ]),
    }
}
