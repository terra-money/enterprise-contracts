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
use multisig_membership_api::api::ConfigResponse;
use multisig_membership_api::error::MultisigMembershipResult;

const DEFAULT_QUERY_LIMIT: u8 = 50;
const MAX_QUERY_LIMIT: u8 = 100;

pub fn query_config(qctx: &QueryContext) -> MultisigMembershipResult<ConfigResponse> {
    let enterprise_contract = ENTERPRISE_CONTRACT.load(qctx.deps.storage)?;

    Ok(ConfigResponse {
        enterprise_contract,
    })
}

pub fn query_user_weight(
    qctx: &QueryContext,
    params: UserWeightParams,
) -> MultisigMembershipResult<UserWeightResponse> {
    let user = qctx.deps.api.addr_validate(&params.user)?;

    let user_weight = get_member_weight(qctx.deps.storage, user.clone())?;

    Ok(UserWeightResponse {
        user,
        weight: user_weight,
    })
}

pub fn query_total_weight(
    qctx: &QueryContext,
    params: TotalWeightParams,
) -> MultisigMembershipResult<TotalWeightResponse> {
    let total_weight = match params.expiration {
        Expiration::AtHeight(height) => load_total_weight_at_height(qctx.deps.storage, height)?,
        Expiration::AtTime(time) => load_total_weight_at_time(qctx.deps.storage, time)?,
        Expiration::Never {} => load_total_weight(qctx.deps.storage)?,
    };

    Ok(TotalWeightResponse { total_weight })
}

pub fn query_members(
    qctx: &QueryContext,
    params: MembersParams,
) -> MultisigMembershipResult<MembersResponse> {
    let start_after = params
        .start_after
        .map(|addr| qctx.deps.api.addr_validate(&addr))
        .transpose()?
        .map(Bound::exclusive);
    let limit = params
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT as u32)
        .min(MAX_QUERY_LIMIT as u32);

    let members = MEMBER_WEIGHTS()
        .idx
        .user
        .range(qctx.deps.storage, start_after, None, Ascending)
        .take(limit as usize)
        .map(|res| res.map(|(addr, weight)| (addr, weight.weight)))
        .collect::<StdResult<Vec<(Addr, Uint128)>>>()?
        .into_iter()
        .map(|(user, weight)| UserWeightResponse { user, weight })
        .collect();

    Ok(MembersResponse { members })
}
