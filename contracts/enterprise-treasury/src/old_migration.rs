use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_json_binary, Addr, DepsMut, StdError, SubMsg, Uint128, Uint64, WasmMsg};
use cw2::{get_contract_version, set_contract_version};
use cw_asset::AssetInfo;
use cw_storage_plus::{Item, Map};
use enterprise_factory_api::api::ConfigResponse;
use enterprise_factory_api::msg::QueryMsg::Config;
use enterprise_protocol::api::DaoType;
use enterprise_treasury_api::error::EnterpriseTreasuryResult;
use enterprise_versioning_api::api::{Version, VersionParams, VersionResponse};
use DaoType::{Multisig, Nft, Token};

const CONTRACT_NAME: &str = "crates.io:enterprise";

#[cw_serde]
struct FundsDistributorMigrateMsg {
    pub minimum_eligible_weight: Option<Uint128>,
}

#[cw_serde]
struct GovernanceMigrateMsg {}

const DAO_TYPE: Item<DaoType> = Item::new("dao_type");

const DAO_CODE_VERSION: Item<Uint64> = Item::new("dao_code_version");

const DAO_MEMBERSHIP_CONTRACT: Item<Addr> = Item::new("dao_membership_contract");

const ENTERPRISE_GOVERNANCE_CONTRACT: Item<Addr> = Item::new("enterprise_governance_contract");
const FUNDS_DISTRIBUTOR_CONTRACT: Item<Addr> = Item::new("funds_distributor_contract");
const ENTERPRISE_FACTORY_CONTRACT: Item<Addr> = Item::new("enterprise_factory_contract");

/// Old migration code, for migrating version codes below 5 (i.e. prior to 0.5.0) to version code 5.
pub fn migrate_to_v_0_5_0(mut deps: DepsMut) -> EnterpriseTreasuryResult<Vec<SubMsg>> {
    let contract_version = get_contract_version(deps.storage)?;

    if contract_version.version == "0.5.0" {
        // already on version 0.5.0, nothing to migrate
        return Ok(vec![]);
    }

    let mut submsgs: Vec<SubMsg> = vec![];

    if vec!["0.1.0", "0.2.0", "0.3.0"].contains(&contract_version.version.as_ref()) {
        let enterprise_factory = ENTERPRISE_FACTORY_CONTRACT.load(deps.storage)?;
        let factory_config: ConfigResponse = deps
            .querier
            .query_wasm_smart(enterprise_factory.to_string(), &Config {})?;

        let version_0_5_0 = Version {
            major: 0,
            minor: 5,
            patch: 0,
        };
        let version_0_5_0_info: VersionResponse = deps.querier.query_wasm_smart(
            factory_config.config.enterprise_versioning.to_string(),
            &enterprise_versioning_api::msg::QueryMsg::Version(VersionParams {
                version: version_0_5_0,
            }),
        )?;

        let funds_distributor = FUNDS_DISTRIBUTOR_CONTRACT.load(deps.storage)?;

        submsgs.push(SubMsg::new(WasmMsg::Migrate {
            contract_addr: funds_distributor.to_string(),
            new_code_id: version_0_5_0_info.version.funds_distributor_code_id,
            msg: to_json_binary(&FundsDistributorMigrateMsg {
                minimum_eligible_weight: None,
            })?,
        }));

        let enterprise_governance = ENTERPRISE_GOVERNANCE_CONTRACT.load(deps.storage)?;

        submsgs.push(SubMsg::new(WasmMsg::Migrate {
            contract_addr: enterprise_governance.to_string(),
            new_code_id: version_0_5_0_info.version.enterprise_governance_code_id,
            msg: to_json_binary(&GovernanceMigrateMsg {})?,
        }));
    }

    migrate_asset_whitelist(deps.branch())?;
    whitelist_dao_membership_asset(deps.branch())?;

    DAO_CODE_VERSION.save(deps.storage, &Uint64::from(5u8))?;
    set_contract_version(deps.storage, CONTRACT_NAME, "0.5.0")?;

    Ok(submsgs)
}

const ASSET_WHITELIST: Item<Vec<AssetInfo>> = Item::new("asset_whitelist");
const NFT_WHITELIST: Map<Addr, ()> = Map::new("nft_whitelist");

const NATIVE_ASSET_WHITELIST: Map<String, ()> = Map::new("native_asset_whitelist");
const CW20_ASSET_WHITELIST: Map<Addr, ()> = Map::new("cw20_asset_whitelist");
const CW1155_ASSET_WHITELIST: Map<(Addr, String), ()> = Map::new("cw1155_asset_whitelist");

fn migrate_asset_whitelist(deps: DepsMut) -> EnterpriseTreasuryResult<()> {
    let assets = ASSET_WHITELIST.may_load(deps.storage)?.unwrap_or_default();
    ASSET_WHITELIST.remove(deps.storage);

    add_whitelisted_assets(deps, assets)?;

    Ok(())
}

fn whitelist_dao_membership_asset(deps: DepsMut) -> EnterpriseTreasuryResult<()> {
    let dao_type = DAO_TYPE.load(deps.storage)?;
    match dao_type {
        Token => {
            let membership_token = DAO_MEMBERSHIP_CONTRACT.load(deps.storage)?;
            add_whitelisted_assets(deps, vec![AssetInfo::cw20(membership_token)])?;
        }
        Nft => {
            let membership_nft = DAO_MEMBERSHIP_CONTRACT.load(deps.storage)?;
            NFT_WHITELIST.save(deps.storage, membership_nft, &())?;
        }
        Multisig => {
            // no-op
        }
        DaoType::Denom => {
            return Err(StdError::generic_err("denom type did not exist in this version!").into())
        }
    }

    Ok(())
}

fn add_whitelisted_assets(deps: DepsMut, assets: Vec<AssetInfo>) -> EnterpriseTreasuryResult<()> {
    for asset in assets {
        match asset {
            AssetInfo::Native(denom) => NATIVE_ASSET_WHITELIST.save(deps.storage, denom, &())?,
            AssetInfo::Cw20(addr) => {
                let addr = deps.api.addr_validate(addr.as_ref())?;
                CW20_ASSET_WHITELIST.save(deps.storage, addr, &())?;
            }
            AssetInfo::Cw1155(addr, id) => {
                let addr = deps.api.addr_validate(addr.as_ref())?;
                CW1155_ASSET_WHITELIST.save(deps.storage, (addr, id), &())?;
            }
            _ => return Err(StdError::generic_err("unknown asset type").into()),
        }
    }

    Ok(())
}
