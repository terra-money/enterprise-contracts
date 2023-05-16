import { MsgMigrateContract } from "@terra-money/terra.js";
import task, { info } from "terrariums";

const ENTERPRISE = "enterprise";
const ENTERPRISE_GOVERNANCE = "enterprise-governance";
const ENTERPRISE_FACTORY = "enterprise-factory";
const FUNDS_DISTRIBUTOR = "funds-distributor";
const TOKEN_STAKING = "token-staking";
const NFT_STAKING = "nft-staking";

task(async ({ deployer, signer, refs, network }) => {
  deployer.buildContract(ENTERPRISE);
  deployer.optimizeContract(ENTERPRISE);

  const enterpriseCodeId = await deployer.storeCode(ENTERPRISE);
  await new Promise((resolve) => setTimeout(resolve, 3000));

  const enterpriseGovernanceCodeId = await deployer.storeCode(ENTERPRISE_GOVERNANCE);
  await new Promise((resolve) => setTimeout(resolve, 3000));

  const fundsDistributorCodeId = await deployer.storeCode(FUNDS_DISTRIBUTOR);
  await new Promise((resolve) => setTimeout(resolve, 3000));

  const tokenStakingCodeId = await deployer.storeCode(TOKEN_STAKING);
  await new Promise((resolve) => setTimeout(resolve, 3000));

  const nftStakingCodeId = await deployer.storeCode(NFT_STAKING);
  await new Promise((resolve) => setTimeout(resolve, 3000));

  await deployer.storeCode(ENTERPRISE_FACTORY);
  await new Promise((resolve) => setTimeout(resolve, 3000));

  const contract = refs.getContract(network, ENTERPRISE_FACTORY);

  let msg = new MsgMigrateContract(
    signer.key.accAddress,
    contract.address!,
    parseInt(contract.codeId!),
    {
      new_enterprise_code_id: parseInt(enterpriseCodeId),
      new_enterprise_governance_code_id: parseInt(enterpriseGovernanceCodeId),
      new_funds_distributor_code_id: parseInt(fundsDistributorCodeId),
      new_token_staking_code_id: parseInt(tokenStakingCodeId),
      new_nft_staking_code_id: parseInt(nftStakingCodeId),
    }
  );

  console.log("enterpriseFactoryCodeId", contract.codeId);
  console.log("enterpriseCodeId", enterpriseCodeId);
  console.log("enterpriseGovernanceCodeId", enterpriseGovernanceCodeId);
  console.log("tokenStakingCodeId", tokenStakingCodeId);
  console.log("nftStakingCodeId", nftStakingCodeId);

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
