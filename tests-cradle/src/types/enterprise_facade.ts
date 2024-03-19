export type AdaptedMsg =
  | {
      execute: AdaptedExecuteMsg
    }
  | {
      bank: AdaptedBankMsg
    }
export type Uint128 = string
export type Addr = string
export interface AdapterResponse {
  msgs: AdaptedMsg[]
}
export interface AdaptedExecuteMsg {
  funds: Coin[]
  msg: string
  target_contract: Addr
}
export interface Coin {
  amount: Uint128
  denom: string
}
export interface AdaptedBankMsg {
  funds: Coin[]
  receiver: Addr
}
export type AssetInfoBaseFor_Addr =
  | {
      native: string
    }
  | {
      cw20: Addr
    }
  | {
      /**
       * @minItems 2
       * @maxItems 2
       */
      cw1155: [Addr, string]
    }
export interface AssetWhitelistResponse {
  assets: AssetInfoBaseFor_Addr[]
}
export type ClaimAsset =
  | {
      cw20: Cw20ClaimAsset
    }
  | {
      cw721: Cw721ClaimAsset
    }
  | {
      denom: DenomClaimAsset
    }
export type ReleaseAt =
  | {
      timestamp: Timestamp
    }
  | {
      height: Uint64
    }
export type Timestamp = Uint64
export type Uint64 = string
export interface ClaimsResponse {
  claims: Claim[]
}
export interface Claim {
  asset: ClaimAsset
  release_at: ReleaseAt
}
export interface Cw20ClaimAsset {
  amount: Uint128
}
export interface Cw721ClaimAsset {
  tokens: string[]
}
export interface DenomClaimAsset {
  amount: Uint128
}
export interface ComponentContractsResponse {
  attestation_contract?: Addr | null
  council_membership_contract?: Addr | null
  enterprise_contract: Addr
  enterprise_factory_contract: Addr
  enterprise_governance_contract?: Addr | null
  enterprise_governance_controller_contract?: Addr | null
  enterprise_outposts_contract?: Addr | null
  enterprise_treasury_contract?: Addr | null
  funds_distributor_contract: Addr
  membership_contract?: Addr | null
}
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
export type Decimal = string
export type DaoType = 'denom' | 'token' | 'nft' | 'multisig'
export type Duration =
  | {
      height: number
    }
  | {
      time: number
    }
export type Logo =
  | 'none'
  | {
      url: string
    }
export interface DaoInfoResponse {
  creation_date: Timestamp
  dao_code_version: Uint64
  dao_council?: DaoCouncil | null
  dao_membership_contract: string
  dao_type: DaoType
  dao_version: Version
  enterprise_factory_contract: Addr
  funds_distributor_contract: Addr
  gov_config: GovConfigFacade
  metadata: DaoMetadata
}
export interface DaoCouncil {
  allowed_proposal_action_types: ProposalActionType[]
  members: Addr[]
  quorum: Decimal
  threshold: Decimal
}
export interface Version {
  major: number
  minor: number
  patch: number
}
export interface GovConfigFacade {
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
   * Duration that has to pass for unstaked membership tokens to be claimable
   */
  unlocking_period: Duration
  /**
   * Portion of votes assigned to veto option from all the votes cast in the given proposal required to veto the proposal. Will default to the threshold set for all proposal options.
   */
  veto_threshold: Decimal
  /**
   * Duration of proposals before they end, expressed in seconds
   */
  vote_duration: number
}
export interface DaoMetadata {
  description?: string | null
  logo: Logo
  name: string
  socials: DaoSocialData
}
export interface DaoSocialData {
  discord_username?: string | null
  github_username?: string | null
  telegram_username?: string | null
  twitter_username?: string | null
}
export interface ExecuteMsg {}
export interface InstantiateMsg {
  enterprise_facade_v1: string
  enterprise_facade_v2: string
}
export interface MemberInfoResponse {
  voting_power: Decimal
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
export interface MultisigMembersResponse {
  members: MultisigMember[]
}
export interface MultisigMember {
  address: string
  weight: Uint128
}
export interface NftWhitelistResponse {
  nfts: Addr[]
}
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
export type ModifyValueFor_Decimal =
  | 'no_change'
  | {
      change: Decimal
    }
export type ModifyValueFor_Duration =
  | 'no_change'
  | {
      change: Duration
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
export type ProposalType = 'general' | 'council'
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
  proposer?: Addr | null
  started_at: Timestamp
  status: ProposalStatus
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
      treasury_address: {
        contract: Addr
      }
    }
  | {
      dao_info: {
        contract: Addr
      }
    }
  | {
      component_contracts: {
        contract: Addr
      }
    }
  | {
      member_info: {
        contract: Addr
        msg: QueryMemberInfoMsg
      }
    }
  | {
      list_multisig_members: {
        contract: Addr
        msg: ListMultisigMembersMsg
      }
    }
  | {
      asset_whitelist: {
        contract: Addr
        params: AssetWhitelistParams
      }
    }
  | {
      nft_whitelist: {
        contract: Addr
        params: NftWhitelistParams
      }
    }
  | {
      proposal: {
        contract: Addr
        params: ProposalParams
      }
    }
  | {
      proposals: {
        contract: Addr
        params: ProposalsParams
      }
    }
  | {
      proposal_status: {
        contract: Addr
        params: ProposalStatusParams
      }
    }
  | {
      member_vote: {
        contract: Addr
        params: MemberVoteParams
      }
    }
  | {
      proposal_votes: {
        contract: Addr
        params: ProposalVotesParams
      }
    }
  | {
      user_stake: {
        contract: Addr
        params: UserStakeParams
      }
    }
  | {
      total_staked_amount: {
        contract: Addr
      }
    }
  | {
      staked_nfts: {
        contract: Addr
        params: StakedNftsParams
      }
    }
  | {
      claims: {
        contract: Addr
        params: ClaimsParams
      }
    }
  | {
      releasable_claims: {
        contract: Addr
        params: ClaimsParams
      }
    }
  | {
      cross_chain_treasuries: {
        contract: Addr
        params: CrossChainTreasuriesParams
      }
    }
  | {
      has_incomplete_v2_migration: {
        contract: Addr
      }
    }
  | {
      has_unmoved_stakes_or_claims: {
        contract: Addr
      }
    }
  | {
      v2_migration_stage: {
        contract: Addr
      }
    }
  | {
      create_proposal_adapted: {
        contract: Addr
        params: CreateProposalMsg
      }
    }
  | {
      create_proposal_with_denom_deposit_adapted: {
        contract: Addr
        params: CreateProposalWithDenomDepositMsg
      }
    }
  | {
      create_proposal_with_token_deposit_adapted: {
        contract: Addr
        params: CreateProposalWithTokenDepositMsg
      }
    }
  | {
      create_proposal_with_nft_deposit_adapted: {
        contract: Addr
        params: CreateProposalWithNftDepositMsg
      }
    }
  | {
      create_council_proposal_adapted: {
        contract: Addr
        params: CreateProposalMsg
      }
    }
  | {
      cast_vote_adapted: {
        contract: Addr
        params: CastVoteMsg
      }
    }
  | {
      cast_council_vote_adapted: {
        contract: Addr
        params: CastVoteMsg
      }
    }
  | {
      execute_proposal_adapted: {
        contract: Addr
        params: ExecuteProposalMsg
      }
    }
  | {
      stake_adapted: {
        contract: Addr
        params: StakeMsg
      }
    }
  | {
      unstake_adapted: {
        contract: Addr
        params: UnstakeMsg
      }
    }
  | {
      claim_adapted: {
        contract: Addr
      }
    }
export type ProposalStatusFilter = 'in_progress' | 'passed' | 'rejected'
export type VoteOutcome = 'yes' | 'no' | 'abstain' | 'veto'
export type StakeMsg =
  | {
      cw20: StakeCw20Msg
    }
  | {
      cw721: StakeCw721Msg
    }
  | {
      denom: StakeDenomMsg
    }
export type UnstakeMsg =
  | {
      cw20: UnstakeCw20Msg
    }
  | {
      cw721: UnstakeCw721Msg
    }
  | {
      denom: UnstakeDenomMsg
    }
export interface QueryMemberInfoMsg {
  member_address: string
}
export interface ListMultisigMembersMsg {
  limit?: number | null
  start_after?: string | null
}
export interface AssetWhitelistParams {
  limit?: number | null
  start_after?: AssetInfoBaseFor_String | null
}
export interface NftWhitelistParams {
  limit?: number | null
  start_after?: string | null
}
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
export interface UserStakeParams {
  limit?: number | null
  start_after?: string | null
  user: string
}
export interface StakedNftsParams {
  limit?: number | null
  start_after?: string | null
}
export interface ClaimsParams {
  owner: string
}
export interface CrossChainTreasuriesParams {
  limit?: number | null
  start_after?: string | null
}
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
export interface CreateProposalWithDenomDepositMsg {
  create_proposal_msg: CreateProposalMsg
  deposit_amount: Uint128
}
export interface CreateProposalWithTokenDepositMsg {
  create_proposal_msg: CreateProposalMsg
  deposit_amount: Uint128
}
export interface CreateProposalWithNftDepositMsg {
  create_proposal_msg: CreateProposalMsg2
  /**
   * Tokens that the user wants to deposit to create the proposal. These tokens are expected to be owned by the user or approved for them, otherwise this fails. governance-controller expects to have an approval for those tokens.
   */
  deposit_tokens: string[]
}
export interface CreateProposalMsg2 {
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
export interface CastVoteMsg {
  outcome: VoteOutcome
  proposal_id: number
}
export interface ExecuteProposalMsg {
  proposal_id: number
}
export interface StakeCw20Msg {
  amount: Uint128
  user: string
}
export interface StakeCw721Msg {
  tokens: string[]
  user: string
}
export interface StakeDenomMsg {
  amount: Uint128
  user: string
}
export interface UnstakeCw20Msg {
  amount: Uint128
}
export interface UnstakeCw721Msg {
  tokens: string[]
}
export interface UnstakeDenomMsg {
  amount: Uint128
}
export interface StakedNftsResponse {
  nfts: string[]
}
export interface TotalStakedAmountResponse {
  total_staked_amount: Uint128
}
export type UserStake =
  | 'none'
  | {
      denom: DenomUserStake
    }
  | {
      token: TokenUserStake
    }
  | {
      nft: NftUserStake
    }
export interface UserStakeResponse {
  user_stake: UserStake
}
export interface DenomUserStake {
  amount: Uint128
}
export interface TokenUserStake {
  amount: Uint128
}
export interface NftUserStake {
  amount: Uint128
  tokens: string[]
}
