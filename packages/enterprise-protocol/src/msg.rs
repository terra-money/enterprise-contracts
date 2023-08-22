use crate::api::{
    AddCrossChainTreasury, ComponentContractsResponse, CrossChainTreasuriesParams,
    CrossChainTreasuriesResponse, CrossChainTreasuryParams, CrossChainTreasuryResponse,
    DaoInfoResponse, DaoMetadata, FinalizeInstantiationMsg, IsRestrictedUserParams,
    IsRestrictedUserResponse, SetAttestationMsg, UpdateMetadataMsg, UpgradeDaoMsg,
};
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub enterprise_factory_contract: String,
    pub enterprise_versioning_contract: String,
    pub dao_metadata: DaoMetadata,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateMetadata(UpdateMetadataMsg),
    UpgradeDao(UpgradeDaoMsg),

    SetAttestation(SetAttestationMsg),
    RemoveAttestation {},

    AddCrossChainTreasury(AddCrossChainTreasury),

    // called by this contract itself
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

    #[returns(CrossChainTreasuryResponse)]
    CrossChainTreasury(CrossChainTreasuryParams),

    /// Query whether a user should be restricted from certain DAO actions, such as governance and
    /// rewards claiming.
    /// Is determined by checking if there is an attestation, and if the user has signed it or not.
    #[returns(IsRestrictedUserResponse)]
    IsRestrictedUser(IsRestrictedUserParams),
}
