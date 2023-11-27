import { Coin } from "@terra-money/terra.js";
import task, {Deployer, Executor, Refs} from "@terra-money/terrariums";
import {Signer} from "@terra-money/terrariums/lib/src/signers";

const WARP_CONTROLLER_ADDRESS = "terra1mg93d4g69tsf3x6sa9nkmkzc9wl38gdrygu0sewwcwj6l2a4089sdd7fgj";
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
            // "terra1sa47q8myqdpw3pcdwahef99nmp3rawvhs87tqmj92akrtjjl6vpqzd2s0k",
            // "terra1gxw5u64xud9y5dv8y3uk4x3cftf3a055v6tn5puksxq7aezcag0q5nwx30",
            // "terra1exj6fxvrg6xuukgx4l90ujg3vh6420540mdr6scrj62u2shk33sqnp0stl",
            // "terra1vm3v334jttp7ur7n6cqcq3w6xq78t49q5n4sw4a7jkddzhfuqntqpcgamt",
            // "terra1c88xa92l0rewxs27dv5r7j98kuzyz2er9vk699nqzlksy838tt6qq3xupw",
            // "terra17c6ts8grcfrgquhj3haclg44le8s7qkx6l2yx33acguxhpf000xqhnl3je",
            // "terra1ydkvywwnl3j84tcntcwjmzgjc5u2vrqpcyjzn3slvwcpjke6nzhstm5a0g",
            // "terra15ysxwg90y3yy3hrd3vyf6smf7lk9a7an8q0fryc48ssr3j7werdqr8n9zw",
            // "terra14zs05y3hc3ran6wyvuj8fr2kg3v6vpknzen6n9xdt9exf68w6sjq36975d",
            // "terra1mjhu6tnf8djhnnnntfzs3s58trh8qgp57g3ppx90xxrhh3u36x6qzej956",
            // "terra1v4sjvr7v59p33h3tvmu98acswvy4zhkq4fs0wyaf9v2kugj0nxuqssfpnn",
            // "terra1m8yppctc5x6u4hp4xnw7e9yyg9gc96agdx2t9umfrxq9t0p6qywq3j4yzs",
            // "terra1szv4tucduxym52q305v3x6dejyqsxhqt63vjy2rpjff8ap40jdkskkx3hy",
            // "terra1z2sgjtez2tuqrtdvz7g7yhxj8mc7x6v877fvt4940zqp3gszen6s9gxu2r",
            // "terra1290d4q6av48d3r8y99s4d5fqr5k75hn7l7ytz27pu3fqvg3f4jhsqr9vju",
            // "terra12agp7scuht4qdtpldyen0l4cxz5xe2q0hws9hk4acw5hkdr2dx6qc8cwu5",
            // "terra18kp32m7jfvcx43m8ymr87ut3cavcjx6s6y830cgmz2hlx505xjwshhw7v8",
            // "terra1ylmjvlayldxler4s9rhm6ycny2l62x3rgyfjrjs6n6p4sr98rj7q22w0ek",
            // "terra1x8wyy2tmvwn5nm23maxry80mkpxn65x2ghs0q3ktnk5y62wj5x7s5vsg79",
            // "terra1vmaggjxf4u3ft5upz3e9wuwu9msl34szuept4mpjwtnud4l65eaqvxyh5u",
            // "terra175v68wt6jtrp8p0xw0rm4cm0ygvcea40mtd3vywd6l28eral7stsgf84zn",
            // "terra1ragx7ypjt2haf9956z55phr3f2degljf7n5gq5las05u5uly94wsyejml0",
            // "terra16vl35edwt5c2904l7zlezv5kr6fwzjk78mc6wmf9rzutxxc7nfksymzuce",
            // "terra1yl6dkpe9tcnlnn6gxnqry8ny42fcmwwh4wv6fqe7vjl4qgveaqyqsdlnkp",
            // "terra1q2j9ezy43gu8sd5juh7ayhhwlevq437fyzl24fhw2wp2asvfhcgq6pa4vn",
            // "terra1qlcwa4k7zpx7ep2uh4cstv7gumjwk2lg0dtavx7h0x8sr0hprumqurjevx",
            // "terra1f43s2vecnmlany8q87e6qafj6mnu249k0yqzg477qsdhzsv39dhq5kxdsj",
            // "terra1ae2fzd44p6ler8csyfclhhav34kzz45pjd8ymy4ye3zjgc984cvqw5ystm",
            // "terra17z87eqstn3d3tt8pypl74wrdrwktt7rl9va7z88pln7ke5c0srnsmtaven",
            "terra18x6fq9m9r9c4c0ev6g6epv9jeqg5qgyls37j40nl40lr78zz5j6s8zgr68",
            "terra1ysfmzka4yacrjpdryxjslgwf2amhgvayp8use2x66h92px0lvgrqn4vhjq",
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

        const facade_query_msg_encoded = Buffer.from(`{"has_unmoved_stakes_or_claims":{"contract":"${dao_address}"}}`).toString('base64');

        const perform_migration_step_msg_encoded = Buffer.from(`{\"perform_next_migration_step\":{\"submsgs_limit\":${submsgs_limit}}}`).toString('base64');

        console.log("perform migration step msg encoded:", perform_migration_step_msg_encoded);

        const vars = `[{"query":{"reinitialize":false,"name":"hasUnmovedStakesOrClaims","init_fn":{"query":{"wasm":{"smart":{"contract_addr":"${facade_address}","msg":"${facade_query_msg_encoded}"}}},"selector":"$.has_unmoved_stakes_or_claims"},"update_fn":null,"kind":"bool","encode":false}}]`;

        console.log("vars:", vars);

        const msgs = `[{\"wasm\":{\"execute\":{\"contract_addr\":\"${dao_address}\",\"msg\":\"${perform_migration_step_msg_encoded}\",\"funds\":[]}}}]`;

        console.log("msgs:", msgs);

        await executor.execute(
            warp_controller_address,
            {
                create_job: {
                    name: `Migration for DAO ${dao_address}`,
                    description: "Performs next migration step for a DAO with migration in progress",
                    labels: [],
                    condition: "{\"expr\":{\"bool\":\"$warp.variable.hasUnmovedStakesOrClaims\"}}",
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
        // const facade_address = refs.getAddress(network, ENTERPRISE_FACADE);
        const facade_address = "terra1dzgr060p4hlc54ynu4z75fhky6rchr8xaskhslxr50tf0g5gj4gq7q4tva";

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
