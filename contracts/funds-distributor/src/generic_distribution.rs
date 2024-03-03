use crate::asset_types::RewardAsset;
use crate::distributing::assert_assets_whitelisted;
use crate::repository::asset_repository::{
    asset_distribution_repository_mut, AssetDistributionRepository, AssetDistributionRepositoryMut,
};
use crate::repository::weights_repository::{weights_repository_mut, WeightsRepository};
use common::cw::Context;
use cosmwasm_std::{Decimal, Uint128};
use cw_asset::AssetInfo;
use funds_distributor_api::error::DistributorError::ZeroTotalWeight;
use funds_distributor_api::error::DistributorResult;

// TODO: rename this file

// TODO: doesn't have to take ctx but w/e
pub fn distribute(ctx: &mut Context, assets: Vec<(RewardAsset, Uint128)>) -> DistributorResult<()> {
    let distribution_assets = assets
        .iter()
        .map(|(asset, _)| asset.into())
        .collect::<Vec<AssetInfo>>();

    assert_assets_whitelisted(ctx, distribution_assets.clone())?;

    let weights_repository = weights_repository_mut(ctx.deps.branch());

    let total_weight = weights_repository.get_total_weight()?;
    if total_weight.is_zero() {
        return Err(ZeroTotalWeight);
    }

    let mut asset_distributions_repository = asset_distribution_repository_mut(ctx.deps.branch());

    for (reward_asset, amount) in assets {
        // TODO: what if an asset appears multiple times?
        let global_index = asset_distributions_repository
            .get_global_index(reward_asset.clone())?
            .unwrap_or(Decimal::zero());

        // calculate how many units of the asset we're distributing per unit of total user weight
        // and add that to the global index for the asset
        let index_increment = Decimal::from_ratio(amount, total_weight);

        let new_global_index = global_index.checked_add(index_increment)?;

        asset_distributions_repository.set_global_index(reward_asset, new_global_index)?;
    }

    Ok(())
}
