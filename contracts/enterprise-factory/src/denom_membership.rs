use crate::contract::MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID;
use crate::state::{DaoBeingCreated, DAO_BEING_CREATED};
use cosmwasm_std::CosmosMsg::Wasm;
use cosmwasm_std::WasmMsg::Instantiate;
use cosmwasm_std::{to_binary, DepsMut, StdResult, SubMsg};
use cw_utils::Duration;
use denom_staking_api::msg::InstantiateMsg;
use enterprise_protocol::api::DaoType;
use enterprise_protocol::error::DaoResult;

pub fn instantiate_denom_staking_membership_contract(
    deps: DepsMut,
    denom: String,
    unlocking_period: Duration,
    admin: String,
) -> DaoResult<SubMsg> {
    let dao_being_created = DAO_BEING_CREATED.load(deps.storage)?;
    let enterprise_contract = dao_being_created.require_enterprise_address()?;
    let version_info = dao_being_created.require_version_info()?;

    DAO_BEING_CREATED.update(deps.storage, |info| -> StdResult<DaoBeingCreated> {
        Ok(DaoBeingCreated {
            dao_type: Some(DaoType::Token),
            ..info
        })
    })?;

    let submsg = SubMsg::reply_on_success(
        Wasm(Instantiate {
            admin: Some(enterprise_contract.to_string()),
            code_id: version_info.denom_staking_membership_code_id,
            msg: to_binary(&InstantiateMsg {
                admin,
                denom,
                unlocking_period,
            })?,
            funds: vec![],
            label: "Denom staking membership".to_string(),
        }),
        MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID,
    );

    Ok(submsg)
}
