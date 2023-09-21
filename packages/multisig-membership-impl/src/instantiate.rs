use crate::validate::dedup_user_weights;
use common::cw::Context;
use cosmwasm_std::Uint128;
use membership_common::enterprise_contract::set_enterprise_contract;
use membership_common::member_weights::{get_member_weight, set_member_weight};
use membership_common::total_weight::{load_total_weight, save_total_weight};
use membership_common::weight_change_hooks::save_initial_weight_change_hooks;
use multisig_membership_api::api::UserWeight;
use multisig_membership_api::error::MultisigMembershipResult;
use multisig_membership_api::msg::InstantiateMsg;

pub fn instantiate(ctx: &mut Context, msg: InstantiateMsg) -> MultisigMembershipResult<()> {
    set_enterprise_contract(ctx.deps.branch(), msg.enterprise_contract)?;

    if let Some(initial_weights) = msg.initial_weights {
        save_initial_weights(ctx, initial_weights)?;
    } else {
        save_total_weight(ctx.deps.storage, &Uint128::zero(), &ctx.env.block)?;
    }

    if let Some(weight_change_hooks) = msg.weight_change_hooks {
        save_initial_weight_change_hooks(ctx, weight_change_hooks)?;
    }

    Ok(())
}

fn save_initial_weights(
    ctx: &mut Context,
    initial_weights: Vec<UserWeight>,
) -> MultisigMembershipResult<()> {
    let deduped_weights = dedup_user_weights(ctx, initial_weights)?;

    let mut total_weight = load_total_weight(ctx.deps.storage)?;

    for (user, weight) in deduped_weights {
        let existing_weight = get_member_weight(ctx.deps.storage, user.clone())?;
        set_member_weight(ctx.deps.storage, user, weight)?;

        total_weight = total_weight - existing_weight + weight;
    }

    save_total_weight(ctx.deps.storage, &total_weight, &ctx.env.block)?;

    Ok(())
}
