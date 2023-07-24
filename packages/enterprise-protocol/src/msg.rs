use crate::api::{
    ComponentContractsResponse, CrossChainTreasuriesParams, CrossChainTreasuriesResponse,
    DaoInfoResponse, DaoMetadata, EditCrossChainTreasuriesMsg, FinalizeInstantiationMsg,
    IsCrossChainTreasuryParams, IsCrossChainTreasuryResponse, IsRestrictedUserParams,
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

    EditCrossChainTreasuries(EditCrossChainTreasuriesMsg),

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

    #[returns(IsCrossChainTreasuryResponse)]
    IsCrossChainTreasury(IsCrossChainTreasuryParams),

    /// Query whether a user should be restricted from certain DAO actions, such as governance and
    /// rewards claiming.
    /// Is determined by checking if there is an attestation, and if the user has signed it or not.
    #[returns(IsRestrictedUserResponse)]
    IsRestrictedUser(IsRestrictedUserParams),
}
