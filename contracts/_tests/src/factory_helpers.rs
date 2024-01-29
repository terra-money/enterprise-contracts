use crate::helpers::{ADDR_FACTORY, USER_DAO_CREATOR};
use cosmwasm_std::{Addr, Uint128};
use cw20::Cw20Coin;
use cw_multi_test::App;
use cw_utils::Duration;
use enterprise_factory_api::api::{
    AllDaosResponse, CreateDaoMembershipMsg, NewCw20MembershipMsg, NewMultisigMembershipMsg,
    QueryAllDaosMsg,
};
use enterprise_factory_api::msg::QueryMsg::AllDaos;
use enterprise_protocol::error::{DaoError, DaoResult};
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

// TODO: create an interface to the factory
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
