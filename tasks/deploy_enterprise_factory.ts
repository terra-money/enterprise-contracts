import task from "terrariums";

const ATTESTATION = "attestation";
const DENOM_STAKING_MEMBERSHIP = "denom-staking-membership";
const ENTERPRISE = "enterprise";
const ENTERPRISE_FACADE = "enterprise-facade";
const ENTERPRISE_FACTORY = "enterprise-factory";
const ENTERPRISE_GOVERNANCE = "enterprise-governance";
const ENTERPRISE_GOVERNANCE_CONTROLLER = "enterprise-governance-controller";
const ENTERPRISE_TREASURY = "enterprise-treasury";
const ENTERPRISE_VERSIONING = "enterprise-versioning";
const FUNDS_DISTRIBUTOR = "funds-distributor";
const MULTISIG_MEMBERSHIP = "multisig-membership";
const TOKEN_STAKING_MEMBERSHIP = "token-staking-membership";
const NFT_STAKING_MEMBERSHIP = "nft-staking-membership";

// assets
const DENOM_LUNA = "uluna";
const DENOM_AXL_USDC = "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4";
const DENOM_AXL_USDT = "ibc/CBF67A2BCF6CAE343FDF251E510C8E18C361FC02B23430C121116E0811835DEF";
const DENOM_AXL_WBTC = "ibc/05D299885B07905B6886F554B39346EA6761246076A1120B1950049B92B922DD";
const DENOM_AXL_WETH = "ibc/BC8A77AFBD872FDC32A348D3FB10CC09277C266CFE52081DE341C7EC6752E674";

task(async ({ network, deployer, executor, signer, refs }) => {
  // deployer.buildContract(ENTERPRISE);
  // deployer.optimizeContract(ENTERPRISE);

  // const enterpriseFacadeCodeId = await deployer.storeCode(ENTERPRISE_FACADE);
  // await new Promise((resolve) => setTimeout(resolve, 5000));
  //
  // try {
  //   await deployer.instantiate("enterprise-facade", {}, {
  //     admin: signer.key.accAddress,
  //     label: "Enterprise facade",
  //   });
  // } catch (err) {
  //   console.log(err);
  // }
  //
  // refs.saveRefs();

  // await deployer.storeCode(ENTERPRISE_VERSIONING);
  // await new Promise((resolve) => setTimeout(resolve, 5000));
  //
  // const versioningInstantiateMsg = {
  //   admin: signer.key.accAddress,
  // };
  //
  // try {
  //   await deployer.instantiate(ENTERPRISE_VERSIONING, versioningInstantiateMsg, {
  //     admin: signer.key.accAddress,
  //   });
  // } catch (err) {
  //   console.log(err);
  // }

  // const enterpriseVersioning = refs.getContract(network, ENTERPRISE_VERSIONING);
  // const cw20CodeId = refs.getContract(network, "cw20_base").codeId;
  // const cw721CodeId = refs.getContract(network, "cw721_base").codeId;
  //
  // await deployer.storeCode(ENTERPRISE_FACTORY);
  // await new Promise((resolve) => setTimeout(resolve, 5000));
  //
  // const factoryInstantiateMsg = {
  //   config: {
  //     enterprise_versioning: enterpriseVersioning.address,
  //     cw20_code_id: parseInt(cw20CodeId),
  //     cw721_code_id: parseInt(cw721CodeId),
  //   },
  // };
  //
  // console.log(JSON.stringify(factoryInstantiateMsg));
  //
  // try {
  //   await deployer.instantiate(ENTERPRISE_FACTORY, factoryInstantiateMsg, {
  //     admin: signer.key.accAddress,
  //   });
  // } catch (err) {
  //   console.log(err);
  // }

  const attestationCodeId = await deployer.storeCode(ATTESTATION);
  await new Promise((resolve) => setTimeout(resolve, 5000));

  const denomStakingMembershipCodeId = await deployer.storeCode(DENOM_STAKING_MEMBERSHIP);
  await new Promise((resolve) => setTimeout(resolve, 5000));

  let enterpriseCodeId;
  try {
    enterpriseCodeId = await deployer.storeCode(ENTERPRISE);
    await new Promise((resolve) => setTimeout(resolve, 5000));
  } catch (e) {
    console.log(e);
  }

  const enterpriseGovernanceCodeId = await deployer.storeCode(ENTERPRISE_GOVERNANCE);
  await new Promise((resolve) => setTimeout(resolve, 5000));

  const enterpriseGovernanceControllerCodeId = await deployer.storeCode(ENTERPRISE_GOVERNANCE_CONTROLLER);
  await new Promise((resolve) => setTimeout(resolve, 5000));

  const enterpriseTreasuryCodeId = await deployer.storeCode(ENTERPRISE_TREASURY);
  await new Promise((resolve) => setTimeout(resolve, 5000));

  const fundsDistributorCodeId = await deployer.storeCode(FUNDS_DISTRIBUTOR);
  await new Promise((resolve) => setTimeout(resolve, 5000));

  const multisigMembershipCodeId = await deployer.storeCode(MULTISIG_MEMBERSHIP);
  await new Promise((resolve) => setTimeout(resolve, 5000));

  const tokenStakingMembershipCodeId = await deployer.storeCode(TOKEN_STAKING_MEMBERSHIP);
  await new Promise((resolve) => setTimeout(resolve, 5000));

  const nftStakingMembershipCodeId = await deployer.storeCode(NFT_STAKING_MEMBERSHIP);
  await new Promise((resolve) => setTimeout(resolve, 5000));

  const enterpriseVersioningAddr = refs.getAddress(network, ENTERPRISE_VERSIONING);

  try {
    await executor.execute(enterpriseVersioningAddr, {
      add_version: {
        version: {
          version: {
            major: 1,
            minor: 0,
            patch: 0,
          },
          changelog: [],
          attestation_code_id: parseInt(attestationCodeId),
          enterprise_code_id: parseInt(enterpriseCodeId),
          enterprise_governance_code_id: parseInt(enterpriseGovernanceCodeId),
          enterprise_governance_controller_code_id: parseInt(enterpriseGovernanceControllerCodeId),
          enterprise_treasury_code_id: parseInt(enterpriseTreasuryCodeId),
          funds_distributor_code_id: parseInt(fundsDistributorCodeId),
          token_staking_membership_code_id: parseInt(tokenStakingMembershipCodeId),
          denom_staking_membership_code_id: parseInt(denomStakingMembershipCodeId),
          nft_staking_membership_code_id: parseInt(nftStakingMembershipCodeId),
          multisig_membership_code_id: parseInt(multisigMembershipCodeId),
        }
      }
    })
  } catch (e) {
    console.log(e);
  }

  refs.saveRefs();
});
