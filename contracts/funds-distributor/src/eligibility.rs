use crate::distributing::query_enterprise_components;
use crate::repository::era_repository::{get_current_era, set_current_era, EraId};
use crate::repository::weights_repository::weights_repository_mut;
use crate::state::ADMIN;
use common::cw::{Context, QueryContext};
use cosmwasm_std::{DepsMut, Response, Uint128};
use cw_storage_plus::Map;
use funds_distributor_api::api::DistributionType::Membership;
use funds_distributor_api::api::{MinimumEligibleWeightResponse, UpdateMinimumEligibleWeightMsg};
use funds_distributor_api::error::DistributorError::Unauthorized;
use funds_distributor_api::error::DistributorResult;
use funds_distributor_api::response::execute_update_minimum_eligible_weight_response;
use membership_common_api::api::{TotalWeightAboveParams, TotalWeightResponse};
use membership_common_api::msg::QueryMsg::TotalWeightAbove;

/// Minimum weight that a user should have to be eligible for receiving rewards.
/// Defined per-era.
pub const MINIMUM_ELIGIBLE_WEIGHT: Map<EraId, Uint128> = Map::new("minimum_eligible_weight");

pub fn execute_update_minimum_eligible_weight(
    ctx: &mut Context,
    msg: UpdateMinimumEligibleWeightMsg,
) -> DistributorResult<Response> {
    let admin = ADMIN.load(ctx.deps.storage)?;

    if ctx.info.sender != admin {
        return Err(Unauthorized);
    }

    let current_era = get_current_era(ctx.deps.as_ref(), Membership)?;

    let old_minimum_weight = MINIMUM_ELIGIBLE_WEIGHT.load(ctx.deps.storage, current_era)?;
    let new_minimum_weight = msg.minimum_eligible_weight;

    update_minimum_eligible_weight(
        ctx.deps.branch(),
        current_era,
        old_minimum_weight,
        new_minimum_weight,
    )?;

    Ok(execute_update_minimum_eligible_weight_response(
        old_minimum_weight,
        new_minimum_weight,
    ))
}

/// Update minimum eligible weight for rewards by going through all the users
/// between the old and the new minimum and updating their effective weight (to either their
/// actual weight, or 0, depending on whether they're above or below the new minimum).
// TODO: the name is very similar to the above, but this does not check for unauthorized use; reveal this through the name somehow
pub fn update_minimum_eligible_weight(
    mut deps: DepsMut,
    current_era: EraId,
    old_minimum_weight: Uint128,
    new_minimum_weight: Uint128,
) -> DistributorResult<()> {
    if old_minimum_weight == new_minimum_weight {
        return Ok(());
    }

    let next_era = current_era + 1;
    set_current_era(deps.branch(), next_era, Membership)?;

    MINIMUM_ELIGIBLE_WEIGHT.save(deps.storage, next_era, &new_minimum_weight)?;

    let membership_contract = query_enterprise_components(deps.as_ref())?.membership_contract;

    let total_weight_above: TotalWeightResponse = deps.querier.query_wasm_smart(
        membership_contract.to_string(),
        &TotalWeightAbove(TotalWeightAboveParams {
            above_weight_inclusive: new_minimum_weight,
        }),
    )?;

    weights_repository_mut(deps.branch(), Membership)
        .set_total_weight(total_weight_above.total_weight, next_era)?;

    Ok(())
}

pub fn query_minimum_eligible_weight(
    qctx: QueryContext,
) -> DistributorResult<MinimumEligibleWeightResponse> {
    let current_era = get_current_era(qctx.deps, Membership)?;
    let minimum_eligible_weight = MINIMUM_ELIGIBLE_WEIGHT.load(qctx.deps.storage, current_era)?;

    Ok(MinimumEligibleWeightResponse {
        minimum_eligible_weight,
    })
}
