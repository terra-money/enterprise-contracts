use crate::state::CONFIG;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Storage;
use cw_storage_plus::Item;
use enterprise_factory_api::api::Config;
use enterprise_protocol::error::DaoResult;

#[cw_serde]
struct ConfigV1 {
    pub enterprise_code_id: u64,
    pub cw3_fixed_multisig_code_id: u64,
    pub cw20_code_id: u64,
    pub cw721_code_id: u64,
}

const CONFIG_V1: Item<ConfigV1> = Item::new("config");

pub fn migrate_v1_to_v2(storage: &mut dyn Storage) -> DaoResult<()> {
    let config_v1 = CONFIG_V1.load(storage)?;

    let config = Config {
        enterprise_code_id: config_v1.enterprise_code_id,
        enterprise_governance_code_id: 0,
        funds_distributor_code_id: 0,
        cw3_fixed_multisig_code_id: config_v1.cw3_fixed_multisig_code_id,
        cw20_code_id: config_v1.cw20_code_id,
        cw721_code_id: config_v1.cw721_code_id,
    };

    CONFIG.save(storage, &config)?;

    Ok(())
}
