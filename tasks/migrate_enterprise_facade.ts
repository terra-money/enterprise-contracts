import { MsgMigrateContract } from "@terra-money/terra.js";
import task, { info } from "@terra-money/terrariums";

const ENTERPRISE_FACADE = "enterprise-facade";
const ENTERPRISE_FACADE_V1 = "enterprise-facade-v1";
const ENTERPRISE_FACADE_V2 = "enterprise-facade-v2";

task(async ({ deployer, signer, refs, network }) => {
  // deployer.buildContract(ENTERPRISE_FACADE);
  // deployer.optimizeContract(ENTERPRISE_FACADE);

  await deployer.storeCode(ENTERPRISE_FACADE_V1);
  await waitForNewBlock();
  await deployer.storeCode(ENTERPRISE_FACADE_V2);
  await waitForNewBlock();

  await deployer.storeCode(ENTERPRISE_FACADE);
  await waitForNewBlock();

  const contract = refs.getContract(network, ENTERPRISE_FACADE);

  let msg = new MsgMigrateContract(
    signer.key.accAddress,
    contract.address!,
    parseInt(contract.codeId!),
    {
      enterprise_facade_v1: refs.getAddress(network, ENTERPRISE_FACADE_V1),
      enterprise_facade_v2: refs.getAddress(network, ENTERPRISE_FACADE_V2),
    }
  );

  console.log("enterpriseFacadeV1 code ID:", refs.getCodeId(network, ENTERPRISE_FACADE_V1));
  console.log("enterpriseFacadeV1 address:", refs.getAddress(network, ENTERPRISE_FACADE_V1));

  console.log("enterpriseFacadeV2 code ID:", refs.getCodeId(network, ENTERPRISE_FACADE_V2));
  console.log("enterpriseFacadeV2 address:", refs.getAddress(network, ENTERPRISE_FACADE_V2));

  try {
    let tx = await signer.createAndSignTx({
      msgs: [msg],
    });
    await signer.lcd.tx.broadcast(tx);
    info(`Migrated ${ENTERPRISE_FACADE} contract.`);
  } catch (e) {
    info(`Migrating ${ENTERPRISE_FACADE} contract has failed.`);
    info(JSON.stringify(e));
  }

  refs.saveRefs();
});

const waitForNewBlock = async (): Promise<void> => new Promise((resolve) => setTimeout(resolve, 5000))