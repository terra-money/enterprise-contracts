use crate::member_weights::MEMBER_WEIGHTS;
use crate::total_weight::{
    load_total_weight, load_total_weight_at_height, load_total_weight_at_time,
};
use common::cw::QueryContext;
use cw_utils::Expiration;
use multisig_membership_api::api::{
    TotalWeightParams, TotalWeightResponse, UserWeightParams, UserWeightResponse,
};
use multisig_membership_api::error::MultisigMembershipResult;

pub fn query_user_weight(
    qctx: &QueryContext,
    params: UserWeightParams,
) -> MultisigMembershipResult<UserWeightResponse> {
    let user = qctx.deps.api.addr_validate(&params.user)?;

    let user_weight = MEMBER_WEIGHTS
        .may_load(qctx.deps.storage, user.clone())?
        .unwrap_or_default();

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
