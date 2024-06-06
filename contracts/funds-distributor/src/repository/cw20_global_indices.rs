use crate::repository::era_repository::EraId;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal};
use cw_storage_plus::{Index, IndexList, IndexedMap, MultiIndex};
use funds_distributor_api::api::DistributionType;
use funds_distributor_api::api::DistributionType::{Membership, Participation};

// TODO: perhaps a regular Map<(EraId, Addr), Decimal> would do, EraId can be prefixed with it, no need for index
// TODO: double check to ensure the storage namespaces don't collide with the other distributions namespaces

const NAMESPACE_CW20_GLOBAL_INDICES_MEMBERSHIP: &str = "cw20_global_indices_membership";
const NAMESPACE_CW20_GLOBAL_INDICES_MEMBERSHIP_IDX_ERA: &str =
    "cw20_global_indices_membership__era";

const NAMESPACE_CW20_GLOBAL_INDICES_PARTICIPATION: &str = "cw20_global_indices_participation";
const NAMESPACE_CW20_GLOBAL_INDICES_PARTICIPATION_IDX_ERA: &str =
    "cw20_global_indices_participation__era";

#[cw_serde]
/// State of a single user's specific CW20 rewards.
pub struct Cw20GlobalIndex {
    pub era_id: EraId,
    pub cw20_asset: Addr,
    pub global_index: Decimal,
}

pub struct Cw20GlobalIndicesIndexes<'a> {
    pub era: MultiIndex<'a, EraId, Cw20GlobalIndex, (EraId, Addr)>,
    // TODO: perhaps add denom index?
}

impl IndexList<Cw20GlobalIndex> for Cw20GlobalIndicesIndexes<'_> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Cw20GlobalIndex>> + '_> {
        let v: Vec<&dyn Index<Cw20GlobalIndex>> = vec![&self.era];
        Box::new(v.into_iter())
    }
}

#[allow(non_snake_case)]
pub fn CW20_GLOBAL_INDICES<'a>(
    distribution_type: &DistributionType,
) -> IndexedMap<'a, (EraId, Addr), Cw20GlobalIndex, Cw20GlobalIndicesIndexes<'a>> {
    // TODO: can use Rc here instead of this filth
    let (namespace, namespace_era_idx) = match distribution_type {
        Membership => (
            NAMESPACE_CW20_GLOBAL_INDICES_MEMBERSHIP,
            NAMESPACE_CW20_GLOBAL_INDICES_MEMBERSHIP_IDX_ERA,
        ),
        Participation => (
            NAMESPACE_CW20_GLOBAL_INDICES_PARTICIPATION,
            NAMESPACE_CW20_GLOBAL_INDICES_PARTICIPATION_IDX_ERA,
        ),
    };

    let indexes = Cw20GlobalIndicesIndexes {
        era: MultiIndex::new(
            |_, global_index| global_index.era_id,
            namespace,
            namespace_era_idx,
        ),
    };

    IndexedMap::new(namespace, indexes)
}
