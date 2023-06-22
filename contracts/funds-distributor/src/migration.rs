use crate::state::ADMIN;
use cosmwasm_std::{Addr, DepsMut};
use cw_storage_plus::Item;
use funds_distributor_api::error::DistributorResult;
use funds_distributor_api::msg::MigrateMsg;

const ENTERPRISE_CONTRACT: Item<Addr> = Item::new("enterprise_contract");

pub fn migrate_to_v1_0_0(deps: DepsMut, msg: MigrateMsg) -> DistributorResult<()> {
    ENTERPRISE_CONTRACT.remove(deps.storage);

    let admin = deps.api.addr_validate(&msg.new_admin)?;
    ADMIN.save(deps.storage, &admin)?;

    Ok(())
}
