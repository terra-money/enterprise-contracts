use crate::asset_types::RewardAsset;
use crate::asset_types::RewardAsset::{Cw20, Native};
use crate::repository::cw20_global_indices::{
    Cw20GlobalIndex, Cw20GlobalIndicesIndexes, CW20_GLOBAL_INDICES,
};
use crate::repository::era_repository::EraId;
use crate::repository::native_global_indices::{
    NativeGlobalIndex, NativeGlobalIndicesIndexes, NATIVE_GLOBAL_INDICES,
};
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{Addr, Decimal, Deps, DepsMut};
use cw_storage_plus::IndexedMap;
use funds_distributor_api::api::DistributionType;
use funds_distributor_api::error::DistributorResult;

pub trait GlobalIndicesRepository {
    fn get_global_index(
        &self,
        asset: RewardAsset,
        era: EraId,
    ) -> DistributorResult<Option<Decimal>>;

    fn get_all_global_indices(&self, era: EraId) -> DistributorResult<Vec<(RewardAsset, Decimal)>>;
}

pub trait GlobalIndicesRepositoryMut: GlobalIndicesRepository {
    fn set_global_index(
        &mut self,
        asset: RewardAsset,
        global_index: Decimal,
        era_id: EraId,
    ) -> DistributorResult<()>;
}

/////////////////////////////
////////// GENERAL //////////
/////////////////////////////

fn native_global_indices(
    distribution_type: &DistributionType,
) -> IndexedMap<(EraId, String), NativeGlobalIndex, NativeGlobalIndicesIndexes> {
    NATIVE_GLOBAL_INDICES(distribution_type)
}

fn cw20_global_indices(
    distribution_type: &DistributionType,
) -> IndexedMap<(EraId, Addr), Cw20GlobalIndex, Cw20GlobalIndicesIndexes> {
    CW20_GLOBAL_INDICES(distribution_type)
}

pub struct GeneralGlobalIndicesRepository<'a> {
    deps: Deps<'a>,
    distribution_type: DistributionType,
}

impl<'a> GlobalIndicesRepository for GeneralGlobalIndicesRepository<'a> {
    fn get_global_index(
        &self,
        asset: RewardAsset,
        era_id: EraId,
    ) -> DistributorResult<Option<Decimal>> {
        let global_index = match asset {
            Native { denom } => native_global_indices(&self.distribution_type)
                .may_load(self.deps.storage, (era_id, denom))?
                .map(|it| it.global_index),
            Cw20 { addr } => cw20_global_indices(&self.distribution_type)
                .may_load(self.deps.storage, (era_id, addr))?
                .map(|it| it.global_index),
        };
        Ok(global_index)
    }

    // TODO: store all assets encountered in only one map, that has no notion of specific era
    fn get_all_global_indices(
        &self,
        era_id: EraId,
    ) -> DistributorResult<Vec<(RewardAsset, Decimal)>> {
        let mut global_indices = vec![];

        native_global_indices(&self.distribution_type)
            .idx
            .era
            .prefix(era_id)
            .range(self.deps.storage, None, None, Ascending)
            .try_for_each(|res| match res {
                Ok(((_, denom), index)) => {
                    global_indices.push((RewardAsset::native(denom), index.global_index));
                    Ok(())
                }
                Err(e) => Err(e),
            })?;

        cw20_global_indices(&self.distribution_type)
            .idx
            .era
            .prefix(era_id)
            .range(self.deps.storage, None, None, Ascending)
            .try_for_each(|res| match res {
                Ok(((_, cw20), index)) => {
                    global_indices.push((RewardAsset::cw20(cw20), index.global_index));
                    Ok(())
                }
                Err(e) => Err(e),
            })?;

        Ok(global_indices)
    }
}

pub struct GeneralGlobalIndicesRepositoryMut<'a> {
    deps: DepsMut<'a>,
    distribution_type: DistributionType,
}

impl<'a> GlobalIndicesRepository for GeneralGlobalIndicesRepositoryMut<'a> {
    fn get_global_index(
        &self,
        asset: RewardAsset,
        era_id: EraId,
    ) -> DistributorResult<Option<Decimal>> {
        self.as_ref().get_global_index(asset, era_id)
    }

    fn get_all_global_indices(
        &self,
        era_id: EraId,
    ) -> DistributorResult<Vec<(RewardAsset, Decimal)>> {
        self.as_ref().get_all_global_indices(era_id)
    }
}

impl<'a> GlobalIndicesRepositoryMut for GeneralGlobalIndicesRepositoryMut<'a> {
    fn set_global_index(
        &mut self,
        asset: RewardAsset,
        global_index: Decimal,
        era_id: EraId,
    ) -> DistributorResult<()> {
        match asset {
            Native { denom } => {
                native_global_indices(&self.distribution_type).save(
                    self.deps.storage,
                    (era_id, denom.clone()),
                    &NativeGlobalIndex {
                        era_id,
                        denom,
                        global_index,
                    },
                )?;
            }
            Cw20 { addr } => {
                cw20_global_indices(&self.distribution_type).save(
                    self.deps.storage,
                    (era_id, addr.clone()),
                    &Cw20GlobalIndex {
                        era_id,
                        cw20_asset: addr,
                        global_index,
                    },
                )?;
            }
        }
        Ok(())
    }
}

impl<'a> GeneralGlobalIndicesRepositoryMut<'a> {
    pub fn as_ref(&self) -> GeneralGlobalIndicesRepository {
        GeneralGlobalIndicesRepository {
            deps: self.deps.as_ref(),
            distribution_type: self.distribution_type.clone(),
        }
    }
}

pub fn global_indices_repository(
    deps: Deps,
    distribution_type: DistributionType,
) -> GeneralGlobalIndicesRepository {
    GeneralGlobalIndicesRepository {
        deps,
        distribution_type,
    }
}

pub fn global_indices_repository_mut(
    deps: DepsMut,
    distribution_type: DistributionType,
) -> GeneralGlobalIndicesRepositoryMut {
    GeneralGlobalIndicesRepositoryMut {
        deps,
        distribution_type,
    }
}
