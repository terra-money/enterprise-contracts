import {MsgExecuteContract, TxSuccess} from "@terra-money/feather.js";
import {ExecuteCtx, executeTx, findAttribute, fundAddressWithLuna, fundAddressWithLunaMsg} from "./txUtils";
import {DaoContracts} from "./main";
import {queryDaoCouncil} from "./council";
import {ProposalAction, ProposalType, VersionMigrateMsg, VoteOutcome} from "./types/enterprise_governance_controller";
import {Version} from "./types/enterprise_versioning";
import {DaoInfoResponse} from "./types/enterprise";
import {queryAllMembers, queryOneMember} from "./membership_utils";
import {queryGovConfig} from "./gov_controller_utils";
import {isNotContract, queryDaoInfo} from "./dao_queries";
import {findRandomTokenHolder, queryTokenConfig} from "./token_dao_utils";
import {findRandomNftHolder, queryNftConfig} from "./nft_dao_utils";
import {approveTokens, queryAllOwnerTokens} from "./cw721_utils";
import {advanceTimeBy} from "./cradle_utils";


export const createProposal = async (ctx: ExecuteCtx, dao: DaoContracts, proposalAction: ProposalAction, proposalType: ProposalType): Promise<number> => {
    switch (proposalType) {
        case "general":
            return await createGeneralProposal(ctx, dao, proposalAction);
        case "council":
            return await createCouncilProposal(ctx, dao, proposalAction);
        default:
            throw new Error(`Unknown proposal type: ${proposalType}`);
    }
}

export const createGeneralProposal = async (ctx: ExecuteCtx, dao: DaoContracts, proposalAction: ProposalAction): Promise<number> => {
    const govConfig = await queryGovConfig(ctx.lcd, dao);
    if (govConfig.gov_config.minimum_deposit === undefined || govConfig.gov_config.minimum_deposit === null || govConfig.gov_config.minimum_deposit === '0') {
        return await createGeneralProposalWithoutDeposit(ctx, dao, proposalAction);
    } else {
        const minimumDeposit = parseInt(govConfig.gov_config.minimum_deposit, 10);
        return await createGeneralProposalWithDeposit(ctx, dao, proposalAction, minimumDeposit);
    }
}

export const createGeneralProposalWithoutDeposit = async (ctx: ExecuteCtx, dao: DaoContracts, proposalAction: ProposalAction): Promise<number> => {
    const member = await queryOneMember(ctx.lcd, dao);

    await fundAddressWithLuna(ctx, member.user);

    const result = await executeTx(ctx, [new MsgExecuteContract(member.user, dao.enterprise_governance_controller_contract, {
        create_proposal: {
            title: "Test general proposal",
            description: "Massive jabaronies",
            proposal_actions: [proposalAction],
        }
    })]);

    return parseProposalId(result)
}

export const createGeneralProposalWithDeposit = async (ctx: ExecuteCtx, dao: DaoContracts, proposalAction: ProposalAction, minimumDeposit: number): Promise<number> => {
    const daoInfo = await queryDaoInfo(ctx.lcd, dao);

    switch (daoInfo.dao_type) {
        case "token":
            return await createProposalWithTokenDeposit(ctx, dao, proposalAction, minimumDeposit);
        case "nft":
            return await createProposalWithNftDeposit(ctx, dao, proposalAction, minimumDeposit);
        case "denom":
            throw new Error(`Not implemented yet`); // TODO: implement this
        case "multisig":
            throw new Error(`Multisig DAOs don't have deposits on proposals`);
    }
}

const createProposalWithTokenDeposit = async (ctx: ExecuteCtx, dao: DaoContracts, proposalAction: ProposalAction, minimumDeposit: number): Promise<number> => {
    const tokenConfig = await queryTokenConfig(ctx.lcd, dao);

    const createProposalMsg = {
        create_proposal: {
            title: "Test general proposal",
            description: "Massive jabaronies",
            proposal_actions: [proposalAction],
        }
    };

    const proposer = await findRandomTokenHolder(ctx.lcd, dao, minimumDeposit);
    await fundAddressWithLuna(ctx, proposer);

    console.log("Proposer chosen to be:", proposer);

    const result = await executeTx(ctx, [new MsgExecuteContract(proposer, tokenConfig.token_contract, {
        send: {
            contract: dao.enterprise_governance_controller_contract,
            amount: minimumDeposit.toString(),
            msg: Buffer.from(JSON.stringify(createProposalMsg)).toString('base64'),
        }
    })]);

    return parseProposalId(result)
}

const createProposalWithNftDeposit = async (ctx: ExecuteCtx, dao: DaoContracts, proposalAction: ProposalAction, minimumDeposit: number): Promise<number> => {
    const nftConfig = await queryNftConfig(ctx.lcd, dao);

    const proposer = await findRandomNftHolder(ctx.lcd, dao, minimumDeposit);
    await fundAddressWithLuna(ctx, proposer);

    const allOwnerTokens = await queryAllOwnerTokens(ctx.lcd, nftConfig.nft_contract, proposer);

    const depositTokens = allOwnerTokens.slice(0, minimumDeposit);

    await approveTokens(ctx, nftConfig.nft_contract, proposer, depositTokens, dao.enterprise_governance_controller_contract);

    const result = await executeTx(ctx, [new MsgExecuteContract(proposer, dao.enterprise_governance_controller_contract, {
        create_proposal_with_nft_deposit: {
            create_proposal_msg: {
                title: "Test council proposal",
                description: "Jabaronies",
                proposal_actions: [proposalAction],
            },
            deposit_tokens: depositTokens,
        }
    })]);

    return parseProposalId(result)
}

export const createCouncilProposal = async (ctx: ExecuteCtx, dao: DaoContracts, proposalAction: ProposalAction): Promise<number> => {
    const council = await queryDaoCouncil(ctx.lcd, dao);

    await fundAddressWithLuna(ctx, council.members[0]);

    const result = await executeTx(ctx, [new MsgExecuteContract(council.members[0], dao.enterprise_governance_controller_contract, {
        create_council_proposal: {
            title: "Test council proposal",
            description: "Jabaronies",
            proposal_actions: [proposalAction],
        }
    })]);

    return parseProposalId(result)
}

export const castVotes = async (ctx: ExecuteCtx, dao: DaoContracts, proposalId: number, members: string[], outcome: VoteOutcome) => {
    const chunkSize = 5;
    const chunkedArray: string[][] = [];

    for (let i = 0; i < members.length; i += chunkSize) {
        const chunk = members.slice(i, i + chunkSize);
        chunkedArray.push(chunk);
    }

    for (const membersChunk of chunkedArray) {
        const msgs = membersChunk.flatMap((member) => [
            fundAddressWithLunaMsg(ctx, member),
            new MsgExecuteContract(
                member,
                dao.enterprise_governance_controller_contract,
                {
                    cast_vote: {
                        proposal_id: proposalId,
                        outcome,
                    }
                }
            )
        ]);
        await executeTx(ctx, msgs);
    }
}

export const castCouncilVote = async (ctx: ExecuteCtx, dao: DaoContracts, proposalId: number, member: string, outcome: VoteOutcome) => {
    await fundAddressWithLuna(ctx, member);
    await executeTx(ctx, [new MsgExecuteContract(
        member,
        dao.enterprise_governance_controller_contract,
        {
            cast_council_vote: {
                proposal_id: proposalId,
                outcome,
            }
        }
    )])
}

export const executeProposal = async (ctx: ExecuteCtx, dao: DaoContracts, proposalId: number) => {
    ctx.wallet.key.accAddress(ctx.lcd.config[ctx.chainId].prefix);
    await executeTx(ctx, [new MsgExecuteContract(
        ctx.wallet.key.accAddress(ctx.lcd.config[ctx.chainId].prefix),
        dao.enterprise_governance_controller_contract,
        {
            execute_proposal: {
                proposal_id: proposalId,
            }
        }
    )])
}

export const passProposal = async (ctx: ExecuteCtx, dao: DaoContracts, proposalId: number, proposalType: ProposalType) => {
    switch (proposalType) {
        case "general":
            return await passGeneralProposal(ctx, dao, proposalId);
        case "council":
            return await passCouncilProposal(ctx, dao, proposalId);
        default:
            throw new Error(`Unknown proposal type: ${proposalType}`);
    }
}

export const passGeneralProposal = async (ctx: ExecuteCtx, dao: DaoContracts, proposalId: number) => {
    const members = await queryAllMembers(ctx.lcd, dao);

    // TODO: consider voting from all members, instead of just some (might be flaky this way)
    const highestMembers = members.sort((member1, member2) => parseInt(member2.weight) - parseInt(member1.weight))
        .slice(0, 50);

    let votingMembers: string[] = [];
    for (const member of highestMembers) {
        if (parseInt(member.weight) > 0 && await isNotContract(ctx.lcd, member.user)) {
            votingMembers.push(member.user)
        }
    }

    await castVotes(ctx, dao, proposalId, votingMembers, 'yes');

    console.log("Members voted, moving to the end of proposal voting period.");
    const govConfig = await queryGovConfig(ctx.lcd, dao);
    await advanceTimeBy(govConfig.gov_config.vote_duration * 1000);
}

export const passCouncilProposal = async (ctx: ExecuteCtx, dao: DaoContracts, proposalId: number) => {
    const council = await queryDaoCouncil(ctx.lcd, dao);

    // TODO: consider only voting from enough members, instead of all (might reveal bugs)
    for (const member of council.members) {
        await castCouncilVote(ctx, dao, proposalId, member, 'yes');
    }
}

export const createAndExecuteProposal = async (ctx: ExecuteCtx, dao: DaoContracts, proposalAction: ProposalAction, proposalType: ProposalType) => {
    console.log('Creating proposal...');
    const proposalId = await createProposal(ctx, dao, proposalAction, proposalType);
    console.log(`Proposal ${proposalId} created, now voting on it.`);
    await passProposal(ctx, dao, proposalId, proposalType);
    console.log(`Executing proposal.`);
    await executeProposal(ctx, dao, proposalId);
}

export const upgradeDaoAction = (newVersion: Version, migrateMsgs: VersionMigrateMsg[]): ProposalAction => {
    return {
        upgrade_dao: {
            new_version: newVersion,
            migrate_msgs: migrateMsgs,
        }
    }
}

export const upgradeDao = async (ctx: ExecuteCtx, dao: DaoContracts, newVersion: Version, migrateMsgs: VersionMigrateMsg[], proposalType: ProposalType) => {
    const daoInfo = await ctx.lcd.wasm.contractQuery<DaoInfoResponse>(dao.enterprise, {dao_info: {}});

    console.log(`Upgrading ${daoInfo.metadata.name} to version ${newVersion.major}.${newVersion.minor}.${newVersion.patch} via ${proposalType} governance.`);

    await createAndExecuteProposal(ctx, dao, upgradeDaoAction(newVersion, migrateMsgs), proposalType);

    console.log(`Upgraded DAO successfully`);
}

const parseProposalId = (result: TxSuccess): number => {
    const proposalId = findAttribute(result.logs, 'proposal_id');
    return parseInt(proposalId, 10)
}