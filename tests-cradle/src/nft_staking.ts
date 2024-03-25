import {ExecuteCtx, executeTx, fundAddressWithLuna} from "./txUtils";
import {DaoContracts} from "./main";
import {LCDClient, MsgExecuteContract} from "@terra-money/feather.js";
import {ReleaseAt} from "./types/enterprise_facade";
import assert = require("node:assert");
import {assertUserWeight, queryUserWeight} from "./membership_utils";
import {advanceTimeBy} from "./cradle_utils";
import {findRandomNftHolder, queryNftConfig} from "./nft_dao_utils";
import {queryAllOwnerTokens} from "./cw721_utils";

export const stakeNfts = async (ctx: ExecuteCtx, dao: DaoContracts, staker: string, tokens: string[]) => {
    console.log(`${staker} staking NFTs: ${JSON.stringify(tokens)}...`);
    const nftConfig = await queryNftConfig(ctx.lcd, dao);

    const stakeHookMsg = {
        stake: {
            user: staker,
        }
    };

    const msgs = tokens.map((token) => new MsgExecuteContract(
        staker,
        nftConfig.nft_contract,
        {
            send_nft: {
                contract: dao.membership_contract,
                token_id: token,
                msg: Buffer.from(JSON.stringify(stakeHookMsg)).toString('base64'),
            }
        }
    ));

    await executeTx(ctx, msgs);

}

export const unstakeNfts = async (ctx: ExecuteCtx, dao: DaoContracts, staker: string, tokens: string[]) => {
    console.log(`${staker} unstaking NFTs: ${JSON.stringify(tokens)}...`);
    await executeTx(ctx, [new MsgExecuteContract(
        staker,
        dao.membership_contract,
        {
            unstake: {
                nft_ids: tokens,
            }
        }
    )]);
}

export const claimNfts = async (ctx: ExecuteCtx, dao: DaoContracts, staker: string) => {
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

export interface NftClaimsResponse {
    claims: NftClaim[]
}

export interface NftClaim {
    id: number,
    user: string,
    nft_ids: string[],
    release_at: ReleaseAt,
}

export const queryNftClaims = async (lcd: LCDClient, dao: DaoContracts, user: string): Promise<NftClaim[]> => {
    let claimsResponse = await lcd.wasm.contractQuery<NftClaimsResponse>(dao.membership_contract, {
        claims: {
            user,
        }
    });
    return claimsResponse.claims;
}

export const queryNftReleasableClaims = async (lcd: LCDClient, dao: DaoContracts, user: string): Promise<NftClaim[]> => {
    let claimsResponse = await lcd.wasm.contractQuery<NftClaimsResponse>(dao.membership_contract, {
        releasable_claims: {
            user,
        }
    });
    return claimsResponse.claims;
}

export const assertLastUserNftClaim = async (lcd: LCDClient, dao: DaoContracts, user: string, tokens: string[], releaseAtMillis: number) => {
    const claims = await queryNftClaims(lcd, dao, user);
    const lastClaim = claims[claims.length - 1];

    assert(JSON.stringify(lastClaim.nft_ids.sort()) === JSON.stringify(tokens.sort()), "User's last claim's NFTs do not match the expected");

    if ('timestamp' in lastClaim.release_at) {
        const timestamp = parseInt(lastClaim.release_at.timestamp, 10);
        assert(releaseAtMillis === Math.floor(timestamp / 1_000_000), "User's last claim release date does not match the expected");
    } else {
        throw new Error("Height-based release dates for claims are not supported");
    }
}

export const assertNftClaimReleasable = async (lcd: LCDClient, dao: DaoContracts, user: string, tokens: string[], releaseAtMillis: number) => {
    const releasableClaims = await queryNftReleasableClaims(lcd, dao, user);

    const expectedClaim = releasableClaims.find((claim) => {
        if ('timestamp' in claim.release_at) {
            return Math.floor(parseInt(claim.release_at.timestamp, 10) / 1_000_000) === releaseAtMillis && JSON.stringify(claim.nft_ids.sort()) === JSON.stringify(tokens.sort())
        } else {
            throw new Error("Height-based release dates for claims are not supported");
        }
    });

    assert(expectedClaim !== undefined, "Expected to find a claim within releasable ones, but it was not present");
}

export const verifyNftStakingWorks = async (ctx: ExecuteCtx, dao: DaoContracts) => {
    const user = await findRandomNftHolder(ctx.lcd, dao);
    console.log(`Using ${user} to test NFT staking.`);

    await fundAddressWithLuna(ctx, user);

    const nftConfig = await queryNftConfig(ctx.lcd, dao);

    const holderNfts = await queryAllOwnerTokens(ctx.lcd, nftConfig.nft_contract, user);
    const initialUserWeight = await queryUserWeight(ctx.lcd, dao, user);

    console.log(`User holds ${holderNfts.length} DAO NFTs, and has ${initialUserWeight} staked.`);

    await stakeNfts(ctx, dao, user, holderNfts);

    let expectedWeight = initialUserWeight + holderNfts.length;
    console.log(`Asserting user's weight after staking is ${expectedWeight}.`);
    await assertUserWeight(ctx.lcd, dao, user, expectedWeight);

    await unstakeNfts(ctx, dao, user, holderNfts);

    const unstakingBlock = await ctx.lcd.tendermint.blockInfo(ctx.chainId);

    console.log(`Asserting user's weight after unstaking is the same as initial (${initialUserWeight}).`);
    await assertUserWeight(ctx.lcd, dao, user, initialUserWeight);

    const unlockingPeriod = nftConfig.unlocking_period;
    let unlockingPeriodMillis: number;
    if ('time' in unlockingPeriod) {
        unlockingPeriodMillis = unlockingPeriod.time * 1000;
    } else {
        throw new Error("Height unlocking periods are not supported");
    }
    const expectedReleaseMillis = new Date(unstakingBlock.block.header.time).getTime() + unlockingPeriodMillis;

    console.log(`Checking that the unstaked NFTs are added to user's claims.`);
    await assertLastUserNftClaim(ctx.lcd, dao, user, holderNfts, expectedReleaseMillis);

    await advanceTimeBy(unlockingPeriodMillis);

    const releasableClaims = await queryNftReleasableClaims(ctx.lcd, dao, user);
    const releasableNfts = releasableClaims.reduce<string[]>((acc, claim) => [...acc, ...claim.nft_ids], []);

    await assertNftClaimReleasable(ctx.lcd, dao, user, holderNfts, expectedReleaseMillis);

    await claimNfts(ctx, dao, user);

    const userNftHoldings = await queryAllOwnerTokens(ctx.lcd, nftConfig.nft_contract, user);
    assert(JSON.stringify(releasableNfts.sort()) === JSON.stringify(userNftHoldings.sort()), "NFTs that were claimed do not match the expected");
}