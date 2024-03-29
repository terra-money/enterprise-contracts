use crate::helpers::cw_multitest_helpers::{
    ADDR_FACTORY, CW20_TOKEN1, NFT_TOKEN1, USER1, USER2, USER_DAO_CREATOR,
};
use crate::helpers::facade_helpers::TestFacade;
use crate::traits::{IntoAddr, IntoStringVec};
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw20::Cw20Coin;
use cw_asset::AssetInfoUnchecked;
use cw_multi_test::{App, AppResponse, Executor};
use cw_utils::Duration;
use enterprise_factory_api::api::{
    AllDaosResponse, CreateDaoMembershipMsg, CreateDaoMsg, ImportCw20MembershipMsg,
    ImportCw3MembershipMsg, ImportCw721MembershipMsg, NewCw20MembershipMsg, NewCw721MembershipMsg,
    NewDenomMembershipMsg, NewMultisigMembershipMsg, QueryAllDaosMsg, TokenMarketingInfo,
};
use enterprise_factory_api::msg::QueryMsg::AllDaos;
use enterprise_governance_controller_api::api::{DaoCouncilSpec, GovConfig, ProposalActionType};
use enterprise_protocol::api::{DaoMetadata, DaoSocialData, Logo};
use enterprise_protocol::error::DaoResult;
use multisig_membership_api::api::UserWeight;

pub fn default_create_dao_msg() -> CreateDaoMsg {
    CreateDaoMsg {
        dao_metadata: default_dao_metadata(),
        gov_config: default_gov_config(),
        dao_council: Some(default_dao_council()),
        dao_membership: new_token_membership(default_new_token_membership()),
        asset_whitelist: Some(vec![AssetInfoUnchecked::cw20(CW20_TOKEN1)]),
        nft_whitelist: Some(vec![NFT_TOKEN1.to_string()]),
        minimum_weight_for_rewards: Some(2u8.into()),
        proposals_tracked_for_participation_rewards: None,
        cross_chain_treasuries: None,
    }
}

pub fn new_multisig_membership(members: Vec<(impl Into<String>, u8)>) -> CreateDaoMembershipMsg {
    CreateDaoMembershipMsg::NewMultisig(NewMultisigMembershipMsg {
        multisig_members: members
            .into_iter()
            .map(|(addr, weight)| UserWeight {
                user: addr.into(),
                weight: weight.into(),
            })
            .collect(),
    })
}

pub fn import_cw3_membership(cw3_contract: Addr) -> CreateDaoMembershipMsg {
    CreateDaoMembershipMsg::ImportCw3(ImportCw3MembershipMsg {
        cw3_contract: cw3_contract.to_string(),
    })
}

pub fn new_token_membership(token_membership: NewCw20MembershipMsg) -> CreateDaoMembershipMsg {
    CreateDaoMembershipMsg::NewCw20(Box::new(token_membership))
}

pub fn default_new_token_membership() -> NewCw20MembershipMsg {
    NewCw20MembershipMsg {
        token_name: "Token name".to_string(),
        token_symbol: "TKNM".to_string(),
        token_decimals: 6,
        initial_token_balances: vec![Cw20Coin {
            address: USER_DAO_CREATOR.to_string(),
            amount: Uint128::one(),
        }],
        initial_dao_balance: Some(100u8.into()),
        token_mint: None,
        token_marketing: None,
        unlocking_period: Duration::Time(300),
    }
}

pub fn import_cw20_membership(
    cw20_contract: impl Into<String>,
    unlocking_period: u64,
) -> CreateDaoMembershipMsg {
    CreateDaoMembershipMsg::ImportCw20(ImportCw20MembershipMsg {
        cw20_contract: cw20_contract.into(),
        unlocking_period: Duration::Time(unlocking_period),
    })
}

pub fn new_nft_membership(nft_membership: NewCw721MembershipMsg) -> CreateDaoMembershipMsg {
    CreateDaoMembershipMsg::NewCw721(nft_membership)
}

pub fn import_cw721_membership(
    cw721_contract: String,
    unlocking_period: u64,
) -> CreateDaoMembershipMsg {
    CreateDaoMembershipMsg::ImportCw721(ImportCw721MembershipMsg {
        cw721_contract,
        unlocking_period: Duration::Time(unlocking_period),
    })
}

pub fn new_denom_membership(
    denom: impl Into<String>,
    unlocking_period: u64,
) -> CreateDaoMembershipMsg {
    CreateDaoMembershipMsg::NewDenom(NewDenomMembershipMsg {
        denom: denom.into(),
        unlocking_period: Duration::Time(unlocking_period),
    })
}

pub fn default_dao_metadata() -> DaoMetadata {
    DaoMetadata {
        name: "DAO name".to_string(),
        description: None,
        logo: Logo::None,
        socials: DaoSocialData {
            github_username: None,
            discord_username: None,
            twitter_username: None,
            telegram_username: None,
        },
    }
}

pub fn default_gov_config() -> GovConfig {
    GovConfig {
        quorum: Decimal::percent(31),
        threshold: Decimal::percent(52),
        veto_threshold: Some(Decimal::percent(17)),
        vote_duration: 250,
        minimum_deposit: None,
        allow_early_proposal_execution: false,
    }
}

pub fn default_dao_council() -> DaoCouncilSpec {
    DaoCouncilSpec {
        members: vec![USER1, USER2].into_string(),
        quorum: Decimal::percent(35),
        threshold: Decimal::percent(55),
        allowed_proposal_action_types: Some(vec![
            ProposalActionType::DeployCrossChainTreasury,
        ]),
    }
}

pub fn default_token_marketing_info() -> TokenMarketingInfo {
    TokenMarketingInfo {
        project: Some("Some project name".to_string()),
        description: Some("Project description".to_string()),
        marketing_owner: Some("marketing_owner".to_string()),
        logo_url: Some("logo_url".to_string()),
    }
}

// TODO: a lot of manual construction of this going on, replace wit this convenience
pub fn asset_whitelist(native: Vec<&str>, cw20: Vec<&str>) -> Option<Vec<AssetInfoUnchecked>> {
    let mut assets = vec![];

    native.into_iter().for_each(|it| assets.push(AssetInfoUnchecked::native(it)));
    cw20.into_iter().for_each(|it| assets.push(AssetInfoUnchecked::cw20(it)));

    Some(assets)
}

// TODO: create an interface to the factory
pub fn create_dao(app: &mut App, msg: CreateDaoMsg) -> anyhow::Result<AppResponse> {
    let response = app.execute_contract(
        USER_DAO_CREATOR.into_addr(),
        ADDR_FACTORY.into_addr(),
        &enterprise_factory_api::msg::ExecuteMsg::CreateDao(Box::new(msg)),
        &[],
    )?;
    Ok(response)
}

// TODO: there are many uses of create_dao where facade is then created, replace them with this
pub fn create_dao_and_get_facade(app: &mut App, msg: CreateDaoMsg) -> anyhow::Result<TestFacade> {
    create_dao(app, msg)?;

    // TODO: use DAO ID from response above, instead of fetching just the first one
    let dao_addr = get_first_dao(&app)?;
    let facade = TestFacade { app, dao_addr };

    Ok(facade)
}

// TODO: we should either switch to this and then instantiate TestFacade every time we need it and consume it immediately,
// TODO: or RefCell in the facade
pub fn create_dao_and_get_addr(app: &mut App, msg: CreateDaoMsg) -> anyhow::Result<Addr> {
    create_dao(app, msg)?;

    // TODO: use DAO ID from response above, instead of fetching just the first one
    let dao_addr = get_first_dao(&app)?;

    Ok(dao_addr)
}

pub fn query_all_daos(app: &App) -> DaoResult<AllDaosResponse> {
    app.wrap()
        .query_wasm_smart(
            ADDR_FACTORY,
            &AllDaos(QueryAllDaosMsg {
                start_after: None,
                limit: None,
            }),
        )
        .map_err(|e| e.into())
}

pub fn get_first_dao(app: &App) -> DaoResult<Addr> {
    query_all_daos(app).map(|it| it.daos.first().cloned().unwrap().dao_address)
}
