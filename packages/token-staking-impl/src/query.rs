use crate::claims::{get_claims, get_releasable_claims};
use crate::config::CONFIG;
use common::cw::QueryContext;
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{Addr, StdResult, Uint128};
use cw_storage_plus::Bound;
use cw_utils::Expiration;
use membership_common::enterprise_contract::ENTERPRISE_CONTRACT;
use membership_common::member_weights::{get_member_weight, MEMBER_WEIGHTS};
use membership_common::total_weight::{
    load_total_weight, load_total_weight_at_height, load_total_weight_at_time,
};
use membership_common_api::api::{
    MembersParams, MembersResponse, TotalWeightParams, TotalWeightResponse, UserWeightParams,
    UserWeightResponse,
};
use token_staking_api::api::{ClaimsParams, ClaimsResponse, TokenConfigResponse};
use token_staking_api::error::TokenStakingResult;

const MAX_QUERY_LIMIT: u8 = 100;
const DEFAULT_QUERY_LIMIT: u8 = 50;

pub fn query_token_config(qctx: &QueryContext) -> TokenStakingResult<TokenConfigResponse> {
    let config = CONFIG.load(qctx.deps.storage)?;

    let enterprise_contract = ENTERPRISE_CONTRACT.load(qctx.deps.storage)?;

    Ok(TokenConfigResponse {
        enterprise_contract,
        token_contract: config.token_contract,
        unlocking_period: config.unlocking_period,
    })
}

pub fn query_user_weight(
    qctx: &QueryContext,
    params: UserWeightParams,
) -> TokenStakingResult<UserWeightResponse> {
    let user = qctx.deps.api.addr_validate(&params.user)?;

    let user_stake = get_member_weight(qctx.deps.storage, user.clone())?;

    Ok(UserWeightResponse {
        user,
        weight: user_stake,
    })
}

pub fn query_total_weight(
    qctx: &QueryContext,
    params: TotalWeightParams,
) -> TokenStakingResult<TotalWeightResponse> {
    let total_staked_amount = match params.expiration {
        Expiration::AtHeight(height) => load_total_weight_at_height(qctx.deps.storage, height)?,
        Expiration::AtTime(time) => load_total_weight_at_time(qctx.deps.storage, time)?,
        Expiration::Never {} => load_total_weight(qctx.deps.storage)?,
    };

    Ok(TotalWeightResponse {
        total_weight: total_staked_amount,
    })
}

pub fn query_claims(
    qctx: &QueryContext,
    params: ClaimsParams,
) -> TokenStakingResult<ClaimsResponse> {
    let user = qctx.deps.api.addr_validate(&params.user)?;

    get_claims(qctx.deps.storage, user)
}

pub fn query_releasable_claims(
    qctx: &QueryContext,
    params: ClaimsParams,
) -> TokenStakingResult<ClaimsResponse> {
    let user = qctx.deps.api.addr_validate(&params.user)?;

    get_releasable_claims(qctx.deps.storage, &qctx.env.block, user)
}

pub fn query_members(
    qctx: &QueryContext,
    params: MembersParams,
) -> TokenStakingResult<MembersResponse> {
    let start_after = params
        .start_after
        .map(|addr| qctx.deps.api.addr_validate(&addr))
        .transpose()?
        .map(Bound::exclusive);
    let limit = params
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT as u32)
        .min(MAX_QUERY_LIMIT as u32);

    let stakers = MEMBER_WEIGHTS()
        .range(qctx.deps.storage, start_after, None, Ascending)
        .take(limit as usize)
        .map(|res| res.map(|(addr, weight)| (addr, weight.weight)))
        .collect::<StdResult<Vec<(Addr, Uint128)>>>()?
        .into_iter()
        .map(|(user, weight)| UserWeightResponse { user, weight })
        .collect();

    Ok(MembersResponse { members: stakers })
}
