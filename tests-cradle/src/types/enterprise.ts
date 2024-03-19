export type Addr = string
export interface ComponentContractsResponse {
  attestation_contract?: Addr | null
  council_membership_contract: Addr
  enterprise_factory_contract: Addr
  enterprise_governance_contract: Addr
  enterprise_governance_controller_contract: Addr
  enterprise_outposts_contract: Addr
  enterprise_treasury_contract: Addr
  funds_distributor_contract: Addr
  membership_contract: Addr
}
export type Timestamp = Uint64
export type Uint64 = string
export type DaoType = 'denom' | 'token' | 'nft' | 'multisig'
export type Logo =
  | 'none'
  | {
      url: string
    }
export interface DaoInfoResponse {
  creation_date: Timestamp
  dao_type: DaoType
  dao_version: Version
  metadata: DaoMetadata
}
export interface Version {
  major: number
  minor: number
  patch: number
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
export type ExecuteMsg =
  | {
      update_metadata: UpdateMetadataMsg
    }
  | {
      upgrade_dao: UpgradeDaoMsg
    }
  | {
      set_attestation: SetAttestationMsg
    }
  | {
      remove_attestation: {}
    }
  | {
      execute_msgs: ExecuteMsgsMsg
    }
  | {
      finalize_instantiation: FinalizeInstantiationMsg
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
export type Binary = string
export interface UpdateMetadataMsg {
  description: ModifyValueFor_Nullable_String
  discord_username: ModifyValueFor_Nullable_String
  github_username: ModifyValueFor_Nullable_String
  logo: ModifyValueFor_Logo
  name: ModifyValueFor_String
  telegram_username: ModifyValueFor_Nullable_String
  twitter_username: ModifyValueFor_Nullable_String
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
export interface SetAttestationMsg {
  attestation_text: string
}
export interface ExecuteMsgsMsg {
  msgs: string[]
}
export interface FinalizeInstantiationMsg {
  attestation_contract?: string | null
  council_membership_contract: string
  enterprise_governance_contract: string
  enterprise_governance_controller_contract: string
  enterprise_outposts_contract: string
  enterprise_treasury_contract: string
  funds_distributor_contract: string
  membership_contract: string
}
export interface InstantiateMsg {
  dao_creation_date?: Timestamp | null
  dao_metadata: DaoMetadata
  dao_type: DaoType
  dao_version: Version
  enterprise_factory_contract: string
  enterprise_versioning_contract: string
}
export interface MigrateMsg {}
export type QueryMsg =
  | {
      dao_info: {}
    }
  | {
      component_contracts: {}
    }
  | {
      is_restricted_user: IsRestrictedUserParams
    }
export interface IsRestrictedUserParams {
  user: string
}
