use cosmwasm_std::{Addr, DepsMut};
use cw_storage_plus::Item;
use membership_common_api::error::MembershipResult;

pub const ENTERPRISE_CONTRACT: Item<Addr> = Item::new("membership_common__enterprise_contract");

pub fn set_enterprise_contract(
    deps: DepsMut,
    enterprise_contract: impl Into<String>,
) -> MembershipResult<Addr> {
    let enterprise_contract = deps.api.addr_validate(&enterprise_contract.into())?;

    ENTERPRISE_CONTRACT.save(deps.storage, &enterprise_contract)?;

    Ok(enterprise_contract)
}
