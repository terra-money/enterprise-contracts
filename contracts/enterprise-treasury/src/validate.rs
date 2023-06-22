use crate::state::CONFIG;
use common::cw::Context;
use enterprise_treasury_api::error::EnterpriseTreasuryError::Unauthorized;
use enterprise_treasury_api::error::EnterpriseTreasuryResult;

/// Verifies that the caller is one of the Enterprise contracts of this DAO.
pub fn admin_only(ctx: &Context) -> EnterpriseTreasuryResult<()> {
    let config = CONFIG.load(ctx.deps.storage)?;

    if ctx.info.sender != config.admin {
        return Err(Unauthorized);
    }

    Ok(())
}
