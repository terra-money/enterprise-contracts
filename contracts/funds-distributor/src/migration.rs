use crate::eligibility::{update_minimum_eligible_weight, MINIMUM_ELIGIBLE_WEIGHT};
use crate::user_weights::{EFFECTIVE_USER_WEIGHTS, USER_WEIGHTS};
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{Addr, DepsMut, StdResult, Uint128};
use funds_distributor_api::error::DistributorResult;

pub fn migrate_v1_to_v2(
    deps: DepsMut,
    minimum_eligible_weight: Option<Uint128>,
) -> DistributorResult<()> {
    // set all effective user weights to actual weights
    USER_WEIGHTS
        .range(deps.storage, None, None, Ascending)
        .collect::<StdResult<Vec<(Addr, Uint128)>>>()?
        .into_iter()
        .try_for_each(|(user, weight)| EFFECTIVE_USER_WEIGHTS.save(deps.storage, user, &weight))?;

    let minimum_eligible_weight = minimum_eligible_weight.unwrap_or_default();

    MINIMUM_ELIGIBLE_WEIGHT.save(deps.storage, &minimum_eligible_weight)?;

    // update minimum eligible weight, acting as if the previously set minimum was 0
    update_minimum_eligible_weight(deps, Uint128::zero(), minimum_eligible_weight)?;

    Ok(())
}
