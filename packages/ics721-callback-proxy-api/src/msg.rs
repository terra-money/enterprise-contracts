use crate::api::ConfigResponse;
use cosmwasm_schema::{cw_serde, QueryResponses};
use ics721_types::types::Ics721ReceiveCallbackMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub ics721_proxy: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Ics721ReceiveCallback(Ics721ReceiveCallbackMsg),
}

#[cw_serde]
pub enum Ics721HookMsg {
    Stake {
        user: String,
        membership_contract: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
}

#[cw_serde]
pub struct MigrateMsg {}
