use crate::config::CONFIG;
use common::cw::Context;
use cosmwasm_std::Addr;
use token_staking_api::error::TokenStakingError::Unauthorized;
use token_staking_api::error::TokenStakingResult;

/// Assert that the caller is admin.
/// If the validation succeeds, returns the admin address.
pub fn admin_caller_only(ctx: &Context) -> TokenStakingResult<Addr> {
    let config = CONFIG.load(ctx.deps.storage)?;
    let admin = config.admin;

    // only current admin can change the admin
    if ctx.info.sender != admin {
        return Err(Unauthorized);
    }

    Ok(admin)
}
