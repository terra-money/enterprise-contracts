use crate::validate::enterprise_governance_controller_only;
use common::cw::Context;
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{wasm_execute, Addr, Response, StdResult, SubMsg};
use cw_storage_plus::Map;
use membership_common_api::api::{UserWeightChange, WeightChangeHookMsg, WeightsChangedMsg};
use membership_common_api::error::MembershipResult;
use membership_common_api::msg::WeightChangeHook;

pub const WEIGHT_CHANGE_HOOKS: Map<Addr, ()> = Map::new("membership_common__weight_change_hooks");

/// Stores initial weight change hooks, without checking who the sender is.
pub fn save_initial_weight_change_hooks(
    ctx: &mut Context,
    weight_change_hooks: Vec<String>,
) -> MembershipResult<()> {
    for hook in weight_change_hooks {
        let hook_addr = ctx.deps.api.addr_validate(&hook)?;
        WEIGHT_CHANGE_HOOKS.save(ctx.deps.storage, hook_addr.clone(), &())?;
    }

    Ok(())
}

/// Add an address to which weight changes will be reported. Only the current admin can execute this.
pub fn add_weight_change_hook(
    ctx: &mut Context,
    msg: WeightChangeHookMsg,
) -> MembershipResult<Response> {
    // only governance controller can execute this
    enterprise_governance_controller_only(ctx, None)?;

    let hook_addr = ctx.deps.api.addr_validate(&msg.hook_addr)?;

    WEIGHT_CHANGE_HOOKS.save(ctx.deps.storage, hook_addr.clone(), &())?;

    Ok(Response::new()
        .add_attribute("action", "add_weight_change_hook")
        .add_attribute("hook_addr", hook_addr.to_string()))
}

/// Remove an address to which weight changes were being reported. Only the current admin can execute this.
pub fn remove_weight_change_hook(
    ctx: &mut Context,
    msg: WeightChangeHookMsg,
) -> MembershipResult<Response> {
    // only governance controller can execute this
    enterprise_governance_controller_only(ctx, None)?;

    let hook_addr = ctx.deps.api.addr_validate(&msg.hook_addr)?;

    WEIGHT_CHANGE_HOOKS.remove(ctx.deps.storage, hook_addr.clone());

    Ok(Response::new()
        .add_attribute("action", "remove_weight_change_hook")
        .add_attribute("hook_addr", hook_addr.to_string()))
}

/// Construct submsgs that send out updates to weight change hooks.
pub fn report_weight_change_submsgs(
    ctx: &mut Context,
    weight_changes: Vec<UserWeightChange>,
) -> MembershipResult<Vec<SubMsg>> {
    let hook_msg = WeightChangeHook::WeightsChanged(WeightsChangedMsg { weight_changes });

    let hook_submsgs = WEIGHT_CHANGE_HOOKS
        .range(ctx.deps.storage, None, None, Ascending)
        .collect::<StdResult<Vec<(Addr, ())>>>()?
        .into_iter()
        .map(|(addr, _)| wasm_execute(addr.to_string(), &hook_msg, vec![]))
        .map(|res| res.map(SubMsg::new))
        .collect::<StdResult<Vec<SubMsg>>>()?;

    Ok(hook_submsgs)
}
