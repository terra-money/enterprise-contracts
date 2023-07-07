use crate::asset_whitelist::{
    add_whitelisted_assets, get_whitelisted_assets_starting_with_cw1155,
    get_whitelisted_assets_starting_with_cw20, get_whitelisted_assets_starting_with_native,
    remove_whitelisted_assets,
};
use crate::state::{Config, CONFIG, ENTERPRISE_CONTRACT, NFT_WHITELIST};
use crate::validate::admin_only;
use common::cw::{Context, QueryContext};
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{
    coin, entry_point, to_binary, wasm_execute, Addr, Binary, Coin, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Reply, Response, StdError, StdResult, SubMsg,
};
use cw2::set_contract_version;
use cw_asset::{Asset, AssetInfoUnchecked};
use cw_storage_plus::Bound;
use enterprise_protocol::api::ComponentContractsResponse;
use enterprise_protocol::msg::QueryMsg::ComponentContracts;
use enterprise_treasury_api::api::{
    AssetWhitelistParams, AssetWhitelistResponse, ConfigResponse, DistributeFundsMsg,
    ExecuteCosmosMsgsMsg, NftWhitelistParams, NftWhitelistResponse, SpendMsg,
    UpdateAssetWhitelistMsg, UpdateConfigMsg, UpdateNftWhitelistMsg,
};
use enterprise_treasury_api::error::EnterpriseTreasuryError::{InvalidCosmosMessage, Std};
use enterprise_treasury_api::error::EnterpriseTreasuryResult;
use enterprise_treasury_api::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use funds_distributor_api::msg::Cw20HookMsg::Distribute;
use funds_distributor_api::msg::ExecuteMsg::DistributeNative;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:enterprise-treasury";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

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

    let enterprise_contract = deps.api.addr_validate(&msg.enterprise_contract)?;
    ENTERPRISE_CONTRACT.save(deps.storage, &enterprise_contract)?;

    add_whitelisted_assets(deps.branch(), msg.asset_whitelist.unwrap_or_default())?;

    for nft in msg.nft_whitelist.unwrap_or_default() {
        let nft_addr = deps.api.addr_validate(&nft)?;
        NFT_WHITELIST.save(deps.storage, nft_addr, &())?;
    }

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", admin.to_string()))
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
        ExecuteMsg::UpdateConfig(msg) => update_config(ctx, msg),
        ExecuteMsg::UpdateAssetWhitelist(msg) => update_asset_whitelist(ctx, msg),
        ExecuteMsg::UpdateNftWhitelist(msg) => update_nft_whitelist(ctx, msg),
        ExecuteMsg::Spend(msg) => spend(ctx, msg),
        ExecuteMsg::DistributeFunds(msg) => distribute_funds(ctx, msg),
        ExecuteMsg::ExecuteCosmosMsgs(msg) => execute_cosmos_msgs(ctx, msg),
    }
}

fn update_config(ctx: &mut Context, msg: UpdateConfigMsg) -> EnterpriseTreasuryResult<Response> {
    admin_only(ctx)?;

    let new_admin = ctx.deps.api.addr_validate(&msg.new_admin)?;

    CONFIG.save(
        ctx.deps.storage,
        &Config {
            admin: new_admin.clone(),
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("new_admin", new_admin.to_string()))
}

fn update_asset_whitelist(
    ctx: &mut Context,
    msg: UpdateAssetWhitelistMsg,
) -> EnterpriseTreasuryResult<Response> {
    admin_only(ctx)?;

    add_whitelisted_assets(ctx.deps.branch(), msg.add)?;
    remove_whitelisted_assets(ctx.deps.branch(), msg.remove)?;

    Ok(Response::new().add_attribute("action", "update_asset_whitelist"))
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

    Ok(Response::new().add_attribute("action", "update_nft_whitelist"))
}

fn spend(ctx: &mut Context, msg: SpendMsg) -> EnterpriseTreasuryResult<Response> {
    admin_only(ctx)?;

    // TODO: does not work with CW1155, make sure it does in the future
    let spend_submsgs = msg
        .assets
        .into_iter()
        .map(|asset_unchecked| asset_unchecked.check(ctx.deps.api, None))
        .map(|asset_res| match asset_res {
            Ok(asset) => asset.transfer_msg(msg.recipient.clone()),
            Err(e) => Err(e),
        })
        .collect::<StdResult<Vec<CosmosMsg>>>()?
        .into_iter()
        .map(SubMsg::new)
        .collect::<Vec<SubMsg>>();

    Ok(Response::new()
        .add_attribute("action", "spend")
        .add_submessages(spend_submsgs))
}

fn distribute_funds(
    ctx: &mut Context,
    msg: DistributeFundsMsg,
) -> EnterpriseTreasuryResult<Response> {
    admin_only(ctx)?;

    let enterprise_contract = ENTERPRISE_CONTRACT.load(ctx.deps.storage)?;
    let component_contracts: ComponentContractsResponse = ctx
        .deps
        .querier
        .query_wasm_smart(enterprise_contract.to_string(), &ComponentContracts {})?;
    let funds_distributor = component_contracts.funds_distributor_contract;

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
                    to_binary(&Distribute {})?,
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

    submsgs.push(SubMsg::new(wasm_execute(
        funds_distributor.to_string(),
        &DistributeNative {},
        native_funds,
    )?));

    Ok(Response::new().add_attribute("action", "distribute_funds"))
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

    Ok(Response::new()
        .add_attribute("action", "execute_cosmos_msgs")
        .add_submessages(submsgs))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> EnterpriseTreasuryResult<Response> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> EnterpriseTreasuryResult<Binary> {
    let qctx = QueryContext { deps, env };

    let response = match msg {
        QueryMsg::Config {} => to_binary(&query_config(qctx)?)?,
        QueryMsg::AssetWhitelist(params) => to_binary(&query_asset_whitelist(qctx, params)?)?,
        QueryMsg::NftWhitelist(params) => to_binary(&query_nft_whitelist(qctx, params)?)?,
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> EnterpriseTreasuryResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("action", "migrate"))
}
