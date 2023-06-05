use crate::config::{Config, CONFIG};
use common::cw::Context;
use multisig_membership_api::error::MultisigMembershipResult;
use multisig_membership_api::msg::InstantiateMsg;

pub fn instantiate(ctx: &mut Context, msg: InstantiateMsg) -> MultisigMembershipResult<()> {
    let admin = ctx.deps.api.addr_validate(&msg.admin)?;

    let config = Config { admin };

    CONFIG.save(ctx.deps.storage, &config)?;

    Ok(())
}
