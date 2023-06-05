use crate::config::CONFIG;
use common::cw::Context;
use cosmwasm_std::Addr;
use nft_staking_api::error::NftStakingError::Unauthorized;
use nft_staking_api::error::NftStakingResult;

/// Assert that the caller is admin.
/// If the validation succeeds, returns the admin address.
pub fn admin_caller_only(ctx: &Context) -> NftStakingResult<Addr> {
    let config = CONFIG.load(ctx.deps.storage)?;
    let admin = config.admin;

    // only current admin can change the admin
    if ctx.info.sender != admin {
        return Err(Unauthorized);
    }

    Ok(admin)
}
