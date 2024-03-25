import {Coin} from "@terra-money/terra.js";
import task, {Deployer, Executor, Refs} from "@terra-money/terrariums";
import {Signer} from "@terra-money/terrariums/lib/src/signers";

const ATTESTATION = "attestation";
const DENOM_STAKING_MEMBERSHIP = "denom-staking-membership";
const ENTERPRISE = "enterprise";
const ENTERPRISE_FACADE = "enterprise-facade";
const ENTERPRISE_FACADE_V1 = "enterprise-facade-v1";
const ENTERPRISE_FACADE_V2 = "enterprise-facade-v2";
const ENTERPRISE_FACTORY = "enterprise-factory";
const ENTERPRISE_GOVERNANCE = "enterprise-governance";
const ENTERPRISE_GOVERNANCE_CONTROLLER = "enterprise-governance-controller";
const ENTERPRISE_TREASURY = "enterprise-treasury";
const ENTERPRISE_OUTPOSTS = "enterprise-outposts";
const ENTERPRISE_VERSIONING = "enterprise-versioning";
const FUNDS_DISTRIBUTOR = "funds-distributor";
const ICS721_CALLBACK_PROXY = "ics721-callback-proxy";
const MULTISIG_MEMBERSHIP = "multisig-membership";
const TOKEN_STAKING_MEMBERSHIP = "token-staking-membership";
const NFT_STAKING_MEMBERSHIP = "nft-staking-membership";

const CW20_BASE = "cw20_base";
const CW721_METADATA_ONCHAIN = "cw721_metadata_onchain";

// assets
const DENOM_LUNA = "uluna";
const DENOM_AXL_USDC = "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4";
const DENOM_AXL_USDT = "ibc/CBF67A2BCF6CAE343FDF251E510C8E18C361FC02B23430C121116E0811835DEF";
const DENOM_AXL_WBTC = "ibc/05D299885B07905B6886F554B39346EA6761246076A1120B1950049B92B922DD";
const DENOM_AXL_WETH = "ibc/BC8A77AFBD872FDC32A348D3FB10CC09277C266CFE52081DE341C7EC6752E674";

const ICS721_PROXY_ADDR = "terra1ed3qw4y4ca3lpj82ugg2jqsjr9czd0yyldcpp5n5yd7hu6udqafslz0nmg";

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

task(async ({network, deployer, executor, signer, refs}) => {
    // deployer.buildContract(ENTERPRISE);
    // deployer.optimizeContract(ENTERPRISE);

    await deployIcs721CallbackProxy(refs, network, deployer, signer);

    // await deployEnterpriseVersioning(refs, network, deployer, signer);

    // await deployEnterpriseFacade(refs, network, deployer, signer);

    // await deployEnterpriseFactory(refs, network, deployer, signer);

    // await deployNewEnterpriseVersion(refs, network, deployer, executor, 1, 1, 1);

    // await instantiateDao(refs, network, executor);

    try {
        // const enterprise_contract = "terra1mg4gvn7svq7clshyn8qt6evwsv4yjrvfpfdjjpt29tmqdlcc700srphtm3";
        //
        // const component_contracts = await executor.query(enterprise_contract, {component_contracts: {}}) as ComponentContracts;
        //
        // const membership_contract = component_contracts.membership_contract;
        // const governance_controller = component_contracts.enterprise_governance_controller_contract;
        //
        // const token_config = await executor.query(membership_contract, {token_config: {}}) as TokenConfig;
        //
        // const token_contract = token_config.token_contract;

        const proposal_id = 1;

        // await stakeTokens(executor, token_contract, membership_contract);

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

        // await createProposal(executor, governance_controller, deployCrossChainTreasuryProposalAction);
        //
        // await castYesVote(executor, governance_controller, proposal_id);
        //
        // await executeProposal(executor, governance_controller, proposal_id)
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

const waitForNewBlock = async (): Promise<void> => new Promise((resolve) => setTimeout(resolve, 5000))

const deployEnterpriseFacade = async (refs: Refs, network: string, deployer: Deployer, signer: Signer): Promise<void> => {
    await deployer.storeCode(ENTERPRISE_FACADE_V1);
    await waitForNewBlock();
    await deployer.storeCode(ENTERPRISE_FACADE_V2);
    await waitForNewBlock();

    await deployer.storeCode(ENTERPRISE_FACADE);
    await waitForNewBlock();

    try {
        await deployer.instantiate(ENTERPRISE_FACADE_V1, {
                enterprise_versioning: refs.getAddress(network, ENTERPRISE_VERSIONING),
            },
            {
                admin: signer.key.accAddress,
                label: "Enterprise facade V1",
            });
        await waitForNewBlock();

        await deployer.instantiate(ENTERPRISE_FACADE_V2, {},
            {
                admin: signer.key.accAddress,
                label: "Enterprise facade V2",
            });

        refs.saveRefs();

        await deployer.instantiate(ENTERPRISE_FACADE, {
                enterprise_facade_v1: refs.getAddress(network, ENTERPRISE_FACADE_V1),
                enterprise_facade_v2: refs.getAddress(network, ENTERPRISE_FACADE_V2),
            },
            {
                admin: signer.key.accAddress,
                label: "Enterprise facade",
            });
        await waitForNewBlock();

    } catch (err) {
        console.log(err);
    }

    refs.saveRefs();
}

const deployIcs721CallbackProxy = async (refs: Refs, network: string, deployer: Deployer, signer: Signer): Promise<void> => {
    await deployer.storeCode(ICS721_CALLBACK_PROXY);
    await new Promise((resolve) => setTimeout(resolve, 5000));

    const instantiateMsg = {
        ics721_proxy: ICS721_PROXY_ADDR
    };

    try {
        await deployer.instantiate(ICS721_CALLBACK_PROXY, instantiateMsg, {
            admin: signer.key.accAddress,
            label: "ICS721 callback proxy",
        });
        await waitForNewBlock();
    } catch (err) {
        console.log(err);
    }

    refs.saveRefs();
}

const deployEnterpriseVersioning = async (refs: Refs, network: string, deployer: Deployer, signer: Signer): Promise<void> => {
    await deployer.storeCode(ENTERPRISE_VERSIONING);
    await new Promise((resolve) => setTimeout(resolve, 5000));

    const versioningInstantiateMsg = {
        admin: signer.key.accAddress,
    };

    try {
        await deployer.instantiate(ENTERPRISE_VERSIONING, versioningInstantiateMsg, {
            admin: signer.key.accAddress,
            label: "Enterprise versioning",
        });
        await waitForNewBlock();
    } catch (err) {
        console.log(err);
    }

    refs.saveRefs();
}

const deployEnterpriseFactory = async (refs: Refs, network: string, deployer: Deployer, signer: Signer): Promise<void> => {
    const enterpriseVersioning = refs.getAddress(network, ENTERPRISE_VERSIONING);
    const cw20CodeId = refs.getCodeId(network, CW20_BASE);
    const cw721CodeId = refs.getCodeId(network, CW721_METADATA_ONCHAIN);

    await deployer.storeCode(ENTERPRISE_FACTORY);
    await new Promise((resolve) => setTimeout(resolve, 5000));

    const factoryInstantiateMsg = {
        config: {
            admin: signer.key.accAddress,
            enterprise_versioning: enterpriseVersioning,
            cw20_code_id: parseInt(cw20CodeId),
            cw721_code_id: parseInt(cw721CodeId),
        },
    };

    console.log(JSON.stringify(factoryInstantiateMsg));

    try {
        await deployer.instantiate(ENTERPRISE_FACTORY, factoryInstantiateMsg, {
            admin: signer.key.accAddress,
        });
        await waitForNewBlock();
    } catch (err) {
        console.log(err);
    }

    refs.saveRefs();
}

const deployNewEnterpriseVersion = async (refs: Refs, network: string, deployer: Deployer, executor: Executor, major: number, minor: number, patch: number): Promise<void> => {
    await deployer.storeCode(ATTESTATION);
    await new Promise((resolve) => setTimeout(resolve, 5000));

    await deployer.storeCode(ENTERPRISE);
    await new Promise((resolve) => setTimeout(resolve, 5000));

    await deployer.storeCode(ENTERPRISE_GOVERNANCE);
    await new Promise((resolve) => setTimeout(resolve, 5000));

    await deployer.storeCode(ENTERPRISE_GOVERNANCE_CONTROLLER);
    await new Promise((resolve) => setTimeout(resolve, 5000));

    await deployer.storeCode(ENTERPRISE_TREASURY);
    await new Promise((resolve) => setTimeout(resolve, 5000));

    await deployer.storeCode(ENTERPRISE_OUTPOSTS);
    await new Promise((resolve) => setTimeout(resolve, 5000));

    await deployer.storeCode(FUNDS_DISTRIBUTOR);
    await new Promise((resolve) => setTimeout(resolve, 5000));

    await deployer.storeCode(TOKEN_STAKING_MEMBERSHIP);
    await new Promise((resolve) => setTimeout(resolve, 5000));

    await deployer.storeCode(DENOM_STAKING_MEMBERSHIP);
    await new Promise((resolve) => setTimeout(resolve, 5000));

    await deployer.storeCode(NFT_STAKING_MEMBERSHIP);
    await new Promise((resolve) => setTimeout(resolve, 5000));

    await deployer.storeCode(MULTISIG_MEMBERSHIP);
    await new Promise((resolve) => setTimeout(resolve, 5000));

    const enterpriseVersioningAddr = refs.getAddress(network, ENTERPRISE_VERSIONING);

    try {
        await executor.execute(enterpriseVersioningAddr, {
            add_version: {
                version: {
                    version: {
                        major: major,
                        minor: minor,
                        patch: patch,
                    },
                    changelog: [],
                    attestation_code_id: parseInt(refs.getCodeId(network, ATTESTATION)),
                    enterprise_code_id: parseInt(refs.getCodeId(network, ENTERPRISE)),
                    enterprise_governance_code_id: parseInt(refs.getCodeId(network, ENTERPRISE_GOVERNANCE)),
                    enterprise_governance_controller_code_id: parseInt(refs.getCodeId(network, ENTERPRISE_GOVERNANCE_CONTROLLER)),
                    enterprise_treasury_code_id: parseInt(refs.getCodeId(network, ENTERPRISE_TREASURY)),
                    enterprise_outposts_code_id: parseInt(refs.getCodeId(network, ENTERPRISE_OUTPOSTS)),
                    funds_distributor_code_id: parseInt(refs.getCodeId(network, FUNDS_DISTRIBUTOR)),
                    token_staking_membership_code_id: parseInt(refs.getCodeId(network, TOKEN_STAKING_MEMBERSHIP)),
                    denom_staking_membership_code_id: parseInt(refs.getCodeId(network, DENOM_STAKING_MEMBERSHIP)),
                    nft_staking_membership_code_id: parseInt(refs.getCodeId(network, NFT_STAKING_MEMBERSHIP)),
                    multisig_membership_code_id: parseInt(refs.getCodeId(network, MULTISIG_MEMBERSHIP)),
                }
            }
        })
    } catch (e) {
        console.log(e);
    }

    refs.saveRefs();
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