use crate::contract::{execute, instantiate};
use crate::rewards::query_user_rewards;
use common::cw::testing::{mock_ctx, mock_info};
use common::cw::{Context, QueryContext};
use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::{coins, to_binary, Addr, Coin, Response, SubMsg, Uint128};
use cw20::Cw20ReceiveMsg;
use cw_asset::Asset;
use funds_distributor_api::api::{
    ClaimRewardsMsg, Cw20Reward, NativeReward, UpdateMinimumEligibleWeightMsg,
    UpdateUserWeightsMsg, UserRewardsParams, UserWeight,
};
use funds_distributor_api::error::DistributorError::{Unauthorized, ZeroTotalWeight};
use funds_distributor_api::error::DistributorResult;
use funds_distributor_api::msg::{Cw20HookMsg, ExecuteMsg, InstantiateMsg};
use itertools::Itertools;

const ADMIN: &str = "admin";
const ENTERPRISE_CONTRACT: &str = "enterprise_contract";

const LUNA: &str = "uluna";
const CW20_TOKEN: &str = "cw20_token";

#[test]
pub fn no_rewards_after_instantiate() -> DistributorResult<()> {
    let mut deps = mock_dependencies();
    let ctx = &mut mock_ctx(deps.as_mut());

    instantiate_default(ctx)?;

    let user_rewards = query_user_rewards(
        ctx.to_query(),
        UserRewardsParams {
            user: "user".to_string(),
            native_denoms: vec!["uluna".to_string()],
            cw20_assets: vec!["cw20_token".to_string()],
        },
    )?;

    assert_eq!(user_rewards.native_rewards, vec![native_reward(LUNA, 0u8)]);
    assert_eq!(
        user_rewards.cw20_rewards,
        vec![cw20_reward(CW20_TOKEN, 0u8)]
    );

    Ok(())
}

#[test]
pub fn distribute_native_zero_total_weight_fails() -> DistributorResult<()> {
    let mut deps = mock_dependencies();
    let ctx = &mut mock_ctx(deps.as_mut());

    instantiate_default(ctx)?;

    let result = distribute_native(ctx, &coins(100u128, LUNA));

    assert_eq!(result, Err(ZeroTotalWeight));

    Ok(())
}

#[test]
pub fn distribute_cw20_zero_total_weight_fails() -> DistributorResult<()> {
    let mut deps = mock_dependencies();
    let ctx = &mut mock_ctx(deps.as_mut());

    instantiate_default(ctx)?;

    let result = distribute_cw20(ctx, CW20_TOKEN, 100u8);

    assert_eq!(result, Err(ZeroTotalWeight));

    Ok(())
}

#[test]
pub fn update_user_weight_by_non_enterprise_fails() -> DistributorResult<()> {
    let mut deps = mock_dependencies();
    let ctx = &mut mock_ctx(deps.as_mut());

    instantiate_default(ctx)?;

    let result = update_user_weights(ctx, "not_enterprise", vec![user_weight("user", 0u8)]);

    assert_eq!(result, Err(Unauthorized));

    Ok(())
}

#[ignore = "to be fixed"]
#[test]
pub fn update_user_weight_updates_pending_rewards() -> DistributorResult<()> {
    let mut deps = mock_dependencies();
    let ctx = &mut mock_ctx(deps.as_mut());

    instantiate_default(ctx)?;

    update_user_weights(ctx, ENTERPRISE_CONTRACT, vec![user_weight("user", 1u8)])?;

    assert_user_rewards(
        ctx,
        "user",
        vec![LUNA],
        vec![CW20_TOKEN],
        vec![native_reward(LUNA, 0u8)],
        vec![cw20_reward(CW20_TOKEN, 0u8)],
    )?;

    distribute_native(ctx, &coins(50, LUNA))?;
    distribute_cw20(ctx, CW20_TOKEN, 100u8)?;

    assert_user_rewards(
        ctx,
        "user",
        vec![LUNA],
        vec![CW20_TOKEN],
        vec![native_reward(LUNA, 50u8)],
        vec![cw20_reward(CW20_TOKEN, 100u8)],
    )?;

    Ok(())
}

#[ignore = "to be fixed"]
#[test]
pub fn distribute_rewards_distributes_proportional_to_total_weight() -> DistributorResult<()> {
    let mut deps = mock_dependencies();
    let ctx = &mut mock_ctx(deps.as_mut());

    instantiate_default(ctx)?;

    update_user_weights(ctx, ENTERPRISE_CONTRACT, vec![user_weight("user1", 1u8)])?;

    update_user_weights(ctx, ENTERPRISE_CONTRACT, vec![user_weight("user2", 2u8)])?;

    distribute_native(ctx, &coins(30, LUNA))?;
    distribute_cw20(ctx, CW20_TOKEN, 60u8)?;

    assert_user_rewards(
        ctx,
        "user1",
        vec![LUNA],
        vec![CW20_TOKEN],
        vec![native_reward(LUNA, 10u8)],
        vec![cw20_reward(CW20_TOKEN, 20u8)],
    )?;

    assert_user_rewards(
        ctx,
        "user2",
        vec![LUNA],
        vec![CW20_TOKEN],
        vec![native_reward(LUNA, 20u8)],
        vec![cw20_reward(CW20_TOKEN, 40u8)],
    )?;

    Ok(())
}

#[ignore = "to be fixed"]
#[test]
pub fn rewards_calculated_properly_for_users_coming_after_distribution() -> DistributorResult<()> {
    let mut deps = mock_dependencies();
    let ctx = &mut mock_ctx(deps.as_mut());

    instantiate_default(ctx)?;

    update_user_weights(ctx, ENTERPRISE_CONTRACT, vec![user_weight("user1", 1u8)])?;

    distribute_native(ctx, &coins(30, LUNA))?;
    distribute_cw20(ctx, CW20_TOKEN, 60u8)?;

    update_user_weights(ctx, ENTERPRISE_CONTRACT, vec![user_weight("user2", 2u8)])?;

    assert_user_rewards(
        ctx,
        "user1",
        vec![LUNA],
        vec![CW20_TOKEN],
        vec![native_reward(LUNA, 30u8)],
        vec![cw20_reward(CW20_TOKEN, 60u8)],
    )?;

    assert_user_rewards(
        ctx,
        "user2",
        vec![LUNA],
        vec![CW20_TOKEN],
        vec![native_reward(LUNA, 0u8)],
        vec![cw20_reward(CW20_TOKEN, 0u8)],
    )?;

    Ok(())
}

#[ignore = "to be fixed"]
#[test]
pub fn claiming_pending_rewards_sends_messages() -> DistributorResult<()> {
    let mut deps = mock_dependencies();
    let ctx = &mut mock_ctx(deps.as_mut());

    instantiate_default(ctx)?;

    update_user_weights(ctx, ENTERPRISE_CONTRACT, vec![user_weight("user", 1u8)])?;

    distribute_native(ctx, &coins(30, LUNA))?;
    distribute_cw20(ctx, CW20_TOKEN, 60u8)?;

    let response = claim(ctx, "user", vec![LUNA], vec![CW20_TOKEN])?;

    assert_eq!(
        response.messages,
        vec![
            SubMsg::new(Asset::native(LUNA, 30u8).transfer_msg(Addr::unchecked("user"))?),
            SubMsg::new(
                Asset::cw20(Addr::unchecked(CW20_TOKEN), 60u8)
                    .transfer_msg(Addr::unchecked("user"))?
            ),
        ],
    );

    assert_user_rewards(
        ctx,
        "user",
        vec![LUNA],
        vec![CW20_TOKEN],
        vec![native_reward(LUNA, 0u8)],
        vec![cw20_reward(CW20_TOKEN, 0u8)],
    )?;

    Ok(())
}

#[ignore = "to be fixed"]
#[test]
pub fn claiming_pending_rewards_after_weight_change_sends_messages() -> DistributorResult<()> {
    let mut deps = mock_dependencies();
    let ctx = &mut mock_ctx(deps.as_mut());

    instantiate_default(ctx)?;

    update_user_weights(ctx, ADMIN, vec![user_weight("user", 1u8)])?;

    distribute_native(ctx, &coins(30, LUNA))?;
    distribute_cw20(ctx, CW20_TOKEN, 60u8)?;

    update_user_weights(ctx, ADMIN, vec![user_weight("user", 3u8)])?;

    let response = claim(ctx, "user", vec![LUNA], vec![CW20_TOKEN])?;

    assert_eq!(
        response.messages,
        vec![
            SubMsg::new(Asset::native(LUNA, 30u8).transfer_msg(Addr::unchecked("user"))?),
            SubMsg::new(
                Asset::cw20(Addr::unchecked(CW20_TOKEN), 60u8)
                    .transfer_msg(Addr::unchecked("user"))?
            ),
        ],
    );

    assert_user_rewards(
        ctx,
        "user",
        vec![LUNA],
        vec![CW20_TOKEN],
        vec![native_reward(LUNA, 0u8)],
        vec![cw20_reward(CW20_TOKEN, 0u8)],
    )?;

    Ok(())
}

#[ignore = "to be fixed"]
#[test]
pub fn claiming_with_no_rewards_sends_no_msgs() -> DistributorResult<()> {
    let mut deps = mock_dependencies();
    let ctx = &mut mock_ctx(deps.as_mut());

    instantiate_default(ctx)?;

    update_user_weights(ctx, ADMIN, vec![user_weight("user1", 1u8)])?;

    distribute_native(ctx, &coins(30, LUNA))?;
    distribute_cw20(ctx, CW20_TOKEN, 60u8)?;

    let response = claim(ctx, "user2", vec![LUNA], vec![CW20_TOKEN])?;

    assert!(response.messages.is_empty());

    Ok(())
}

#[ignore = "to be fixed"]
#[test]
pub fn users_under_minimum_eligible_weight_receive_no_rewards() -> DistributorResult<()> {
    let mut deps = mock_dependencies();
    let ctx = &mut mock_ctx(deps.as_mut());

    instantiate(
        ctx.deps.branch(),
        ctx.env.clone(),
        ctx.info.clone(),
        InstantiateMsg {
            admin: ADMIN.to_string(),
            enterprise_contract: ENTERPRISE_CONTRACT.to_string(),
            initial_weights: vec![],
            minimum_eligible_weight: Some(4u8.into()),
        },
    )?;

    update_user_weights(ctx, ENTERPRISE_CONTRACT, vec![user_weight("user1", 3u8)])?;
    update_user_weights(ctx, ENTERPRISE_CONTRACT, vec![user_weight("user2", 4u8)])?;

    distribute_native(ctx, &coins(30, LUNA))?;
    distribute_cw20(ctx, CW20_TOKEN, 60u8)?;

    assert_user_rewards(
        ctx,
        "user1",
        vec![LUNA],
        vec![CW20_TOKEN],
        vec![native_reward(LUNA, 0u8)],
        vec![cw20_reward(CW20_TOKEN, 0u8)],
    )?;
    assert_user_rewards(
        ctx,
        "user2",
        vec![LUNA],
        vec![CW20_TOKEN],
        vec![native_reward(LUNA, 30u8)],
        vec![cw20_reward(CW20_TOKEN, 60u8)],
    )?;

    Ok(())
}

#[ignore = "to be fixed"]
#[test]
pub fn minimum_eligible_weight_increase_calculates_existing_rewards_properly(
) -> DistributorResult<()> {
    let mut deps = mock_dependencies();
    let ctx = &mut mock_ctx(deps.as_mut());
    instantiate_default(ctx)?;

    update_user_weights(ctx, ENTERPRISE_CONTRACT, vec![user_weight("user1", 4u8)])?;
    update_user_weights(ctx, ENTERPRISE_CONTRACT, vec![user_weight("user2", 6u8)])?;

    distribute_native(ctx, &coins(30, LUNA))?;
    distribute_cw20(ctx, CW20_TOKEN, 60u8)?;

    update_minimum_eligible_weight(ctx, ENTERPRISE_CONTRACT, 5u8)?;

    distribute_native(ctx, &coins(30, LUNA))?;
    distribute_cw20(ctx, CW20_TOKEN, 60u8)?;

    assert_user_rewards(
        ctx,
        "user1",
        vec![LUNA],
        vec![CW20_TOKEN],
        vec![native_reward(LUNA, 12u8)],
        vec![cw20_reward(CW20_TOKEN, 24u8)],
    )?;
    assert_user_rewards(
        ctx,
        "user2",
        vec![LUNA],
        vec![CW20_TOKEN],
        vec![native_reward(LUNA, 48u8)],
        vec![cw20_reward(CW20_TOKEN, 96u8)],
    )?;

    Ok(())
}

#[ignore = "to be fixed"]
#[test]
pub fn minimum_eligible_weight_decrease_calculates_existing_rewards_properly(
) -> DistributorResult<()> {
    let mut deps = mock_dependencies();
    let ctx = &mut mock_ctx(deps.as_mut());
    instantiate_default(ctx)?;

    instantiate(
        ctx.deps.branch(),
        ctx.env.clone(),
        ctx.info.clone(),
        InstantiateMsg {
            admin: ADMIN.to_string(),
            enterprise_contract: ENTERPRISE_CONTRACT.to_string(),
            initial_weights: vec![user_weight("user1", 4u8), user_weight("user2", 6u8)],
            minimum_eligible_weight: Some(5u8.into()),
        },
    )?;

    distribute_native(ctx, &coins(30, LUNA))?;
    distribute_cw20(ctx, CW20_TOKEN, 60u8)?;

    update_minimum_eligible_weight(ctx, ENTERPRISE_CONTRACT, 3u8)?;

    assert_user_rewards(
        ctx,
        "user1",
        vec![LUNA],
        vec![CW20_TOKEN],
        vec![native_reward(LUNA, 0u8)],
        vec![cw20_reward(CW20_TOKEN, 0u8)],
    )?;
    assert_user_rewards(
        ctx,
        "user2",
        vec![LUNA],
        vec![CW20_TOKEN],
        vec![native_reward(LUNA, 30u8)],
        vec![cw20_reward(CW20_TOKEN, 60u8)],
    )?;

    distribute_native(ctx, &coins(30, LUNA))?;
    distribute_cw20(ctx, CW20_TOKEN, 60u8)?;

    assert_user_rewards(
        ctx,
        "user1",
        vec![LUNA],
        vec![CW20_TOKEN],
        vec![native_reward(LUNA, 12u8)],
        vec![cw20_reward(CW20_TOKEN, 24u8)],
    )?;
    assert_user_rewards(
        ctx,
        "user2",
        vec![LUNA],
        vec![CW20_TOKEN],
        vec![native_reward(LUNA, 48u8)],
        vec![cw20_reward(CW20_TOKEN, 96u8)],
    )?;

    Ok(())
}

///////////////////////
/////// HELPERS ///////
///////////////////////

fn instantiate_default(ctx: &mut Context) -> DistributorResult<()> {
    instantiate(
        ctx.deps.branch(),
        ctx.env.clone(),
        ctx.info.clone(),
        InstantiateMsg {
            admin: ADMIN.to_string(),
            enterprise_contract: ENTERPRISE_CONTRACT.to_string(),
            initial_weights: vec![],
            minimum_eligible_weight: None,
        },
    )?;
    Ok(())
}

fn native_reward(denom: impl Into<String>, amount: impl Into<Uint128>) -> NativeReward {
    NativeReward {
        denom: denom.into(),
        amount: amount.into(),
    }
}

fn cw20_reward(asset: impl Into<String>, amount: impl Into<Uint128>) -> Cw20Reward {
    Cw20Reward {
        asset: asset.into(),
        amount: amount.into(),
    }
}

fn distribute_native(ctx: &mut Context, funds: &[Coin]) -> DistributorResult<Response> {
    execute(
        ctx.deps.branch(),
        ctx.env.clone(),
        mock_info(ctx.info.sender.as_ref(), funds),
        ExecuteMsg::DistributeNative {},
    )
}

fn distribute_cw20(
    ctx: &mut Context,
    token: &str,
    amount: impl Into<Uint128>,
) -> DistributorResult<Response> {
    execute(
        ctx.deps.branch(),
        ctx.env.clone(),
        mock_info(token, &ctx.info.funds),
        ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: ctx.info.sender.to_string(),
            amount: amount.into(),
            msg: to_binary(&Cw20HookMsg::Distribute {})?,
        }),
    )
}

fn claim(
    ctx: &mut Context,
    user: &str,
    native_denoms: Vec<impl Into<String>>,
    cw20_assets: Vec<impl Into<String>>,
) -> DistributorResult<Response> {
    execute(
        ctx.deps.branch(),
        ctx.env.clone(),
        ctx.info.clone(),
        ExecuteMsg::ClaimRewards(ClaimRewardsMsg {
            user: user.to_string(),
            native_denoms: native_denoms
                .into_iter()
                .map(|denom| denom.into())
                .collect_vec(),
            cw20_assets: cw20_assets
                .into_iter()
                .map(|asset| asset.into())
                .collect_vec(),
        }),
    )
}

fn user_weight(user: impl Into<String>, weight: impl Into<Uint128>) -> UserWeight {
    UserWeight {
        user: user.into(),
        weight: weight.into(),
    }
}

fn update_user_weights(
    ctx: &mut Context,
    sender: &str,
    new_user_weights: Vec<UserWeight>,
) -> DistributorResult<Response> {
    execute(
        ctx.deps.branch(),
        ctx.env.clone(),
        mock_info(sender, &vec![]),
        ExecuteMsg::UpdateUserWeights(UpdateUserWeightsMsg { new_user_weights }),
    )
}

fn update_minimum_eligible_weight(
    ctx: &mut Context,
    sender: &str,
    new_minimum_eligible_weight: impl Into<Uint128>,
) -> DistributorResult<Response> {
    execute(
        ctx.deps.branch(),
        ctx.env.clone(),
        mock_info(sender, &vec![]),
        ExecuteMsg::UpdateMinimumEligibleWeight(UpdateMinimumEligibleWeightMsg {
            minimum_eligible_weight: new_minimum_eligible_weight.into(),
        }),
    )
}

fn assert_user_rewards(
    ctx: &mut Context,
    user: &str,
    native_denoms: Vec<impl Into<String>>,
    cw20_assets: Vec<impl Into<String>>,
    expected_native_rewards: Vec<NativeReward>,
    expected_cw20_rewards: Vec<Cw20Reward>,
) -> DistributorResult<()> {
    let qctx = QueryContext {
        deps: ctx.deps.as_ref(),
        env: ctx.env.clone(),
    };

    let native_denoms = native_denoms
        .into_iter()
        .map(|denom| denom.into())
        .collect_vec();
    let cw20_assets = cw20_assets
        .into_iter()
        .map(|asset| asset.into())
        .collect_vec();

    let user_rewards = query_user_rewards(
        qctx,
        UserRewardsParams {
            user: user.to_string(),
            native_denoms,
            cw20_assets,
        },
    )?;

    assert_eq!(user_rewards.native_rewards, expected_native_rewards);
    assert_eq!(user_rewards.cw20_rewards, expected_cw20_rewards);

    Ok(())
}
