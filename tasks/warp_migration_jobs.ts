import { Coin } from "@terra-money/terra.js";
import task, {Deployer, Executor, Refs} from "@terra-money/terrariums";
import {Signer} from "@terra-money/terrariums/lib/src/signers";

const WARP_CONTROLLER_ADDRESS = "terra1pz9r2wtnkh72kyyzw82279nkjh92c9tmwhkc3gw5cgwpke5yw3gszxlnk7";
const ENTERPRISE_FACADE = "enterprise-facade";

task(async ({network, executor, refs }) => {
    // await createWarpAccount(executor, WARP_CONTROLLER_ADDRESS, 100_000_000);
    //
    // await createMigrationStepsOldWarpJob(refs, network, executor, WARP_CONTROLLER_ADDRESS, "terra1a9qnerqlhnkqummr9vyky6qmenvhqldy2gnvkdd97etsyt7amp6ss3r237", 20);
    //
    // await executeWarpJob(executor, 1);

    await createMigrationStepsOldWarpJobMultiple(
        refs,
        network,
        executor,
        WARP_CONTROLLER_ADDRESS,
        20,
        [
            "terra1jhglkunpl9kumrafjnq9xayhq249ardungkrf7gpj8dkdmnt6d9suuy00t"
        ]
    );
});

const createWarpAccount = async(executor: Executor, warp_controller_address: string, uluna_deposit: number): Promise<void> => {
    try {
        await executor.execute(
            warp_controller_address,
            {
                create_account: {}
            },
            {
                coins: [new Coin('uluna', uluna_deposit)],
            }
        )
    } catch (e) {
        console.log(e);
    }
}

const createMigrationStepsOldWarpJobMultiple = async (refs: Refs, network: string, executor: Executor, warp_controller_address: string, submsgs_limit: number | undefined, daos: string[]): Promise<void> => {
    for (const i in daos) {
        console.log("creating a job for DAO:", daos[i]);
        await createMigrationStepsOldWarpJob(refs, network, executor, warp_controller_address, daos[i], submsgs_limit);
    }
}

const createMigrationStepsOldWarpJob = async (refs: Refs, network: string, executor: Executor, warp_controller_address: string, dao_address: string, submsgs_limit: number | undefined): Promise<void> => {
    try {
        const facade_address = "terra1dqg237pzmqklsecd7azzap4vw3jeq58e96u32ch75qh9klrcrq2qsmdu4r";

        const facade_query_msg_encoded = Buffer.from(`{"v2_migration_stage":{"contract":"${dao_address}"}}`).toString('base64');

        const perform_migration_step_msg_encoded = Buffer.from(`{\"perform_next_migration_step\":{\"submsgs_limit\":${submsgs_limit}}}`).toString('base64');

        console.log("perform migration step msg encoded:", perform_migration_step_msg_encoded);

        const vars = `[{"query":{"reinitialize":false,"name":"v2MigrationStage","init_fn":{"query":{"wasm":{"smart":{"contract_addr":"${facade_address}","msg":"${facade_query_msg_encoded}"}}},"selector":"$.stage"},"update_fn":null,"kind":"string","encode":false}}]`;

        console.log("vars:", vars);

        const msgs = `[{\"wasm\":{\"execute\":{\"contract_addr\":\"${dao_address}\",\"msg\":\"${perform_migration_step_msg_encoded}\",\"funds\":[]}}}]`;

        console.log("msgs:", msgs);

        await executor.execute(
            warp_controller_address,
            {
                create_job: {
                    name: "Test migration",
                    description: "Migrates a 'stuck' migration of a DAO",
                    labels: [],
                    condition: "{\"expr\":{\"string\":{\"left\":{\"ref\":\"$warp.variable.v2MigrationStage\"},\"right\":{\"simple\":\"migration_in_progress\"},\"op\":\"eq\"}}}",
                    msgs: `[{\"wasm\":{\"execute\":{\"contract_addr\":\"${dao_address}\",\"msg\":\"${perform_migration_step_msg_encoded}\",\"funds\":[]}}}]`,
                    vars: vars,
                    recurring: true,
                    requeue_on_evict: false,
                    reward: "20000",
                }
            }
        );
    } catch (e) {
        console.log(e);
    }
}

const executeWarpJob = async (executor: Executor, id: number): Promise<void> => {
    try {
        await executor.execute(
            WARP_CONTROLLER_ADDRESS,
            {
                execute_job: {
                    id: id.toString()
                }
            },
        );
    } catch (e) {
        console.log(e);
    }
}

const createMigrationStepsWarpJob = async (refs: Refs, network: string, executor: Executor, dao_address: string, submsgs_limit: number | undefined): Promise<void> => {
    try {
        const facade_address = refs.getAddress(network, ENTERPRISE_FACADE);

        const facade_query_msg_encoded = Buffer.from(`{"v2_migration_stage":{"contract":"${dao_address}"}}`).toString('base64');

        const perform_migration_step_msg_encoded = Buffer.from(`{"perform_next_migration_step":{"submsgs_limit":${submsgs_limit}}`).toString('base64');

        const vars = `[{"query":{"reinitialize":false,"name":"v2MigrationStage","init_fn":{"query":{"wasm":{"smart":{"contract_addr":"${facade_address}","msg":"${facade_query_msg_encoded}"}}},"selector":"$.stage"},"update_fn":null,"kind":"string","encode":false}}]`;

        console.log("vars:", vars);

        await executor.execute(
            "terra1fqcfh8vpqsl7l5yjjtq5wwu6sv989txncq5fa756tv7lywqexraq5vnjvt",
            {
                create_job: {
                    name: "Test migration",
                    description: "Migrates a 'stuck' migration of a DAO",
                    labels: [],
                    executions: [
                        {
                            condition: "{\"expr\":{\"string\":{\"left\":{\"ref\":\"$warp.variable.v2MigrationStage\"},\"right\":{\"simple\":\"migration_in_progress\"},\"op\":\"eq\"}}}",
                            msgs: `[{\"wasm\":{\"execute\":{\"contract_addr\":\"${dao_address}\",\"msg\":\"${perform_migration_step_msg_encoded}\",\"funds\":[]}}}]`,
                        },
                    ],
                    terminate_condition: "{\"expr\":{\"string\":{\"left\":{\"ref\":\"$warp.variable.v2MigrationStage\"},\"right\":{\"simple\":\"migration_completed\"},\"op\":\"eq\"}}}",
                    vars: vars,
                    recurring: true,
                    requeue_on_evict: false,
                    reward: "20000",
                    duration_days: "730",
                }
            }
        );
    } catch (e) {
        console.log(e);
    }
}

const waitForNewBlock = async (): Promise<void> => new Promise((resolve) => setTimeout(resolve, 5000))
