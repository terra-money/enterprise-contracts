use crate::member_weights::MEMBER_WEIGHTS;
use crate::validate::dedup_user_weights;
use common::cw::Context;
use cosmwasm_std::Uint128;
use membership_common::admin::ADMIN;
use membership_common::total_weight::{load_total_weight, save_total_weight};
use multisig_membership_api::api::UserWeight;
use multisig_membership_api::error::MultisigMembershipResult;
use multisig_membership_api::msg::InstantiateMsg;

pub fn instantiate(ctx: &mut Context, msg: InstantiateMsg) -> MultisigMembershipResult<()> {
    let admin = ctx.deps.api.addr_validate(&msg.admin)?;
    ADMIN.save(ctx.deps.storage, &admin)?;

    if let Some(initial_weights) = msg.initial_weights {
        save_initial_weights(ctx, initial_weights)?;
    } else {
        save_total_weight(ctx.deps.storage, &Uint128::zero(), &ctx.env.block)?;
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
        let existing_weight = MEMBER_WEIGHTS
            .may_load(ctx.deps.storage, user.clone())?
            .unwrap_or_default();
        MEMBER_WEIGHTS.save(ctx.deps.storage, user, &weight)?;

        total_weight = total_weight - existing_weight + weight;
    }

    save_total_weight(ctx.deps.storage, &total_weight, &ctx.env.block)?;

    Ok(())
}
