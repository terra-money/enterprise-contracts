use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Index, IndexList, IndexedMap, MultiIndex};

#[cw_serde]
// TODO: write a docs-style description for what this represents
pub struct Cw20Distribution {
    // TODO: rename to Cw20DistributionInfo?
    pub user: Addr,
    pub cw20_asset: Addr,
    pub user_index: Decimal,
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
            "cw20_distributions__staker",
        ),
    };
    IndexedMap::new("cw20_distributions", indexes)
}
