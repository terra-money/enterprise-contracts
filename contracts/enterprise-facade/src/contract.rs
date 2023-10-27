use crate::facade::get_facade;
use crate::state::{ENTERPRISE_FACADE_V1, ENTERPRISE_FACADE_V2};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
};
use cw2::set_contract_version;
use enterprise_facade_api::api::{
    AdapterResponse, AssetWhitelistResponse, ClaimsResponse, DaoInfoResponse, MemberInfoResponse,
    MemberVoteResponse, MultisigMembersResponse, NftWhitelistResponse, ProposalResponse,
    ProposalStatusResponse, ProposalVotesResponse, ProposalsResponse, StakedNftsResponse,
    TotalStakedAmountResponse, TreasuryAddressResponse, UserStakeResponse,
};
use enterprise_facade_api::error::EnterpriseFacadeResult;
use enterprise_facade_api::msg::QueryMsg::{ExecuteProposalAdapted, TreasuryAddress};
use enterprise_facade_api::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use enterprise_outposts_api::api::CrossChainTreasuriesResponse;
use QueryMsg::{
    AssetWhitelist, CastCouncilVoteAdapted, CastVoteAdapted, ClaimAdapted, Claims,
    CreateCouncilProposalAdapted, CreateProposalAdapted, CreateProposalWithDenomDepositAdapted,
    CreateProposalWithNftDepositAdapted, CreateProposalWithTokenDepositAdapted,
    CrossChainTreasuries, DaoInfo, ListMultisigMembers, MemberInfo, MemberVote, NftWhitelist,
    Proposal, ProposalStatus, ProposalVotes, Proposals, ReleasableClaims, StakeAdapted, StakedNfts,
    TotalStakedAmount, UnstakeAdapted, UserStake,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:enterprise-facade";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> EnterpriseFacadeResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let facade_v1 = deps.api.addr_validate(&msg.enterprise_facade_v1)?;
    ENTERPRISE_FACADE_V1.save(deps.storage, &facade_v1)?;

    let facade_v2 = deps.api.addr_validate(&msg.enterprise_facade_v2)?;
    ENTERPRISE_FACADE_V2.save(deps.storage, &facade_v2)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> EnterpriseFacadeResult<Response> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> EnterpriseFacadeResult<Response> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> EnterpriseFacadeResult<Binary> {
    let response = match msg {
        TreasuryAddress { contract } => {
            let facade = get_facade(deps, contract)?;

            let response: TreasuryAddressResponse = deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &TreasuryAddress {
                    contract: facade.dao_address,
                },
            )?;
            to_binary(&response)?
        }
        DaoInfo { contract } => {
            let facade = get_facade(deps, contract)?;

            let response: DaoInfoResponse = deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &DaoInfo {
                    contract: facade.dao_address,
                },
            )?;
            to_binary(&response)?
        }
        MemberInfo { contract, msg } => {
            let facade = get_facade(deps, contract)?;

            let response: MemberInfoResponse = deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &MemberInfo {
                    contract: facade.dao_address,
                    msg,
                },
            )?;
            to_binary(&response)?
        }
        ListMultisigMembers { contract, msg } => {
            let facade = get_facade(deps, contract)?;

            let response: MultisigMembersResponse = deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &ListMultisigMembers {
                    contract: facade.dao_address,
                    msg,
                },
            )?;
            to_binary(&response)?
        }
        AssetWhitelist { contract, params } => {
            let facade = get_facade(deps, contract)?;

            let response: AssetWhitelistResponse = deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &AssetWhitelist {
                    contract: facade.dao_address,
                    params,
                },
            )?;
            to_binary(&response)?
        }
        NftWhitelist { contract, params } => {
            let facade = get_facade(deps, contract)?;

            let response: NftWhitelistResponse = deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &NftWhitelist {
                    contract: facade.dao_address,
                    params,
                },
            )?;
            to_binary(&response)?
        }
        Proposal { contract, params } => {
            let facade = get_facade(deps, contract)?;

            let response: ProposalResponse = deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &Proposal {
                    contract: facade.dao_address,
                    params,
                },
            )?;
            to_binary(&response)?
        }
        Proposals { contract, params } => {
            let facade = get_facade(deps, contract)?;

            let response: ProposalsResponse = deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &Proposals {
                    contract: facade.dao_address,
                    params,
                },
            )?;
            to_binary(&response)?
        }
        ProposalStatus { contract, params } => {
            let facade = get_facade(deps, contract)?;

            let response: ProposalStatusResponse = deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &ProposalStatus {
                    contract: facade.dao_address,
                    params,
                },
            )?;
            to_binary(&response)?
        }
        MemberVote { contract, params } => {
            let facade = get_facade(deps, contract)?;

            let response: MemberVoteResponse = deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &MemberVote {
                    contract: facade.dao_address,
                    params,
                },
            )?;
            to_binary(&response)?
        }
        ProposalVotes { contract, params } => {
            let facade = get_facade(deps, contract)?;

            let response: ProposalVotesResponse = deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &ProposalVotes {
                    contract: facade.dao_address,
                    params,
                },
            )?;
            to_binary(&response)?
        }
        UserStake { contract, params } => {
            let facade = get_facade(deps, contract)?;

            let response: UserStakeResponse = deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &UserStake {
                    contract: facade.dao_address,
                    params,
                },
            )?;
            to_binary(&response)?
        }
        TotalStakedAmount { contract } => {
            let facade = get_facade(deps, contract)?;

            let response: TotalStakedAmountResponse = deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &TotalStakedAmount {
                    contract: facade.dao_address,
                },
            )?;
            to_binary(&response)?
        }
        StakedNfts { contract, params } => {
            let facade = get_facade(deps, contract)?;

            let response: StakedNftsResponse = deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &StakedNfts {
                    contract: facade.dao_address,
                    params,
                },
            )?;
            to_binary(&response)?
        }
        Claims { contract, params } => {
            let facade = get_facade(deps, contract)?;

            let response: ClaimsResponse = deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &Claims {
                    contract: facade.dao_address,
                    params,
                },
            )?;
            to_binary(&response)?
        }
        ReleasableClaims { contract, params } => {
            let facade = get_facade(deps, contract)?;

            let response: ClaimsResponse = deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &ReleasableClaims {
                    contract: facade.dao_address,
                    params,
                },
            )?;
            to_binary(&response)?
        }
        CrossChainTreasuries { contract, params } => {
            let facade = get_facade(deps, contract)?;

            let response: CrossChainTreasuriesResponse = deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &CrossChainTreasuries {
                    contract: facade.dao_address,
                    params,
                },
            )?;
            to_binary(&response)?
        }
        CreateProposalAdapted { contract, params } => {
            let facade = get_facade(deps, contract)?;

            let response: AdapterResponse = deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &CreateProposalAdapted {
                    contract: facade.dao_address,
                    params,
                },
            )?;
            to_binary(&response)?
        }
        CreateProposalWithDenomDepositAdapted { contract, params } => {
            let facade = get_facade(deps, contract)?;

            let response: AdapterResponse = deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &CreateProposalWithDenomDepositAdapted {
                    contract: facade.dao_address,
                    params,
                },
            )?;
            to_binary(&response)?
        }
        CreateProposalWithTokenDepositAdapted { contract, params } => {
            let facade = get_facade(deps, contract)?;

            let response: AdapterResponse = deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &CreateProposalWithTokenDepositAdapted {
                    contract: facade.dao_address,
                    params,
                },
            )?;
            to_binary(&response)?
        }
        CreateProposalWithNftDepositAdapted { contract, params } => {
            let facade = get_facade(deps, contract)?;

            let response: AdapterResponse = deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &CreateProposalWithNftDepositAdapted {
                    contract: facade.dao_address,
                    params,
                },
            )?;
            to_binary(&response)?
        }
        CreateCouncilProposalAdapted { contract, params } => {
            let facade = get_facade(deps, contract)?;

            let response: AdapterResponse = deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &CreateCouncilProposalAdapted {
                    contract: facade.dao_address,
                    params,
                },
            )?;
            to_binary(&response)?
        }
        CastVoteAdapted { contract, params } => {
            let facade = get_facade(deps, contract)?;

            let response: AdapterResponse = deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &CastVoteAdapted {
                    contract: facade.dao_address,
                    params,
                },
            )?;
            to_binary(&response)?
        }
        CastCouncilVoteAdapted { contract, params } => {
            let facade = get_facade(deps, contract)?;

            let response: AdapterResponse = deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &CastCouncilVoteAdapted {
                    contract: facade.dao_address,
                    params,
                },
            )?;
            to_binary(&response)?
        }
        ExecuteProposalAdapted { contract, params } => {
            let facade = get_facade(deps, contract)?;

            let response: AdapterResponse = deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &ExecuteProposalAdapted {
                    contract: facade.dao_address,
                    params,
                },
            )?;
            to_binary(&response)?
        }
        StakeAdapted { contract, params } => {
            let facade = get_facade(deps, contract)?;

            let response: AdapterResponse = deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &StakeAdapted {
                    contract: facade.dao_address,
                    params,
                },
            )?;
            to_binary(&response)?
        }
        UnstakeAdapted { contract, params } => {
            let facade = get_facade(deps, contract)?;

            let response: AdapterResponse = deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &UnstakeAdapted {
                    contract: facade.dao_address,
                    params,
                },
            )?;
            to_binary(&response)?
        }
        ClaimAdapted { contract } => {
            let facade = get_facade(deps, contract)?;

            let response: AdapterResponse = deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &ClaimAdapted {
                    contract: facade.dao_address,
                },
            )?;
            to_binary(&response)?
        }
    };
    Ok(response)
}
