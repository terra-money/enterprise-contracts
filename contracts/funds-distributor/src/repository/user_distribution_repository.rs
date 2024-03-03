use crate::asset_types::RewardAsset;
use crate::cw20_distributions::{Cw20Distribution, CW20_DISTRIBUTIONS};
use crate::native_distributions::{NativeDistribution, NATIVE_DISTRIBUTIONS};
use crate::rewards::calculate_user_reward;
use cosmwasm_std::{Addr, Decimal, Deps, DepsMut, Uint128};
use funds_distributor_api::error::DistributorResult;
use RewardAsset::{Cw20, Native};

pub struct UserDistributionInfo {
    pub user_index: Decimal,
    pub pending_rewards: Uint128,
}

impl From<UserDistributionInfo> for (Decimal, Uint128) {
    fn from(value: UserDistributionInfo) -> Self {
        (value.user_index, value.pending_rewards)
    }
}

pub trait UserDistributionRepository {
    fn get_distribution_info(
        &self,
        asset: RewardAsset,
        user: Addr,
    ) -> DistributorResult<Option<UserDistributionInfo>>;
}

pub trait UserDistributionRepositoryMut: UserDistributionRepository {
    fn set_distribution_info(
        &mut self,
        asset: RewardAsset,
        user: Addr,
        distribution_info: UserDistributionInfo,
    ) -> DistributorResult<()>;

    fn initialize_distribution_info(
        &mut self,
        global_indices: Vec<(RewardAsset, Decimal)>,
        user: Addr,
    ) -> DistributorResult<()> {
        for (asset, global_index) in global_indices {
            let distribution = self.get_distribution_info(asset.clone(), user.clone())?;
            if distribution.is_none() {
                self.set_distribution_info(
                    asset,
                    user.clone(),
                    UserDistributionInfo {
                        user_index: global_index,
                        pending_rewards: Uint128::zero(),
                    },
                )?;
            }
        }

        Ok(())
    }

    fn update_user_indices(
        &mut self,
        user: Addr,
        all_global_indices: Vec<(RewardAsset, Decimal)>,
        old_user_weight: Uint128,
    ) -> DistributorResult<()> {
        for (asset, global_index) in all_global_indices {
            let distribution = self.get_distribution_info(asset.clone(), user.clone())?;

            let reward = calculate_user_reward(global_index, distribution, old_user_weight)?;

            self.set_distribution_info(
                asset,
                user.clone(),
                UserDistributionInfo {
                    user_index: global_index,
                    pending_rewards: reward,
                },
            )?;
        }

        Ok(())
    }
}

/////////////////////////////
////////// GENERAL //////////
/////////////////////////////

pub struct GeneralUserDistributionRepository<'a> {
    deps: Deps<'a>,
}

impl UserDistributionRepository for GeneralUserDistributionRepository<'_> {
    fn get_distribution_info(
        &self,
        asset: RewardAsset,
        user: Addr,
    ) -> DistributorResult<Option<UserDistributionInfo>> {
        let distribution = match asset {
            Native { denom } => NATIVE_DISTRIBUTIONS()
                .may_load(self.deps.storage, (user, denom))?
                .map(|it| it.into()),
            Cw20 { addr } => CW20_DISTRIBUTIONS()
                .may_load(self.deps.storage, (user, addr))?
                .map(|it| it.into()),
        };
        Ok(distribution)
    }
}

pub struct GeneralUserDistributionRepositoryMut<'a> {
    deps: DepsMut<'a>,
}

impl GeneralUserDistributionRepositoryMut<'_> {
    pub fn as_ref(&self) -> GeneralUserDistributionRepository {
        GeneralUserDistributionRepository {
            deps: self.deps.as_ref(),
        }
    }
}

impl UserDistributionRepository for GeneralUserDistributionRepositoryMut<'_> {
    fn get_distribution_info(
        &self,
        asset: RewardAsset,
        user: Addr,
    ) -> DistributorResult<Option<UserDistributionInfo>> {
        self.as_ref().get_distribution_info(asset, user)
    }
}

impl UserDistributionRepositoryMut for GeneralUserDistributionRepositoryMut<'_> {
    fn set_distribution_info(
        &mut self,
        asset: RewardAsset,
        user: Addr,
        distribution_info: UserDistributionInfo,
    ) -> DistributorResult<()> {
        match asset {
            Native { denom } => {
                NATIVE_DISTRIBUTIONS().save(
                    self.deps.storage,
                    (user.clone(), denom.clone()),
                    &NativeDistribution {
                        user,
                        denom,
                        user_index: distribution_info.user_index,
                        pending_rewards: distribution_info.pending_rewards,
                    },
                )?;
            }
            Cw20 { addr } => {
                CW20_DISTRIBUTIONS().save(
                    self.deps.storage,
                    (user.clone(), addr.clone()),
                    &Cw20Distribution {
                        user,
                        cw20_asset: addr,
                        user_index: distribution_info.user_index,
                        pending_rewards: distribution_info.pending_rewards,
                    },
                )?;
            }
        }

        Ok(())
    }
}

pub fn user_distribution_repository(deps: Deps) -> GeneralUserDistributionRepository {
    GeneralUserDistributionRepository { deps }
}

pub fn user_distribution_repository_mut(deps: DepsMut) -> GeneralUserDistributionRepositoryMut {
    GeneralUserDistributionRepositoryMut { deps }
}
