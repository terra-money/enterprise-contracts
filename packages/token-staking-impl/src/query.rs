use crate::claims::{get_claims, get_releasable_claims};
use crate::config::CONFIG;
use crate::token_staking::get_user_stake;
use crate::total_staked::{
    load_total_staked, load_total_staked_at_height, load_total_staked_at_time,
};
use common::cw::QueryContext;
use cw_utils::Expiration;
use token_staking_api::api::{
    ClaimsParams, ClaimsResponse, ConfigResponse, TotalStakedAmountParams,
    TotalStakedAmountResponse, UserTokenStakeParams, UserTokenStakeResponse,
};
use token_staking_api::error::TokenStakingResult;

pub fn query_config(qctx: &QueryContext) -> TokenStakingResult<ConfigResponse> {
    let config = CONFIG.load(qctx.deps.storage)?;

    Ok(ConfigResponse {
        admin: config.admin,
        token_contract: config.token_contract,
        unlocking_period: config.unlocking_period,
    })
}

pub fn query_user_token_stake(
    qctx: &QueryContext,
    params: UserTokenStakeParams,
) -> TokenStakingResult<UserTokenStakeResponse> {
    let user = qctx.deps.api.addr_validate(&params.user)?;

    let user_stake = get_user_stake(qctx.deps.storage, user.clone())?;

    Ok(UserTokenStakeResponse {
        user,
        staked_amount: user_stake,
    })
}

pub fn query_total_staked_amount(
    qctx: &QueryContext,
    params: TotalStakedAmountParams,
) -> TokenStakingResult<TotalStakedAmountResponse> {
    let total_staked_amount = match params.expiration {
        Expiration::AtHeight(height) => load_total_staked_at_height(qctx.deps.storage, height)?,
        Expiration::AtTime(time) => load_total_staked_at_time(qctx.deps.storage, time)?,
        Expiration::Never {} => load_total_staked(qctx.deps.storage)?,
    };

    Ok(TotalStakedAmountResponse {
        total_staked_amount,
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
