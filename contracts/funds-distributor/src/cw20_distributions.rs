use crate::rewards::calculate_user_reward;
use crate::state::CW20_GLOBAL_INDICES;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{Addr, Decimal, DepsMut, StdResult, Uint128};
use cw_storage_plus::{Index, IndexList, IndexedMap, MultiIndex};
use funds_distributor_api::error::DistributorResult;

#[cw_serde]
/// State of a single user's specific CW20 rewards.
pub struct Cw20Distribution {
    pub user: Addr,
    pub cw20_asset: Addr,
    /// The last global index at which the user's pending rewards were calculated
    pub user_index: Decimal,
    /// User's unclaimed rewards
    pub pending_rewards: Uint128,
}

pub struct Cw20DistributionIndexes<'a> {
    pub user: MultiIndex<'a, Addr, Cw20Distribution, (Addr, Addr)>,
}

impl IndexList<Cw20Distribution> for Cw20DistributionIndexes<'_> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Cw20Distribution>> + '_> {
        let v: Vec<&dyn Index<Cw20Distribution>> = vec![&self.user];
        Box::new(v.into_iter())
    }
}

#[allow(non_snake_case)]
pub fn CW20_DISTRIBUTIONS<'a>(
) -> IndexedMap<'a, (Addr, Addr), Cw20Distribution, Cw20DistributionIndexes<'a>> {
    let indexes = Cw20DistributionIndexes {
        user: MultiIndex::new(
            |_, cw20_distribution| cw20_distribution.user.clone(),
            "cw20_distributions",
            "cw20_distributions__user",
        ),
    };
    IndexedMap::new("cw20_distributions", indexes)
}

// convenience trait to unify duplicate code between this and native distributions
impl From<Cw20Distribution> for (Decimal, Uint128) {
    fn from(item: Cw20Distribution) -> Self {
        (item.user_index, item.pending_rewards)
    }
}

/// Updates user's reward indices for all CW20 assets.
///
/// Will calculate newly pending rewards since the last update to the user's reward index until now,
/// using their last weight to calculate the newly accrued rewards.
pub fn update_user_cw20_distributions(
    deps: DepsMut,
    user: Addr,
    old_user_weight: Uint128,
) -> DistributorResult<()> {
    let cw20_global_indices = CW20_GLOBAL_INDICES
        .range(deps.storage, None, None, Ascending)
        .collect::<StdResult<Vec<(Addr, Decimal)>>>()?;

    for (cw20_asset, global_index) in cw20_global_indices {
        let distribution =
            CW20_DISTRIBUTIONS().may_load(deps.storage, (user.clone(), cw20_asset.clone()))?;

        let reward = calculate_user_reward(global_index, distribution, old_user_weight);

        CW20_DISTRIBUTIONS().save(
            deps.storage,
            (user.clone(), cw20_asset.clone()),
            &Cw20Distribution {
                user: user.clone(),
                cw20_asset,
                user_index: global_index,
                pending_rewards: reward,
            },
        )?;
    }

    Ok(())
}
