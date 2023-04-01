use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Index, IndexList, IndexedMap, MultiIndex};

#[cw_serde]
/// State of a single user's specific CW20 rewards.
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

impl From<NativeDistribution> for (Decimal, Uint128) {
    fn from(item: NativeDistribution) -> Self {
        (item.user_index, item.pending_rewards)
    }
}
