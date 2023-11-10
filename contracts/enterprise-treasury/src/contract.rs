use crate::asset_whitelist::{
    add_whitelisted_assets, get_whitelisted_assets_starting_with_cw1155,
    get_whitelisted_assets_starting_with_cw20, get_whitelisted_assets_starting_with_native,
    remove_whitelisted_assets,
};
use crate::migration::{
    council_membership_contract_created, enterprise_contract_created,
    enterprise_outposts_contract_created, finalize_initial_migration_step,
    governance_controller_contract_created, membership_contract_created, migrate_to_rewrite,
    perform_next_migration_step,
};
use crate::migration_stages::MigrationStage::MigrationInProgress;
use crate::migration_stages::{MigrationStage, MIGRATION_TO_V_1_0_0_STAGE};
use crate::state::{Config, CONFIG, NFT_WHITELIST};
use crate::validate::admin_only;
use common::cw::{Context, QueryContext};
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{
    coin, entry_point, to_json_binary, wasm_execute, Addr, Binary, Coin, CosmosMsg, Deps, DepsMut,
    Env, MessageInfo, Reply, Response, StdError, StdResult, SubMsg,
};
use cw2::set_contract_version;
use cw_asset::{Asset, AssetInfoUnchecked};
use cw_storage_plus::Bound;
use cw_utils::parse_reply_instantiate_data;
use enterprise_treasury_api::api::{
    AssetWhitelistParams, AssetWhitelistResponse, ConfigResponse, DistributeFundsMsg,
    ExecuteCosmosMsgsMsg, HasIncompleteV2MigrationResponse, NftWhitelistParams,
    NftWhitelistResponse, SetAdminMsg, SpendMsg, UpdateAssetWhitelistMsg, UpdateNftWhitelistMsg,
};
use enterprise_treasury_api::error::EnterpriseTreasuryError::{InvalidCosmosMessage, Std};
use enterprise_treasury_api::error::EnterpriseTreasuryResult;
use enterprise_treasury_api::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use enterprise_treasury_api::response::{
    execute_distribute_funds_response, execute_execute_cosmos_msgs_response,
    execute_set_admin_response, execute_spend_response, execute_update_asset_whitelist_response,
    execute_update_nft_whitelist_response, instantiate_response,
};
use funds_distributor_api::msg::Cw20HookMsg::Distribute;
use funds_distributor_api::msg::ExecuteMsg::DistributeNative;
use std::ops::Not;
use MigrationStage::{Finalized, MigrationNotStarted};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:enterprise-treasury";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const ENTERPRISE_INSTANTIATE_REPLY_ID: u64 = 1;
pub const MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID: u64 = 2;
pub const COUNCIL_MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID: u64 = 3;
pub const ENTERPRISE_GOVERNANCE_CONTROLLER_INSTANTIATE_REPLY_ID: u64 = 4;
pub const ENTERPRISE_OUTPOSTS_INSTANTIATE_REPLY_ID: u64 = 5;
pub const EXECUTE_PROPOSAL_ACTIONS_REPLY_ID: u64 = 6;

const DEFAULT_QUERY_LIMIT: u8 = 30;
const MAX_QUERY_LIMIT: u8 = 100;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> EnterpriseTreasuryResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let admin = deps.api.addr_validate(&msg.admin)?;
    CONFIG.save(
        deps.storage,
        &Config {
            admin: admin.clone(),
        },
    )?;

    add_whitelisted_assets(deps.branch(), msg.asset_whitelist.unwrap_or_default())?;

    for nft in msg.nft_whitelist.unwrap_or_default() {
        let nft_addr = deps.api.addr_validate(&nft)?;
        NFT_WHITELIST.save(deps.storage, nft_addr, &())?;
    }

    Ok(instantiate_response(admin.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> EnterpriseTreasuryResult<Response> {
    let ctx = &mut Context { deps, env, info };

    match msg {
        ExecuteMsg::SetAdmin(msg) => set_admin(ctx, msg),
        ExecuteMsg::UpdateAssetWhitelist(msg) => update_asset_whitelist(ctx, msg),
        ExecuteMsg::UpdateNftWhitelist(msg) => update_nft_whitelist(ctx, msg),
        ExecuteMsg::Spend(msg) => spend(ctx, msg),
        ExecuteMsg::DistributeFunds(msg) => distribute_funds(ctx, msg),
        ExecuteMsg::ExecuteCosmosMsgs(msg) => execute_cosmos_msgs(ctx, msg),

        ExecuteMsg::FinalizeInitialMigrationStep {} => finalize_initial_migration_step(ctx),
        ExecuteMsg::PerformNextMigrationStep { submsgs_limit } => {
            perform_next_migration_step(ctx, submsgs_limit)
        }
    }
}

fn set_admin(ctx: &mut Context, msg: SetAdminMsg) -> EnterpriseTreasuryResult<Response> {
    admin_only(ctx)?;

    let new_admin = ctx.deps.api.addr_validate(&msg.new_admin)?;

    CONFIG.save(
        ctx.deps.storage,
        &Config {
            admin: new_admin.clone(),
        },
    )?;

    Ok(execute_set_admin_response(new_admin.to_string()))
}

fn update_asset_whitelist(
    ctx: &mut Context,
    msg: UpdateAssetWhitelistMsg,
) -> EnterpriseTreasuryResult<Response> {
    admin_only(ctx)?;

    add_whitelisted_assets(ctx.deps.branch(), msg.add)?;
    remove_whitelisted_assets(ctx.deps.branch(), msg.remove)?;

    Ok(execute_update_asset_whitelist_response())
}

fn update_nft_whitelist(
    ctx: &mut Context,
    msg: UpdateNftWhitelistMsg,
) -> EnterpriseTreasuryResult<Response> {
    admin_only(ctx)?;

    for add in msg.add {
        NFT_WHITELIST.save(ctx.deps.storage, ctx.deps.api.addr_validate(&add)?, &())?;
    }
    for remove in msg.remove {
        NFT_WHITELIST.remove(
            ctx.deps.storage,
            ctx.deps.api.addr_validate(remove.as_ref())?,
        );
    }

    Ok(execute_update_nft_whitelist_response())
}

fn spend(ctx: &mut Context, msg: SpendMsg) -> EnterpriseTreasuryResult<Response> {
    admin_only(ctx)?;

    // TODO: does not work with CW1155, make sure it does in the future
    let spend_submsgs = msg
        .assets
        .into_iter()
        .map(|asset_unchecked| asset_unchecked.check(ctx.deps.api, None))
        .map(|asset_res| match asset_res {
            Ok(asset) => asset.transfer_msg(msg.recipient.clone()).map(SubMsg::new),
            Err(e) => Err(e),
        })
        .collect::<StdResult<Vec<SubMsg>>>()?;

    Ok(execute_spend_response().add_submessages(spend_submsgs))
}

fn distribute_funds(
    ctx: &mut Context,
    msg: DistributeFundsMsg,
) -> EnterpriseTreasuryResult<Response> {
    admin_only(ctx)?;

    let funds_distributor = ctx
        .deps
        .api
        .addr_validate(&msg.funds_distributor_contract)?;

    let mut native_funds: Vec<Coin> = vec![];
    let mut submsgs: Vec<SubMsg> = vec![];

    for asset in msg.funds {
        match asset.info {
            AssetInfoUnchecked::Native(denom) => {
                native_funds.push(coin(asset.amount.u128(), denom))
            }
            AssetInfoUnchecked::Cw20(addr) => {
                let addr = ctx.deps.api.addr_validate(&addr)?;
                let asset = Asset::cw20(addr, asset.amount);
                submsgs.push(SubMsg::new(asset.send_msg(
                    funds_distributor.to_string(),
                    to_json_binary(&Distribute {})?,
                )?))
            }
            AssetInfoUnchecked::Cw1155(_, _) => {
                return Err(Std(StdError::generic_err(
                    "cw1155 assets are not supported at this time",
                )))
            }
            _ => return Err(Std(StdError::generic_err("unknown asset type"))),
        }
    }

    if native_funds.is_empty().not() {
        submsgs.push(SubMsg::new(wasm_execute(
            funds_distributor.to_string(),
            &DistributeNative {},
            native_funds,
        )?));
    }

    Ok(execute_distribute_funds_response().add_submessages(submsgs))
}

fn execute_cosmos_msgs(
    ctx: &mut Context,
    msg: ExecuteCosmosMsgsMsg,
) -> EnterpriseTreasuryResult<Response> {
    admin_only(ctx)?;

    let mut submsgs: Vec<SubMsg> = vec![];
    for msg in msg.msgs {
        submsgs.push(SubMsg::new(
            serde_json_wasm::from_str::<CosmosMsg>(msg.as_str())
                .map_err(|_| InvalidCosmosMessage)?,
        ))
    }

    Ok(execute_execute_cosmos_msgs_response().add_submessages(submsgs))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> EnterpriseTreasuryResult<Response> {
    match msg.id {
        ENTERPRISE_INSTANTIATE_REPLY_ID => {
            let enterprise_contract = parse_instantiated_contract_addr(deps.as_ref(), msg)?;

            enterprise_contract_created(deps, enterprise_contract)
        }
        ENTERPRISE_GOVERNANCE_CONTROLLER_INSTANTIATE_REPLY_ID => {
            let governance_controller_contract =
                parse_instantiated_contract_addr(deps.as_ref(), msg)?;

            governance_controller_contract_created(deps, governance_controller_contract)
        }
        COUNCIL_MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID => {
            let council_membership_contract = parse_instantiated_contract_addr(deps.as_ref(), msg)?;

            council_membership_contract_created(deps, council_membership_contract)
        }
        MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID => {
            let membership_contract = parse_instantiated_contract_addr(deps.as_ref(), msg)?;

            membership_contract_created(deps, membership_contract)
        }
        ENTERPRISE_OUTPOSTS_INSTANTIATE_REPLY_ID => {
            let enterprise_outposts_contract =
                parse_instantiated_contract_addr(deps.as_ref(), msg)?;

            enterprise_outposts_contract_created(deps, enterprise_outposts_contract)
        }
        EXECUTE_PROPOSAL_ACTIONS_REPLY_ID => {
            // no actions, regardless of the result
            // here for compatibility while migrating old DAOs
            Ok(Response::new())
        }
        _ => Err(Std(StdError::generic_err("No such reply ID found"))),
    }
}

fn parse_instantiated_contract_addr(deps: Deps, msg: Reply) -> EnterpriseTreasuryResult<Addr> {
    let contract_address = parse_reply_instantiate_data(msg)
        .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
        .contract_address;
    let addr = deps.api.addr_validate(&contract_address)?;

    Ok(addr)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> EnterpriseTreasuryResult<Binary> {
    let qctx = QueryContext { deps, env };

    let response = match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(qctx)?)?,
        QueryMsg::AssetWhitelist(params) => to_json_binary(&query_asset_whitelist(qctx, params)?)?,
        QueryMsg::NftWhitelist(params) => to_json_binary(&query_nft_whitelist(qctx, params)?)?,
        QueryMsg::HasIncompleteV2Migration {} => {
            to_json_binary(&query_has_incomplete_v2_migration(qctx)?)?
        }
    };

    Ok(response)
}

pub fn query_config(qctx: QueryContext) -> EnterpriseTreasuryResult<ConfigResponse> {
    let config = CONFIG.load(qctx.deps.storage)?;

    Ok(ConfigResponse {
        admin: config.admin,
    })
}

pub fn query_asset_whitelist(
    qctx: QueryContext,
    params: AssetWhitelistParams,
) -> EnterpriseTreasuryResult<AssetWhitelistResponse> {
    let limit = params
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT as u32)
        .min(MAX_QUERY_LIMIT as u32) as usize;

    let assets = if let Some(start_after) = params.start_after {
        match start_after {
            AssetInfoUnchecked::Native(denom) => {
                get_whitelisted_assets_starting_with_native(qctx, Some(denom), limit)?
            }
            AssetInfoUnchecked::Cw20(addr) => {
                let addr = qctx.deps.api.addr_validate(addr.as_ref())?;
                get_whitelisted_assets_starting_with_cw20(qctx, Some(addr), limit)?
            }
            AssetInfoUnchecked::Cw1155(addr, id) => {
                let addr = qctx.deps.api.addr_validate(addr.as_ref())?;
                get_whitelisted_assets_starting_with_cw1155(qctx, Some((addr, id)), limit)?
            }
            _ => return Err(StdError::generic_err("unknown asset type").into()),
        }
    } else {
        get_whitelisted_assets_starting_with_native(qctx, None, limit)?
    };

    Ok(AssetWhitelistResponse { assets })
}

pub fn query_nft_whitelist(
    qctx: QueryContext,
    params: NftWhitelistParams,
) -> EnterpriseTreasuryResult<NftWhitelistResponse> {
    let start_after = params
        .start_after
        .map(|addr| qctx.deps.api.addr_validate(&addr))
        .transpose()?
        .map(Bound::exclusive);

    let limit = params
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT as u32)
        .min(MAX_QUERY_LIMIT as u32);

    let nfts = NFT_WHITELIST
        .range(qctx.deps.storage, start_after, None, Ascending)
        .take(limit as usize)
        .collect::<StdResult<Vec<(Addr, ())>>>()?
        .into_iter()
        .map(|(addr, _)| addr)
        .collect();

    Ok(NftWhitelistResponse { nfts })
}

pub fn query_has_incomplete_v2_migration(
    qctx: QueryContext,
) -> EnterpriseTreasuryResult<HasIncompleteV2MigrationResponse> {
    let migration_stage = MIGRATION_TO_V_1_0_0_STAGE.may_load(qctx.deps.storage)?;

    let has_incomplete_migration = match migration_stage {
        None => false,
        Some(migration_stage) => match migration_stage {
            MigrationInProgress => true,
            MigrationNotStarted | Finalized => false,
        },
    };

    Ok(HasIncompleteV2MigrationResponse {
        has_incomplete_migration,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    mut deps: DepsMut,
    env: Env,
    _msg: MigrateMsg,
) -> EnterpriseTreasuryResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let submsgs = migrate_to_rewrite(deps.branch(), env)?;

    Ok(Response::new()
        .add_attribute("action", "migrate")
        .add_submessages(submsgs))
}
