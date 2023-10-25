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

  // await deployEnterpriseVersioning(refs, network, deployer, signer);

  // await deployEnterpriseFacade(refs, network, deployer, signer);

  // await deployEnterpriseFactory(refs, network, deployer, signer);

  await deployNewEnterpriseVersion(refs, network, deployer, executor, 2, 4, 0);

  // await instantiateDao(refs, network, executor);

  // try {
  //   await deployer.instantiate(MULTISIG_MEMBERSHIP, {
  //     enterprise_contract: "terra14zwkusypmm9pdhdlnqygmzu0d7mpmz7ml0aw3aw0m968wwfna97s540dh7",
  //     initial_weights: []
  //   });
  // } catch (e) {
  //   console.log(e);
  // }
});

const deployEnterpriseFacade = async (refs: Refs, network: string, deployer: Deployer, signer: Signer): Promise<void> => {
  await deployer.storeCode(ENTERPRISE_FACADE_V1);
  await new Promise((resolve) => setTimeout(resolve, 5000));

  await deployer.storeCode(ENTERPRISE_FACADE_V2);
  await new Promise((resolve) => setTimeout(resolve, 5000));

  await deployer.storeCode(ENTERPRISE_FACADE);
  await new Promise((resolve) => setTimeout(resolve, 5000));

  try {
    await deployer.instantiate(ENTERPRISE_FACADE_V1, {
          enterprise_versioning: refs.getAddress(network, ENTERPRISE_VERSIONING),
        },
        {
          admin: signer.key.accAddress,
          label: "Enterprise facade V1",
        });
    await new Promise((resolve) => setTimeout(resolve, 5000));

    await deployer.instantiate(ENTERPRISE_FACADE_V2,
        {},
        {
          admin: signer.key.accAddress,
          label: "Enterprise facade V2",
        });
    await new Promise((resolve) => setTimeout(resolve, 5000));

    await deployer.instantiate(ENTERPRISE_FACADE, {
          enterprise_facade_v1: refs.getAddress(network, ENTERPRISE_FACADE_V1),
          enterprise_facade_v2: refs.getAddress(network, ENTERPRISE_FACADE_V2),
        },
        {
          admin: signer.key.accAddress,
          label: "Enterprise facade",
        });
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
  } catch (err) {
    console.log(err);
  }

  refs.saveRefs();
}

const deployEnterpriseFactory = async (refs: Refs, network: string, deployer: Deployer, signer: Signer): Promise<void> => {
  const enterpriseVersioning = refs.getContract(network, ENTERPRISE_VERSIONING);
  const cw20CodeId = refs.getContract(network, "cw20_base").codeId;
  const cw721CodeId = refs.getContract(network, "cw721_base").codeId;

  await deployer.storeCode(ENTERPRISE_FACTORY);
  await new Promise((resolve) => setTimeout(resolve, 5000));

  const factoryInstantiateMsg = {
    config: {
      enterprise_versioning: enterpriseVersioning.address,
      cw20_code_id: parseInt(cw20CodeId),
      cw721_code_id: parseInt(cw721CodeId),
    },
  };

  console.log(JSON.stringify(factoryInstantiateMsg));

  try {
    await deployer.instantiate(ENTERPRISE_FACTORY, factoryInstantiateMsg, {
      admin: signer.key.accAddress,
    });
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

const instantiateDao = async(refs: Refs, network: string, executor: Executor): Promise<void> => {
  const enterpriseFactoryAddr = refs.getAddress(network, ENTERPRISE_FACTORY);

  console.log("enterprise factory addr", enterpriseFactoryAddr);

  try {
    await executor.execute(enterpriseFactoryAddr, {
      create_dao: {
        dao_metadata: TEST_DAO_METADATA,
        gov_config: TEST_GOV_CONFIG,
        // dao_council: TEST_DAO_COUNCIL,
        dao_membership: TEST_NEW_CW721_DAO_MEMBERSHIP,
        // asset_whitelist: [
        //   {native: DENOM_LUNA},
        // ],
        // nft_whitelist: [
        //   "terra1x5zsfdfxj6xg5pqm0999lagmccmrwk54495e9v"
        // ],
        // minimum_weight_for_rewards: "3",
        // attestation_text: "Attest that you're not a criminal",
      }
    })
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