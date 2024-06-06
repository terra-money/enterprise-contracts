use crate::contract::MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID;
use crate::state::{DaoBeingCreated, DAO_BEING_CREATED};
use crate::validate::validate_existing_cw3_contract;
use cosmwasm_std::CosmosMsg::Wasm;
use cosmwasm_std::WasmMsg::Instantiate;
use cosmwasm_std::{to_json_binary, DepsMut, SubMsg, Uint128};
use cw3::Cw3QueryMsg::ListVoters;
use cw3::VoterListResponse;
use enterprise_factory_api::api::{ImportCw3MembershipMsg, NewMultisigMembershipMsg};
use enterprise_protocol::error::DaoError::MultisigDaoWithNoInitialMembers;
use enterprise_protocol::error::DaoResult;
use multisig_membership_api::api::UserWeight;
use multisig_membership_api::msg::InstantiateMsg;

const LIST_VOTERS_QUERY_LIMIT: u32 = 100;

pub fn import_cw3_membership(
    deps: DepsMut,
    msg: ImportCw3MembershipMsg,
    weight_change_hooks: Option<Vec<String>>,
) -> DaoResult<SubMsg> {
    let cw3_address = deps.api.addr_validate(&msg.cw3_contract)?;

    validate_existing_cw3_contract(deps.as_ref(), cw3_address.as_ref())?;

    // TODO: gotta do an integration test for this
    let mut initial_weights: Vec<UserWeight> = vec![];

    let mut total_weight = Uint128::zero();

    let mut last_voter: Option<String> = None;
    while {
        let query_msg = ListVoters {
            start_after: last_voter.clone(),
            limit: Some(LIST_VOTERS_QUERY_LIMIT),
        };

        last_voter = None;

        let voters: VoterListResponse = deps
            .querier
            .query_wasm_smart(&msg.cw3_contract, &query_msg)?;

        for voter in voters.voters {
            last_voter = Some(voter.addr.clone());

            let weight: Uint128 = voter.weight.into();

            initial_weights.push(UserWeight {
                user: voter.addr,
                weight,
            });

            total_weight += weight;
        }

        last_voter.is_some()
    } {}

    // multisig DAO with no initial members is meaningless - it's locked from the get-go
    if total_weight.is_zero() {
        return Err(MultisigDaoWithNoInitialMembers);
    }

    instantiate_multisig_membership_contract(
        deps,
        initial_weights,
        weight_change_hooks,
        MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID,
        true,
    )
}

pub fn instantiate_new_multisig_membership(
    deps: DepsMut,
    msg: NewMultisigMembershipMsg,
    weight_change_hooks: Option<Vec<String>>,
    store_initial_weights: bool,
) -> DaoResult<SubMsg> {
    instantiate_multisig_membership_contract(
        deps,
        msg.multisig_members,
        weight_change_hooks,
        MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID,
        store_initial_weights,
    )
}

pub fn instantiate_multisig_membership_contract(
    deps: DepsMut,
    initial_weights: Vec<UserWeight>,
    weight_change_hooks: Option<Vec<String>>,
    reply_id: u64,
    store_initial_weights: bool,
) -> DaoResult<SubMsg> {
    let dao_being_created = DAO_BEING_CREATED.load(deps.storage)?;

    let enterprise_contract = dao_being_created.require_enterprise_address()?;
    let version_info = dao_being_created.require_version_info()?;

    // TODO: this is a dirty hack in order to prevent creation of council multisig from mutating this
    if store_initial_weights {
        DAO_BEING_CREATED.save(
            deps.storage,
            &DaoBeingCreated {
                initial_weights: Some(initial_weights.clone()),
                ..dao_being_created
            },
        )?;
    }

    let submsg = SubMsg::reply_on_success(
        Wasm(Instantiate {
            admin: Some(enterprise_contract.to_string()),
            code_id: version_info.multisig_membership_code_id,
            msg: to_json_binary(&InstantiateMsg {
                enterprise_contract: enterprise_contract.to_string(),
                initial_weights: Some(initial_weights),
                weight_change_hooks,
                total_weight_by_height_checkpoints: None,
                total_weight_by_seconds_checkpoints: None,
            })?,
            funds: vec![],
            label: "Multisig staking membership".to_string(),
        }),
        reply_id,
    );

    Ok(submsg)
}
