use crate::config::CONFIG;
use common::cw::Context;
use cosmwasm_std::{Addr, StdResult, Uint128};
use itertools::Itertools;
use multisig_membership_api::api::UserWeight;
use multisig_membership_api::error::MultisigMembershipError::Unauthorized;
use multisig_membership_api::error::{MultisigMembershipError, MultisigMembershipResult};
use MultisigMembershipError::DuplicateUserWeightFound;

/// Assert that the caller is admin.
/// If the validation succeeds, returns the admin address.
pub fn admin_caller_only(ctx: &Context) -> MultisigMembershipResult<Addr> {
    let config = CONFIG.load(ctx.deps.storage)?;
    let admin = config.admin;

    // only current admin can change the admin
    if ctx.info.sender != admin {
        return Err(Unauthorized);
    }

    Ok(admin)
}

/// Will validate each of the user addresses, and fail if there are any duplicate addresses found.
/// Otherwise, returns a vector of (user Addr, weight).
pub fn dedup_user_weights(
    ctx: &Context,
    user_weights: Vec<UserWeight>,
) -> MultisigMembershipResult<Vec<(Addr, Uint128)>> {
    let user_weights_length = user_weights.len();

    let deduped_user_weights: Vec<(Addr, Uint128)> = user_weights
        .into_iter()
        // validate each of the user addresses
        .map(|user_weight| {
            ctx.deps
                .api
                .addr_validate(&user_weight.user)
                .map(|user| (user, user_weight.weight))
        })
        .collect::<StdResult<Vec<(Addr, Uint128)>>>()?
        .into_iter()
        // de-duplicate the vector by user address
        .unique_by(|(user, _)| user.clone())
        .collect();

    if deduped_user_weights.len() != user_weights_length {
        // if there are less elements in the de-duplicated vector, it means there were duplicates
        return Err(DuplicateUserWeightFound);
    }

    Ok(deduped_user_weights)
}
