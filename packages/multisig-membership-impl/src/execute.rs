use crate::validate::dedup_user_weights;
use common::cw::Context;
use cosmwasm_std::{Addr, Order, Response, StdResult, Uint128};
use membership_common::member_weights::{get_member_weight, set_member_weight, MEMBER_WEIGHTS};
use membership_common::total_weight::{load_total_weight, save_total_weight};
use membership_common::validate::enterprise_governance_controller_only;
use membership_common::weight_change_hooks::report_weight_change_submsgs;
use membership_common_api::api::UserWeightChange;
use multisig_membership_api::api::{SetMembersMsg, UpdateMembersMsg};
use multisig_membership_api::error::MultisigMembershipResult;
use std::collections::HashMap;

/// Update members' weights. Only the current admin can execute this.
pub fn update_members(
    ctx: &mut Context,
    msg: UpdateMembersMsg,
) -> MultisigMembershipResult<Response> {
    // only governance controller can execute this
    enterprise_governance_controller_only(ctx, None)?;

    let deduped_edit_members = dedup_user_weights(ctx, msg.update_members)?;

    let mut total_weight = load_total_weight(ctx.deps.storage)?;

    let mut weight_changes: Vec<UserWeightChange> = vec![];

    for (member, weight) in deduped_edit_members {
        let old_weight = get_member_weight(ctx.deps.storage, member.clone())?;

        total_weight = total_weight - old_weight + weight;

        weight_changes.push(UserWeightChange {
            user: member.to_string(),
            old_weight,
            new_weight: weight,
        });

        set_member_weight(ctx.deps.storage, member, weight)?;
    }

    save_total_weight(ctx.deps.storage, &total_weight, &ctx.env.block)?;

    let report_weight_change_submsgs = report_weight_change_submsgs(ctx, weight_changes)?;

    Ok(Response::new()
        .add_attribute("action", "update_members")
        .add_submessages(report_weight_change_submsgs))
}

/// Clear existing members and replace with the given members' weights. Only the current admin can execute this.
pub fn set_members(ctx: &mut Context, msg: SetMembersMsg) -> MultisigMembershipResult<Response> {
    // only governance controller can execute this
    enterprise_governance_controller_only(ctx, None)?;

    let old_member_weights = MEMBER_WEIGHTS()
        .idx
        .user
        .range(ctx.deps.storage, None, None, Order::Ascending)
        .map(|res| res.map(|(addr, weight)| (addr, weight.weight)))
        .collect::<StdResult<HashMap<Addr, Uint128>>>()?;

    MEMBER_WEIGHTS().clear(ctx.deps.storage);

    let deduped_edit_members = dedup_user_weights(ctx, msg.new_members)?;

    let mut total_weight = Uint128::zero();

    let mut weight_changes: Vec<UserWeightChange> = vec![];

    for (member, weight) in deduped_edit_members {
        total_weight += weight;

        let old_weight = old_member_weights.get(&member).cloned().unwrap_or_default();
        weight_changes.push(UserWeightChange {
            user: member.to_string(),
            old_weight,
            new_weight: weight,
        });

        set_member_weight(ctx.deps.storage, member, weight)?;
    }

    save_total_weight(ctx.deps.storage, &total_weight, &ctx.env.block)?;

    let report_weight_change_submsgs = report_weight_change_submsgs(ctx, weight_changes)?;

    Ok(Response::new()
        .add_attribute("action", "set_members")
        .add_submessages(report_weight_change_submsgs))
}
