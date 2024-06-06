use crate::api::{
    ClaimMsg, ClaimsParams, ClaimsResponse, NftConfigResponse, NftContract,
    NftContractConfigResponse, ReceiveNftMsg, StakedNftsParams, StakedNftsResponse, UnstakeMsg,
    UpdateUnlockingPeriodMsg, UserNftStakeParams, UserNftStakeResponse,
};
use common::cw::ReleaseAt;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_utils::Duration;
use membership_common_api::api::{
    MembersParams, MembersResponse, TotalWeightAboveParams, TotalWeightCheckpoint,
    TotalWeightParams, TotalWeightResponse, UserWeightParams, UserWeightResponse,
    WeightChangeHookMsg,
};

#[cw_serde]
pub struct InstantiateMsg {
    pub enterprise_contract: String,
    pub nft_contract: NftContract,
    pub unlocking_period: Duration,
    pub weight_change_hooks: Option<Vec<String>>,
    pub total_weight_by_height_checkpoints: Option<Vec<TotalWeightCheckpoint>>,
    pub total_weight_by_seconds_checkpoints: Option<Vec<TotalWeightCheckpoint>>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Unstake(UnstakeMsg),
    Claim(ClaimMsg),
    UpdateUnlockingPeriod(UpdateUnlockingPeriodMsg),
    ReceiveNft(ReceiveNftMsg),
    AddWeightChangeHook(WeightChangeHookMsg),
    RemoveWeightChangeHook(WeightChangeHookMsg),
}

#[cw_serde]
pub enum Cw721HookMsg {
    Stake { user: String },
    AddClaim { user: String, release_at: ReleaseAt },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(NftConfigResponse)]
    NftConfig {},
    #[returns(NftContractConfigResponse)]
    NftContractConfig {},
    #[returns(UserNftStakeResponse)]
    UserStake(UserNftStakeParams),
    #[returns(UserWeightResponse)]
    UserWeight(UserWeightParams),
    #[returns(StakedNftsResponse)]
    StakedNfts(StakedNftsParams),
    #[returns(TotalWeightResponse)]
    TotalWeight(TotalWeightParams),
    #[returns(TotalWeightResponse)]
    TotalWeightAbove(TotalWeightAboveParams),
    #[returns(ClaimsResponse)]
    Claims(ClaimsParams),
    #[returns(ClaimsResponse)]
    ReleasableClaims(ClaimsParams),
    #[returns(MembersResponse)]
    Members(MembersParams),
}

#[cw_serde]
pub struct MigrateMsg {}
