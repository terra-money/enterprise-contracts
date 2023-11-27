use crate::migration_copy_storage::MIGRATED_USER_WEIGHTS;
use crate::migration_stages::MigrationStage::MigrationCompleted;
use crate::migration_stages::{MigrationStage, MIGRATION_TO_V_1_0_0_STAGE};
use crate::nft_staking::{load_all_nft_stakes_for_user, NFT_STAKES};
use crate::staking::CW20_STAKES;
use common::cw::{Context, ReleaseAt};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{
    to_json_binary, wasm_execute, Addr, Deps, DepsMut, Response, StdError, StdResult, Storage,
    SubMsg, Uint128,
};
use cw_asset::Asset;
use cw_storage_plus::{Item, Map};
use enterprise_protocol::api::DaoType;
use enterprise_treasury_api::error::EnterpriseTreasuryError::{InvalidMigrationStage, Std};
use enterprise_treasury_api::error::EnterpriseTreasuryResult;
use enterprise_versioning_api::api::VersionInfo;
use nft_staking_api::api::NftTokenId;
use nft_staking_api::msg::Cw721HookMsg::AddClaim;
use token_staking_api::api::{UserClaim, UserStake};
use token_staking_api::msg::Cw20HookMsg::AddClaims;
use MigrationStage::{MigrationInProgress, MigrationNotStarted};

const DEFAULT_CW20_SUBMSGS_LIMIT: u32 = 100;
const DEFAULT_NFT_SUBMSGS_LIMIT: u32 = 20;

const DAO_TYPE: Item<DaoType> = Item::new("dao_type");

const DAO_MEMBERSHIP_CONTRACT: Item<Addr> = Item::new("dao_membership_contract");

const MULTISIG_MEMBERS: Map<Addr, Uint128> = Map::new("multisig_members");

pub const CLAIMS: Map<&Addr, Vec<Claim>> = Map::new("claims");

#[cw_serde]
pub struct Claim {
    pub asset: ClaimAsset,
    pub release_at: ReleaseAt,
}

#[cw_serde]
pub enum ClaimAsset {
    Cw20(Cw20ClaimAsset),
    Cw721(Cw721ClaimAsset),
}

#[cw_serde]
pub struct Cw20ClaimAsset {
    pub amount: Uint128,
}

#[cw_serde]
pub struct Cw721ClaimAsset {
    pub tokens: Vec<NftTokenId>,
}

#[cw_serde]
struct MigrationInfo {
    pub version_info: VersionInfo,
    pub enterprise_contract: Option<Addr>,
    pub enterprise_governance_controller_contract: Option<Addr>,
    pub enterprise_outposts_contract: Option<Addr>,
    pub membership_contract: Option<Addr>,
    pub council_membership_contract: Option<Addr>,
    pub initial_submsgs_limit: Option<u32>,
}
const MIGRATION_INFO: Item<MigrationInfo> = Item::new("migration_info");

/// Carries result of an operation that consumes a certain amount of items to perform itself.
// TODO: this is a really shitty name, think of a better one
struct ResultWithItemsConsumed<T> {
    pub result: T,
    pub items_consumed: u32,
}

fn finalize_membership_contract_submsgs(
    mut deps: DepsMut,
    membership_contract: Addr,
    limit: u32,
) -> EnterpriseTreasuryResult<ResultWithItemsConsumed<Vec<SubMsg>>> {
    let dao_type = DAO_TYPE.load(deps.storage)?;

    let finalize_membership_msgs = match dao_type {
        DaoType::Denom => {
            return Err(Std(StdError::generic_err(
                "Denom membership was not supported prior to this migration!",
            )))
        }
        DaoType::Token => {
            let cw20_contract = DAO_MEMBERSHIP_CONTRACT.load(deps.storage)?;

            let ResultWithItemsConsumed {
                result: stakes_submsg,
                items_consumed: items_consumed_stakes,
            } = migrate_and_clear_cw20_stakes_submsg(
                deps.branch(),
                cw20_contract.clone(),
                membership_contract.clone(),
                limit,
            )?;

            let ResultWithItemsConsumed {
                result: claims_submsg,
                items_consumed: items_consumed_claims,
            } = migrate_and_clear_cw20_claims_submsg(
                deps.branch(),
                cw20_contract,
                membership_contract,
                limit - items_consumed_stakes,
            )?;

            let items_consumed = items_consumed_stakes + items_consumed_claims;

            ResultWithItemsConsumed {
                result: vec![stakes_submsg, claims_submsg]
                    .into_iter()
                    .flatten()
                    .collect(),
                items_consumed,
            }
        }
        DaoType::Nft => {
            let cw721_contract = DAO_MEMBERSHIP_CONTRACT.load(deps.storage)?;

            let mut migrate_stakes_submsgs = migrate_and_clear_cw721_stakes_submsgs(
                deps.branch(),
                cw721_contract.clone(),
                membership_contract.clone(),
                limit,
            )?;

            let remaining_limit = limit - migrate_stakes_submsgs.len() as u32;

            let mut claim_submsgs = migrate_and_clear_cw721_claims_submsgs(
                deps,
                cw721_contract,
                membership_contract,
                remaining_limit,
            )?;

            migrate_stakes_submsgs.append(&mut claim_submsgs);

            let items_consumed = migrate_stakes_submsgs.len() as u32;

            ResultWithItemsConsumed {
                result: migrate_stakes_submsgs,
                items_consumed,
            }
        }
        DaoType::Multisig => {
            // nothing to finalize in multisig DAOs
            ResultWithItemsConsumed {
                result: vec![],
                items_consumed: 0,
            } // TODO: will this prevent multisigs from being finalized? test
        }
    };

    Ok(finalize_membership_msgs)
}

fn migrate_and_clear_cw20_stakes_submsg(
    deps: DepsMut,
    cw20_contract: Addr,
    membership_contract: Addr,
    limit: u32,
) -> EnterpriseTreasuryResult<ResultWithItemsConsumed<Option<SubMsg>>> {
    let mut total_stakes = Uint128::zero();
    let mut stakers_to_remove: Vec<(Addr, Uint128)> = vec![];

    let stakers = CW20_STAKES
        .range(deps.storage, None, None, Ascending)
        .take(limit as usize)
        .map(|res| {
            res.map(|(user, amount)| {
                total_stakes += amount;
                stakers_to_remove.push((user.clone(), amount));
                UserStake {
                    user: user.to_string(),
                    staked_amount: amount,
                }
            })
        })
        .collect::<StdResult<Vec<UserStake>>>()?;

    let items_consumed = stakers_to_remove.len() as u32;

    for (staker, weight) in stakers_to_remove {
        CW20_STAKES.remove(deps.storage, staker.clone());
        MIGRATED_USER_WEIGHTS.save(deps.storage, staker, &weight)?;
    }

    if total_stakes.is_zero() {
        Ok(ResultWithItemsConsumed {
            result: None,
            items_consumed,
        })
    } else {
        Ok(ResultWithItemsConsumed {
            result: Some(SubMsg::new(
                Asset::cw20(cw20_contract, total_stakes).send_msg(
                    membership_contract,
                    to_json_binary(&token_staking_api::msg::Cw20HookMsg::AddStakes { stakers })?,
                )?,
            )),
            items_consumed,
        })
    }
}

fn migrate_and_clear_cw20_claims_submsg(
    deps: DepsMut,
    cw20_contract: Addr,
    membership_contract: Addr,
    limit: u32,
) -> EnterpriseTreasuryResult<ResultWithItemsConsumed<Option<SubMsg>>> {
    let mut total_claims_amount = Uint128::zero();

    let mut claims_included = 0u32;

    let mut claims_to_send: Vec<UserClaim> = vec![];

    while claims_included < limit && !CLAIMS.is_empty(deps.storage) {
        let mut claims_to_replace: Vec<(Addr, Vec<Claim>)> = vec![];

        for claim in CLAIMS
            .range(deps.storage, None, None, Ascending)
            .take(limit as usize)
        {
            if claims_included >= limit {
                break;
            }

            let (user, claims) = claim?;

            let mut claims_remaining: Vec<Claim> = vec![];

            for claim in claims {
                match claim.asset {
                    ClaimAsset::Cw20(asset) if claims_included < limit => {
                        total_claims_amount += asset.amount;
                        claims_included += 1;
                        claims_to_send.push(UserClaim {
                            user: user.to_string(),
                            claim_amount: asset.amount,
                            release_at: claim.release_at,
                        });
                    }
                    _ => claims_remaining.push(claim),
                }
            }

            claims_to_replace.push((user, claims_remaining));
        }

        for (user, claims_remaining) in claims_to_replace {
            if claims_remaining.is_empty() {
                CLAIMS.remove(deps.storage, &user);
            } else {
                CLAIMS.save(deps.storage, &user, &claims_remaining)?;
            }
        }
    }

    let items_consumed = claims_included;

    if total_claims_amount.is_zero() {
        Ok(ResultWithItemsConsumed {
            result: None,
            items_consumed,
        })
    } else {
        let migrate_claims_submsg = SubMsg::new(wasm_execute(
            cw20_contract.to_string(),
            &cw20::Cw20ExecuteMsg::Send {
                contract: membership_contract.to_string(),
                amount: total_claims_amount,
                msg: to_json_binary(&AddClaims {
                    claims: claims_to_send,
                })?,
            },
            vec![],
        )?);

        Ok(ResultWithItemsConsumed {
            result: Some(migrate_claims_submsg),
            items_consumed,
        })
    }
}

fn migrate_and_clear_cw721_stakes_submsgs(
    deps: DepsMut,
    cw721_contract: Addr,
    membership_contract: Addr,
    limit: u32,
) -> EnterpriseTreasuryResult<Vec<SubMsg>> {
    let mut migrate_stakes_submsgs = vec![];

    let mut stakes_to_remove: Vec<(NftTokenId, Addr)> = vec![];

    for stake_res in NFT_STAKES()
        .range(deps.storage, None, None, Ascending)
        .take(limit as usize)
    {
        let (key, stake) = stake_res?;

        let submsg = wasm_execute(
            cw721_contract.to_string(),
            &cw721::Cw721ExecuteMsg::SendNft {
                contract: membership_contract.to_string(),
                token_id: stake.token_id,
                msg: to_json_binary(&nft_staking_api::msg::Cw721HookMsg::Stake {
                    user: stake.staker.to_string(),
                })?,
            },
            vec![],
        )?;

        migrate_stakes_submsgs.push(SubMsg::new(submsg));

        stakes_to_remove.push((key, stake.staker));
    }

    for (stake_key, staker) in stakes_to_remove {
        NFT_STAKES().remove(deps.storage, stake_key)?;

        let previously_migrated_stake = MIGRATED_USER_WEIGHTS
            .may_load(deps.storage, staker.clone())?
            .unwrap_or_default();
        MIGRATED_USER_WEIGHTS.save(
            deps.storage,
            staker,
            &(previously_migrated_stake.checked_add(Uint128::one())?),
        )?;
    }

    Ok(migrate_stakes_submsgs)
}

fn migrate_and_clear_cw721_claims_submsgs(
    deps: DepsMut,
    cw721_contract: Addr,
    membership_contract: Addr,
    limit: u32,
) -> EnterpriseTreasuryResult<Vec<SubMsg>> {
    let mut claim_submsgs = vec![];

    let mut claims_included = 0u32;

    while claims_included < limit && !CLAIMS.is_empty(deps.storage) {
        let mut claim_keys_to_remove: Vec<Addr> = vec![];

        for claim_res in CLAIMS
            .range(deps.storage, None, None, Ascending)
            .take(limit as usize)
        {
            let (user, claims) = claim_res?;

            for claim in claims {
                match claim.asset {
                    ClaimAsset::Cw20(_) => continue,
                    ClaimAsset::Cw721(asset) => {
                        for token in asset.tokens {
                            let submsg = SubMsg::new(wasm_execute(
                                cw721_contract.to_string(),
                                &cw721::Cw721ExecuteMsg::SendNft {
                                    contract: membership_contract.to_string(),
                                    token_id: token,
                                    msg: to_json_binary(&AddClaim {
                                        user: user.to_string(),
                                        release_at: claim.release_at.clone(),
                                    })?,
                                },
                                vec![],
                            )?);
                            claim_submsgs.push(submsg);
                            claims_included += 1;
                        }
                    }
                }
            }

            claim_keys_to_remove.push(user);
        }

        for claim_key in claim_keys_to_remove {
            CLAIMS.remove(deps.storage, &claim_key);
        }
    }

    Ok(claim_submsgs)
}

pub fn perform_next_migration_step(
    ctx: &mut Context,
    submsgs_limit: Option<u32>,
) -> EnterpriseTreasuryResult<Response> {
    let migration_stage = MIGRATION_TO_V_1_0_0_STAGE
        .may_load(ctx.deps.storage)?
        .unwrap_or(MigrationNotStarted);

    match migration_stage {
        MigrationNotStarted => {
            // not allowed to perform the operation
            Err(InvalidMigrationStage)
        }
        MigrationInProgress | MigrationCompleted => {
            perform_next_migration_step_safe(ctx, submsgs_limit)
        }
    }
}

fn perform_next_migration_step_safe(
    ctx: &mut Context,
    submsgs_limit: Option<u32>,
) -> EnterpriseTreasuryResult<Response> {
    let migration_info = MIGRATION_INFO.load(ctx.deps.storage)?;
    let membership_contract =
        migration_info
            .membership_contract
            .ok_or(Std(StdError::generic_err(
                "invalid state - missing membership address",
            )))?;

    let dao_type = DAO_TYPE.load(ctx.deps.storage)?;

    let limit = submsgs_limit.unwrap_or(match dao_type {
        DaoType::Nft => DEFAULT_NFT_SUBMSGS_LIMIT,
        DaoType::Denom | DaoType::Token | DaoType::Multisig => DEFAULT_CW20_SUBMSGS_LIMIT,
    });

    let ResultWithItemsConsumed {
        result: submsgs,
        items_consumed,
    } = finalize_membership_contract_submsgs(ctx.deps.branch(), membership_contract, limit)?;

    if items_consumed < limit {
        set_migration_stage_to_completed(ctx.deps.storage)?;
    } else {
        set_migration_stage_to_in_progress(ctx.deps.storage)?;
    };

    Ok(Response::new()
        .add_attribute("action", "perform_next_migration_step")
        .add_submessages(submsgs))
}

fn set_migration_stage_to_in_progress(storage: &mut dyn Storage) -> EnterpriseTreasuryResult<()> {
    MIGRATION_TO_V_1_0_0_STAGE.save(storage, &MigrationInProgress)?;

    Ok(())
}

fn set_migration_stage_to_completed(storage: &mut dyn Storage) -> EnterpriseTreasuryResult<()> {
    MIGRATION_TO_V_1_0_0_STAGE.save(storage, &MigrationCompleted)?;

    Ok(())
}

/// Loads a user's weight from before migration.
/// As the migration moves weights over to the membership contract, this weight will be removed
/// at some point.
pub fn load_pre_migration_user_weight(
    deps: Deps,
    user: Addr,
) -> EnterpriseTreasuryResult<Option<Uint128>> {
    let dao_type = DAO_TYPE.load(deps.storage)?;

    let weight = match dao_type {
        DaoType::Denom => {
            return Err(StdError::generic_err("No denom DAOs existed pre-migration!").into());
        }
        DaoType::Token => CW20_STAKES.may_load(deps.storage, user)?,
        DaoType::Nft => load_all_nft_stakes_for_user(deps.storage, user)?,
        DaoType::Multisig => MULTISIG_MEMBERS.may_load(deps.storage, user)?,
    };

    Ok(weight)
}
