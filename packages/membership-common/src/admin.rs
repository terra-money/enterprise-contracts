use common::cw::{Context, QueryContext};
use cosmwasm_std::{Addr, Response};
use cw_storage_plus::Item;
use membership_common_api::api::{AdminResponse, UpdateAdminMsg};
use membership_common_api::error::MembershipError::Unauthorized;
use membership_common_api::error::MembershipResult;

pub const ADMIN: Item<Addr> = Item::new("membership_common__admin");

/// Assert that the caller is admin.
/// If the validation succeeds, returns the admin address.
pub fn admin_caller_only(ctx: &Context) -> MembershipResult<Addr> {
    let admin = ADMIN.load(ctx.deps.storage)?;

    // only current admin can change the admin
    if ctx.info.sender != admin {
        return Err(Unauthorized);
    }

    Ok(admin)
}

/// Update the admin. Only the current admin can execute this.
pub fn update_admin(ctx: &mut Context, msg: UpdateAdminMsg) -> MembershipResult<Response> {
    // only admin can execute this
    admin_caller_only(ctx)?;

    let new_admin = ctx.deps.api.addr_validate(&msg.new_admin)?;
    ADMIN.save(ctx.deps.storage, &new_admin)?;

    Ok(Response::new().add_attribute("action", "update_admin"))
}

pub fn query_admin(qctx: &QueryContext) -> MembershipResult<AdminResponse> {
    let admin = ADMIN.load(qctx.deps.storage)?;

    Ok(AdminResponse { admin })
}
