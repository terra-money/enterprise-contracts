use crate::claims::{get_claims, get_releasable_claims};
use crate::config::{NftContractAddr, CONFIG};
use crate::nft_staking::{NftStake, NFT_STAKES};
use common::cw::QueryContext;
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{Addr, Order, StdResult, Uint128};
use cw_storage_plus::Bound;
use cw_utils::Expiration;
use itertools::Itertools;
use membership_common::enterprise_contract::ENTERPRISE_CONTRACT;
use membership_common::member_weights::{get_member_weight, MEMBER_WEIGHTS};
use membership_common::total_weight::{
    load_total_weight, load_total_weight_at_height, load_total_weight_at_time,
};
use membership_common_api::api::{
    MembersParams, MembersResponse, TotalWeightParams, TotalWeightResponse, UserWeightParams,
    UserWeightResponse,
};
use nft_staking_api::api::{
    ClaimsParams, ClaimsResponse, NftConfigResponse, NftContract, NftContractConfigResponse,
    NftTokenId, StakedNftsParams, StakedNftsResponse, UserNftStakeParams, UserNftStakeResponse,
};
use nft_staking_api::error::NftStakingError::Ics721StillNotTransferred;
use nft_staking_api::error::NftStakingResult;

const MAX_QUERY_LIMIT: u8 = 100;
const DEFAULT_QUERY_LIMIT: u8 = 50;

pub fn query_nft_config(qctx: &QueryContext) -> NftStakingResult<NftConfigResponse> {
    let config = CONFIG.load(qctx.deps.storage)?;

    let enterprise_contract = ENTERPRISE_CONTRACT.load(qctx.deps.storage)?;

    let nft_contract = match config.nft_contract_addr {
        NftContractAddr::Cw721 { contract } => contract,
        NftContractAddr::Ics721 { .. } => return Err(Ics721StillNotTransferred),
    };

    Ok(NftConfigResponse {
        enterprise_contract,
        nft_contract,
        unlocking_period: config.unlocking_period,
    })
}

pub fn query_nft_contract_config(
    qctx: &QueryContext,
) -> NftStakingResult<NftContractConfigResponse> {
    let config = CONFIG.load(qctx.deps.storage)?;

    let enterprise_contract = ENTERPRISE_CONTRACT.load(qctx.deps.storage)?;

    let nft_contract = match config.nft_contract_addr {
        NftContractAddr::Cw721 { contract } => NftContract::Cw721 {
            contract: contract.to_string(),
        },
        NftContractAddr::Ics721 { contract, class_id } => NftContract::Ics721 {
            contract: contract.to_string(),
            class_id,
        },
    };

    Ok(NftContractConfigResponse {
        enterprise_contract,
        nft_contract,
        unlocking_period: config.unlocking_period,
    })
}

pub fn query_user_nft_stake(
    qctx: &QueryContext,
    params: UserNftStakeParams,
) -> NftStakingResult<UserNftStakeResponse> {
    let user = qctx.deps.api.addr_validate(&params.user)?;

    let start_after = params.start_after.map(Bound::exclusive);
    let limit = params
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT as u32)
        .min(MAX_QUERY_LIMIT as u32);

    let user_stake = NFT_STAKES()
        .idx
        .staker
        .prefix(user.clone())
        .range(qctx.deps.storage, start_after, None, Order::Ascending)
        .take(limit as usize)
        .map_ok(|(_, stake)| stake)
        .collect::<StdResult<Vec<NftStake>>>()?;

    let total_user_stake = get_member_weight(qctx.deps.storage, user.clone())?;
    let tokens = user_stake.into_iter().map(|stake| stake.token_id).collect();

    Ok(UserNftStakeResponse {
        user,
        tokens,
        total_user_stake,
    })
}

pub fn query_user_weight(
    qctx: &QueryContext,
    params: UserWeightParams,
) -> NftStakingResult<UserWeightResponse> {
    let user = qctx.deps.api.addr_validate(&params.user)?;

    let weight = get_member_weight(qctx.deps.storage, user.clone())?;

    Ok(UserWeightResponse { user, weight })
}

pub fn query_total_weight(
    qctx: &QueryContext,
    params: TotalWeightParams,
) -> NftStakingResult<TotalWeightResponse> {
    let total_weight = match params.expiration {
        Expiration::AtHeight(height) => load_total_weight_at_height(qctx.deps.storage, height)?,
        Expiration::AtTime(time) => load_total_weight_at_time(qctx.deps.storage, time)?,
        Expiration::Never {} => load_total_weight(qctx.deps.storage)?,
    };

    Ok(TotalWeightResponse { total_weight })
}

pub fn query_claims(qctx: &QueryContext, params: ClaimsParams) -> NftStakingResult<ClaimsResponse> {
    let user = qctx.deps.api.addr_validate(&params.user)?;

    get_claims(qctx.deps.storage, user)
}

pub fn query_releasable_claims(
    qctx: &QueryContext,
    params: ClaimsParams,
) -> NftStakingResult<ClaimsResponse> {
    let user = qctx.deps.api.addr_validate(&params.user)?;

    get_releasable_claims(qctx.deps.storage, &qctx.env.block, user)
}

pub fn query_members(
    qctx: &QueryContext,
    params: MembersParams,
) -> NftStakingResult<MembersResponse> {
    let start_after = params
        .start_after
        .map(|addr| qctx.deps.api.addr_validate(&addr))
        .transpose()?
        .map(Bound::exclusive);
    let limit = params
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT as u32)
        .min(MAX_QUERY_LIMIT as u32);

    let members = MEMBER_WEIGHTS
        .range(qctx.deps.storage, start_after, None, Ascending)
        .take(limit as usize)
        .collect::<StdResult<Vec<(Addr, Uint128)>>>()?
        .into_iter()
        .map(|(user, weight)| UserWeightResponse { user, weight })
        .collect();

    Ok(MembersResponse { members })
}

pub fn query_staked_nfts(
    qctx: &QueryContext,
    params: StakedNftsParams,
) -> NftStakingResult<StakedNftsResponse> {
    let start_after = params.start_after.map(Bound::exclusive);
    let limit = params
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT as u32)
        .min(MAX_QUERY_LIMIT as u32);

    let nfts = NFT_STAKES()
        .range(qctx.deps.storage, start_after, None, Ascending)
        .take(limit as usize)
        .map(|res| res.map(|(_, nft_stake)| nft_stake.token_id))
        .collect::<StdResult<Vec<NftTokenId>>>()?;

    Ok(StakedNftsResponse { nfts })
}
