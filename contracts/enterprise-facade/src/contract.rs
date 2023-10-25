use crate::facade::get_facade;
use crate::state::{ENTERPRISE_FACADE_V1, ENTERPRISE_FACADE_V2};
use cosmwasm_std::{
    entry_point, wasm_execute, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, SubMsg,
};
use cw2::set_contract_version;
use enterprise_facade_api::error::EnterpriseFacadeResult;
use enterprise_facade_api::msg::QueryMsg::TreasuryAddress;
use enterprise_facade_api::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use ExecuteMsg::ExecuteProposal;
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
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> EnterpriseFacadeResult<Response> {
    match msg {
        ExecuteProposal { contract, msg } => {
            let facade = get_facade(deps.as_ref(), contract)?;

            Ok(Response::new().add_submessage(SubMsg::new(wasm_execute(
                facade.facade_address.to_string(),
                &ExecuteProposal {
                    contract: facade.dao_address,
                    msg,
                },
                vec![],
            )?)))
        }
    }
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
            deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &TreasuryAddress {
                    contract: facade.dao_address,
                },
            )?
        }
        DaoInfo { contract } => {
            let facade = get_facade(deps, contract)?;
            deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &DaoInfo {
                    contract: facade.dao_address,
                },
            )?
        }
        MemberInfo { contract, msg } => {
            let facade = get_facade(deps, contract)?;
            deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &MemberInfo {
                    contract: facade.dao_address,
                    msg,
                },
            )?
        }
        ListMultisigMembers { contract, msg } => {
            let facade = get_facade(deps, contract)?;
            deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &ListMultisigMembers {
                    contract: facade.dao_address,
                    msg,
                },
            )?
        }
        AssetWhitelist { contract, params } => {
            let facade = get_facade(deps, contract)?;
            deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &AssetWhitelist {
                    contract: facade.dao_address,
                    params,
                },
            )?
        }
        NftWhitelist { contract, params } => {
            let facade = get_facade(deps, contract)?;
            deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &NftWhitelist {
                    contract: facade.dao_address,
                    params,
                },
            )?
        }
        Proposal { contract, params } => {
            let facade = get_facade(deps, contract)?;
            deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &Proposal {
                    contract: facade.dao_address,
                    params,
                },
            )?
        }
        Proposals { contract, params } => {
            let facade = get_facade(deps, contract)?;
            deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &Proposals {
                    contract: facade.dao_address,
                    params,
                },
            )?
        }
        ProposalStatus { contract, params } => {
            let facade = get_facade(deps, contract)?;
            deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &ProposalStatus {
                    contract: facade.dao_address,
                    params,
                },
            )?
        }
        MemberVote { contract, params } => {
            let facade = get_facade(deps, contract)?;
            deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &MemberVote {
                    contract: facade.dao_address,
                    params,
                },
            )?
        }
        ProposalVotes { contract, params } => {
            let facade = get_facade(deps, contract)?;
            deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &ProposalVotes {
                    contract: facade.dao_address,
                    params,
                },
            )?
        }
        UserStake { contract, params } => {
            let facade = get_facade(deps, contract)?;
            deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &UserStake {
                    contract: facade.dao_address,
                    params,
                },
            )?
        }
        TotalStakedAmount { contract } => {
            let facade = get_facade(deps, contract)?;
            deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &TotalStakedAmount {
                    contract: facade.dao_address,
                },
            )?
        }
        StakedNfts { contract, params } => {
            let facade = get_facade(deps, contract)?;
            deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &StakedNfts {
                    contract: facade.dao_address,
                    params,
                },
            )?
        }
        Claims { contract, params } => {
            let facade = get_facade(deps, contract)?;
            deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &Claims {
                    contract: facade.dao_address,
                    params,
                },
            )?
        }
        ReleasableClaims { contract, params } => {
            let facade = get_facade(deps, contract)?;
            deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &ReleasableClaims {
                    contract: facade.dao_address,
                    params,
                },
            )?
        }
        CrossChainTreasuries { contract, params } => {
            let facade = get_facade(deps, contract)?;
            deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &CrossChainTreasuries {
                    contract: facade.dao_address,
                    params,
                },
            )?
        }
        CreateProposalAdapted { contract, params } => {
            let facade = get_facade(deps, contract)?;
            deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &CreateProposalAdapted {
                    contract: facade.dao_address,
                    params,
                },
            )?
        }
        CreateProposalWithDenomDepositAdapted { contract, params } => {
            let facade = get_facade(deps, contract)?;
            deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &CreateProposalWithDenomDepositAdapted {
                    contract: facade.dao_address,
                    params,
                },
            )?
        }
        CreateProposalWithTokenDepositAdapted { contract, params } => {
            let facade = get_facade(deps, contract)?;
            deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &CreateProposalWithTokenDepositAdapted {
                    contract: facade.dao_address,
                    params,
                },
            )?
        }
        CreateProposalWithNftDepositAdapted { contract, params } => {
            let facade = get_facade(deps, contract)?;
            deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &CreateProposalWithNftDepositAdapted {
                    contract: facade.dao_address,
                    params,
                },
            )?
        }
        CreateCouncilProposalAdapted { contract, params } => {
            let facade = get_facade(deps, contract)?;
            deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &CreateCouncilProposalAdapted {
                    contract: facade.dao_address,
                    params,
                },
            )?
        }
        CastVoteAdapted { contract, params } => {
            let facade = get_facade(deps, contract)?;
            deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &CastVoteAdapted {
                    contract: facade.dao_address,
                    params,
                },
            )?
        }
        CastCouncilVoteAdapted { contract, params } => {
            let facade = get_facade(deps, contract)?;
            deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &CastCouncilVoteAdapted {
                    contract: facade.dao_address,
                    params,
                },
            )?
        }
        StakeAdapted { contract, params } => {
            let facade = get_facade(deps, contract)?;
            deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &StakeAdapted {
                    contract: facade.dao_address,
                    params,
                },
            )?
        }
        UnstakeAdapted { contract, params } => {
            let facade = get_facade(deps, contract)?;
            deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &UnstakeAdapted {
                    contract: facade.dao_address,
                    params,
                },
            )?
        }
        ClaimAdapted { contract } => {
            let facade = get_facade(deps, contract)?;
            deps.querier.query_wasm_smart(
                facade.facade_address.to_string(),
                &ClaimAdapted {
                    contract: facade.dao_address,
                },
            )?
        }
    };
    Ok(response)
}
