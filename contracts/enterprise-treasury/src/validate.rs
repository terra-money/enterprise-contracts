use crate::state::ENTERPRISE_CONTRACT;
use common::cw::Context;
use enterprise_treasury_api::error::EnterpriseTreasuryError::Unauthorized;
use enterprise_treasury_api::error::EnterpriseTreasuryResult;

/// Verifies that the caller is one of the Enterprise contracts of this DAO.
pub fn dao_contracts_only(ctx: &Context) -> EnterpriseTreasuryResult<()> {
    let enterprise_contract = ENTERPRISE_CONTRACT.load(ctx.deps.storage)?;

    // TODO: also allow governance contract
    if ctx.info.sender != enterprise_contract {
        return Err(Unauthorized);
    }

    Ok(())
}
