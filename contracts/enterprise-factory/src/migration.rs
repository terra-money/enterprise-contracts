use crate::state::CONFIG;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::DepsMut;
use cw_storage_plus::Item;
use enterprise_factory_api::api::Config;
use enterprise_factory_api::msg::MigrateMsg;
use enterprise_protocol::error::DaoResult;

#[cw_serde]
struct OldConfig {
    pub enterprise_code_id: u64,
    pub enterprise_governance_code_id: u64,
    pub funds_distributor_code_id: u64,
    pub cw3_fixed_multisig_code_id: u64,
    pub cw20_code_id: u64,
    pub cw721_code_id: u64,
}

const OLD_CONFIG: Item<OldConfig> = Item::new("config");

pub fn migrate_config(deps: DepsMut, msg: MigrateMsg) -> DaoResult<()> {
    let old_config = OLD_CONFIG.load(deps.storage)?;

    let admin = deps.api.addr_validate(&msg.admin)?;
    let enterprise_versioning = deps.api.addr_validate(&msg.enterprise_versioning_addr)?;

    let new_config = Config {
        admin,
        enterprise_versioning,
        cw20_code_id: msg.cw20_code_id.unwrap_or(old_config.cw20_code_id),
        cw721_code_id: msg.cw721_code_id.unwrap_or(old_config.cw721_code_id),
    };

    CONFIG.save(deps.storage, &new_config)?;

    Ok(())
}
