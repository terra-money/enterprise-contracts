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
export type Addr = string
export interface AssetWhitelistResponse {
  assets: AssetInfoBaseFor_Addr[]
}
export interface ConfigResponse {
  admin: Addr
}
export type ExecuteMsg =
  | {
      set_admin: SetAdminMsg
    }
  | {
      update_asset_whitelist: UpdateAssetWhitelistMsg
    }
  | {
      update_nft_whitelist: UpdateNftWhitelistMsg
    }
  | {
      spend: SpendMsg
    }
  | {
      distribute_funds: DistributeFundsMsg
    }
  | {
      execute_cosmos_msgs: ExecuteCosmosMsgsMsg
    }
  | {
      perform_next_migration_step: {
        submsgs_limit?: number | null
      }
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
export type Uint128 = string
export interface SetAdminMsg {
  new_admin: string
}
export interface UpdateAssetWhitelistMsg {
  /**
   * New assets to add to the whitelist. Will ignore assets that are already whitelisted.
   */
  add: AssetInfoBaseFor_String[]
  /**
   * Assets to remove from the whitelist. Will ignore assets that are not already whitelisted.
   */
  remove: AssetInfoBaseFor_String[]
}
export interface UpdateNftWhitelistMsg {
  /**
   * New NFTs to add to the whitelist. Will ignore NFTs that are already whitelisted.
   */
  add: string[]
  /**
   * NFTs to remove from the whitelist. Will ignore NFTs that are not already whitelisted.
   */
  remove: string[]
}
export interface SpendMsg {
  assets: AssetBaseFor_String[]
  recipient: string
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
export interface DistributeFundsMsg {
  funds: AssetBaseFor_String[]
  funds_distributor_contract: string
}
export interface ExecuteCosmosMsgsMsg {
  /**
   * custom Cosmos msgs to execute
   */
  msgs: string[]
}
export interface InstantiateMsg {
  admin: string
  asset_whitelist?: AssetInfoBaseFor_String[] | null
  nft_whitelist?: string[] | null
}
export interface MigrateMsg {}
export interface NftWhitelistResponse {
  nfts: Addr[]
}
export type QueryMsg =
  | {
      config: {}
    }
  | {
      asset_whitelist: AssetWhitelistParams
    }
  | {
      nft_whitelist: NftWhitelistParams
    }
  | {
      user_weight: UserWeightParams
    }
  | {
      total_weight: TotalWeightParams
    }
  | {
      has_incomplete_v2_migration: {}
    }
  | {
      has_unmoved_stakes_or_claims: {}
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
export type Timestamp = Uint64
export type Uint64 = string
export interface AssetWhitelistParams {
  limit?: number | null
  start_after?: AssetInfoBaseFor_String | null
}
export interface NftWhitelistParams {
  limit?: number | null
  start_after?: string | null
}
export interface UserWeightParams {
  user: string
}
export interface TotalWeightParams {
  /**
   * Denotes the moment at which we're interested in the total weight. Expiration::Never is used for current total weight.
   */
  expiration: Expiration
}
