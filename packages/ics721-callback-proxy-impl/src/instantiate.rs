use crate::config::{Config, CONFIG};
use common::cw::Context;
use ics721_callback_proxy_api::error::Ics721CallbackProxyResult;
use ics721_callback_proxy_api::msg::InstantiateMsg;

pub fn instantiate(ctx: &mut Context, msg: InstantiateMsg) -> Ics721CallbackProxyResult<()> {
    let ics721_proxy = ctx.deps.api.addr_validate(&msg.ics721_proxy)?;

    let config = Config { ics721_proxy };

    CONFIG.save(ctx.deps.storage, &config)?;

    Ok(())
}
