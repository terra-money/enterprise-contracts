use cosmwasm_std::{Decimal, Deps, DepsMut};
use funds_distributor_api::error::DistributorResult;
use crate::asset_types::{Cw20Asset, NativeAsset};
use crate::state::{CW20_GLOBAL_INDICES, NATIVE_GLOBAL_INDICES};

pub trait AssetDistributionRepository<A> {
    fn get_global_index(&self, asset: A) -> DistributorResult<Option<Decimal>>;
}

pub trait AssetDistributionRepositoryMut<A>: AssetDistributionRepository<A> {
    fn set_global_index(&mut self, asset: A, global_index: Decimal) -> DistributorResult<()>;
}

////////////////////////////
////////// NATIVE //////////
////////////////////////////

pub struct NativeDistributionRepository<'a> {
    deps: Deps<'a>,
}

impl<'a> AssetDistributionRepository<NativeAsset> for NativeDistributionRepository<'a> {
    fn get_global_index(&self, asset: NativeAsset) -> DistributorResult<Option<Decimal>> {
        let global_index = NATIVE_GLOBAL_INDICES
            .may_load(self.deps.storage, asset.clone())?;
        Ok(global_index)
    }
}

pub struct NativeDistributionRepositoryMut<'a> {
    deps: DepsMut<'a>,
}

impl<'a> AssetDistributionRepository<NativeAsset> for NativeDistributionRepositoryMut<'a> {
    fn get_global_index(&self, asset: NativeAsset) -> DistributorResult<Option<Decimal>> {
        self.as_ref().get_global_index(asset)
    }
}

impl<'a> AssetDistributionRepositoryMut<NativeAsset> for NativeDistributionRepositoryMut<'a> {
    fn set_global_index(&mut self, asset: NativeAsset, global_index: Decimal) -> DistributorResult<()> {
        NATIVE_GLOBAL_INDICES.save(self.deps.storage, asset.clone(), &global_index)?;
        Ok(())
    }
}

impl<'a> NativeDistributionRepositoryMut<'a> {
    pub fn as_ref(&self) -> NativeDistributionRepository {
        NativeDistributionRepository { deps: self.deps.as_ref() }
    }
}

//////////////////////////
////////// CW20 //////////
//////////////////////////

pub struct Cw20DistributionRepository<'a> {
    deps: Deps<'a>,
}

impl<'a> AssetDistributionRepository<Cw20Asset> for Cw20DistributionRepository<'a> {
    fn get_global_index(&self, asset: Cw20Asset) -> DistributorResult<Option<Decimal>> {
        let global_index = CW20_GLOBAL_INDICES
            .may_load(self.deps.storage, asset.clone())?;
        Ok(global_index)
    }
}

pub struct Cw20DistributionRepositoryMut<'a> {
    deps: DepsMut<'a>,
}

impl<'a> AssetDistributionRepository<Cw20Asset> for Cw20DistributionRepositoryMut<'a> {
    fn get_global_index(&self, asset: Cw20Asset) -> DistributorResult<Option<Decimal>> {
        self.as_ref().get_global_index(asset)
    }
}

impl<'a> AssetDistributionRepositoryMut<Cw20Asset> for Cw20DistributionRepositoryMut<'a> {
    fn set_global_index(&mut self, asset: Cw20Asset, global_index: Decimal) -> DistributorResult<()> {
        CW20_GLOBAL_INDICES.save(self.deps.storage, asset.clone(), &global_index)?;
        Ok(())
    }
}

impl<'a> Cw20DistributionRepositoryMut<'a> {
    pub fn as_ref(&self) -> Cw20DistributionRepository {
        Cw20DistributionRepository { deps: self.deps.as_ref() }
    }
}

// TODO: this only returns one type of repository
pub fn asset_distribution_repository(deps: Deps) -> NativeDistributionRepository {
    NativeDistributionRepository { deps }
}

// TODO: this only returns one type of repository
pub fn asset_distribution_repository_mut(deps: DepsMut) -> NativeDistributionRepositoryMut {
    NativeDistributionRepositoryMut { deps }
}