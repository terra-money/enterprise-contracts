use crate::contract::{
    INSTANTIATE_NFT_STAKING_CONTRACT_REPLY_ID, INSTANTIATE_TOKEN_STAKING_CONTRACT_REPLY_ID,
};
use crate::state::{DAO_GOV_CONFIG, DAO_MEMBERSHIP_CONTRACT, DAO_TYPE, STAKING_CONTRACT};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    to_binary, wasm_execute, wasm_instantiate, Addr, DepsMut, Env, Order, Reply, Response,
    StdError, StdResult, SubMsg, Uint128,
};
use cw_storage_plus::{Index, IndexList, IndexedMap, Map, MultiIndex};
use cw_utils::parse_reply_instantiate_data;
use enterprise_protocol::api::{Claim, ClaimAsset, DaoType, NftTokenId};
use enterprise_protocol::error::DaoResult;
use token_staking_api::api::{UserClaim, UserStake};
use token_staking_api::msg::Cw20HookMsg::InitializeStakers;

const CW20_STAKES: Map<Addr, Uint128> = Map::new("stakes");

const CLAIMS: Map<&Addr, Vec<Claim>> = Map::new("claims");

#[cw_serde]
pub struct NftStake {
    pub staker: Addr,
    pub token_id: NftTokenId,
}

pub struct NftStakesIndexes<'a> {
    pub staker: MultiIndex<'a, Addr, NftStake, String>,
}

impl IndexList<NftStake> for NftStakesIndexes<'_> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<NftStake>> + '_> {
        let v: Vec<&dyn Index<NftStake>> = vec![&self.staker];
        Box::new(v.into_iter())
    }
}

#[allow(non_snake_case)]
pub fn NFT_STAKES<'a>() -> IndexedMap<'a, String, NftStake, NftStakesIndexes<'a>> {
    let indexes = NftStakesIndexes {
        staker: MultiIndex::new(
            |_, nft_stake| nft_stake.staker.clone(),
            "nft_stakes",
            "nft_stakes__staker",
        ),
    };
    IndexedMap::new("nft_stakes", indexes)
}

pub fn migrate_staking(deps: DepsMut, env: Env) -> DaoResult<Vec<SubMsg>> {
    let dao_type = DAO_TYPE.load(deps.storage)?;

    match dao_type {
        DaoType::Token => {
            let token_addr = DAO_MEMBERSHIP_CONTRACT.load(deps.storage)?;

            let gov_config = DAO_GOV_CONFIG.load(deps.storage)?;

            let instantiate_msg = SubMsg::reply_on_success(
                wasm_instantiate(
                    0, // TODO: use real code ID
                    &token_staking_api::msg::InstantiateMsg {
                        admin: env.contract.address.to_string(),
                        token_contract: token_addr.to_string(),
                        unlocking_period: gov_config.unlocking_period,
                    },
                    vec![],
                    "Token staking".to_string(),
                )?,
                INSTANTIATE_TOKEN_STAKING_CONTRACT_REPLY_ID,
            );

            Ok(vec![instantiate_msg])
        }
        DaoType::Nft => {
            let nft_addr = DAO_MEMBERSHIP_CONTRACT.load(deps.storage)?;

            let gov_config = DAO_GOV_CONFIG.load(deps.storage)?;

            let instantiate_msg = SubMsg::reply_on_success(
                wasm_instantiate(
                    0, // TODO: use real code ID
                    &nft_staking_api::msg::InstantiateMsg {
                        admin: env.contract.address.to_string(),
                        nft_contract: nft_addr.to_string(),
                        unlocking_period: gov_config.unlocking_period,
                    },
                    vec![],
                    "NFT staking".to_string(),
                )?,
                INSTANTIATE_NFT_STAKING_CONTRACT_REPLY_ID,
            );

            Ok(vec![instantiate_msg])
        }
        DaoType::Multisig => Ok(vec![]),
    }
}

pub fn reply_instantiate_token_staking_contract(deps: DepsMut, msg: Reply) -> DaoResult<Response> {
    if msg.id != INSTANTIATE_TOKEN_STAKING_CONTRACT_REPLY_ID {
        return Err(StdError::generic_err("invalid reply ID").into());
    }

    let token_staking_contract_address = parse_reply_instantiate_data(msg)
        .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
        .contract_address;

    let staking_contract = deps.api.addr_validate(&token_staking_contract_address)?;

    STAKING_CONTRACT.save(deps.storage, &staking_contract)?;

    let stakers = CW20_STAKES
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<(Addr, Uint128)>>>()?;

    let mut total_stake = Uint128::zero();

    let mut staking_contract_stakers = vec![];

    for (staker, stake) in stakers {
        total_stake += stake;

        staking_contract_stakers.push(UserStake {
            user: staker.to_string(),
            staked_amount: stake,
        });

        CW20_STAKES.remove(deps.storage, staker);
    }

    let token_addr = DAO_MEMBERSHIP_CONTRACT.load(deps.storage)?;

    let initialize_stakers_msg = SubMsg::new(wasm_execute(
        token_addr.to_string(),
        &cw20::Cw20ExecuteMsg::Send {
            contract: token_staking_contract_address.clone(),
            amount: total_stake,
            msg: to_binary(&InitializeStakers {
                stakers: staking_contract_stakers,
            })?,
        },
        vec![],
    )?);

    let all_claims = CLAIMS
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<(Addr, Vec<Claim>)>>>()?;

    let mut total_claims_amount = Uint128::zero();

    let mut claims: Vec<UserClaim> = vec![];

    for (user, user_claims) in all_claims {
        for claim in user_claims {
            if let ClaimAsset::Cw20(asset) = claim.asset {
                total_claims_amount += asset.amount;

                claims.push(UserClaim {
                    user: user.to_string(),
                    claim_amount: asset.amount,
                    release_at: claim.release_at.clone(),
                });
            }
        }
    }

    CLAIMS.clear(deps.storage);

    let initialize_claims_msg = SubMsg::new(wasm_execute(
        token_addr.to_string(),
        &cw20::Cw20ExecuteMsg::Send {
            contract: token_staking_contract_address,
            amount: total_claims_amount,
            msg: to_binary(&token_staking_api::msg::Cw20HookMsg::AddClaims { claims })?,
        },
        vec![],
    )?);

    Ok(Response::new()
        .add_submessage(initialize_stakers_msg)
        .add_submessage(initialize_claims_msg))
}

pub fn reply_instantiate_nft_staking_contract(deps: DepsMut, msg: Reply) -> DaoResult<Response> {
    if msg.id != INSTANTIATE_NFT_STAKING_CONTRACT_REPLY_ID {
        return Err(StdError::generic_err("invalid reply ID").into());
    }

    let token_staking_contract_address = parse_reply_instantiate_data(msg)
        .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
        .contract_address;

    let staking_contract = deps.api.addr_validate(&token_staking_contract_address)?;

    STAKING_CONTRACT.save(deps.storage, &staking_contract)?;

    // migrate stakes
    let stakes = NFT_STAKES()
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<(String, NftStake)>>>()?;

    let mut send_nft_stakes_msgs = vec![];

    let nft_addr = DAO_MEMBERSHIP_CONTRACT.load(deps.storage)?;

    for (token_id, stake) in stakes {
        send_nft_stakes_msgs.push(SubMsg::new(wasm_execute(
            nft_addr.to_string(),
            &cw721::Cw721ExecuteMsg::SendNft {
                contract: staking_contract.to_string(),
                token_id,
                msg: to_binary(&nft_staking_api::msg::Cw721HookMsg::Stake {
                    user: stake.staker.to_string(),
                })?,
            },
            vec![],
        )?));
    }

    NFT_STAKES().clear(deps.storage);

    // migrate claims
    let all_claims = CLAIMS
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<(Addr, Vec<Claim>)>>>()?;

    let mut send_claims_msg = vec![];

    for (user, user_claims) in all_claims {
        CLAIMS.remove(deps.storage, &user);

        for claim in user_claims {
            if let ClaimAsset::Cw721(claim_assets) = claim.asset {
                for token_id in claim_assets.tokens {
                    send_claims_msg.push(SubMsg::new(wasm_execute(
                        nft_addr.to_string(),
                        &cw721::Cw721ExecuteMsg::SendNft {
                            contract: staking_contract.to_string(),
                            token_id,
                            msg: to_binary(&nft_staking_api::msg::Cw721HookMsg::AddClaim {
                                user: user.to_string(),
                                release_at: claim.release_at.clone(),
                            })?,
                        },
                        vec![],
                    )?));
                }
            }
        }
    }

    CLAIMS.clear(deps.storage);

    Ok(Response::new()
        .add_submessages(send_nft_stakes_msgs)
        .add_submessages(send_claims_msg))
}
