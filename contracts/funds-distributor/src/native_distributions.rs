use crate::repository::user_distribution_repository::UserDistributionInfo;
use crate::state::EraId;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Index, IndexList, IndexedMap, MultiIndex};
use funds_distributor_api::api::DistributionType;
use funds_distributor_api::api::DistributionType::{Membership, Participation};

// TODO: having to use these constants is ugly, but Rust is uglier
const NAMESPACE_MEMBERSHIP: &str = "native_distributions";
const NAMESPACE_MEMBERSHIP_USER_IDX: &str = "native_distributions__user";

const NAMESPACE_PARTICIPATION: &str = "native_distributions_participation";
const NAMESPACE_PARTICIPATION_USER_IDX: &str = "native_distributions_participation__user";

#[cw_serde]
/// State of a single user's specific native rewards.
pub struct NativeDistribution {
    pub user: Addr,
    pub era_id: EraId,
    pub denom: String,
    /// The last global index at which the user's pending rewards were calculated
    pub user_index: Decimal,
    /// User's unclaimed rewards
    pub pending_rewards: Uint128,
}

impl From<NativeDistribution> for UserDistributionInfo {
    fn from(value: NativeDistribution) -> Self {
        UserDistributionInfo {
            user_index: value.user_index,
            pending_rewards: value.pending_rewards,
        }
    }
}

pub struct NativeDistributionIndexes<'a> {
    pub user: MultiIndex<'a, Addr, NativeDistribution, (Addr, EraId, String)>,
}

impl IndexList<NativeDistribution> for NativeDistributionIndexes<'_> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<NativeDistribution>> + '_> {
        let v: Vec<&dyn Index<NativeDistribution>> = vec![&self.user];
        Box::new(v.into_iter())
    }
}

#[allow(non_snake_case)]
pub fn NATIVE_DISTRIBUTIONS<'a>(
    distribution_type: DistributionType,
) -> IndexedMap<'a, (Addr, EraId, String), NativeDistribution, NativeDistributionIndexes<'a>> {
    let (namespace, namespace_user_idx) = match distribution_type {
        Membership => (NAMESPACE_MEMBERSHIP, NAMESPACE_MEMBERSHIP_USER_IDX),
        Participation => (NAMESPACE_PARTICIPATION, NAMESPACE_PARTICIPATION_USER_IDX),
    };

    let indexes = NativeDistributionIndexes {
        user: MultiIndex::new(
            |_, native_distribution| native_distribution.user.clone(),
            namespace,
            namespace_user_idx,
        ),
    };
    IndexedMap::new(namespace, indexes)
}

// convenience trait to unify duplicate code between this and CW20 distributions
impl From<NativeDistribution> for (Decimal, Uint128) {
    fn from(item: NativeDistribution) -> Self {
        (item.user_index, item.pending_rewards)
    }
}
