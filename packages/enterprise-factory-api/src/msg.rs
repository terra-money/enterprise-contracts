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

    /// Executed only by this contract itself to finalize creation of a DAO.
    /// Not part of the public interface.
    FinalizeDaoCreation {},
}

#[cw_serde]
pub struct MigrateMsg {
    pub enterprise_versioning_addr: String,
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
