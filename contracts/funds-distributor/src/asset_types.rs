use cosmwasm_std::{Addr, Deps, StdResult};
use cw_asset::AssetInfo;
use funds_distributor_api::error::DistributorResult;

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum RewardAsset {
    Native { denom: String },
    Cw20 { addr: Addr },
}

impl RewardAsset {
    pub fn native(denom: impl Into<String>) -> RewardAsset {
        RewardAsset::Native {
            denom: denom.into(),
        }
    }

    pub fn cw20(addr: Addr) -> RewardAsset {
        RewardAsset::Cw20 { addr }
    }
}

impl From<RewardAsset> for AssetInfo {
    fn from(value: RewardAsset) -> Self {
        match value {
            RewardAsset::Native { denom } => AssetInfo::native(denom),
            RewardAsset::Cw20 { addr } => AssetInfo::cw20(addr),
        }
    }
}

impl From<&RewardAsset> for AssetInfo {
    fn from(value: &RewardAsset) -> Self {
        match value {
            RewardAsset::Native { denom } => AssetInfo::native(denom),
            RewardAsset::Cw20 { addr } => AssetInfo::cw20(addr.clone()),
        }
    }
}

pub fn to_reward_assets(
    deps: Deps,
    native_assets: Vec<String>,
    cw20_assets: Vec<String>,
) -> DistributorResult<Vec<RewardAsset>> {
    let mut assets: Vec<RewardAsset> = native_assets.iter().map(RewardAsset::native).collect();
    let mut cw20_assets = cw20_assets
        .iter()
        .map(|addr| deps.api.addr_validate(addr))
        .map(|res| res.map(RewardAsset::cw20))
        .collect::<StdResult<Vec<RewardAsset>>>()?;

    assets.append(&mut cw20_assets);

    Ok(assets)
}
