import {MsgMigrateContract} from "@terra-money/terra.js";
import task, {info} from "@terra-money/terrariums";

const ENTERPRISE_VERSIONING = "enterprise-versioning";

task(async ({deployer, signer, refs, network}) => {
    // deployer.buildContract(ENTERPRISE);
    // deployer.optimizeContract(ENTERPRISE);
    //
    await deployer.storeCode(ENTERPRISE_VERSIONING);
    await new Promise((resolve) => setTimeout(resolve, 5000));

    const contract = refs.getContract(network, ENTERPRISE_VERSIONING);

    let msg = new MsgMigrateContract(
        signer.key.accAddress,
        contract.address!,
        parseInt(contract.codeId!),
        {},
    );

    try {
        let tx = await signer.createAndSignTx({
            msgs: [msg],
        });
        await signer.lcd.tx.broadcast(tx);
        info(`Migrated ${ENTERPRISE_VERSIONING} contract.`);
    } catch (e) {
        info(`Migrating ${ENTERPRISE_VERSIONING} contract has failed.`);
        info(JSON.stringify(e));
    }

    refs.saveRefs();
});
