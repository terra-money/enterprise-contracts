use crate::asset_types::RewardAsset;
use crate::state::{CW20_GLOBAL_INDICES, NATIVE_GLOBAL_INDICES};
use cosmwasm_std::{Decimal, Deps, DepsMut};
use funds_distributor_api::error::DistributorResult;
use RewardAsset::{Cw20, Native};

pub trait AssetDistributionRepository {
    fn get_global_index(&self, asset: RewardAsset) -> DistributorResult<Option<Decimal>>;
}

pub trait AssetDistributionRepositoryMut: AssetDistributionRepository {
    fn set_global_index(
        &mut self,
        asset: RewardAsset,
        global_index: Decimal,
    ) -> DistributorResult<()>;
}

/////////////////////////////
////////// GENERAL //////////
/////////////////////////////

pub struct GeneralDistributionRepository<'a> {
    deps: Deps<'a>,
}

impl<'a> AssetDistributionRepository for GeneralDistributionRepository<'a> {
    fn get_global_index(&self, asset: RewardAsset) -> DistributorResult<Option<Decimal>> {
        let global_index = match asset {
            Native { denom } => NATIVE_GLOBAL_INDICES.may_load(self.deps.storage, denom)?,
            Cw20 { addr } => CW20_GLOBAL_INDICES.may_load(self.deps.storage, addr)?,
        };
        Ok(global_index)
    }
}

pub struct GeneralDistributionRepositoryMut<'a> {
    deps: DepsMut<'a>,
}

impl<'a> AssetDistributionRepository for GeneralDistributionRepositoryMut<'a> {
    fn get_global_index(&self, asset: RewardAsset) -> DistributorResult<Option<Decimal>> {
        self.as_ref().get_global_index(asset)
    }
}

impl<'a> AssetDistributionRepositoryMut for GeneralDistributionRepositoryMut<'a> {
    fn set_global_index(
        &mut self,
        asset: RewardAsset,
        global_index: Decimal,
    ) -> DistributorResult<()> {
        match asset {
            Native { denom } => {
                NATIVE_GLOBAL_INDICES.save(self.deps.storage, denom, &global_index)?;
            }
            Cw20 { addr } => {
                CW20_GLOBAL_INDICES.save(self.deps.storage, addr, &global_index)?;
            }
        }
        Ok(())
    }
}

impl<'a> GeneralDistributionRepositoryMut<'a> {
    pub fn as_ref(&self) -> GeneralDistributionRepository {
        GeneralDistributionRepository {
            deps: self.deps.as_ref(),
        }
    }
}

pub fn asset_distribution_repository(deps: Deps) -> GeneralDistributionRepository {
    GeneralDistributionRepository { deps }
}

pub fn asset_distribution_repository_mut(deps: DepsMut) -> GeneralDistributionRepositoryMut {
    GeneralDistributionRepositoryMut { deps }
}
