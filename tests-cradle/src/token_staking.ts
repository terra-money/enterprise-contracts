import {ExecuteCtx, executeTx, fundAddressWithLuna} from "./txUtils";
import {DaoContracts} from "./main";
import {LCDClient, MsgExecuteContract} from "@terra-money/feather.js";
import {findRandomTokenHolder, queryTokenConfig} from "./token_dao_utils";
import {ReleaseAt} from "./types/enterprise_facade";
import assert = require("node:assert");
import {queryCw20Balance} from "./cw20_utils";
import {assertUserWeight, queryUserWeight} from "./membership_utils";
import {advanceTimeBy} from "./cradle_utils";

export const stakeTokens = async (ctx: ExecuteCtx, dao: DaoContracts, staker: string, amount: number) => {
    console.log(`${staker} staking ${amount} tokens...`);
    const tokenConfig = await queryTokenConfig(ctx.lcd, dao);

    const stakeHookMsg = {
        stake: {
            user: staker,
        }
    };

    await executeTx(ctx, [new MsgExecuteContract(
        staker,
        tokenConfig.token_contract,
        {
            send: {
                contract: dao.membership_contract,
                amount: amount.toString(),
                msg: Buffer.from(JSON.stringify(stakeHookMsg)).toString('base64'),
            }
        }
    )]);
}

export const unstakeTokens = async (ctx: ExecuteCtx, dao: DaoContracts, staker: string, amount: number) => {
    console.log(`${staker} unstaking ${amount} tokens...`);
    await executeTx(ctx, [new MsgExecuteContract(
        staker,
        dao.membership_contract,
        {
            unstake: {
                amount: amount.toString(),
            }
        }
    )]);
}

export const claimTokens = async (ctx: ExecuteCtx, dao: DaoContracts, staker: string) => {
    await executeTx(ctx, [new MsgExecuteContract(
        staker,
        dao.membership_contract,
        {
            claim: {
                user: staker,
            }
        }
    )]);
}

export interface TokenClaimsResponse {
    claims: TokenClaim[]
}

export interface TokenClaim {
    id: number,
    user: string,
    amount: string,
    release_at: ReleaseAt,
}

export const queryTokenClaims = async (lcd: LCDClient, dao: DaoContracts, user: string): Promise<TokenClaim[]> => {
    let claimsResponse = await lcd.wasm.contractQuery<TokenClaimsResponse>(dao.membership_contract, {
        claims: {
            user,
        }
    });
    return claimsResponse.claims;
}

export const queryTokenReleasableClaims = async (lcd: LCDClient, dao: DaoContracts, user: string): Promise<TokenClaim[]> => {
    let claimsResponse = await lcd.wasm.contractQuery<TokenClaimsResponse>(dao.membership_contract, {
        releasable_claims: {
            user,
        }
    });
    return claimsResponse.claims;
}

export const assertLastUserTokenClaim = async (lcd: LCDClient, dao: DaoContracts, user: string, amount: string, releaseAtMillis: number) => {
    const claims = await queryTokenClaims(lcd, dao, user);
    const lastClaim = claims[claims.length - 1];

    assert(lastClaim.amount === amount, "User's last claim amount does not match the expected");

    if ('timestamp' in lastClaim.release_at) {
        const timestamp = parseInt(lastClaim.release_at.timestamp, 10);
        assert(releaseAtMillis === Math.floor(timestamp / 1_000_000), "User's last claim release date does not match the expected");
    } else {
        throw new Error("Height-based release dates for claims are not supported");
    }
}

export const assertUserClaimReleasable = async (lcd: LCDClient, dao: DaoContracts, user: string, amount: string, releaseAtMillis: number) => {
    const releasableClaims = await queryTokenReleasableClaims(lcd, dao, user);

    const expectedClaim = releasableClaims.find((claim) => {
        if ('timestamp' in claim.release_at) {
            return Math.floor(parseInt(claim.release_at.timestamp, 10) / 1_000_000) === releaseAtMillis && claim.amount === amount
        } else {
            throw new Error("Height-based release dates for claims are not supported");
        }
    });

    assert(expectedClaim !== undefined, "Expected to find a claim within releasable ones, but it was not present");
}

export const verifyTokenStakingWorks = async (ctx: ExecuteCtx, dao: DaoContracts) => {
    const user = await findRandomTokenHolder(ctx.lcd, dao);
    console.log(`Using ${user} to test token staking.`);

    await fundAddressWithLuna(ctx, user);

    const tokenConfig = await queryTokenConfig(ctx.lcd, dao);

    const holderBalance = await queryCw20Balance(ctx.lcd, tokenConfig.token_contract, user);

    const initialUserWeight = await queryUserWeight(ctx.lcd, dao, user);

    await stakeTokens(ctx, dao, user, holderBalance);

    let expectedWeight = initialUserWeight + holderBalance;
    console.log(`Asserting user's weight after staking is ${expectedWeight}.`);
    await assertUserWeight(ctx.lcd, dao, user, expectedWeight);

    await unstakeTokens(ctx, dao, user, holderBalance);

    const unstakingBlock = await ctx.lcd.tendermint.blockInfo(ctx.chainId);

    console.log(`Asserting user's weight after unstaking is the same as initial (${initialUserWeight}).`);
    await assertUserWeight(ctx.lcd, dao, user, initialUserWeight);

    const unlockingPeriod = tokenConfig.unlocking_period;
    let unlockingPeriodMillis: number;
    if ('time' in unlockingPeriod) {
        unlockingPeriodMillis = unlockingPeriod.time * 1000;
    } else {
        throw new Error("Height unlocking periods are not supported");
    }
    const expectedReleaseMillis = new Date(unstakingBlock.block.header.time).getTime() + unlockingPeriodMillis;

    console.log(`Checking that the unstaked tokens are added to user's claims.`);
    await assertLastUserTokenClaim(ctx.lcd, dao, user, holderBalance.toString(), expectedReleaseMillis);

    await advanceTimeBy(unlockingPeriodMillis);

    const releasableClaims = await queryTokenReleasableClaims(ctx.lcd, dao, user);
    const releasableAmount = releasableClaims.reduce((acc, claim) => acc + parseInt(claim.amount, 10), 0);

    await assertUserClaimReleasable(ctx.lcd, dao, user, holderBalance.toString(), expectedReleaseMillis);

    await claimTokens(ctx, dao, user);

    const daoTokenUserBalance = await queryCw20Balance(ctx.lcd, tokenConfig.token_contract, user);
    assert(releasableAmount === daoTokenUserBalance, "Amount that was claimed does not match the expected");
}