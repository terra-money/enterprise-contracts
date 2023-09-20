use crate::facade::EnterpriseFacade;
use crate::facade_v5::ExecuteV5Msg::{
    CastCouncilVote, CastVote, Claim, CreateCouncilProposal, CreateProposal, Unstake,
};
use crate::facade_v5::QueryV5Msg::{
    AssetWhitelist, Claims, DaoInfo, ListMultisigMembers, MemberInfo, MemberVote, NftWhitelist,
    Proposal, ProposalVotes, Proposals, ReleasableClaims, StakedNfts, TotalStakedAmount, UserStake,
};
use crate::util::adapter_response_single_msg;
use common::cw::{Context, QueryContext};
use cosmwasm_schema::cw_serde;
use cosmwasm_schema::serde::de::DeserializeOwned;
use cosmwasm_schema::serde::Serialize;
use cosmwasm_std::{to_binary, wasm_execute, Addr, Deps, Response, SubMsg};
use enterprise_facade_api::api::{
    AdaptedMsg, AdapterResponse, AssetWhitelistParams, AssetWhitelistResponse, CastVoteMsg,
    ClaimsParams, ClaimsResponse, CreateProposalMsg, CreateProposalWithDenomDepositMsg,
    CreateProposalWithTokenDepositMsg, DaoInfoResponse, DaoType, ExecuteProposalMsg,
    ListMultisigMembersMsg, MemberInfoResponse, MemberVoteParams, MemberVoteResponse,
    MultisigMembersResponse, NftWhitelistParams, NftWhitelistResponse, ProposalParams,
    ProposalResponse, ProposalStatusParams, ProposalStatusResponse, ProposalVotesParams,
    ProposalVotesResponse, ProposalsParams, ProposalsResponse, QueryMemberInfoMsg, StakeMsg,
    StakedNftsParams, StakedNftsResponse, TotalStakedAmountResponse, UnstakeMsg, UserStakeParams,
    UserStakeResponse,
};
use enterprise_facade_api::error::DaoError::UnsupportedOperationForDaoType;
use enterprise_facade_api::error::EnterpriseFacadeError::Dao;
use enterprise_facade_api::error::{EnterpriseFacadeError, EnterpriseFacadeResult};
use enterprise_governance_controller_api::api::CreateProposalWithNftDepositMsg;
use EnterpriseFacadeError::UnsupportedOperation;
use QueryV5Msg::ProposalStatus;

/// Facade implementation for v0.5.0 of Enterprise (pre-contract-rewrite).
pub struct EnterpriseFacadeV5 {
    pub enterprise_address: Addr,
}

impl EnterpriseFacade for EnterpriseFacadeV5 {
    fn execute_proposal(
        &self,
        _ctx: &mut Context,
        msg: ExecuteProposalMsg,
    ) -> EnterpriseFacadeResult<Response> {
        let submsg = SubMsg::new(wasm_execute(
            self.enterprise_address.to_string(),
            &ExecuteV5Msg::ExecuteProposal(msg),
            vec![],
        )?);
        Ok(Response::new().add_submessage(submsg))
    }

    fn query_dao_info(&self, qctx: QueryContext) -> EnterpriseFacadeResult<DaoInfoResponse> {
        self.query_enterprise_contract(qctx.deps, &DaoInfo {})
    }

    fn query_member_info(
        &self,
        qctx: QueryContext,
        msg: QueryMemberInfoMsg,
    ) -> EnterpriseFacadeResult<MemberInfoResponse> {
        self.query_enterprise_contract(qctx.deps, &MemberInfo(msg))
    }

    fn query_list_multisig_members(
        &self,
        qctx: QueryContext,
        msg: ListMultisigMembersMsg,
    ) -> EnterpriseFacadeResult<MultisigMembersResponse> {
        self.query_enterprise_contract(qctx.deps, &ListMultisigMembers(msg))
    }

    fn query_asset_whitelist(
        &self,
        qctx: QueryContext,
        params: AssetWhitelistParams,
    ) -> EnterpriseFacadeResult<AssetWhitelistResponse> {
        self.query_enterprise_contract(qctx.deps, &AssetWhitelist(params))
    }

    fn query_nft_whitelist(
        &self,
        qctx: QueryContext,
        params: NftWhitelistParams,
    ) -> EnterpriseFacadeResult<NftWhitelistResponse> {
        self.query_enterprise_contract(qctx.deps, &NftWhitelist(params))
    }

    fn query_proposal(
        &self,
        qctx: QueryContext,
        params: ProposalParams,
    ) -> EnterpriseFacadeResult<ProposalResponse> {
        self.query_enterprise_contract(qctx.deps, &Proposal(params))
    }

    fn query_proposals(
        &self,
        qctx: QueryContext,
        params: ProposalsParams,
    ) -> EnterpriseFacadeResult<ProposalsResponse> {
        self.query_enterprise_contract(qctx.deps, &Proposals(params))
    }

    fn query_proposal_status(
        &self,
        qctx: QueryContext,
        params: ProposalStatusParams,
    ) -> EnterpriseFacadeResult<ProposalStatusResponse> {
        self.query_enterprise_contract(qctx.deps, &ProposalStatus(params))
    }

    fn query_member_vote(
        &self,
        qctx: QueryContext,
        params: MemberVoteParams,
    ) -> EnterpriseFacadeResult<MemberVoteResponse> {
        self.query_enterprise_contract(qctx.deps, &MemberVote(params))
    }

    fn query_proposal_votes(
        &self,
        qctx: QueryContext,
        params: ProposalVotesParams,
    ) -> EnterpriseFacadeResult<ProposalVotesResponse> {
        self.query_enterprise_contract(qctx.deps, &ProposalVotes(params))
    }

    fn query_user_stake(
        &self,
        qctx: QueryContext,
        params: UserStakeParams,
    ) -> EnterpriseFacadeResult<UserStakeResponse> {
        self.query_enterprise_contract(
            qctx.deps,
            &UserStake(UserStakeV5Params { user: params.user }),
        )
    }

    fn query_total_staked_amount(
        &self,
        qctx: QueryContext,
    ) -> EnterpriseFacadeResult<TotalStakedAmountResponse> {
        self.query_enterprise_contract(qctx.deps, &TotalStakedAmount {})
    }

    fn query_staked_nfts(
        &self,
        qctx: QueryContext,
        params: StakedNftsParams,
    ) -> EnterpriseFacadeResult<StakedNftsResponse> {
        self.query_enterprise_contract(qctx.deps, &StakedNfts(params))
    }

    fn query_claims(
        &self,
        qctx: QueryContext,
        params: ClaimsParams,
    ) -> EnterpriseFacadeResult<ClaimsResponse> {
        self.query_enterprise_contract(qctx.deps, &Claims(params))
    }

    fn query_releasable_claims(
        &self,
        qctx: QueryContext,
        params: ClaimsParams,
    ) -> EnterpriseFacadeResult<ClaimsResponse> {
        self.query_enterprise_contract(qctx.deps, &ReleasableClaims(params))
    }

    fn adapt_create_proposal(
        &self,
        _: QueryContext,
        params: CreateProposalMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        Ok(adapter_response_single_msg(
            self.enterprise_address.clone(),
            serde_json_wasm::to_string(&CreateProposal(params))?,
            vec![],
        ))
    }

    fn adapt_create_proposal_with_denom_deposit(
        &self,
        _: QueryContext,
        _: CreateProposalWithDenomDepositMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        Err(UnsupportedOperation)
    }

    fn adapt_create_proposal_with_token_deposit(
        &self,
        qctx: QueryContext,
        params: CreateProposalWithTokenDepositMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        let dao_info = self.query_dao_info(qctx.clone())?;
        let dao_type = dao_info.dao_type;

        match dao_type {
            DaoType::Token => Ok(adapter_response_single_msg(
                dao_info.dao_membership_contract,
                serde_json_wasm::to_string(&cw20::Cw20ExecuteMsg::Send {
                    contract: self.enterprise_address.to_string(),
                    amount: params.deposit_amount,
                    msg: to_binary(&Cw20HookV5Msg::CreateProposal(CreateProposalMsg {
                        title: params.create_proposal_msg.title,
                        description: params.create_proposal_msg.description,
                        proposal_actions: params.create_proposal_msg.proposal_actions,
                    }))?,
                })?,
                vec![],
            )),
            _ => Err(Dao(UnsupportedOperationForDaoType {
                dao_type: dao_type.to_string(),
            })),
        }
    }

    fn adapt_create_proposal_with_nft_deposit(
        &self,
        _: QueryContext,
        _: CreateProposalWithNftDepositMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        Err(UnsupportedOperation)
    }

    fn adapt_create_council_proposal(
        &self,
        _: QueryContext,
        params: CreateProposalMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        Ok(adapter_response_single_msg(
            self.enterprise_address.clone(),
            serde_json_wasm::to_string(&CreateCouncilProposal(params))?,
            vec![],
        ))
    }

    fn adapt_cast_vote(
        &self,
        _: QueryContext,
        params: CastVoteMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        Ok(adapter_response_single_msg(
            self.enterprise_address.clone(),
            serde_json_wasm::to_string(&CastVote(params))?,
            vec![],
        ))
    }

    fn adapt_cast_council_vote(
        &self,
        _: QueryContext,
        params: CastVoteMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        Ok(adapter_response_single_msg(
            self.enterprise_address.clone(),
            serde_json_wasm::to_string(&CastCouncilVote(params))?,
            vec![],
        ))
    }

    fn adapt_stake(
        &self,
        qctx: QueryContext,
        params: StakeMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        match params {
            StakeMsg::Cw20(msg) => {
                let token_addr = self.query_dao_info(qctx.clone())?.dao_membership_contract;
                let msg = cw20::Cw20ExecuteMsg::Send {
                    contract: self.enterprise_address.to_string(),
                    amount: msg.amount,
                    msg: to_binary(&Cw20HookV5Msg::Stake {})?,
                };
                Ok(adapter_response_single_msg(
                    token_addr,
                    serde_json_wasm::to_string(&msg)?,
                    vec![],
                ))
            }
            StakeMsg::Cw721(msg) => {
                let nft_addr = self.query_dao_info(qctx.clone())?.dao_membership_contract;

                let stake_msg_binary = to_binary(&Cw721HookV5Msg::Stake {})?;

                let msgs = msg
                    .tokens
                    .into_iter()
                    .map(|token_id| cw721::Cw721ExecuteMsg::SendNft {
                        contract: self.enterprise_address.to_string(),
                        token_id,
                        msg: stake_msg_binary.clone(),
                    })
                    .map(|send_nft_msg| {
                        serde_json_wasm::to_string(&send_nft_msg).map(|send_nft_msg_json| {
                            AdaptedMsg {
                                target_contract: nft_addr.clone(),
                                msg: send_nft_msg_json,
                                funds: vec![],
                            }
                        })
                    })
                    .collect::<serde_json_wasm::ser::Result<Vec<AdaptedMsg>>>()?;

                Ok(AdapterResponse { msgs })
            }
            StakeMsg::Denom(_) => Err(UnsupportedOperation),
        }
    }

    fn adapt_unstake(
        &self,
        _: QueryContext,
        params: UnstakeMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        Ok(adapter_response_single_msg(
            self.enterprise_address.clone(),
            serde_json_wasm::to_string(&Unstake(params))?,
            vec![],
        ))
    }

    fn adapt_claim(&self, _: QueryContext) -> EnterpriseFacadeResult<AdapterResponse> {
        Ok(adapter_response_single_msg(
            self.enterprise_address.clone(),
            serde_json_wasm::to_string(&Claim {})?,
            vec![],
        ))
    }
}

impl EnterpriseFacadeV5 {
    fn query_enterprise_contract<T: DeserializeOwned>(
        &self,
        deps: Deps,
        msg: &impl Serialize,
    ) -> EnterpriseFacadeResult<T> {
        Ok(deps
            .querier
            .query_wasm_smart(self.enterprise_address.to_string(), &msg)?)
    }
}

/// This is what execute messages for Enterprise contract looked like for v5.
#[cw_serde]
enum ExecuteV5Msg {
    // facaded part
    ExecuteProposal(ExecuteProposalMsg),
    // adapted part
    CreateProposal(CreateProposalMsg),
    CreateCouncilProposal(CreateProposalMsg),
    CastVote(CastVoteMsg),
    CastCouncilVote(CastVoteMsg),
    Unstake(UnstakeMsg),
    Claim {},
}

/// This is what CW20-receive hook messages for Enterprise contract looked like for v5.
#[cw_serde]
enum Cw20HookV5Msg {
    Stake {},
    CreateProposal(CreateProposalMsg),
}

/// This is what CW721-receive hook messages for Enterprise contract looked like for v5.
#[cw_serde]
pub enum Cw721HookV5Msg {
    Stake {},
}

/// This is what query messages for Enterprise contract looked like for v5.
/// Looks almost the same as the API for enterprise-facade, but the facade also takes target
/// Enterprise contract address in each of the queries.
#[cw_serde]
pub enum QueryV5Msg {
    DaoInfo {},
    MemberInfo(QueryMemberInfoMsg),
    ListMultisigMembers(ListMultisigMembersMsg),
    AssetWhitelist(AssetWhitelistParams),
    NftWhitelist(NftWhitelistParams),
    Proposal(ProposalParams),
    Proposals(ProposalsParams),
    ProposalStatus(ProposalStatusParams),
    MemberVote(MemberVoteParams),
    ProposalVotes(ProposalVotesParams),
    UserStake(UserStakeV5Params),
    TotalStakedAmount {},
    StakedNfts(StakedNftsParams),
    Claims(ClaimsParams),
    ReleasableClaims(ClaimsParams),
}

#[cw_serde]
pub struct UserStakeV5Params {
    pub user: String,
}
