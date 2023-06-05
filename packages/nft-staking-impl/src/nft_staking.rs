use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, StdResult, Storage, Uint128};
use cw_storage_plus::{Index, IndexList, IndexedMap, Map, MultiIndex};
use nft_staking_api::api::NftTokenId;

pub const USER_TOTAL_STAKED: Map<Addr, Uint128> = Map::new("user_total_staked");

pub fn get_user_total_stake(storage: &dyn Storage, user: Addr) -> StdResult<Uint128> {
    Ok(USER_TOTAL_STAKED
        .may_load(storage, user)?
        .unwrap_or_default())
}

pub fn increment_user_total_staked(storage: &mut dyn Storage, user: Addr) -> StdResult<Uint128> {
    let user_total_staked = USER_TOTAL_STAKED
        .may_load(storage, user.clone())?
        .unwrap_or_default();
    let new_user_total_staked = user_total_staked + Uint128::one();
    USER_TOTAL_STAKED.save(storage, user, &(new_user_total_staked))?;

    Ok(new_user_total_staked)
}

pub fn decrement_user_total_staked(
    storage: &mut dyn Storage,
    user: Addr,
    amount: Uint128,
) -> StdResult<Uint128> {
    let user_total_staked = USER_TOTAL_STAKED
        .may_load(storage, user.clone())?
        .unwrap_or_default();
    let new_user_total_staked = user_total_staked - amount;
    USER_TOTAL_STAKED.save(storage, user, &(new_user_total_staked))?;

    Ok(new_user_total_staked)
}

#[cw_serde]
pub struct NftStake {
    pub staker: Addr,
    pub token_id: NftTokenId,
}

pub struct NftStakesIndexes<'a> {
    pub staker: MultiIndex<'a, Addr, NftStake, String>,
}

impl IndexList<NftStake> for NftStakesIndexes<'_> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<NftStake>> + '_> {
        let v: Vec<&dyn Index<NftStake>> = vec![&self.staker];
        Box::new(v.into_iter())
    }
}

#[allow(non_snake_case)]
pub fn NFT_STAKES<'a>() -> IndexedMap<'a, String, NftStake, NftStakesIndexes<'a>> {
    let indexes = NftStakesIndexes {
        staker: MultiIndex::new(
            |_, nft_stake| nft_stake.staker.clone(),
            "nft_stakes",
            "nft_stakes__staker",
        ),
    };
    IndexedMap::new("nft_stakes", indexes)
}

pub fn save_nft_stake(store: &mut dyn Storage, nft_stake: &NftStake) -> StdResult<()> {
    NFT_STAKES().save(store, nft_stake.token_id.clone(), nft_stake)
}
