export type Cw20HookMsg = {
  distribute: {}
}
export type ExecuteMsg =
  | {
      update_user_weights: UpdateUserWeightsMsg
    }
  | {
      update_minimum_eligible_weight: UpdateMinimumEligibleWeightMsg
    }
  | {
      distribute_native: {}
    }
  | {
      claim_rewards: ClaimRewardsMsg
    }
  | {
      receive: Cw20ReceiveMsg
    }
export type Uint128 = string
export type Binary = string
export interface UpdateUserWeightsMsg {
  /**
   * New weights that the users have, after the change
   */
  new_user_weights: UserWeight[]
}
export interface UserWeight {
  user: string
  weight: Uint128
}
export interface UpdateMinimumEligibleWeightMsg {
  /**
   * New minimum weight that the user must have to be eligible for rewards distributions
   */
  minimum_eligible_weight: Uint128
}
export interface ClaimRewardsMsg {
  /**
   * CW20 asset rewards to be claimed, should be addresses of CW20 tokens
   */
  cw20_assets: string[]
  /**
   * Native denominations to be claimed
   */
  native_denoms: string[]
  user: string
}
export interface Cw20ReceiveMsg {
  amount: Uint128
  msg: Binary
  sender: string
}
export interface InstantiateMsg {
  admin: string
  enterprise_contract: string
  initial_weights: UserWeight[]
  /**
   * Optional minimum weight that the user must have to be eligible for rewards distributions
   */
  minimum_eligible_weight?: Uint128 | null
}
export interface MigrateMsg {}
export interface MinimumEligibleWeightResponse {
  minimum_eligible_weight: Uint128
}
export type QueryMsg =
  | {
      user_rewards: UserRewardsParams
    }
  | {
      minimum_eligible_weight: {}
    }
export interface UserRewardsParams {
  /**
   * Addresses of CW20 tokens to be queried for rewards
   */
  cw20_assets: string[]
  /**
   * Native denominations to be queried for rewards
   */
  native_denoms: string[]
  user: string
}
export interface UserRewardsResponse {
  cw20_rewards: Cw20Reward[]
  native_rewards: NativeReward[]
}
export interface Cw20Reward {
  amount: Uint128
  /**
   * Address of the CW20 token
   */
  asset: string
}
export interface NativeReward {
  amount: Uint128
  denom: string
}
