use common::cw::QueryContext;
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{StdResult, Uint128};
use itertools::Itertools;
use std::ops::Add;

use crate::api::Cw20Claim;
use crate::claims::CLAIMS;

use crate::execute::is_releasable;
use crate::state::{STAKES, TOTAL_STAKED};

// TODO: tests
// TODO: docs
pub fn query_stake(qctx: QueryContext, owner: String) -> StdResult<Uint128> {
    let owner = qctx.deps.api.addr_validate(&owner)?;

    let stake = STAKES
        .may_load(qctx.deps.storage, owner)?
        .unwrap_or_default();

    Ok(stake)
}

// TODO: tests
// TODO: docs
pub fn query_total_staked(qctx: QueryContext) -> StdResult<Uint128> {
    let total_staked = TOTAL_STAKED
        .may_load(qctx.deps.storage)?
        .unwrap_or_default();

    Ok(total_staked)
}

// TODO: tests
// TODO: docs
pub fn query_claims(qctx: QueryContext, owner: String) -> StdResult<Vec<Cw20Claim>> {
    let owner = qctx.deps.api.addr_validate(&owner)?;

    let claims = CLAIMS()
        .idx
        .user
        .prefix(owner)
        .range(qctx.deps.storage, None, None, Ascending)
        .collect::<StdResult<Vec<(u64, Cw20Claim)>>>()?
        .into_iter()
        .map(|(_, claim)| claim)
        .collect_vec();

    Ok(claims)
}

// TODO: tests
// TODO: docs
pub fn query_releasable_amount(qctx: QueryContext, owner: String) -> StdResult<Uint128> {
    let block = qctx.env.block.clone();
    let claims = query_claims(qctx, owner)?;

    let releasable_claims = claims.into_iter().fold(Uint128::zero(), |accum, claim| {
        if is_releasable(&claim, &block) {
            accum.add(claim.amount)
        } else {
            accum
        }
    });

    Ok(releasable_claims)
}
