import { MsgMigrateContract } from "@terra-money/terra.js";
import task, { info } from "terrariums";

const ENTERPRISE = "enterprise";
const ENTERPRISE_FACTORY = "enterprise-factory";

task(async ({ deployer, signer, refs, network }) => {
  deployer.buildContract(ENTERPRISE);
  deployer.optimizeContract(ENTERPRISE);

  const enterpriseCodeId = await deployer.storeCode(ENTERPRISE);
  await new Promise((resolve) => setTimeout(resolve, 3000));

  deployer.buildContract(ENTERPRISE_FACTORY);
  deployer.optimizeContract(ENTERPRISE_FACTORY);

  await deployer.storeCode(ENTERPRISE_FACTORY);
  await new Promise((resolve) => setTimeout(resolve, 3000));

  const contract = refs.getContract(network, ENTERPRISE_FACTORY);

  let msg = new MsgMigrateContract(
    signer.key.accAddress,
    contract.address!,
    parseInt(contract.codeId!),
    {
      new_enterprise_code_id: parseInt(enterpriseCodeId),
    }
  );

  console.log("enterpriseFactoryCodeId", contract.codeId);
  console.log("enterpriseCodeId", enterpriseCodeId);

  try {
    let tx = await signer.createAndSignTx({
      msgs: [msg],
    });
    await signer.lcd.tx.broadcast(tx);
    info(`Migrated ${ENTERPRISE_FACTORY} contract.`);
  } catch (e) {
    info(`Migrating ${ENTERPRISE_FACTORY} contract has failed.`);
    info(JSON.stringify(e));
  }

  refs.saveRefs();
});
