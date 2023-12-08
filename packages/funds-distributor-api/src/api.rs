use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct UpdateUserWeightsMsg {
    /// New weights that the users have, after the change
    pub new_user_weights: Vec<UserWeight>,
}

#[cw_serde]
pub struct UpdateMinimumEligibleWeightMsg {
    /// New minimum weight that the user must have to be eligible for rewards distributions
    pub minimum_eligible_weight: Uint128,
}

#[cw_serde]
pub struct UserWeight {
    pub user: String,
    pub weight: Uint128,
}

#[cw_serde]
pub struct ClaimRewardsMsg {
    /// Kept for backwards-compatibility. Will be used if 'receiver' is None
    #[deprecated(note = "use 'receiver' field instead")]
    pub user: String,
    pub receiver: Option<RewardsReceiver>,
    /// Native denominations to be claimed
    pub native_denoms: Vec<String>,
    /// CW20 asset rewards to be claimed, should be addresses of CW20 tokens
    pub cw20_assets: Vec<String>,
}

#[cw_serde]
pub enum RewardsReceiver {
    Local { address: String },
    CrossChain(CrossChainReceiver),
}

// TODO: same struct as we use for token and denom memberships, unify perhaps
#[cw_serde]
pub struct CrossChainReceiver {
    pub source_port: String,
    pub source_channel: String,
    pub receiver_address: String,
    pub cw20_ics20_contract: String,
    /// How long the packet lives in seconds. If not specified, use default_timeout
    pub timeout_seconds: u64,
}

#[cw_serde]
pub struct UserRewardsParams {
    pub user: String,
    /// Native denominations to be queried for rewards
    pub native_denoms: Vec<String>,
    /// Addresses of CW20 tokens to be queried for rewards
    pub cw20_assets: Vec<String>,
}

#[cw_serde]
pub struct UserRewardsResponse {
    pub native_rewards: Vec<NativeReward>,
    pub cw20_rewards: Vec<Cw20Reward>,
}

#[cw_serde]
pub struct MinimumEligibleWeightResponse {
    pub minimum_eligible_weight: Uint128,
}

#[cw_serde]
pub struct NativeReward {
    pub denom: String,
    pub amount: Uint128,
}

#[cw_serde]
pub struct Cw20Reward {
    /// Address of the CW20 token
    pub asset: String,
    pub amount: Uint128,
}
