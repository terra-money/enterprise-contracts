use crate::asset_types::RewardAsset;
use crate::state::{CW20_GLOBAL_INDICES, NATIVE_GLOBAL_INDICES};
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{Decimal, Deps, DepsMut};
use funds_distributor_api::api::DistributionType;
use funds_distributor_api::error::DistributorResult;
use RewardAsset::{Cw20, Native};

pub trait AssetDistributionRepository {
    fn get_global_index(&self, asset: RewardAsset) -> DistributorResult<Option<Decimal>>;

    fn get_all_global_indices(&self) -> DistributorResult<Vec<(RewardAsset, Decimal)>>;
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

    fn get_all_global_indices(&self) -> DistributorResult<Vec<(RewardAsset, Decimal)>> {
        let mut global_indices = vec![];

        NATIVE_GLOBAL_INDICES
            .range(self.deps.storage, None, None, Ascending)
            .try_for_each(|res| match res {
                Ok((denom, global_index)) => {
                    global_indices.push((RewardAsset::native(denom), global_index));
                    Ok(())
                }
                Err(e) => Err(e),
            })?;

        CW20_GLOBAL_INDICES
            .range(self.deps.storage, None, None, Ascending)
            .try_for_each(|res| match res {
                Ok((cw20, global_index)) => {
                    global_indices.push((RewardAsset::cw20(cw20), global_index));
                    Ok(())
                }
                Err(e) => Err(e),
            })?;

        Ok(global_indices)
    }
}

pub struct GeneralDistributionRepositoryMut<'a> {
    deps: DepsMut<'a>,
}

impl<'a> AssetDistributionRepository for GeneralDistributionRepositoryMut<'a> {
    fn get_global_index(&self, asset: RewardAsset) -> DistributorResult<Option<Decimal>> {
        self.as_ref().get_global_index(asset)
    }

    fn get_all_global_indices(&self) -> DistributorResult<Vec<(RewardAsset, Decimal)>> {
        self.as_ref().get_all_global_indices()
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

pub fn asset_distribution_repository(
    deps: Deps,
    distribution_type: DistributionType,
) -> GeneralDistributionRepository {
    GeneralDistributionRepository { deps }
}

pub fn asset_distribution_repository_mut(
    deps: DepsMut,
    distribution_type: DistributionType,
) -> GeneralDistributionRepositoryMut {
    GeneralDistributionRepositoryMut { deps }
}
