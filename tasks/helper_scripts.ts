import task, {Executor, Refs} from "@terra-money/terrariums";

const ENTERPRISE_FACTORY = "enterprise-factory";

// assets
const DENOM_LUNA = "uluna";
const DENOM_AXL_USDC = "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4";
const DENOM_AXL_USDT = "ibc/CBF67A2BCF6CAE343FDF251E510C8E18C361FC02B23430C121116E0811835DEF";
const DENOM_AXL_WBTC = "ibc/05D299885B07905B6886F554B39346EA6761246076A1120B1950049B92B922DD";
const DENOM_AXL_WETH = "ibc/BC8A77AFBD872FDC32A348D3FB10CC09277C266CFE52081DE341C7EC6752E674";

type ComponentContracts = {
    enterprise_factory_contract: string,
    enterprise_governance_contract: string,
    enterprise_governance_controller_contract: string,
    enterprise_outposts_contract: string,
    enterprise_treasury_contract: string,
    funds_distributor_contract: string,
    membership_contract: string,
    council_membership_contract: string,
    attestation_contract: string | undefined,
}

type TokenConfig = {
    enterprise_contract: string,
    token_contract: string,
    unlocking_period: Object,
}

task(async ({network, executor, refs}) => {
    try {
        await instantiateDao(refs, network, executor);

        const enterprise_contract = "terra1mg4gvn7svq7clshyn8qt6evwsv4yjrvfpfdjjpt29tmqdlcc700srphtm3";

        const component_contracts = await executor.query(enterprise_contract, {component_contracts: {}}) as ComponentContracts;

        const membership_contract = component_contracts.membership_contract;
        const governance_controller = component_contracts.enterprise_governance_controller_contract;

        const token_config = await executor.query(membership_contract, {token_config: {}}) as TokenConfig;

        const token_contract = token_config.token_contract;

        const proposal_id = 1;

        await stakeTokens(executor, token_contract, membership_contract);

        const executeMsgProposalAction = {
            execute_msgs: {
                action_type: "it is what it is",
                msgs: [
                    "{\"stargate\":{\"type_url\":\"/ibc.applications.transfer.v1.MsgTransfer\",\"value\":\"Cgh0cmFuc2ZlchIJY2hhbm5lbC0yGgoKBXVsdW5hEgExIkB0ZXJyYTF0dTl6MHFnZmh0Z242a3o2cjJ4ZGY3cWFhMDUycXA3OTJ1NHVubGt6dnFodG1oemdqampxcWZsbHBtKj9qdW5vMW44eWF1OHk2NXhsNDdoNWR5ajlycjU2dWN5bmp5ZGQyOTBwZ2g0ZHI1Z3ZzMnFmdTJ5cnF2MHJ5MjM41u7hjc7aoMcXQooFeyJ3YXNtIjp7ImNvbnRyYWN0IjoianVubzFuOHlhdTh5NjV4bDQ3aDVkeWo5cnI1NnVjeW5qeWRkMjkwcGdoNGRyNWd2czJxZnUyeXJxdjByeTIzIiwibXNnIjp7ImV4ZWN1dGVfbXNncyI6eyJtc2dzIjpbeyJtc2ciOnsid2FzbSI6eyJpbnN0YW50aWF0ZSI6eyJhZG1pbiI6bnVsbCwiY29kZV9pZCI6MzY4OSwibXNnIjoiZXlKaGJHeHZkMTlqY205emMxOWphR0ZwYmw5dGMyZHpJanAwY25WbExDSnZkMjVsY2lJNkltcDFibTh4Wm1Nd2N6ZzNZMkV5YzIxNll6VXpaalJ4ZW1kamVYVTJhbmR0Y2pSM2VIWjNPRFptTWpJNGJYSmtOMlJrT0dwNGNuQmxjWGxuTTJWdU15SXNJbmRvYVhSbGJHbHpkQ0k2Ym5Wc2JDd2liWE5uY3lJNmJuVnNiSDA9IiwiZnVuZHMiOltdLCJsYWJlbCI6IlByb3h5IGNvbnRyYWN0In19fSwicmVwbHlfY2FsbGJhY2siOnsiY2FsbGJhY2tfaWQiOjEsImliY19wb3J0IjoidHJhbnNmZXIiLCJpYmNfY2hhbm5lbCI6ImNoYW5uZWwtODYiLCJkZW5vbSI6ImliYy8xMDdEMTUyQkIzMTc2RkFFQkY0QzJBODRDNUZGREVFQTdDN0NCNEZFMUJCREFCNzEwRjFGRDI1QkNEMDU1Q0JGIiwicmVjZWl2ZXIiOiJ0ZXJyYTF0dTl6MHFnZmh0Z242a3o2cjJ4ZGY3cWFhMDUycXA3OTJ1NHVubGt6dnFodG1oemdqampxcWZsbHBtIn19XX19fX0=\"}}"
                ]
            }
        };

        const deployCrossChainTreasuryProposalAction = {
            deploy_cross_chain_treasury: {
                cross_chain_msg_spec: {
                    chain_id: "juno-1",
                    chain_bech32_prefix: "juno",
                    src_ibc_port: "transfer",
                    src_ibc_channel: "channel-2",
                    dest_ibc_port: "transfer",
                    dest_ibc_channel: "channel-86",
                    uluna_denom: "ibc/107D152BB3176FAEBF4C2A84C5FFDEEA7C7CB4FE1BBDAB710F1FD25BCD055CBF"
                },
                ics_proxy_code_id: 3689,
                enterprise_treasury_code_id: 3690,
                chain_global_proxy: "juno1n8yau8y65xl47h5dyj9rr56ucynjydd290pgh4dr5gvs2qfu2yrqv0ry23"
            }
        };

        const upgradeDaoProposalAction = {
            upgrade_dao: {
                new_version: {
                    major: 2,
                    minor: 1,
                    patch: 0,
                },
                migrate_msgs: [],
            }
        };

        const spendTreasuryToJunoProposalAction = {
            execute_treasury_msgs: {
                action_type: "spend_treasury_cross_chain",
                msgs: [
                    "{\"stargate\":{\"type_url\":\"/ibc.applications.transfer.v1.MsgTransfer\",\"value\":\"Cgh0cmFuc2ZlchIJY2hhbm5lbC0yGg4KBXVsdW5hEgUxMDAwMCJAdGVycmExd3ZuajZjZHRwbGNxYXcwYXhzd2g0ZzR5ZDBudDI4NTN4NGFjMGt1ODI2YXI5Y2VhanRjc3J3NDU5OCo/anVubzE0ajBuY3d3dXVkbmZuZ3ZqMG1mN3V2dms2NnRscnQ0djk3NmVkZ3FyaHF2OTdjY2F1cXZzeGZxaGU3OICArprFhIvJFw==\"}}"
                ],
            }
        }

        const delegateTreasuryOnJunoProposalAction = {
            execute_treasury_msgs: {
                action_type: "spend_treasury_cross_chain",
                msgs: [
                    "{\"staking\":{\"delegate\":{\"validator\":\"junovaloper1t8ehvswxjfn3ejzkjtntcyrqwvmvuknzmvtaaa\",\"amount\":{\"denom\":\"ujuno\",\"amount\":\"100\"}}}}"
                ],
                remote_treasury_target: {
                    cross_chain_msg_spec: {
                        chain_id: "juno-1",
                        chain_bech32_prefix: "juno",
                        src_ibc_port: "transfer",
                        src_ibc_channel: "channel-2",
                        dest_ibc_port: "transfer",
                        dest_ibc_channel: "channel-86",
                        uluna_denom: "ibc/107D152BB3176FAEBF4C2A84C5FFDEEA7C7CB4FE1BBDAB710F1FD25BCD055CBF",
                    }
                }
            }
        }

        await createProposal(executor, governance_controller, deployCrossChainTreasuryProposalAction);

        await castYesVote(executor, governance_controller, proposal_id);

        await executeProposal(executor, governance_controller, proposal_id)
    } catch (e) {
        console.log(e);
    }
});

const stakeTokens = async (executor: Executor, token_contract: string, membership_contract: string): Promise<void> => {
    await executor.execute(
        token_contract,
        {
            send: {
                contract: membership_contract,
                amount: "10000",
                msg: "eyJzdGFrZSI6eyJ1c2VyIjoidGVycmExeDV6c2ZkZnhqNnhnNXBxbTA5OTlsYWdtY2NtcndrNTQ0OTVlOXYifX0=",
            }
        }
    );
    await waitForNewBlock();
}

const createProposal = async (executor: Executor, governance_controller: string, proposalAction: Object): Promise<void> => {
    await executor.execute(
        governance_controller,
        {
            create_proposal: {
                title: "Test proposal",
                description: "yeye whatevs",
                proposal_actions: [proposalAction]
            }
        }
    );
    await waitForNewBlock();
}

const castYesVote = async (executor: Executor, governance_controller: string, proposal_id: number): Promise<void> => {
    await executor.execute(governance_controller,
        {
            cast_vote: {
                proposal_id: proposal_id,
                outcome: "yes",
            }
        });
}

function executeProposal(executor: Executor, governance_controller: string, proposal_id: number) {
    return executor.execute(governance_controller,
        {
            execute_proposal: {
                proposal_id: proposal_id,
            }
        });
}

const instantiateDao = async (refs: Refs, network: string, executor: Executor): Promise<void> => {
    const enterpriseFactoryAddr = refs.getAddress(network, ENTERPRISE_FACTORY);

    console.log("enterprise factory addr", enterpriseFactoryAddr);

    try {
        await executor.execute(enterpriseFactoryAddr, {
            create_dao: {
                dao_metadata: TEST_DAO_METADATA,
                gov_config: TEST_GOV_CONFIG,
                dao_council: TEST_DAO_COUNCIL,
                dao_membership: TEST_NEW_CW20_DAO_MEMBERSHIP,
                // asset_whitelist: [
                //   {native: DENOM_LUNA},
                // ],
                // nft_whitelist: [
                //   "terra1x5zsfdfxj6xg5pqm0999lagmccmrwk54495e9v"
                // ],
                // minimum_weight_for_rewards: "3",
                // attestation_text: "Attest that you're not a criminal",
            }
        });
    } catch (e) {
        console.log(e);
    }
}

const waitForNewBlock = async (): Promise<void> => new Promise((resolve) => setTimeout(resolve, 5000))

const TEST_DAO_METADATA = {
    name: "test DAO",
    logo: "none",
    socials: {},
};

const TEST_GOV_CONFIG = {
    quorum: "0.3",
    threshold: "0.3",
    veto_threshold: "0.15",
    vote_duration: 300,
    allow_early_proposal_execution: true,
};

const TEST_DAO_COUNCIL = {
    members: [
        "terra1x5zsfdfxj6xg5pqm0999lagmccmrwk54495e9v"
    ],
    quorum: "0.3",
    threshold: "0.3",
};

const TEST_NEW_CW20_DAO_MEMBERSHIP = {
    new_cw20: {
        token_name: "TestToken",
        token_symbol: "TSTKN",
        token_decimals: 6,
        initial_token_balances: [
            {
                address: "terra1x5zsfdfxj6xg5pqm0999lagmccmrwk54495e9v",
                amount: "1000000000",
            },
        ],
        initial_dao_balance: "1000000000",
        token_mint: {
            minter: "terra1x5zsfdfxj6xg5pqm0999lagmccmrwk54495e9v",
            cap: "3000000000"
        },
        token_marketing: {
            project: "My project bro",
            description: "Randomest description ever",
            marketing_owner: "terra1x5zsfdfxj6xg5pqm0999lagmccmrwk54495e9v",
        },
        unlocking_period: {
            time: 300
        },
    }
};

const TEST_NEW_CW721_DAO_MEMBERSHIP = {
    new_cw721: {
        nft_name: "Test NFT",
        nft_symbol: "TSTNFT",
        minter: "terra1x5zsfdfxj6xg5pqm0999lagmccmrwk54495e9v",
        unlocking_period: {
            time: 300
        }
    }
};

const TEST_NEW_MULTISIG_DAO_MEMBERSHIP = {
    new_multisig: {
        multisig_members: [
            {
                user: "terra1x5zsfdfxj6xg5pqm0999lagmccmrwk54495e9v",
                weight: "100"
            }
        ]
    }
};