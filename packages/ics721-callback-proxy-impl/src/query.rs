use crate::config::CONFIG;
use common::cw::QueryContext;
use ics721_callback_proxy_api::api::ConfigResponse;
use ics721_callback_proxy_api::error::Ics721CallbackProxyResult;

pub fn query_config(qctx: &QueryContext) -> Ics721CallbackProxyResult<ConfigResponse> {
    let config = CONFIG.load(qctx.deps.storage)?;

    Ok(ConfigResponse {
        ics721_proxy: config.ics721_proxy,
    })
}
