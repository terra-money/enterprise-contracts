import { Coin } from "@terra-money/terra.js";
import task, {Deployer, Executor, Refs} from "@terra-money/terrariums";
import {Signer} from "@terra-money/terrariums/lib/src/signers";

const WARP_CONTROLLER_ADDRESS = "terra196x8l07ue2mk8gtqxawz4z9dcwhztlg3st4t9we932fwx8rtcz3s9kawlp";
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
            "terra1lyqsev4fzxep0wuaxl9cdg2s3x78ampxh0vavt2a0qyy7ktxr74q2r8lng",
            "terra1qra6m7sjh8299vwxmgep5gdh9chr45xvcgvydje4e0448tfhrgls69wqlq",
            "terra1y83dnmx9lmacmckfu7ek2etqxtzkf8kd69gdnykfrkmcmc8q4lnqk48pqa",
            "terra13yp5fwd77daqm78xz97ad9phtldtx972a5zwcfggl9u4mf3szrzq9nwjqu",
            "terra1e30kcuznfj52v9t4e9m6hpeurr5hhfjmurp2jmn5lga9ggw9nrtsmraxw3"
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
        const facade_address = refs.getAddress(network, ENTERPRISE_FACADE);

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
                    condition: "{\"expr\":{\"string\":{\"left\":{\"ref\":\"$warp.variable.v2MigrationStage\"},\"right\":{\"simple\":\"migration_not_started\"},\"op\":\"neq\"}}}",
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
