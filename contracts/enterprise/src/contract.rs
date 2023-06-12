use crate::migration::migrate_to_rewrite;
use crate::state::{
    ComponentContracts, COMPONENT_CONTRACTS, DAO_CODE_VERSION, DAO_CREATION_DATE, DAO_METADATA,
    DAO_TYPE, ENTERPRISE_FACTORY_CONTRACT, IS_INSTANTIATION_FINALIZED,
};
use common::commons::ModifyValue::Change;
use common::cw::{Context, QueryContext};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError,
    SubMsg, WasmMsg,
};
use cw2::set_contract_version;
use cw_utils::parse_reply_instantiate_data;
use enterprise_protocol::api::{
    ComponentContractsResponse, DaoInfoResponse, FinalizeInstantiationMsg, UpdateMetadataMsg,
    UpgradeDaoMsg,
};
use enterprise_protocol::error::DaoError::{AlreadyInitialized, Unauthorized};
use enterprise_protocol::error::{DaoError, DaoResult};
use enterprise_protocol::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

pub const ENTERPRISE_TREASURY_REPLY_ID: u64 = 1;
pub const ENTERPRISE_GOVERNANCE_CONTROLLER_REPLY_ID: u64 = 2;
pub const MEMBERSHIP_REPLY_ID: u64 = 3;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:enterprise";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const CODE_VERSION: u8 = 5;

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

    DAO_METADATA.save(deps.storage, &msg.dao_metadata)?;
    DAO_CODE_VERSION.save(deps.storage, &CODE_VERSION.into())?;

    IS_INSTANTIATION_FINALIZED.save(deps.storage, &false)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> DaoResult<Response> {
    let ctx = &mut Context { deps, env, info };

    match msg {
        ExecuteMsg::FinalizeInstantiation(msg) => finalize_instantiation(ctx, msg),
        ExecuteMsg::UpdateMetadata(msg) => update_metadata(ctx, msg),
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

// TODO: there should be a message, and then this should dispatch to other contracts
fn _upgrade_dao(env: Env, msg: UpgradeDaoMsg) -> DaoResult<Vec<SubMsg>> {
    Ok(vec![SubMsg::new(WasmMsg::Migrate {
        contract_addr: env.contract.address.to_string(),
        new_code_id: msg.new_dao_code_id,
        msg: msg.migrate_msg,
    })])
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> DaoResult<Response> {
    match msg.id {
        ENTERPRISE_TREASURY_REPLY_ID => {
            let contract_address = parse_reply_instantiate_data(msg)
                .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
                .contract_address;
            let addr = deps.api.addr_validate(&contract_address)?;

            let component_contracts = COMPONENT_CONTRACTS.load(deps.storage)?;

            COMPONENT_CONTRACTS.save(
                deps.storage,
                &ComponentContracts {
                    enterprise_treasury_contract: addr,
                    ..component_contracts
                },
            )?;

            Ok(Response::new())
        }
        ENTERPRISE_GOVERNANCE_CONTROLLER_REPLY_ID => {
            let contract_address = parse_reply_instantiate_data(msg)
                .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
                .contract_address;
            let addr = deps.api.addr_validate(&contract_address)?;

            let component_contracts = COMPONENT_CONTRACTS.load(deps.storage)?;

            COMPONENT_CONTRACTS.save(
                deps.storage,
                &ComponentContracts {
                    enterprise_governance_controller_contract: addr,
                    ..component_contracts
                },
            )?;

            Ok(Response::new())
        }
        MEMBERSHIP_REPLY_ID => {
            let contract_address = parse_reply_instantiate_data(msg)
                .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
                .contract_address;
            let addr = deps.api.addr_validate(&contract_address)?;

            let component_contracts = COMPONENT_CONTRACTS.load(deps.storage)?;

            COMPONENT_CONTRACTS.save(
                deps.storage,
                &ComponentContracts {
                    membership_contract: addr,
                    ..component_contracts
                },
            )?;

            Ok(Response::new())
        }
        _ => Err(DaoError::Std(StdError::generic_err(
            "No such reply ID found",
        ))),
    }
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
    let dao_code_version = DAO_CODE_VERSION.load(qctx.deps.storage)?;

    Ok(DaoInfoResponse {
        creation_date,
        metadata,
        dao_type,
        dao_code_version,
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
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> DaoResult<Response> {
    // TODO: if version < 5, either fail or migrate to version 5 first

    migrate_to_rewrite(
        deps,
        env,
        msg.treasury_code_id,
        msg.governance_controller_code_id,
        msg.token_membership_code_id,
        msg.nft_membership_code_id,
        msg.multisig_membership_code_id,
    )?;

    Ok(Response::new().add_attribute("action", "migrate"))
}
