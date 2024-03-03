use cosmwasm_std::{Decimal, Uint128};
use cw_asset::AssetInfo;
use common::cw::Context;
use funds_distributor_api::error::DistributorError::ZeroTotalWeight;
use funds_distributor_api::error::DistributorResult;
use crate::asset_types::NativeAsset;
use crate::distributing::assert_assets_whitelisted;
use crate::repository::asset_repository::{asset_distribution_repository_mut, AssetDistributionRepository, AssetDistributionRepositoryMut};
use crate::repository::weights_repository::{weights_repository_mut, WeightsRepository};

// TODO: doesn't have to take ctx but w/e
pub fn distribute_native(ctx: &mut Context, assets: Vec<(NativeAsset, Uint128)>) -> DistributorResult<()> {
    let distribution_assets = assets
        .iter()
        .map(|(denom, _)| AssetInfo::native(denom))
        .collect::<Vec<AssetInfo>>();

    assert_assets_whitelisted(ctx, distribution_assets.clone())?;

    let weights_repository = weights_repository_mut(ctx.deps.branch());

    let total_weight = weights_repository.get_total_weight()?;
    if total_weight.is_zero() {
        return Err(ZeroTotalWeight);
    }

    let mut asset_distributions_repository = asset_distribution_repository_mut(ctx.deps.branch());
    for (denom, amount) in assets { // TODO: what if an asset appears multiple times?
        let global_index = asset_distributions_repository.get_global_index(denom.clone())?
            .unwrap_or(Decimal::zero());

        // calculate how many units of the asset we're distributing per unit of total user weight
        // and add that to the global index for the asset
        let index_increment = Decimal::from_ratio(amount, total_weight);

        asset_distributions_repository.set_global_index(denom, global_index.checked_add(index_increment)?)?;
    }

    Ok(())
}