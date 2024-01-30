use crate::helpers::{ADDR_FACTORY, USER1, USER2, USER_DAO_CREATOR};
use crate::traits::{IntoAddr, IntoStringVec};
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw20::Cw20Coin;
use cw_multi_test::{App, AppResponse, Executor};
use cw_utils::Duration;
use enterprise_factory_api::api::{
    AllDaosResponse, CreateDaoMembershipMsg, CreateDaoMsg, ImportCw3MembershipMsg,
    NewCw20MembershipMsg, NewCw721MembershipMsg, NewMultisigMembershipMsg, QueryAllDaosMsg,
};
use enterprise_factory_api::msg::QueryMsg::AllDaos;
use enterprise_governance_controller_api::api::{DaoCouncilSpec, GovConfig, ProposalActionType};
use enterprise_protocol::api::{DaoMetadata, DaoSocialData, Logo};
use enterprise_protocol::error::DaoResult;
use multisig_membership_api::api::UserWeight;

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

pub fn new_nft_membership(nft_membership: NewCw721MembershipMsg) -> CreateDaoMembershipMsg {
    CreateDaoMembershipMsg::NewCw721(nft_membership)
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
        quorum: Decimal::percent(34),
        threshold: Decimal::percent(54),
        allowed_proposal_action_types: Some(vec![
            ProposalActionType::DeployCrossChainTreasury,
            ProposalActionType::RemoveAttestation,
        ]),
    }
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
