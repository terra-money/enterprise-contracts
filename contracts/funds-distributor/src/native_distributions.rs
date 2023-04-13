use crate::rewards::calculate_user_reward;
use crate::state::NATIVE_GLOBAL_INDICES;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{Addr, Decimal, DepsMut, StdResult, Uint128};
use cw_storage_plus::{Index, IndexList, IndexedMap, MultiIndex};
use funds_distributor_api::error::DistributorResult;

#[cw_serde]
/// State of a single user's specific native rewards.
pub struct NativeDistribution {
    pub user: Addr,
    pub denom: String,
    /// The last global index at which the user's pending rewards were calculated
    pub user_index: Decimal,
    /// User's unclaimed rewards
    pub pending_rewards: Uint128,
}

pub struct NativeDistributionIndexes<'a> {
    pub user: MultiIndex<'a, Addr, NativeDistribution, (Addr, String)>,
}

impl IndexList<NativeDistribution> for NativeDistributionIndexes<'_> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<NativeDistribution>> + '_> {
        let v: Vec<&dyn Index<NativeDistribution>> = vec![&self.user];
        Box::new(v.into_iter())
    }
}

#[allow(non_snake_case)]
pub fn NATIVE_DISTRIBUTIONS<'a>(
) -> IndexedMap<'a, (Addr, String), NativeDistribution, NativeDistributionIndexes<'a>> {
    let indexes = NativeDistributionIndexes {
        user: MultiIndex::new(
            |_, native_distribution| native_distribution.user.clone(),
            "native_distributions",
            "native_distributions__user",
        ),
    };
    IndexedMap::new("native_distributions", indexes)
}

// convenience trait to unify duplicate code between this and CW20 distributions
impl From<NativeDistribution> for (Decimal, Uint128) {
    fn from(item: NativeDistribution) -> Self {
        (item.user_index, item.pending_rewards)
    }
}

/// Updates user's reward indices for all native assets.
///
/// Will calculate newly pending rewards since the last update to the user's reward index until now,
/// using their last weight to calculate the newly accrued rewards.
pub fn update_user_native_distributions(
    deps: DepsMut,
    user: Addr,
    old_user_weight: Uint128,
) -> DistributorResult<()> {
    let native_global_indices = NATIVE_GLOBAL_INDICES
        .range(deps.storage, None, None, Ascending)
        .collect::<StdResult<Vec<(String, Decimal)>>>()?;

    for (denom, global_index) in native_global_indices {
        let distribution =
            NATIVE_DISTRIBUTIONS().may_load(deps.storage, (user.clone(), denom.clone()))?;

        let reward = calculate_user_reward(global_index, distribution, old_user_weight);

        NATIVE_DISTRIBUTIONS().save(
            deps.storage,
            (user.clone(), denom.clone()),
            &NativeDistribution {
                user: user.clone(),
                denom,
                user_index: global_index,
                pending_rewards: reward,
            },
        )?;
    }

    Ok(())
}
