use crate::multisig_membership::{import_cw3_membership, instantiate_new_multisig_membership};
use crate::nft_membership::{
    import_cw721_membership, instantiate_new_cw721_membership,
    instantiate_nft_staking_membership_contract,
};
use crate::state::{
    DaoBeingCreated, CONFIG, DAO_ADDRESSES, DAO_BEING_CREATED, DAO_ID_COUNTER, ENTERPRISE_CODE_IDS,
};
use crate::token_membership::{
    import_cw20_membership, instantiate_new_cw20_membership,
    instantiate_token_staking_membership_contract,
};
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{
    entry_point, to_binary, wasm_execute, wasm_instantiate, Addr, Binary, Deps, DepsMut, Env,
    MessageInfo, Reply, Response, StdError, StdResult, SubMsg, Uint64, WasmMsg,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use cw_utils::parse_reply_instantiate_data;
use enterprise_factory_api::api::{
    AllDaosResponse, Config, ConfigResponse, CreateDaoMembershipMsg, CreateDaoMsg,
    EnterpriseCodeIdsMsg, EnterpriseCodeIdsResponse, IsEnterpriseCodeIdMsg,
    IsEnterpriseCodeIdResponse, QueryAllDaosMsg,
};
use enterprise_factory_api::msg::ExecuteMsg::FinalizeDaoCreation;
use enterprise_factory_api::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use enterprise_protocol::api::FinalizeInstantiationMsg;
use enterprise_protocol::error::DaoError::Unauthorized;
use enterprise_protocol::error::{DaoError, DaoResult};
use enterprise_protocol::msg::ExecuteMsg::FinalizeInstantiation;
use funds_distributor_api::api::UserWeight;
use itertools::Itertools;
use CreateDaoMembershipMsg::{ImportCw20, ImportCw3, ImportCw721, NewCw20, NewCw721, NewMultisig};
use ExecuteMsg::CreateDao;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:enterprise-factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MAX_QUERY_LIMIT: u32 = 100;
const DEFAULT_QUERY_LIMIT: u32 = 50;

pub const ENTERPRISE_INSTANTIATE_REPLY_ID: u64 = 1;
pub const CW20_CONTRACT_INSTANTIATE_REPLY_ID: u64 = 2;
pub const CW721_CONTRACT_INSTANTIATE_REPLY_ID: u64 = 3;
pub const MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID: u64 = 4;
pub const FUNDS_DISTRIBUTOR_INSTANTIATE_REPLY_ID: u64 = 5;
pub const ENTERPRISE_GOVERNANCE_INSTANTIATE_REPLY_ID: u64 = 6;
pub const ENTERPRISE_GOVERNANCE_CONTROLLER_INSTANTIATE_REPLY_ID: u64 = 7;
pub const ENTERPRISE_TREASURY_INSTANTIATE_REPLY_ID: u64 = 8;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> DaoResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CONFIG.save(deps.storage, &msg.config)?;
    DAO_ID_COUNTER.save(deps.storage, &1u64)?;

    ENTERPRISE_CODE_IDS.save(deps.storage, msg.config.enterprise_code_id, &())?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> DaoResult<Response> {
    match msg {
        CreateDao(msg) => create_dao(deps, env, *msg),
        FinalizeDaoCreation {} => finalize_dao_creation(deps, env, info),
    }
}

fn create_dao(deps: DepsMut, env: Env, msg: CreateDaoMsg) -> DaoResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    DAO_BEING_CREATED.save(
        deps.storage,
        &DaoBeingCreated {
            create_dao_msg: Some(msg.clone()),
            enterprise_address: None,
            initial_weights: None,
            dao_type: None,
            unlocking_period: None,
            membership_address: None,
            funds_distributor_address: None,
            enterprise_governance_address: None,
            enterprise_governance_controller_address: None,
            enterprise_treasury_address: None,
        },
    )?;

    let instantiate_enterprise_msg = enterprise_protocol::msg::InstantiateMsg {
        dao_metadata: msg.dao_metadata.clone(),
        enterprise_factory_contract: env.contract.address.to_string(),
    };
    let create_dao_submsg = SubMsg::reply_on_success(
        WasmMsg::Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id: config.enterprise_code_id,
            msg: to_binary(&instantiate_enterprise_msg)?,
            funds: vec![],
            label: msg.dao_metadata.name,
        },
        ENTERPRISE_INSTANTIATE_REPLY_ID,
    );

    Ok(Response::new()
        .add_attribute("action", "create_dao")
        .add_submessage(create_dao_submsg))
}

fn finalize_dao_creation(deps: DepsMut, env: Env, info: MessageInfo) -> DaoResult<Response> {
    if info.sender != env.contract.address {
        return Err(Unauthorized);
    }

    let dao_being_created = DAO_BEING_CREATED.load(deps.storage)?;

    DAO_BEING_CREATED.save(
        deps.storage,
        &DaoBeingCreated {
            create_dao_msg: None,
            enterprise_address: None,
            initial_weights: None,
            dao_type: None,
            unlocking_period: None,
            membership_address: None,
            funds_distributor_address: None,
            enterprise_governance_address: None,
            enterprise_governance_controller_address: None,
            enterprise_treasury_address: None,
        },
    )?;

    let finalize_creation_submsg = SubMsg::new(wasm_execute(
        dao_being_created.require_enterprise_address()?.to_string(),
        &FinalizeInstantiation(FinalizeInstantiationMsg {
            enterprise_treasury_contract: dao_being_created
                .require_enterprise_treasury_address()?
                .to_string(),
            enterprise_governance_contract: dao_being_created
                .require_enterprise_governance_address()?
                .to_string(),
            enterprise_governance_controller_contract: dao_being_created
                .require_enterprise_governance_controller_address()?
                .to_string(),
            funds_distributor_contract: dao_being_created
                .require_funds_distributor_address()?
                .to_string(),
            membership_contract: dao_being_created.require_membership_address()?.to_string(),
            dao_type: dao_being_created.require_dao_type()?,
        }),
        vec![],
    )?);

    Ok(Response::new()
        .add_attribute("action", "finalize_dao_creation")
        .add_submessage(finalize_creation_submsg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(mut deps: DepsMut, env: Env, msg: Reply) -> DaoResult<Response> {
    match msg.id {
        ENTERPRISE_INSTANTIATE_REPLY_ID => {
            let contract_address = parse_reply_instantiate_data(msg)
                .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
                .contract_address;
            let enterprise_contract = deps.api.addr_validate(&contract_address)?;

            let id = DAO_ID_COUNTER.load(deps.storage)?;
            let next_id = id + 1;
            DAO_ID_COUNTER.save(deps.storage, &next_id)?;

            DAO_ADDRESSES.save(deps.storage, id, &enterprise_contract)?;

            // Update the admin of the DAO contract to be the DAO itself
            let update_admin_msg = SubMsg::new(WasmMsg::UpdateAdmin {
                contract_addr: contract_address.clone(),
                admin: contract_address.clone(),
            });

            let dao_being_created = DAO_BEING_CREATED.load(deps.storage)?;

            let create_dao_msg = dao_being_created.require_create_dao_msg()?;

            DAO_BEING_CREATED.save(
                deps.storage,
                &DaoBeingCreated {
                    enterprise_address: Some(enterprise_contract.clone()),
                    ..dao_being_created.clone()
                },
            )?;

            let membership_submsg = match create_dao_msg.dao_membership {
                ImportCw20(msg) => import_cw20_membership(deps.branch(), msg)?,
                NewCw20(msg) => instantiate_new_cw20_membership(deps.branch(), *msg)?,
                ImportCw721(msg) => import_cw721_membership(deps.branch(), msg)?,
                NewCw721(msg) => instantiate_new_cw721_membership(deps.branch(), msg)?,
                ImportCw3(msg) => import_cw3_membership(deps.branch(), msg)?,
                NewMultisig(msg) => instantiate_new_multisig_membership(deps.branch(), msg)?,
            };

            let config = CONFIG.load(deps.storage)?;

            let initial_weights = dao_being_created
                .initial_weights
                .unwrap_or_default()
                .iter()
                .map(|user_weight| UserWeight {
                    user: user_weight.user.clone(),
                    weight: user_weight.weight,
                })
                .collect();

            let funds_distributor_submsg = SubMsg::reply_on_success(
                wasm_instantiate(
                    config.funds_distributor_code_id,
                    &funds_distributor_api::msg::InstantiateMsg {
                        enterprise_contract: enterprise_contract.to_string(),
                        initial_weights,
                        minimum_eligible_weight: create_dao_msg.minimum_weight_for_rewards,
                    }, // TODO: supply initial weights?
                    vec![],
                    "Enterprise governance".to_string(),
                )?,
                ENTERPRISE_GOVERNANCE_INSTANTIATE_REPLY_ID,
            );

            let enterprise_governance_submsg = SubMsg::reply_on_success(
                wasm_instantiate(
                    config.enterprise_governance_code_id,
                    &enterprise_governance_api::msg::InstantiateMsg {
                        enterprise_contract: enterprise_contract.to_string(),
                    }, // TODO: supply enterprise-governance-controller instead?
                    vec![],
                    "Enterprise governance".to_string(),
                )?,
                ENTERPRISE_GOVERNANCE_INSTANTIATE_REPLY_ID,
            );

            let enterprise_governance_controller_submsg = SubMsg::reply_on_success(
                wasm_instantiate(
                    config.enterprise_governance_controller_code_id,
                    &enterprise_governance_controller_api::msg::InstantiateMsg {
                        enterprise_contract: enterprise_contract.to_string(),
                        gov_config: create_dao_msg.gov_config,
                    },
                    vec![],
                    "Enterprise governance controller".to_string(),
                )?,
                ENTERPRISE_GOVERNANCE_INSTANTIATE_REPLY_ID,
            );

            let enterprise_treasury_submsg = SubMsg::reply_on_success(
                wasm_instantiate(
                    config.enterprise_treasury_code_id,
                    &enterprise_treasury_api::msg::InstantiateMsg {
                        enterprise_contract: enterprise_contract.to_string(),
                        asset_whitelist: create_dao_msg.asset_whitelist,
                        nft_whitelist: create_dao_msg.nft_whitelist,
                    },
                    vec![],
                    "Enterprise governance controller".to_string(),
                )?,
                ENTERPRISE_GOVERNANCE_INSTANTIATE_REPLY_ID,
            );

            let finalize_submsg = SubMsg::new(wasm_execute(
                env.contract.address.to_string(),
                &FinalizeDaoCreation {},
                vec![],
            )?);

            Ok(Response::new()
                .add_submessage(update_admin_msg)
                .add_submessage(membership_submsg)
                .add_submessage(funds_distributor_submsg)
                .add_submessage(enterprise_governance_submsg)
                .add_submessage(enterprise_governance_controller_submsg)
                .add_submessage(enterprise_treasury_submsg)
                .add_submessage(finalize_submsg)
                .add_attribute("action", "instantiate_dao")
                .add_attribute("dao_address", contract_address))
        }
        CW20_CONTRACT_INSTANTIATE_REPLY_ID => {
            let contract_address = parse_reply_instantiate_data(msg)
                .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
                .contract_address;
            let addr = deps.api.addr_validate(&contract_address)?;

            let unlocking_period = DAO_BEING_CREATED
                .load(deps.storage)?
                .require_unlocking_period()?;

            Ok(
                Response::new().add_submessage(instantiate_token_staking_membership_contract(
                    deps,
                    addr,
                    unlocking_period,
                )?),
            )
        }
        CW721_CONTRACT_INSTANTIATE_REPLY_ID => {
            let contract_address = parse_reply_instantiate_data(msg)
                .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
                .contract_address;
            let addr = deps.api.addr_validate(&contract_address)?;

            let unlocking_period = DAO_BEING_CREATED
                .load(deps.storage)?
                .require_unlocking_period()?;

            Ok(
                Response::new().add_submessage(instantiate_nft_staking_membership_contract(
                    deps,
                    addr,
                    unlocking_period,
                )?),
            )
        }
        MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID => {
            let contract_address = parse_reply_instantiate_data(msg)
                .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
                .contract_address;
            let addr = deps.api.addr_validate(&contract_address)?;

            DAO_BEING_CREATED.update(deps.storage, |info| -> StdResult<DaoBeingCreated> {
                Ok(DaoBeingCreated {
                    membership_address: Some(addr),
                    ..info
                })
            })?;

            Ok(Response::new())
        }
        FUNDS_DISTRIBUTOR_INSTANTIATE_REPLY_ID => {
            let contract_address = parse_reply_instantiate_data(msg)
                .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
                .contract_address;
            let addr = deps.api.addr_validate(&contract_address)?;

            DAO_BEING_CREATED.update(deps.storage, |info| -> StdResult<DaoBeingCreated> {
                Ok(DaoBeingCreated {
                    funds_distributor_address: Some(addr),
                    ..info
                })
            })?;

            Ok(Response::new())
        }
        ENTERPRISE_GOVERNANCE_INSTANTIATE_REPLY_ID => {
            let contract_address = parse_reply_instantiate_data(msg)
                .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
                .contract_address;
            let addr = deps.api.addr_validate(&contract_address)?;

            DAO_BEING_CREATED.update(deps.storage, |info| -> StdResult<DaoBeingCreated> {
                Ok(DaoBeingCreated {
                    enterprise_governance_address: Some(addr),
                    ..info
                })
            })?;

            Ok(Response::new())
        }
        ENTERPRISE_GOVERNANCE_CONTROLLER_INSTANTIATE_REPLY_ID => {
            let contract_address = parse_reply_instantiate_data(msg)
                .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
                .contract_address;
            let addr = deps.api.addr_validate(&contract_address)?;

            DAO_BEING_CREATED.update(deps.storage, |info| -> StdResult<DaoBeingCreated> {
                Ok(DaoBeingCreated {
                    enterprise_governance_controller_address: Some(addr),
                    ..info
                })
            })?;

            Ok(Response::new())
        }
        ENTERPRISE_TREASURY_INSTANTIATE_REPLY_ID => {
            let contract_address = parse_reply_instantiate_data(msg)
                .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
                .contract_address;
            let addr = deps.api.addr_validate(&contract_address)?;

            DAO_BEING_CREATED.update(deps.storage, |info| -> StdResult<DaoBeingCreated> {
                Ok(DaoBeingCreated {
                    enterprise_treasury_address: Some(addr),
                    ..info
                })
            })?;

            Ok(Response::new())
        }
        _ => Err(DaoError::Std(StdError::generic_err(
            "No such reply ID found",
        ))),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> DaoResult<Binary> {
    let response = match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?)?,
        QueryMsg::AllDaos(msg) => to_binary(&query_all_daos(deps, msg)?)?,
        QueryMsg::EnterpriseCodeIds(msg) => to_binary(&query_enterprise_code_ids(deps, msg)?)?,
        QueryMsg::IsEnterpriseCodeId(msg) => to_binary(&query_is_enterprise_code_id(deps, msg)?)?,
    };
    Ok(response)
}

pub fn query_config(deps: Deps) -> DaoResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    Ok(ConfigResponse { config })
}

pub fn query_all_daos(deps: Deps, msg: QueryAllDaosMsg) -> DaoResult<AllDaosResponse> {
    let start_after = msg.start_after.map(Bound::exclusive);
    let limit = msg
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT)
        .min(MAX_QUERY_LIMIT);
    let addresses = DAO_ADDRESSES
        .range(deps.storage, start_after, None, Ascending)
        .take(limit as usize)
        .collect::<StdResult<Vec<(u64, Addr)>>>()?
        .into_iter()
        .map(|(_, addr)| addr)
        .collect_vec();

    Ok(AllDaosResponse { daos: addresses })
}

pub fn query_enterprise_code_ids(
    deps: Deps,
    msg: EnterpriseCodeIdsMsg,
) -> DaoResult<EnterpriseCodeIdsResponse> {
    let start_after = msg.start_after.map(Bound::exclusive);
    let limit = msg
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT)
        .min(MAX_QUERY_LIMIT);

    let code_ids = ENTERPRISE_CODE_IDS
        .range(deps.storage, start_after, None, Ascending)
        .take(limit as usize)
        .collect::<StdResult<Vec<(u64, ())>>>()?
        .into_iter()
        .map(|(code_id, _)| Uint64::from(code_id))
        .collect_vec();

    Ok(EnterpriseCodeIdsResponse { code_ids })
}

pub fn query_is_enterprise_code_id(
    deps: Deps,
    msg: IsEnterpriseCodeIdMsg,
) -> DaoResult<IsEnterpriseCodeIdResponse> {
    let is_enterprise_code_id = ENTERPRISE_CODE_IDS.has(deps.storage, msg.code_id.u64());
    Ok(IsEnterpriseCodeIdResponse {
        is_enterprise_code_id,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> DaoResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    CONFIG.save(
        deps.storage,
        &Config {
            enterprise_code_id: msg.new_enterprise_code_id,
            enterprise_governance_code_id: msg.new_enterprise_governance_code_id,
            funds_distributor_code_id: msg.new_funds_distributor_code_id,
            ..config
        },
    )?;

    ENTERPRISE_CODE_IDS.save(deps.storage, msg.new_enterprise_code_id, &())?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new())
}
