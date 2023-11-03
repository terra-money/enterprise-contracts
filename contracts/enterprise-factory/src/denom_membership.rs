use crate::contract::MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID;
use crate::state::{DaoBeingCreated, DAO_BEING_CREATED};
use crate::validate::validate_unlocking_period;
use cosmwasm_std::CosmosMsg::Wasm;
use cosmwasm_std::WasmMsg::Instantiate;
use cosmwasm_std::{to_json_binary, DepsMut, StdResult, SubMsg};
use cw_utils::Duration;
use denom_staking_api::msg::InstantiateMsg;
use enterprise_protocol::error::DaoResult;

pub fn instantiate_denom_staking_membership_contract(
    deps: DepsMut,
    denom: String,
    unlocking_period: Duration,
    weight_change_hooks: Option<Vec<String>>,
) -> DaoResult<SubMsg> {
    let dao_being_created = DAO_BEING_CREATED.load(deps.storage)?;
    let enterprise_contract = dao_being_created.require_enterprise_address()?;
    let version_info = dao_being_created.require_version_info()?;

    validate_unlocking_period(
        dao_being_created.require_create_dao_msg()?.gov_config,
        unlocking_period,
    )?;

    DAO_BEING_CREATED.update(deps.storage, |info| -> StdResult<DaoBeingCreated> {
        Ok(DaoBeingCreated {
            unlocking_period: Some(unlocking_period),
            ..info
        })
    })?;

    let submsg = SubMsg::reply_on_success(
        Wasm(Instantiate {
            admin: Some(enterprise_contract.to_string()),
            code_id: version_info.denom_staking_membership_code_id,
            msg: to_json_binary(&InstantiateMsg {
                enterprise_contract: enterprise_contract.to_string(),
                denom,
                unlocking_period,
                weight_change_hooks,
            })?,
            funds: vec![],
            label: "Denom staking membership".to_string(),
        }),
        MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID,
    );

    Ok(submsg)
}
