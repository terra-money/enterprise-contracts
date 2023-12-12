import { MsgMigrateContract } from "@terra-money/terra.js";
import task, { info } from "@terra-money/terrariums";

const ENTERPRISE_FACTORY = "enterprise-factory";
const ENTERPRISE_VERSIONING = "enterprise-versioning";
const CW721_METADATA_ONCHAIN = "cw721_metadata_onchain";

task(async ({ deployer, signer, refs, network }) => {
  // deployer.buildContract(ENTERPRISE);
  // deployer.optimizeContract(ENTERPRISE);
  //
  // await deployer.storeCode(ENTERPRISE_FACTORY);
  // await new Promise((resolve) => setTimeout(resolve, 5000));

  const contract = refs.getContract(network, ENTERPRISE_FACTORY);

  let msg = new MsgMigrateContract(
    signer.key.accAddress,
    contract.address!,
    parseInt(contract.codeId!),
    {
      admin: signer.key.accAddress,
      enterprise_versioning_addr: refs.getAddress(network, ENTERPRISE_VERSIONING),
      cw721_code_id: parseInt(refs.getCodeId(network, CW721_METADATA_ONCHAIN)),
    }
  );

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
