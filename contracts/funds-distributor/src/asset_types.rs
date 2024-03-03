use cosmwasm_std::Addr;
use cw_asset::AssetInfo;

#[derive(Clone)]
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
