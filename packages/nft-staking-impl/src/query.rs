use crate::claims::{get_claims, get_releasable_claims};
use crate::config::CONFIG;
use crate::nft_staking::{get_user_total_stake, NftStake, NFT_STAKES, USER_TOTAL_STAKED};
use crate::total_staked::{
    load_total_staked, load_total_staked_at_height, load_total_staked_at_time,
};
use common::cw::QueryContext;
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{Addr, Order, StdResult, Uint128};
use cw_storage_plus::Bound;
use cw_utils::Expiration;
use itertools::Itertools;
use membership_common::api::{
    AdminResponse, MembersParams, MembersResponse, TotalWeightParams, TotalWeightResponse,
    UserWeightParams, UserWeightResponse,
};
use nft_staking_api::api::{
    ClaimsParams, ClaimsResponse, NftConfigResponse, UserNftStakeParams, UserNftStakeResponse,
};
use nft_staking_api::error::NftStakingResult;

const MAX_QUERY_LIMIT: u8 = 100;
const DEFAULT_QUERY_LIMIT: u8 = 50;

pub fn query_admin(qctx: &QueryContext) -> NftStakingResult<AdminResponse> {
    let config = CONFIG.load(qctx.deps.storage)?;

    Ok(AdminResponse {
        admin: config.admin,
    })
}

pub fn query_nft_cnfig(qctx: &QueryContext) -> NftStakingResult<NftConfigResponse> {
    let config = CONFIG.load(qctx.deps.storage)?;

    Ok(NftConfigResponse {
        nft_contract: config.nft_contract,
        unlocking_period: config.unlocking_period,
    })
}

pub fn query_user_nft_stake(
    qctx: &QueryContext,
    params: UserNftStakeParams,
) -> NftStakingResult<UserNftStakeResponse> {
    let user = qctx.deps.api.addr_validate(&params.user)?;

    let start_after = params.start_after.map(Bound::exclusive);
    let limit = params
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT as u32)
        .min(MAX_QUERY_LIMIT as u32);

    let user_stake = NFT_STAKES()
        .idx
        .staker
        .prefix(user.clone())
        .range(qctx.deps.storage, start_after, None, Order::Ascending)
        .take(limit as usize)
        .map_ok(|(_, stake)| stake)
        .collect::<StdResult<Vec<NftStake>>>()?;

    let total_user_stake = get_user_total_stake(qctx.deps.storage, user.clone())?;
    let tokens = user_stake.into_iter().map(|stake| stake.token_id).collect();

    Ok(UserNftStakeResponse {
        user,
        tokens,
        total_user_stake,
    })
}

pub fn query_user_weight(
    qctx: &QueryContext,
    params: UserWeightParams,
) -> NftStakingResult<UserWeightResponse> {
    let user = qctx.deps.api.addr_validate(&params.user)?;

    let weight = get_user_total_stake(qctx.deps.storage, user.clone())?;

    Ok(UserWeightResponse { user, weight })
}

pub fn query_total_weight(
    qctx: &QueryContext,
    params: TotalWeightParams,
) -> NftStakingResult<TotalWeightResponse> {
    let total_weight = match params.expiration {
        Expiration::AtHeight(height) => load_total_staked_at_height(qctx.deps.storage, height)?,
        Expiration::AtTime(time) => load_total_staked_at_time(qctx.deps.storage, time)?,
        Expiration::Never {} => load_total_staked(qctx.deps.storage)?,
    };

    Ok(TotalWeightResponse { total_weight })
}

pub fn query_claims(qctx: &QueryContext, params: ClaimsParams) -> NftStakingResult<ClaimsResponse> {
    let user = qctx.deps.api.addr_validate(&params.user)?;

    get_claims(qctx.deps.storage, user)
}

pub fn query_releasable_claims(
    qctx: &QueryContext,
    params: ClaimsParams,
) -> NftStakingResult<ClaimsResponse> {
    let user = qctx.deps.api.addr_validate(&params.user)?;

    get_releasable_claims(qctx.deps.storage, &qctx.env.block, user)
}

pub fn query_members(
    qctx: &QueryContext,
    params: MembersParams,
) -> NftStakingResult<MembersResponse> {
    let start_after = params
        .start_after
        .map(|addr| qctx.deps.api.addr_validate(&addr))
        .transpose()?
        .map(Bound::exclusive);
    let limit = params
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT as u32)
        .min(MAX_QUERY_LIMIT as u32);

    let members = USER_TOTAL_STAKED
        .range(qctx.deps.storage, start_after, None, Ascending)
        .take(limit as usize)
        .collect::<StdResult<Vec<(Addr, Uint128)>>>()?
        .into_iter()
        .map(|(user, weight)| UserWeightResponse { user, weight })
        .collect();

    Ok(MembersResponse { members })
}
