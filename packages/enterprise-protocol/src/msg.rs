use crate::api::{
    AddCrossChainProxyMsg, AddCrossChainTreasuryMsg, ComponentContractsResponse,
    CrossChainDeploymentsParams, CrossChainDeploymentsResponse, CrossChainTreasuriesParams,
    CrossChainTreasuriesResponse, DaoInfoResponse, DaoMetadata, DaoType, FinalizeInstantiationMsg,
    IsRestrictedUserParams, IsRestrictedUserResponse, SetAttestationMsg, UpdateMetadataMsg,
    UpgradeDaoMsg,
};
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub enterprise_factory_contract: String,
    pub enterprise_versioning_contract: String,
    pub dao_metadata: DaoMetadata,
    pub dao_type: DaoType,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateMetadata(UpdateMetadataMsg),
    UpgradeDao(UpgradeDaoMsg),

    SetAttestation(SetAttestationMsg),
    RemoveAttestation {},

    AddCrossChainProxy(AddCrossChainProxyMsg),
    AddCrossChainTreasury(AddCrossChainTreasuryMsg),

    // called only right after instantiation
    FinalizeInstantiation(FinalizeInstantiationMsg),
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(DaoInfoResponse)]
    DaoInfo {},

    #[returns(ComponentContractsResponse)]
    ComponentContracts {},

    #[returns(CrossChainTreasuriesResponse)]
    CrossChainTreasuries(CrossChainTreasuriesParams),

    #[returns(CrossChainDeploymentsResponse)]
    CrossChainDeployments(CrossChainDeploymentsParams),

    /// Query whether a user should be restricted from certain DAO actions, such as governance and
    /// rewards claiming.
    /// Is determined by checking if there is an attestation, and if the user has signed it or not.
    #[returns(IsRestrictedUserResponse)]
    IsRestrictedUser(IsRestrictedUserParams),
}
