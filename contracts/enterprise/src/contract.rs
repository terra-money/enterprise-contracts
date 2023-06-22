use crate::migration::{
    council_membership_contract_created, finalize_migration,
    governance_controller_contract_created, membership_contract_created, migrate_to_rewrite,
    treasury_contract_created,
};
use crate::state::{
    ComponentContracts, COMPONENT_CONTRACTS, DAO_CREATION_DATE, DAO_METADATA, DAO_TYPE,
    DAO_VERSION, ENTERPRISE_FACTORY_CONTRACT, ENTERPRISE_VERSIONING_CONTRACT,
    IS_INSTANTIATION_FINALIZED,
};
use common::commons::ModifyValue::Change;
use common::cw::{Context, QueryContext};
use cosmwasm_std::CosmosMsg::Wasm;
use cosmwasm_std::WasmMsg::Migrate;
use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdError, SubMsg,
};
use cw2::set_contract_version;
use cw_utils::parse_reply_instantiate_data;
use enterprise_protocol::api::{
    ComponentContractsResponse, DaoInfoResponse, FinalizeInstantiationMsg, UpdateMetadataMsg,
    UpgradeDaoMsg,
};
use enterprise_protocol::error::DaoError::{
    AlreadyInitialized, InvalidMigrateMsgMap, MigratingToLowerVersion, Unauthorized,
};
use enterprise_protocol::error::{DaoError, DaoResult};
use enterprise_protocol::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use enterprise_versioning_api::api::{Version, VersionInfo, VersionsParams, VersionsResponse};
use enterprise_versioning_api::msg::QueryMsg::Versions;
use serde_json::json;
use serde_json::Value::Object;
use std::str::FromStr;

pub const ENTERPRISE_TREASURY_REPLY_ID: u64 = 1;
pub const ENTERPRISE_GOVERNANCE_CONTROLLER_REPLY_ID: u64 = 2;
pub const COUNCIL_MEMBERSHIP_REPLY_ID: u64 = 3;
pub const MEMBERSHIP_REPLY_ID: u64 = 4;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:enterprise";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const DEFAULT_QUERY_LIMIT: u8 = 50;
pub const MAX_QUERY_LIMIT: u8 = 100;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> DaoResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    DAO_CREATION_DATE.save(deps.storage, &env.block.time)?;

    let enterprise_factory_contract = deps.api.addr_validate(&msg.enterprise_factory_contract)?;
    ENTERPRISE_FACTORY_CONTRACT.save(deps.storage, &enterprise_factory_contract)?;

    let enterprise_versioning_contract = deps
        .api
        .addr_validate(&msg.enterprise_versioning_contract)?;
    ENTERPRISE_VERSIONING_CONTRACT.save(deps.storage, &enterprise_versioning_contract)?;

    DAO_METADATA.save(deps.storage, &msg.dao_metadata)?;

    DAO_VERSION.save(deps.storage, &Version::from_str(CONTRACT_VERSION)?)?;

    IS_INSTANTIATION_FINALIZED.save(deps.storage, &false)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> DaoResult<Response> {
    let ctx = &mut Context { deps, env, info };

    match msg {
        ExecuteMsg::FinalizeInstantiation(msg) => finalize_instantiation(ctx, msg),
        ExecuteMsg::UpdateMetadata(msg) => update_metadata(ctx, msg),
        ExecuteMsg::UpgradeDao(msg) => upgrade_dao(ctx, msg),
        ExecuteMsg::FinalizeMigration {} => finalize_migration(ctx),
    }
}

fn finalize_instantiation(ctx: &mut Context, msg: FinalizeInstantiationMsg) -> DaoResult<Response> {
    let is_instantiation_finalized = IS_INSTANTIATION_FINALIZED.load(ctx.deps.storage)?;

    if is_instantiation_finalized {
        return Err(AlreadyInitialized);
    }

    let enterprise_factory = ENTERPRISE_FACTORY_CONTRACT.load(ctx.deps.storage)?;

    if ctx.info.sender != enterprise_factory {
        return Err(Unauthorized);
    }

    let component_contracts = ComponentContracts {
        enterprise_governance_contract: ctx
            .deps
            .api
            .addr_validate(&msg.enterprise_governance_contract)?,
        enterprise_governance_controller_contract: ctx
            .deps
            .api
            .addr_validate(&msg.enterprise_governance_controller_contract)?,
        enterprise_treasury_contract: ctx
            .deps
            .api
            .addr_validate(&msg.enterprise_treasury_contract)?,
        funds_distributor_contract: ctx
            .deps
            .api
            .addr_validate(&msg.funds_distributor_contract)?,
        membership_contract: ctx.deps.api.addr_validate(&msg.membership_contract)?,
    };

    COMPONENT_CONTRACTS.save(ctx.deps.storage, &component_contracts)?;

    DAO_TYPE.save(ctx.deps.storage, &msg.dao_type)?;

    IS_INSTANTIATION_FINALIZED.save(ctx.deps.storage, &true)?;

    Ok(Response::new()
        .add_attribute("action", "finalize_instantiation")
        .add_attribute(
            "enterprise_governance_contract",
            component_contracts
                .enterprise_governance_contract
                .to_string(),
        )
        .add_attribute(
            "enterprise_governance_controller_contract",
            component_contracts
                .enterprise_governance_controller_contract
                .to_string(),
        )
        .add_attribute(
            "enterprise_treasury_contract",
            component_contracts.enterprise_treasury_contract.to_string(),
        )
        .add_attribute(
            "funds_distributor_contract",
            component_contracts.funds_distributor_contract.to_string(),
        )
        .add_attribute(
            "membership_contract",
            component_contracts.membership_contract.to_string(),
        ))
}

fn update_metadata(ctx: &mut Context, msg: UpdateMetadataMsg) -> DaoResult<Response> {
    let component_contracts = COMPONENT_CONTRACTS.load(ctx.deps.storage)?;

    if component_contracts.enterprise_governance_controller_contract != ctx.info.sender {
        return Err(Unauthorized);
    }

    let mut metadata = DAO_METADATA.load(ctx.deps.storage)?;

    if let Change(name) = msg.name {
        metadata.name = name;
    }

    if let Change(description) = msg.description {
        metadata.description = description;
    }

    if let Change(logo) = msg.logo {
        metadata.logo = logo;
    }

    if let Change(github) = msg.github_username {
        metadata.socials.github_username = github;
    }
    if let Change(twitter) = msg.twitter_username {
        metadata.socials.twitter_username = twitter;
    }
    if let Change(discord) = msg.discord_username {
        metadata.socials.discord_username = discord;
    }
    if let Change(telegram) = msg.telegram_username {
        metadata.socials.telegram_username = telegram;
    }

    DAO_METADATA.save(ctx.deps.storage, &metadata)?;

    Ok(Response::new().add_attribute("action", "update_metadata"))
}

fn upgrade_dao(ctx: &mut Context, msg: UpgradeDaoMsg) -> DaoResult<Response> {
    let current_version = DAO_VERSION.load(ctx.deps.storage)?;

    if current_version >= msg.new_version {
        return Err(MigratingToLowerVersion {
            current: current_version,
            target: msg.new_version,
        });
    }

    let component_contracts = COMPONENT_CONTRACTS.load(ctx.deps.storage)?;

    if component_contracts.enterprise_governance_controller_contract != ctx.info.sender {
        return Err(Unauthorized);
    }

    let migrate_msg_json: serde_json::Value = serde_json::from_slice(msg.migrate_msg.as_slice())
        .map_err(|e| DaoError::Std(StdError::generic_err(e.to_string())))?;

    if let Object(migrate_msgs_map) = migrate_msg_json {
        let versions =
            get_versions_between_current_and_target(ctx, current_version, msg.new_version.clone())?;

        let mut submsgs = vec![];

        for version in versions {
            let msg = migrate_msgs_map.get(&version.version.to_string());
            let migrate_msg = match msg {
                Some(msg) => to_binary(msg)?,
                None => to_binary(&json!({}))?, // if no msg was supplied, just use an empty one
            };

            submsgs.push(SubMsg::new(Wasm(Migrate {
                contract_addr: ctx.env.contract.address.to_string(),
                new_code_id: version.enterprise_code_id,
                msg: migrate_msg,
            })));
        }

        Ok(Response::new()
            .add_attribute("action", "upgrade_dao")
            .add_attribute("new_version", msg.new_version.to_string())
            .add_submessages(submsgs))
    } else {
        Err(InvalidMigrateMsgMap)
    }
}

fn get_versions_between_current_and_target(
    ctx: &Context,
    current_version: Version,
    target_version: Version,
) -> DaoResult<Vec<VersionInfo>> {
    let enterprise_versioning = ENTERPRISE_VERSIONING_CONTRACT.load(ctx.deps.storage)?;

    let mut versions: Vec<VersionInfo> = vec![];
    let mut last_version = Some(current_version);

    loop {
        let versions_response: VersionsResponse = ctx.deps.querier.query_wasm_smart(
            enterprise_versioning.to_string(),
            &Versions(VersionsParams {
                start_after: last_version.clone(),
                limit: None,
            }),
        )?;

        if versions_response.versions.is_empty() {
            break;
        }

        last_version = versions_response
            .versions
            .last()
            .map(|info| info.version.clone());

        for version in versions_response.versions {
            if version.version > target_version {
                break;
            }

            versions.push(version.clone());

            if version.version == target_version {
                break;
            }
        }
    }

    Ok(versions)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> DaoResult<Response> {
    match msg.id {
        ENTERPRISE_TREASURY_REPLY_ID => {
            let addr = parse_instantiated_contract_addr(deps.as_ref(), msg)?;

            treasury_contract_created(deps, addr)
        }
        ENTERPRISE_GOVERNANCE_CONTROLLER_REPLY_ID => {
            let addr = parse_instantiated_contract_addr(deps.as_ref(), msg)?;

            governance_controller_contract_created(deps, addr)
        }
        COUNCIL_MEMBERSHIP_REPLY_ID => {
            let addr = parse_instantiated_contract_addr(deps.as_ref(), msg)?;

            council_membership_contract_created(deps, env, addr)
        }
        MEMBERSHIP_REPLY_ID => {
            let addr = parse_instantiated_contract_addr(deps.as_ref(), msg)?;

            membership_contract_created(deps, addr)
        }
        _ => Err(DaoError::Std(StdError::generic_err(
            "No such reply ID found",
        ))),
    }
}

fn parse_instantiated_contract_addr(deps: Deps, msg: Reply) -> DaoResult<Addr> {
    let contract_address = parse_reply_instantiate_data(msg)
        .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
        .contract_address;
    let addr = deps.api.addr_validate(&contract_address)?;

    Ok(addr)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> DaoResult<Binary> {
    let qctx = QueryContext::from(deps, env);

    let response = match msg {
        QueryMsg::DaoInfo {} => to_binary(&query_dao_info(qctx)?)?,
        QueryMsg::ComponentContracts {} => to_binary(&query_dao_info(qctx)?)?,
    };
    Ok(response)
}

pub fn query_dao_info(qctx: QueryContext) -> DaoResult<DaoInfoResponse> {
    let creation_date = DAO_CREATION_DATE.load(qctx.deps.storage)?;
    let metadata = DAO_METADATA.load(qctx.deps.storage)?;
    let dao_type = DAO_TYPE.load(qctx.deps.storage)?;
    let dao_version = DAO_VERSION.load(qctx.deps.storage)?;

    Ok(DaoInfoResponse {
        creation_date,
        metadata,
        dao_type,
        dao_version,
    })
}

pub fn query_component_contracts(qctx: QueryContext) -> DaoResult<ComponentContractsResponse> {
    let component_contracts = COMPONENT_CONTRACTS.load(qctx.deps.storage)?;
    let enterprise_factory_contract = ENTERPRISE_FACTORY_CONTRACT.load(qctx.deps.storage)?;

    Ok(ComponentContractsResponse {
        enterprise_factory_contract,
        enterprise_governance_contract: component_contracts.enterprise_governance_contract,
        enterprise_governance_controller_contract: component_contracts
            .enterprise_governance_controller_contract,
        enterprise_treasury_contract: component_contracts.enterprise_treasury_contract,
        funds_distributor_contract: component_contracts.funds_distributor_contract,
        membership_contract: component_contracts.membership_contract,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(mut deps: DepsMut, env: Env, _msg: MigrateMsg) -> DaoResult<Response> {
    // TODO: if version < 5, either fail or migrate to version 5 first

    let submsgs = migrate_to_rewrite(deps.branch(), env)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    DAO_VERSION.save(deps.storage, &Version::from_str(CONTRACT_VERSION)?)?;

    Ok(Response::new()
        .add_attribute("action", "migrate")
        .add_submessages(submsgs))
}
