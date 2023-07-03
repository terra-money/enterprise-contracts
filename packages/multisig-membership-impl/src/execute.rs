use crate::member_weights::MEMBER_WEIGHTS;
use crate::total_weight::{load_total_weight, save_total_weight};
use crate::validate::dedup_user_weights;
use common::cw::Context;
use cosmwasm_std::{Response, Uint128};
use membership_common::admin::admin_caller_only;
use multisig_membership_api::api::{SetMembersMsg, UpdateMembersMsg};
use multisig_membership_api::error::MultisigMembershipResult;

/// Update members' weights. Only the current admin can execute this.
pub fn update_members(
    ctx: &mut Context,
    msg: UpdateMembersMsg,
) -> MultisigMembershipResult<Response> {
    // only admin can execute this
    admin_caller_only(ctx)?;

    let deduped_edit_members = dedup_user_weights(ctx, msg.update_members)?;

    let mut total_weight = load_total_weight(ctx.deps.storage)?;

    for (member, weight) in deduped_edit_members {
        let old_weight = MEMBER_WEIGHTS
            .may_load(ctx.deps.storage, member.clone())?
            .unwrap_or_default();

        total_weight = total_weight - old_weight + weight;

        MEMBER_WEIGHTS.save(ctx.deps.storage, member, &weight)?;
    }

    save_total_weight(ctx.deps.storage, &total_weight, &ctx.env.block)?;

    Ok(Response::new().add_attribute("action", "update_members"))
}

/// Clear existing members and replace with the given members' weights. Only the current admin can execute this.
pub fn set_members(ctx: &mut Context, msg: SetMembersMsg) -> MultisigMembershipResult<Response> {
    // only admin can execute this
    admin_caller_only(ctx)?;

    MEMBER_WEIGHTS.clear(ctx.deps.storage);

    let deduped_edit_members = dedup_user_weights(ctx, msg.new_members)?;

    let mut total_weight = Uint128::zero();

    for (member, weight) in deduped_edit_members {
        total_weight += weight;

        MEMBER_WEIGHTS.save(ctx.deps.storage, member, &weight)?;
    }

    save_total_weight(ctx.deps.storage, &total_weight, &ctx.env.block)?;

    Ok(Response::new().add_attribute("action", "set_members"))
}

// TODO: move to common
// /// Add an address to which weight changes will be reported. Only the current admin can execute this.
// pub fn add_weight_change_hook(
//     ctx: &mut Context,
//     msg: WeightChangeHookMsg,
// ) -> MultisigMembershipResult<Response> {
//     // only admin can execute this
//     admin_caller_only(ctx)?;
//
//     let hook_addr = ctx.deps.api.addr_validate(&msg.hook_addr)?;
//
//     WEIGHT_CHANGE_HOOKS.save(ctx.deps.storage, hook_addr.clone(), &())?;
//
//     Ok(Response::new()
//         .add_attribute("action", "add_weight_change_hook")
//         .add_attribute("hook_addr", hook_addr.to_string()))
// }
//
// /// Remove an address to which weight changes were being reported. Only the current admin can execute this.
// pub fn remove_weight_change_hook(
//     ctx: &mut Context,
//     msg: WeightChangeHookMsg,
// ) -> MultisigMembershipResult<Response> {
//     // only admin can execute this
//     admin_caller_only(ctx)?;
//
//     let hook_addr = ctx.deps.api.addr_validate(&msg.hook_addr)?;
//
//     WEIGHT_CHANGE_HOOKS.remove(ctx.deps.storage, hook_addr.clone());
//
//     Ok(Response::new()
//         .add_attribute("action", "remove_weight_change_hook")
//         .add_attribute("hook_addr", hook_addr.to_string()))
// }
