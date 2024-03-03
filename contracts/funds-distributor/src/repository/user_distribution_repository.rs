use cosmwasm_std::{Addr, Decimal, Deps, DepsMut, Uint128};
use funds_distributor_api::error::DistributorResult;
use crate::asset_types::{Cw20Asset, NativeAsset};
use crate::cw20_distributions::{CW20_DISTRIBUTIONS, Cw20Distribution};
use crate::native_distributions::{NATIVE_DISTRIBUTIONS, NativeDistribution};

pub struct UserDistributionInfo {
    pub user_index: Decimal,
    pub pending_rewards: Uint128,
}

pub trait UserDistributionRepository<A> {
    fn get_distribution_info(&self, asset: A, user: Addr) -> DistributorResult<Option<UserDistributionInfo>>;
}

pub trait UserDistributionRepositoryMut<A>: UserDistributionRepository<A> {
    fn set_distribution_info(&mut self, asset: A, user: Addr, distribution_info: UserDistributionInfo) -> DistributorResult<()>;
}

////////////////////////////
////////// NATIVE //////////
////////////////////////////

pub struct NativeUserDistributionRepository<'a> {
    deps: Deps<'a>,
}

impl UserDistributionRepository<NativeAsset> for NativeUserDistributionRepository<'_> {
    fn get_distribution_info(&self, asset: NativeAsset, user: Addr) -> DistributorResult<Option<UserDistributionInfo>> {
        let distribution = NATIVE_DISTRIBUTIONS().may_load(self.deps.storage, (user, asset))?;
        Ok(distribution.map(|it| UserDistributionInfo { user_index: it.user_index, pending_rewards: it.pending_rewards })) // TODO: perhaps introduce a 'From' for this conversion
    }
}

pub struct NativeUserDistributionRepositoryMut<'a> {
    deps: DepsMut<'a>,
}

impl NativeUserDistributionRepositoryMut<'_> {
    pub fn as_ref(&self) -> NativeUserDistributionRepository {
        NativeUserDistributionRepository { deps: self.deps.as_ref() }
    }
}

impl UserDistributionRepository<NativeAsset> for NativeUserDistributionRepositoryMut<'_> {
    fn get_distribution_info(&self, asset: NativeAsset, user: Addr) -> DistributorResult<Option<UserDistributionInfo>> {
        self.as_ref().get_distribution_info(asset, user)
    }
}

impl UserDistributionRepositoryMut<NativeAsset> for NativeUserDistributionRepositoryMut<'_> {
    fn set_distribution_info(&mut self, asset: NativeAsset, user: Addr, distribution_info: UserDistributionInfo) -> DistributorResult<()> {
        NATIVE_DISTRIBUTIONS().save(
            self.deps.storage,
            (user.clone(), asset.clone()),
            &NativeDistribution {
                user,
                denom: asset,
                user_index: distribution_info.user_index,
                pending_rewards: distribution_info.pending_rewards,
            },
        )?;
        Ok(())
    }
}

//////////////////////////
////////// CW20 //////////
//////////////////////////

pub struct Cw20UserDistributionRepository<'a> {
    deps: Deps<'a>,
}

impl UserDistributionRepository<Cw20Asset> for Cw20UserDistributionRepository<'_> {
    fn get_distribution_info(&self, asset: Cw20Asset, user: Addr) -> DistributorResult<Option<UserDistributionInfo>> {
        let distribution = CW20_DISTRIBUTIONS().may_load(self.deps.storage, (user, asset))?;
        Ok(distribution.map(|it| UserDistributionInfo { user_index: it.user_index, pending_rewards: it.pending_rewards })) // TODO: perhaps introduce a 'From' for this conversion
    }
}

pub struct Cw20UserDistributionRepositoryMut<'a> {
    deps: DepsMut<'a>,
}

impl Cw20UserDistributionRepositoryMut<'_> {
    pub fn as_ref(&self) -> Cw20UserDistributionRepository {
        Cw20UserDistributionRepository { deps: self.deps.as_ref() }
    }
}

impl UserDistributionRepository<Cw20Asset> for Cw20UserDistributionRepositoryMut<'_> {
    fn get_distribution_info(&self, asset: Cw20Asset, user: Addr) -> DistributorResult<Option<UserDistributionInfo>> {
        self.as_ref().get_distribution_info(asset, user)
    }
}

impl UserDistributionRepositoryMut<Cw20Asset> for Cw20UserDistributionRepositoryMut<'_> {
    fn set_distribution_info(&mut self, asset: Cw20Asset, user: Addr, distribution_info: UserDistributionInfo) -> DistributorResult<()> {
        CW20_DISTRIBUTIONS().save(
            self.deps.storage,
            (user.clone(), asset.clone()),
            &Cw20Distribution {
                user,
                cw20_asset: asset,
                user_index: distribution_info.user_index,
                pending_rewards: distribution_info.pending_rewards,
            },
        )?;
        Ok(())
    }
}