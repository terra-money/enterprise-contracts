use crate::api::{
    AllDaosResponse, Config, ConfigResponse, CreateDaoMsg, EnterpriseCodeIdsMsg,
    EnterpriseCodeIdsResponse, IsEnterpriseCodeIdMsg, IsEnterpriseCodeIdResponse, QueryAllDaosMsg,
};
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub config: Config,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateDao(Box<CreateDaoMsg>),
    FinalizeDaoCreation {},
}

#[cw_serde]
pub struct MigrateMsg {
    pub new_enterprise_code_id: u64,
    pub new_enterprise_governance_code_id: u64,
    pub new_funds_distributor_code_id: u64,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(AllDaosResponse)]
    AllDaos(QueryAllDaosMsg),
    #[returns(EnterpriseCodeIdsResponse)]
    EnterpriseCodeIds(EnterpriseCodeIdsMsg),
    #[returns(IsEnterpriseCodeIdResponse)]
    IsEnterpriseCodeId(IsEnterpriseCodeIdMsg),
}
