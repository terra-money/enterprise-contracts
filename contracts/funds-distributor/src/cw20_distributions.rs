use crate::repository::era_repository::EraId;
use crate::repository::user_distribution_repository::UserDistributionInfo;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Index, IndexList, IndexedMap, MultiIndex};
use funds_distributor_api::api::DistributionType;
use DistributionType::{Membership, Participation};

// TODO: having to use these constants is ugly, but Rust is uglier
const NAMESPACE_MEMBERSHIP: &str = "cw20_distributions";
const NAMESPACE_MEMBERSHIP_USER_IDX: &str = "cw20_distributions__user";

const NAMESPACE_PARTICIPATION: &str = "cw20_distributions_participation";
const NAMESPACE_PARTICIPATION_USER_IDX: &str = "cw20_distributions_participation__user";

#[cw_serde]
/// State of a single user's specific CW20 rewards.
pub struct Cw20Distribution {
    pub user: Addr,
    pub era_id: EraId,
    pub cw20_asset: Addr,
    /// The last global index at which the user's pending rewards were calculated
    pub user_index: Decimal,
    /// User's unclaimed rewards
    pub pending_rewards: Uint128,
}

impl From<Cw20Distribution> for UserDistributionInfo {
    fn from(value: Cw20Distribution) -> Self {
        UserDistributionInfo {
            user_index: value.user_index,
            pending_rewards: value.pending_rewards,
        }
    }
}

pub struct Cw20DistributionIndexes<'a> {
    pub user: MultiIndex<'a, Addr, Cw20Distribution, (Addr, EraId, Addr)>,
}

impl IndexList<Cw20Distribution> for Cw20DistributionIndexes<'_> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Cw20Distribution>> + '_> {
        let v: Vec<&dyn Index<Cw20Distribution>> = vec![&self.user];
        Box::new(v.into_iter())
    }
}

#[allow(non_snake_case)]
pub fn CW20_DISTRIBUTIONS<'a>(
    distribution_type: DistributionType,
) -> IndexedMap<'a, (Addr, EraId, Addr), Cw20Distribution, Cw20DistributionIndexes<'a>> {
    let (namespace, namespace_user_idx) = match distribution_type {
        Membership => (NAMESPACE_MEMBERSHIP, NAMESPACE_MEMBERSHIP_USER_IDX),
        Participation => (NAMESPACE_PARTICIPATION, NAMESPACE_PARTICIPATION_USER_IDX),
    };

    let indexes = Cw20DistributionIndexes {
        user: MultiIndex::new(
            |_, cw20_distribution| cw20_distribution.user.clone(),
            namespace,
            namespace_user_idx,
        ),
    };
    IndexedMap::new(namespace, indexes)
}

// convenience trait to unify duplicate code between this and native distributions
impl From<Cw20Distribution> for (Decimal, Uint128) {
    fn from(item: Cw20Distribution) -> Self {
        (item.user_index, item.pending_rewards)
    }
}
