use crate::helpers::USER_DAO_CREATOR;
use cosmwasm_std::Uint128;
use cw20::Cw20Coin;
use cw_utils::Duration;
use enterprise_factory_api::api::{
    CreateDaoMembershipMsg, NewCw20MembershipMsg, NewMultisigMembershipMsg,
};
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
