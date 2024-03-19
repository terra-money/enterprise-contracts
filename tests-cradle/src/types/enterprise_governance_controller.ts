export type Cw20HookMsg = {
  create_proposal: CreateProposalMsg
}
export type ProposalAction =
  | {
      update_metadata: UpdateMetadataMsg
    }
  | {
      update_gov_config: UpdateGovConfigMsg
    }
  | {
      update_council: UpdateCouncilMsg
    }
  | {
      update_asset_whitelist: UpdateAssetWhitelistProposalActionMsg
    }
  | {
      update_nft_whitelist: UpdateNftWhitelistProposalActionMsg
    }
  | {
      request_funding_from_dao: RequestFundingFromDaoMsg
    }
  | {
      upgrade_dao: UpgradeDaoMsg
    }
  | {
      execute_msgs: ExecuteMsgsMsg
    }
  | {
      execute_treasury_msgs: ExecuteTreasuryMsgsMsg
    }
  | {
      execute_enterprise_msgs: ExecuteEnterpriseMsgsMsg
    }
  | {
      modify_multisig_membership: ModifyMultisigMembershipMsg
    }
  | {
      distribute_funds: DistributeFundsMsg
    }
  | {
      update_minimum_weight_for_rewards: UpdateMinimumWeightForRewardsMsg
    }
  | {
      add_attestation: AddAttestationMsg
    }
  | {
      remove_attestation: {}
    }
  | {
      deploy_cross_chain_treasury: DeployCrossChainTreasuryMsg
    }
export type ModifyValueFor_Nullable_String =
  | 'no_change'
  | {
      change: string | null
    }
export type ModifyValueFor_Logo =
  | 'no_change'
  | {
      change: Logo
    }
export type Logo =
  | 'none'
  | {
      url: string
    }
export type ModifyValueFor_String =
  | 'no_change'
  | {
      change: string
    }
export type ModifyValueFor_Boolean =
  | 'no_change'
  | {
      change: boolean
    }
export type ModifyValueFor_Nullable_Uint128 =
  | 'no_change'
  | {
      change: Uint128 | null
    }
export type Uint128 = string
export type ModifyValueFor_Decimal =
  | 'no_change'
  | {
      change: Decimal
    }
export type Decimal = string
export type ModifyValueFor_Duration =
  | 'no_change'
  | {
      change: Duration
    }
export type Duration =
  | {
      height: number
    }
  | {
      time: number
    }
export type ModifyValueFor_Nullable_Decimal =
  | 'no_change'
  | {
      change: Decimal | null
    }
export type ModifyValueFor_Uint64 =
  | 'no_change'
  | {
      change: Uint64
    }
export type Uint64 = string
export type ProposalActionType =
  | 'update_metadata'
  | 'update_gov_config'
  | 'update_council'
  | 'update_asset_whitelist'
  | 'update_nft_whitelist'
  | 'request_funding_from_dao'
  | 'upgrade_dao'
  | 'execute_msgs'
  | 'execute_treasury_msgs'
  | 'execute_enterprise_msgs'
  | 'modify_multisig_membership'
  | 'distribute_funds'
  | 'update_minimum_weight_for_rewards'
  | 'add_attestation'
  | 'remove_attestation'
  | 'deploy_cross_chain_treasury'
export type AssetInfoBaseFor_String =
  | {
      native: string
    }
  | {
      cw20: string
    }
  | {
      /**
       * @minItems 2
       * @maxItems 2
       */
      cw1155: [string, string]
    }
export type Binary = string
export interface CreateProposalMsg {
  /**
   * Optionally define the owner of the proposal deposit. If None, will default to the proposer themselves.
   */
  deposit_owner?: string | null
  /**
   * Optional description text of the proposal
   */
  description?: string | null
  /**
   * Actions to be executed, in order, if the proposal passes
   */
  proposal_actions: ProposalAction[]
  /**
   * Title of the proposal
   */
  title: string
}
export interface UpdateMetadataMsg {
  description: ModifyValueFor_Nullable_String
  discord_username: ModifyValueFor_Nullable_String
  github_username: ModifyValueFor_Nullable_String
  logo: ModifyValueFor_Logo
  name: ModifyValueFor_String
  telegram_username: ModifyValueFor_Nullable_String
  twitter_username: ModifyValueFor_Nullable_String
}
export interface UpdateGovConfigMsg {
  allow_early_proposal_execution: ModifyValueFor_Boolean
  minimum_deposit: ModifyValueFor_Nullable_Uint128
  quorum: ModifyValueFor_Decimal
  threshold: ModifyValueFor_Decimal
  unlocking_period: ModifyValueFor_Duration
  veto_threshold: ModifyValueFor_Nullable_Decimal
  voting_duration: ModifyValueFor_Uint64
}
export interface UpdateCouncilMsg {
  dao_council?: DaoCouncilSpec | null
}
export interface DaoCouncilSpec {
  /**
   * Proposal action types allowed in proposals that are voted on by the council. Effectively defines what types of actions council can propose and vote on. If None, will default to a predefined set of actions.
   */
  allowed_proposal_action_types?: ProposalActionType[] | null
  /**
   * Addresses of council members. Each member has equal voting power.
   */
  members: string[]
  /**
   * Portion of total available votes cast in a proposal to consider it valid e.g. quorum of 30% means that 30% of all available votes have to be cast in the proposal, otherwise it fails automatically when it expires
   */
  quorum: Decimal
  /**
   * Portion of votes assigned to a single option from all the votes cast in the given proposal required to determine the 'winning' option e.g. 51% threshold means that an option has to have at least 51% of the cast votes to win
   */
  threshold: Decimal
}
export interface UpdateAssetWhitelistProposalActionMsg {
  /**
   * New assets to add to the whitelist. Will ignore assets that are already whitelisted.
   */
  add: AssetInfoBaseFor_String[]
  remote_treasury_target?: RemoteTreasuryTarget | null
  /**
   * Assets to remove from the whitelist. Will ignore assets that are not already whitelisted.
   */
  remove: AssetInfoBaseFor_String[]
}
export interface RemoteTreasuryTarget {
  /**
   * Spec for the cross-chain message to send. Treasury address will be determined using chain-id given in the spec.
   */
  cross_chain_msg_spec: CrossChainMsgSpec
}
export interface CrossChainMsgSpec {
  chain_bech32_prefix: string
  chain_id: string
  dest_ibc_channel: string
  dest_ibc_port: string
  src_ibc_channel: string
  src_ibc_port: string
  /**
   * Optional timeout for the cross-chain messages. Formatted in nanoseconds.
   */
  timeout_nanos?: number | null
  /**
   * uluna IBC denom on the remote chain. Currently, can be calculated as 'ibc/' + uppercase(sha256('{port}/{channel}/uluna'))
   */
  uluna_denom: string
}
export interface UpdateNftWhitelistProposalActionMsg {
  /**
   * New NFTs to add to the whitelist. Will ignore NFTs that are already whitelisted.
   */
  add: string[]
  remote_treasury_target?: RemoteTreasuryTarget | null
  /**
   * NFTs to remove from the whitelist. Will ignore NFTs that are not already whitelisted.
   */
  remove: string[]
}
export interface RequestFundingFromDaoMsg {
  assets: AssetBaseFor_String[]
  recipient: string
  remote_treasury_target?: RemoteTreasuryTarget | null
}
export interface AssetBaseFor_String {
  /**
   * Specifies the asset's amount
   */
  amount: Uint128
  /**
   * Specifies the asset's type (CW20 or native)
   */
  info: AssetInfoBaseFor_String
}
export interface UpgradeDaoMsg {
  /**
   * Expects an array of (version, migrate msg for that version). E.g. [ { "version": { "major": 2, "minor": 0, "patch": 0 }, "migrate_msg": <MigrateMsg JSON for 2.0.0> }, { "version": { "major": 2, "minor": 1, "patch": 3 }, "migrate_msg": <MigrateMsg JSON for 2.1.3> } ]
   */
  migrate_msgs: VersionMigrateMsg[]
  new_version: Version
}
export interface VersionMigrateMsg {
  migrate_msg: Binary
  version: Version
}
export interface Version {
  major: number
  minor: number
  patch: number
}
export interface ExecuteMsgsMsg {
  action_type: string
  msgs: string[]
}
export interface ExecuteTreasuryMsgsMsg {
  action_type: string
  msgs: string[]
  remote_treasury_target?: RemoteTreasuryTarget | null
}
export interface ExecuteEnterpriseMsgsMsg {
  action_type: string
  msgs: string[]
}
export interface ModifyMultisigMembershipMsg {
  /**
   * Members to be edited. Can contain existing members, in which case their new weight will be the one specified in this message. This effectively allows removing of members (by setting their weight to 0).
   */
  edit_members: UserWeight[]
}
export interface UserWeight {
  user: string
  weight: Uint128
}
export interface DistributeFundsMsg {
  funds: AssetBaseFor_String[]
}
export interface UpdateMinimumWeightForRewardsMsg {
  minimum_weight_for_rewards: Uint128
}
export interface AddAttestationMsg {
  attestation_text: string
}
export interface DeployCrossChainTreasuryMsg {
  asset_whitelist?: AssetInfoBaseFor_String[] | null
  /**
   * Proxy contract serving globally for the given chain, with no specific permission model.
   */
  chain_global_proxy: string
  cross_chain_msg_spec: CrossChainMsgSpec
  enterprise_treasury_code_id: number
  ics_proxy_code_id: number
  nft_whitelist?: string[] | null
}
export type ExecuteMsg =
  | {
      create_proposal: CreateProposalMsg
    }
  | {
      create_proposal_with_nft_deposit: CreateProposalWithNftDepositMsg
    }
  | {
      create_council_proposal: CreateProposalMsg
    }
  | {
      cast_vote: CastVoteMsg
    }
  | {
      cast_council_vote: CastVoteMsg
    }
  | {
      execute_proposal: ExecuteProposalMsg
    }
  | {
      receive: Cw20ReceiveMsg
    }
  | {
      weights_changed: WeightsChangedMsg
    }
  | {
      execute_proposal_actions: ExecuteProposalMsg
    }
  | {
      deploy_initial_cross_chain_treasuries: {}
    }
export type VoteOutcome = 'yes' | 'no' | 'abstain' | 'veto'
export interface CreateProposalWithNftDepositMsg {
  create_proposal_msg: CreateProposalMsg
  /**
   * Tokens that the user wants to deposit to create the proposal. These tokens are expected to be owned by the user or approved for them, otherwise this fails. governance-controller expects to have an approval for those tokens.
   */
  deposit_tokens: string[]
}
export interface CastVoteMsg {
  outcome: VoteOutcome
  proposal_id: number
}
export interface ExecuteProposalMsg {
  proposal_id: number
}
export interface Cw20ReceiveMsg {
  amount: Uint128
  msg: Binary
  sender: string
}
export interface WeightsChangedMsg {
  weight_changes: UserWeightChange[]
}
export interface UserWeightChange {
  new_weight: Uint128
  old_weight: Uint128
  user: string
}
export type Addr = string
export interface GovConfigResponse {
  council_gov_config?: CouncilGovConfig | null
  dao_council_membership_contract: Addr
  dao_membership_contract: Addr
  gov_config: GovConfig
}
export interface CouncilGovConfig {
  allowed_proposal_action_types: ProposalActionType[]
  quorum: Decimal
  threshold: Decimal
}
export interface GovConfig {
  /**
   * If set to true, this will allow DAOs to execute proposals that have reached quorum and threshold, even before their voting period ends.
   */
  allow_early_proposal_execution: boolean
  /**
   * Optional minimum amount of DAO's governance unit to be required to create a deposit.
   */
  minimum_deposit?: Uint128 | null
  /**
   * Portion of total available votes cast in a proposal to consider it valid e.g. quorum of 30% means that 30% of all available votes have to be cast in the proposal, otherwise it fails automatically when it expires
   */
  quorum: Decimal
  /**
   * Portion of votes assigned to a single option from all the votes cast in the given proposal required to determine the 'winning' option e.g. 51% threshold means that an option has to have at least 51% of the cast votes to win
   */
  threshold: Decimal
  /**
   * Portion of votes assigned to veto option from all the votes cast in the given proposal required to veto the proposal. If None, will default to the threshold set for all proposal options.
   */
  veto_threshold?: Decimal | null
  /**
   * Duration of proposals before they end, expressed in seconds
   */
  vote_duration: number
}
export type DaoType = 'denom' | 'token' | 'nft' | 'multisig'
export type Timestamp = Uint64
export type ProposalDepositAsset =
  | {
      denom: {
        amount: Uint128
        denom: string
      }
    }
  | {
      cw20: {
        amount: Uint128
        token_addr: Addr
      }
    }
  | {
      cw721: {
        nft_addr: Addr
        tokens: string[]
      }
    }
export type ProposalType = 'general' | 'council'
export interface InstantiateMsg {
  council_gov_config?: DaoCouncilSpec | null
  dao_type: DaoType
  enterprise_contract: string
  gov_config: GovConfig
  initial_cross_chain_treasuries?: DeployCrossChainTreasuryMsg[] | null
  proposal_infos?: [number, ProposalInfo][] | null
}
export interface ProposalInfo {
  /**
   * The earliest time at which the proposal's actions can be executed, if it passed. If None, can be executed as soon as the proposal passes
   */
  earliest_execution?: Timestamp | null
  executed_at?: BlockInfo | null
  proposal_actions: ProposalAction[]
  proposal_deposit?: ProposalDeposit | null
  proposal_type: ProposalType
}
export interface BlockInfo {
  chain_id: string
  /**
   * The height of a block is the number of blocks preceding it in the blockchain.
   */
  height: number
  /**
   * Absolute time of the block creation in seconds since the UNIX epoch (00:00:00 on 1970-01-01 UTC).
   *
   * The source of this is the [BFT Time in Tendermint](https://github.com/tendermint/tendermint/blob/58dc1726/spec/consensus/bft-time.md), which has the same nanosecond precision as the `Timestamp` type.
   *
   * # Examples
   *
   * Using chrono:
   *
   * ``` # use cosmwasm_std::{Addr, BlockInfo, ContractInfo, Env, MessageInfo, Timestamp, TransactionInfo}; # let env = Env { #     block: BlockInfo { #         height: 12_345, #         time: Timestamp::from_nanos(1_571_797_419_879_305_533), #         chain_id: "cosmos-testnet-14002".to_string(), #     }, #     transaction: Some(TransactionInfo { index: 3 }), #     contract: ContractInfo { #         address: Addr::unchecked("contract"), #     }, # }; # extern crate chrono; use chrono::NaiveDateTime; let seconds = env.block.time.seconds(); let nsecs = env.block.time.subsec_nanos(); let dt = NaiveDateTime::from_timestamp(seconds as i64, nsecs as u32); ```
   *
   * Creating a simple millisecond-precision timestamp (as used in JavaScript):
   *
   * ``` # use cosmwasm_std::{Addr, BlockInfo, ContractInfo, Env, MessageInfo, Timestamp, TransactionInfo}; # let env = Env { #     block: BlockInfo { #         height: 12_345, #         time: Timestamp::from_nanos(1_571_797_419_879_305_533), #         chain_id: "cosmos-testnet-14002".to_string(), #     }, #     transaction: Some(TransactionInfo { index: 3 }), #     contract: ContractInfo { #         address: Addr::unchecked("contract"), #     }, # }; let millis = env.block.time.nanos() / 1_000_000; ```
   */
  time: Timestamp
}
export interface ProposalDeposit {
  asset: ProposalDepositAsset
  depositor: Addr
}
export interface MemberVoteResponse {
  vote?: Vote | null
}
export interface Vote {
  /**
   * Number of votes on the outcome.
   */
  amount: number
  /**
   * The outcome, 0-indexed.
   */
  outcome: number
  /**
   * Unique identifier for the poll.
   */
  poll_id: number
  /**
   * Voter address.
   */
  voter: Addr
}
export interface MigrateMsg {}
export type Expiration =
  | {
      at_height: number
    }
  | {
      at_time: Timestamp
    }
  | {
      never: {}
    }
export type ProposalStatus =
  | 'in_progress'
  | 'in_progress_can_execute_early'
  | 'passed'
  | 'rejected'
  | 'executed'
export interface ProposalResponse {
  proposal: Proposal
  proposal_status: ProposalStatus
  /**
   * Total vote-count (value) for each outcome (key).
   */
  results: [number, Uint128][]
  total_votes_available: Uint128
}
export interface Proposal {
  description: string
  expires: Expiration
  id: number
  proposal_actions: ProposalAction[]
  proposal_type: ProposalType
  proposer: Addr
  started_at: Timestamp
  status: ProposalStatus
  title: string
}
export interface ProposalStatusResponse {
  expires: Expiration
  /**
   * Total vote-count (value) for each outcome (key).
   */
  results: [number, Uint128][]
  status: ProposalStatus
}
export interface ProposalVotesResponse {
  votes: Vote[]
}
export interface ProposalsResponse {
  proposals: ProposalResponse[]
}
export type QueryMsg =
  | {
      config: {}
    }
  | {
      gov_config: {}
    }
  | {
      proposal: ProposalParams
    }
  | {
      proposals: ProposalsParams
    }
  | {
      proposal_status: ProposalStatusParams
    }
  | {
      member_vote: MemberVoteParams
    }
  | {
      proposal_votes: ProposalVotesParams
    }
export type ProposalStatusFilter = 'in_progress' | 'passed' | 'rejected'
export interface ProposalParams {
  proposal_id: number
}
export interface ProposalsParams {
  /**
   * Optional proposal status to filter for.
   */
  filter?: ProposalStatusFilter | null
  limit?: number | null
  start_after?: number | null
}
export interface ProposalStatusParams {
  proposal_id: number
}
export interface MemberVoteParams {
  member: string
  proposal_id: number
}
export interface ProposalVotesParams {
  limit?: number | null
  proposal_id: number
  /**
   * Optional pagination data, will return votes after the given voter address
   */
  start_after?: string | null
}
