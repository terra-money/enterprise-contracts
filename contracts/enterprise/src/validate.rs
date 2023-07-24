use crate::state::{COMPONENT_CONTRACTS, ENTERPRISE_FACTORY_CONTRACT};
use common::cw::Context;
use cosmwasm_std::{Addr, Deps};
use enterprise_protocol::api::EditCrossChainTreasuriesMsg;
use enterprise_protocol::error::DaoError::{EditingTreasuryAddressMultipleTimes, Unauthorized};
use enterprise_protocol::error::DaoResult;
use std::collections::HashMap;

/// Asserts that the caller is enterprise-factory contract.
pub fn enterprise_factory_caller_only(ctx: &Context) -> DaoResult<()> {
    let enterprise_factory = ENTERPRISE_FACTORY_CONTRACT.load(ctx.deps.storage)?;

    if ctx.info.sender != enterprise_factory {
        Err(Unauthorized)
    } else {
        Ok(())
    }
}

/// Asserts that the caller is enterprise-governance-controller contract.
pub fn enterprise_governance_controller_caller_only(ctx: &Context) -> DaoResult<()> {
    let component_contracts = COMPONENT_CONTRACTS.load(ctx.deps.storage)?;

    if ctx.info.sender != component_contracts.enterprise_governance_controller_contract {
        Err(Unauthorized)
    } else {
        Ok(())
    }
}

pub fn validate_and_dedup_edit_treasuries(
    deps: Deps,
    msg: EditCrossChainTreasuriesMsg,
) -> DaoResult<ValidatedTreasuryEdits> {
    let mut treasury_addresses_being_edited: HashMap<Addr, ()> = HashMap::new();

    let mut add_treasuries: Vec<Addr> = vec![];
    for treasury in msg.add_treasuries {
        let treasury_addr = deps.api.addr_validate(&treasury)?;
        if treasury_addresses_being_edited.contains_key(&treasury_addr) {
            return Err(EditingTreasuryAddressMultipleTimes);
        } else {
            treasury_addresses_being_edited.insert(treasury_addr.clone(), ());
            add_treasuries.push(treasury_addr);
        }
    }

    let mut remove_treasuries: Vec<Addr> = vec![];
    for treasury in msg.remove_treasuries {
        let treasury_addr = deps.api.addr_validate(&treasury)?;
        if treasury_addresses_being_edited.contains_key(&treasury_addr) {
            return Err(EditingTreasuryAddressMultipleTimes);
        } else {
            treasury_addresses_being_edited.insert(treasury_addr.clone(), ());
            remove_treasuries.push(treasury_addr);
        }
    }

    Ok(ValidatedTreasuryEdits {
        add_treasuries,
        remove_treasuries,
    })
}

pub struct ValidatedTreasuryEdits {
    pub add_treasuries: Vec<Addr>,
    pub remove_treasuries: Vec<Addr>,
}
