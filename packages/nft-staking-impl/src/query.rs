use crate::claims::{get_claims, get_releasable_claims};
use crate::nft_staking::{get_user_total_stake, NftStake, NFT_STAKES};
use crate::total_staked::{
    load_total_staked, load_total_staked_at_height, load_total_staked_at_time,
};
use common::cw::QueryContext;
use cosmwasm_std::{Order, StdResult};
use cw_storage_plus::Bound;
use cw_utils::Expiration;
use itertools::Itertools;
use nft_staking_api::api::{
    ClaimsResponse, UserNftStakeParams, UserNftStakeResponse, UserNftTotalStakeResponse,
};
use nft_staking_api::error::NftStakingResult;
use staking_common::api::{
    ClaimsParams, TotalStakedAmountParams, TotalStakedAmountResponse, UserTotalStakeParams,
};

const MAX_QUERY_LIMIT: u32 = 100;
const DEFAULT_QUERY_LIMIT: u32 = 50;

pub fn query_user_nft_stake(
    qctx: &QueryContext,
    params: UserNftStakeParams,
) -> NftStakingResult<UserNftStakeResponse> {
    let user = qctx.deps.api.addr_validate(&params.user)?;

    let start_after = params.start_after.map(Bound::exclusive);
    let limit = params
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT)
        .min(MAX_QUERY_LIMIT);

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

pub fn query_user_total_stake(
    qctx: &QueryContext,
    params: UserTotalStakeParams,
) -> NftStakingResult<UserNftTotalStakeResponse> {
    let user = qctx.deps.api.addr_validate(&params.user)?;

    let total_user_stake = get_user_total_stake(qctx.deps.storage, user.clone())?;

    Ok(UserNftTotalStakeResponse {
        user,
        total_user_stake,
    })
}

pub fn query_total_staked_amount(
    qctx: &QueryContext,
    params: TotalStakedAmountParams,
) -> NftStakingResult<TotalStakedAmountResponse> {
    let total_staked_amount = match params.expiration {
        Expiration::AtHeight(height) => load_total_staked_at_height(qctx.deps.storage, height)?,
        Expiration::AtTime(time) => load_total_staked_at_time(qctx.deps.storage, time)?,
        Expiration::Never {} => load_total_staked(qctx.deps.storage)?,
    };

    Ok(TotalStakedAmountResponse {
        total_staked_amount,
    })
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
