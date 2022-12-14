use crate::state::{DAO_TYPE, ENTERPRISE_FACTORY_CONTRACT};
use common::cw::Context;
use cosmwasm_std::{Addr, CosmosMsg, Decimal, Deps, StdResult};
use cw20::TokenInfoResponse;
use cw3::VoterListResponse;
use cw721::NumTokensResponse;
use cw_asset::AssetInfo;
use cw_utils::Duration;
use enterprise_factory_api::api::{IsEnterpriseCodeIdMsg, IsEnterpriseCodeIdResponse};
use enterprise_protocol::api::DaoType::{Multisig, Nft, Token};
use enterprise_protocol::api::ProposalAction::{ExecuteMsgs, ModifyMultisigMembership, UpgradeDao};
use enterprise_protocol::api::{
    DaoGovConfig, DaoType, ExecuteMsgsMsg, ModifyMultisigMembershipMsg, ProposalAction,
    ProposalDeposit, UpgradeDaoMsg,
};
use enterprise_protocol::error::DaoError::{
    InsufficientProposalDeposit, InvalidArgument, InvalidCosmosMessage, MinimumDepositNotAllowed,
};
use enterprise_protocol::error::{DaoError, DaoResult};
use std::collections::HashSet;
use DaoError::{
    InvalidEnterpriseCodeId, InvalidExistingMultisigContract, InvalidExistingNftContract,
    InvalidExistingTokenContract, UnsupportedOperationForDaoType, VoteDurationLongerThanUnstaking,
};
use ProposalAction::{UpdateAssetWhitelist, UpdateNftWhitelist};

pub fn validate_dao_gov_config(dao_type: &DaoType, dao_gov_config: &DaoGovConfig) -> DaoResult<()> {
    if let Duration::Time(unlocking_time) = dao_gov_config.unlocking_period {
        if unlocking_time < dao_gov_config.vote_duration {
            return Err(VoteDurationLongerThanUnstaking {});
        }
    }

    if dao_gov_config.quorum > Decimal::one() || dao_gov_config.quorum == Decimal::zero() {
        return Err(InvalidArgument {
            msg: "Invalid quorum, must be 0 < quorum <= 1".to_string(),
        });
    }

    if dao_gov_config.threshold > Decimal::one() || dao_gov_config.threshold == Decimal::zero() {
        return Err(InvalidArgument {
            msg: "Invalid threshold, must be 0 < threshold <= 1".to_string(),
        });
    }

    if dao_gov_config.minimum_deposit.is_some() && (dao_type == &Nft || dao_type == &Multisig) {
        return Err(MinimumDepositNotAllowed {});
    }

    Ok(())
}

pub fn validate_deposit(
    gov_config: &DaoGovConfig,
    deposit: &Option<ProposalDeposit>,
) -> DaoResult<()> {
    match gov_config.minimum_deposit {
        None => Ok(()),
        Some(required_amount) => {
            let deposited_amount = deposit
                .as_ref()
                .map(|deposit| deposit.amount)
                .unwrap_or_default();

            if deposited_amount >= required_amount {
                Ok(())
            } else {
                Err(InsufficientProposalDeposit { required_amount })
            }
        }
    }
}

pub fn validate_existing_dao_contract(
    ctx: &Context,
    dao_type: &DaoType,
    contract: &str,
) -> DaoResult<()> {
    match dao_type {
        Token => {
            let query = cw20::Cw20QueryMsg::TokenInfo {};
            let result: StdResult<TokenInfoResponse> =
                ctx.deps.querier.query_wasm_smart(contract, &query);

            result.map_err(|_| InvalidExistingTokenContract)?;
        }
        Nft => {
            let query = cw721::Cw721QueryMsg::NumTokens {};
            let result: StdResult<NumTokensResponse> =
                ctx.deps.querier.query_wasm_smart(contract, &query);

            result.map_err(|_| InvalidExistingNftContract)?;
        }
        Multisig => {
            let query = cw3::Cw3QueryMsg::ListVoters {
                start_after: None,
                limit: Some(10u32),
            };
            let result: StdResult<VoterListResponse> =
                ctx.deps.querier.query_wasm_smart(contract, &query);

            result.map_err(|_| InvalidExistingMultisigContract)?;
        }
    }

    Ok(())
}

pub fn validate_proposal_actions(
    deps: Deps,
    proposal_actions: &Vec<ProposalAction>,
) -> DaoResult<()> {
    for proposal_action in proposal_actions {
        match proposal_action {
            UpdateAssetWhitelist(msg) => validate_asset_whitelist_changes(&msg.add, &msg.remove)?,
            UpdateNftWhitelist(msg) => validate_nft_whitelist_changes(&msg.add, &msg.remove)?,
            UpgradeDao(msg) => validate_upgrade_dao(deps, msg)?,
            ExecuteMsgs(msg) => validate_execute_msgs(msg)?,
            ModifyMultisigMembership(msg) => validate_modify_multisig_membership(deps, msg)?,
            _ => {}
        }
    }

    Ok(())
}

fn validate_asset_whitelist_changes(
    add: &Vec<AssetInfo>,
    remove: &Vec<AssetInfo>,
) -> DaoResult<()> {
    let add_asset_hashsets = split_asset_hashsets(add)?;
    let remove_asset_hashsets = split_asset_hashsets(remove)?;

    if add_asset_hashsets
        .native
        .intersection(&remove_asset_hashsets.native)
        .count()
        > 0usize
    {
        return Err(DaoError::AssetPresentInBothAddAndRemove);
    }
    if add_asset_hashsets
        .cw20
        .intersection(&remove_asset_hashsets.cw20)
        .count()
        > 0usize
    {
        return Err(DaoError::AssetPresentInBothAddAndRemove);
    }
    if add_asset_hashsets
        .cw1155
        .intersection(&remove_asset_hashsets.cw1155)
        .count()
        > 0usize
    {
        return Err(DaoError::AssetPresentInBothAddAndRemove);
    }

    Ok(())
}

fn split_asset_hashsets(assets: &Vec<AssetInfo>) -> DaoResult<AssetInfoHashSets> {
    let mut native_assets: HashSet<String> = HashSet::new();
    let mut cw20_assets: HashSet<Addr> = HashSet::new();
    let mut cw1155_assets: HashSet<(Addr, String)> = HashSet::new();
    for asset in assets {
        match asset {
            AssetInfo::Native(denom) => {
                if native_assets.contains(denom) {
                    return Err(DaoError::DuplicateAssetFound);
                } else {
                    native_assets.insert(denom.clone());
                }
            }
            AssetInfo::Cw20(addr) => {
                if cw20_assets.contains(addr) {
                    return Err(DaoError::DuplicateAssetFound);
                } else {
                    cw20_assets.insert(addr.clone());
                }
            }
            AssetInfo::Cw1155(addr, id) => {
                if cw1155_assets.contains(&(addr.clone(), id.to_string())) {
                    return Err(DaoError::DuplicateAssetFound);
                } else {
                    cw1155_assets.insert((addr.clone(), id.to_string()));
                }
            }
            _ => {
                return Err(DaoError::CustomError {
                    val: "Unsupported whitelist asset type".to_string(),
                })
            }
        }
    }

    Ok(AssetInfoHashSets {
        native: native_assets,
        cw20: cw20_assets,
        cw1155: cw1155_assets,
    })
}

struct AssetInfoHashSets {
    pub native: HashSet<String>,
    pub cw20: HashSet<Addr>,
    pub cw1155: HashSet<(Addr, String)>,
}

fn validate_nft_whitelist_changes(add: &Vec<Addr>, remove: &Vec<Addr>) -> DaoResult<()> {
    let mut add_nfts: HashSet<&Addr> = HashSet::new();
    for nft in add {
        if add_nfts.contains(nft) {
            return Err(DaoError::DuplicateNftFound);
        } else {
            add_nfts.insert(nft);
        }
    }

    let mut remove_nfts: HashSet<&Addr> = HashSet::new();
    for nft in remove {
        if remove_nfts.contains(nft) {
            return Err(DaoError::DuplicateNftFound);
        } else {
            remove_nfts.insert(nft);
        }
    }

    if add_nfts.intersection(&remove_nfts).count() > 0usize {
        return Err(DaoError::NftPresentInBothAddAndRemove);
    }

    Ok(())
}

fn validate_upgrade_dao(deps: Deps, msg: &UpgradeDaoMsg) -> DaoResult<()> {
    let enterprise_factory = ENTERPRISE_FACTORY_CONTRACT.load(deps.storage)?;
    let response: IsEnterpriseCodeIdResponse = deps.querier.query_wasm_smart(
        enterprise_factory.to_string(),
        &enterprise_factory_api::msg::QueryMsg::IsEnterpriseCodeId(IsEnterpriseCodeIdMsg {
            code_id: msg.new_dao_code_id.into(),
        }),
    )?;

    if !response.is_enterprise_code_id {
        Err(InvalidEnterpriseCodeId {
            code_id: msg.new_dao_code_id,
        })
    } else {
        Ok(())
    }
}

fn validate_execute_msgs(msg: &ExecuteMsgsMsg) -> DaoResult<()> {
    for msg in msg.msgs.iter() {
        serde_json_wasm::from_str::<CosmosMsg>(msg.as_str()).map_err(|_| InvalidCosmosMessage)?;
    }
    Ok(())
}

pub fn validate_modify_multisig_membership(
    deps: Deps,
    _msg: &ModifyMultisigMembershipMsg,
) -> DaoResult<()> {
    let dao_type = DAO_TYPE.load(deps.storage)?;

    if dao_type != Multisig {
        return Err(UnsupportedOperationForDaoType {
            dao_type: dao_type.to_string(),
        });
    }
    Ok(())
}
