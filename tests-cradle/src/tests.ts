import {ExecuteCtx} from "./txUtils";
import {Version} from "./types/enterprise_versioning";
import {VersionMigrateMsg} from "./types/enterprise_governance_controller";
import {getDaoComponents} from "./enterprise_utils";
import {verifyTokenStakingWorks} from "./token_staking";
import {queryAllMembers} from "./membership_utils";
import {upgradeDao} from "./proposals";
import {queryDaoInfo} from "./dao_queries";
import {verifyNftStakingWorks} from "./nft_staking";
import {hasDaoCouncil} from "./council";
import {DaoContracts} from "./main";
import assert = require("node:assert");

export const verifyUpgradeCorrectness = async (ctx: ExecuteCtx, daoEnterprise: string, newVersion: Version, migrateMsgs: VersionMigrateMsg[], nextVersion: Version, secondNextVersion: Version) => {
    const dao = await getDaoComponents(ctx.lcd, daoEnterprise);

    const daoInfo = await queryDaoInfo(ctx.lcd, dao);
    console.log(`Running tests for ${daoInfo.metadata.name}.`);

    const initialMembers = await queryAllMembers(ctx.lcd, dao);

    const hasCouncil = await hasDaoCouncil(ctx.lcd, dao);

    await upgradeDao(ctx, dao, newVersion, migrateMsgs, hasCouncil ? 'council' : 'general');

    console.log('\n');

    await verifyStakingCorrectness(ctx, dao);

    console.log('Querying all members to compare with original weights.');
    const currentMembers = await queryAllMembers(ctx.lcd, dao);

    console.log('Asserting member weights have remained unchanged.');
    assert(JSON.stringify(currentMembers.sort()) === JSON.stringify(initialMembers.sort()), "Member weights should remain the same");

    await upgradeDao(ctx, dao, nextVersion, [], 'general');
    await upgradeDao(ctx, dao, secondNextVersion, [], 'council');

    const newDaoInfo = await queryDaoInfo(ctx.lcd, dao);
    console.log('New DAO version:', JSON.stringify(newDaoInfo.dao_version));
}

const verifyStakingCorrectness = async (ctx: ExecuteCtx, dao: DaoContracts) => {
    const daoInfo = await queryDaoInfo(ctx.lcd, dao);
    switch (daoInfo.dao_type) {
        case "token":
            await verifyTokenStakingWorks(ctx, dao);
            break;
        case "nft":
            await verifyNftStakingWorks(ctx, dao);
            break;
        case "denom":
            throw new Error("Denom staking checks not yet implemented");
    }
}