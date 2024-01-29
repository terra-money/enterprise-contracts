use cw_asset::AssetInfoUnchecked;

pub fn cw20_unchecked(cw20_addr: impl Into<String>) -> AssetInfoUnchecked {
    AssetInfoUnchecked::cw20(cw20_addr.into())
}
