use crate::state::{ADMIN, ENTERPRISE_CONTRACT};
use cosmwasm_std::DepsMut;
use funds_distributor_api::error::DistributorResult;
use funds_distributor_api::msg::MigrateMsg;

pub fn migrate_to_v1_0_0(deps: DepsMut, msg: MigrateMsg) -> DistributorResult<()> {
    let admin = deps.api.addr_validate(&msg.new_admin)?;
    ADMIN.save(deps.storage, &admin)?;

    let enterprise_contract = deps.api.addr_validate(&msg.new_enterprise_contract)?;
    ENTERPRISE_CONTRACT.save(deps.storage, &enterprise_contract)?;

    Ok(())
}
