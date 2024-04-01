use crate::state::EraId;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Decimal;
use cw_storage_plus::{Index, IndexList, IndexedMap, MultiIndex};
use funds_distributor_api::api::DistributionType;
use funds_distributor_api::api::DistributionType::{Membership, Participation};
use std::rc::Rc;

// TODO: perhaps a regular Map<(EraId, String), Decimal> would do, EraId can be prefixed with it, no need for index
// TODO: double check to ensure the storage namespaces don't collide with the other distributions namespaces

const NAMESPACE_NATIVE_GLOBAL_INDICES_MEMBERSHIP: &str = "native_global_indices_membership";
const NAMESPACE_NATIVE_GLOBAL_INDICES_MEMBERSHIP_IDX_ERA: &str =
    "native_global_indices_membership__era";

const NAMESPACE_NATIVE_GLOBAL_INDICES_PARTICIPATION: &str = "native_global_indices_participation";
const NAMESPACE_NATIVE_GLOBAL_INDICES_PARTICIPATION_IDX_ERA: &str =
    "native_global_indices_participation__era";

#[cw_serde]
/// State of a single user's specific native rewards.
pub struct NativeGlobalIndex {
    pub era_id: EraId,
    pub denom: String,
    pub global_index: Decimal,
}

pub struct NativeGlobalIndicesIndexes<'a> {
    pub era: MultiIndex<'a, EraId, NativeGlobalIndex, (EraId, String)>,
    // TODO: perhaps add denom index?
}

impl IndexList<NativeGlobalIndex> for NativeGlobalIndicesIndexes<'_> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<NativeGlobalIndex>> + '_> {
        let v: Vec<&dyn Index<NativeGlobalIndex>> = vec![&self.era];
        Box::new(v.into_iter())
    }
}

#[allow(non_snake_case)]
pub fn NATIVE_GLOBAL_INDICES<'a>(
    distribution_type: &DistributionType,
) -> IndexedMap<'a, (EraId, String), NativeGlobalIndex, NativeGlobalIndicesIndexes<'a>> {
    // TODO: can use Rc here instead of this filth
    let (namespace, namespace_era_idx) = match distribution_type {
        Membership => (
            NAMESPACE_NATIVE_GLOBAL_INDICES_MEMBERSHIP,
            NAMESPACE_NATIVE_GLOBAL_INDICES_MEMBERSHIP_IDX_ERA,
        ),
        Participation => (
            NAMESPACE_NATIVE_GLOBAL_INDICES_PARTICIPATION,
            NAMESPACE_NATIVE_GLOBAL_INDICES_PARTICIPATION_IDX_ERA,
        ),
    };

    let indexes = NativeGlobalIndicesIndexes {
        era: MultiIndex::new(
            |_, global_index| global_index.era_id,
            namespace,
            namespace_era_idx,
        ),
    };

    IndexedMap::new(namespace, indexes)
}
