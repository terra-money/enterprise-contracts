import task, { Executor } from "terrariums";
import { LoremIpsum } from "lorem-ipsum";
import { Signer } from "terrariums/lib/src/signers";

const sleep = () => new Promise((resolve) => setTimeout(resolve, 1000));

const lorem = new LoremIpsum({
  sentencesPerParagraph: {
    max: 8,
    min: 4,
  },
  wordsPerSentence: {
    max: 16,
    min: 4,
  },
});

const createMultisigDaoMsg = (
  name: string,
  logo: string,
  members: string[],
  twitter?: string
) => {
  return {
    create_dao: {
      asset_whitelist: null,
      dao_membership: {
        new_membership: {
          new_multisig: {
            multisig_members: members.map((member) => {
              return { address: member, weight: "1" };
            }),
          },
        },
      },
      dao_metadata: {
        logo: { url: logo },
        name,
        socials: {
          ...(twitter ? { twitter_username: twitter } : {}),
        },
      },
      dao_gov_config: {
        quorum: "0.51",
        threshold: "0.5",
        unlocking_period: {
          height: 1,
        },
        vote_duration: 3600,
      },
    },
  };
};

const createTokenDaoMsg = (
  name: string,
  logo: string,
  tokenName: string,
  tokenSymbol: string,
  holders: string[],
  twitter?: string
) => {
  return {
    create_dao: {
      asset_whitelist: [
        {
          native: "uluna",
        },
      ],
      dao_membership: {
        new_membership: {
          new_token: {
            initial_token_balances: holders.map((h) => {
              return { address: h, amount: "1000000000" };
            }),
            token_decimals: 6,
            token_marketing: null,
            token_mint: null,
            token_name: tokenName,
            token_symbol: tokenSymbol,
          },
        },
      },
      dao_metadata: {
        logo: { url: logo },
        name,
        socials: {
          ...(twitter ? { twitter_username: twitter } : {}),
        },
      },
      dao_gov_config: {
        quorum: "0.51",
        threshold: "0.5",
        unlocking_period: {
          height: 1,
        },
        vote_duration: 3600,
      },
    },
  };
};

const createNftDaoMsg = (
  name: string,
  logo: string,
  nftName: string,
  nftSymbol: string,
  minter: string,
  twitter?: string
) => {
  return {
    create_dao: {
      asset_whitelist: [
        {
          native: "uluna",
        },
      ],
      dao_membership: {
        new_membership: {
          new_nft: {
            minter,
            nft_name: nftName,
            nft_symbol: nftSymbol,
          },
        },
      },
      dao_metadata: {
        logo: { url: logo },
        name,
        socials: {
          ...(twitter ? { twitter_username: twitter } : {}),
        },
      },
      dao_gov_config: {
        quorum: "0.51",
        threshold: "0.5",
        unlocking_period: {
          height: 1,
        },
        vote_duration: 3600,
      },
    },
  };
};

const createPollMsg = (title: string, description: string) => {
  return {
    create_proposal: {
      title,
      description,
      proposal_actions: [],
    },
  };
};

const createRandomTextProposals = async (
  executor: Executor,
  daoAddress: string,
  count: number
) => {
  for (let i = 0; i < count; i++) {
    console.log(`Creating poll ${i + 1} for ${daoAddress}`);

    const msg = createPollMsg(
      lorem.generateSentences(1),
      lorem.generateParagraphs(1)
    );

    try {
      await executor.execute(daoAddress, msg);
    } catch (e) {
      console.log(e);
    }
    await sleep();
  }
};

const DEV_WALLETS = [
  "terra1k529hl5nvrvavnzv4jm3um2lllxujrshpn5um2",
  "terra1w8c76dcepu5ekuduh0nqra3heachtyxxz5qqjs",
  "terra1th52jsckfhjhl84agtmmnf3wmetjxqqj7zveel",
];

const createDaos = (walletAddresses: string[]) => {
  return [
    // createMultisigDaoMsg(
    //   "Metaplex DAO",
    //   "https://app.realms.today/realms/metaplex/img/black-circle.png",
    //   walletAddresses,
    //   "@metaplex"
    // ),
    // createMultisigDaoMsg(
    //   "Serum",
    //   "https://assets.website-files.com/61284dcff241c2f0729af9f3/61285237ce2e301255d09108_logo-serum.png",
    //   walletAddresses
    // ),
    // createNftDaoMsg(
    //   "Friends and Family DAO",
    //   "https://app.realms.today/realms/FAFD/img/fafd_logo.png",
    //   lorem.generateWords(1),
    //   createRandomTicker(3),
    //   walletAddresses[0]
    // ),
    // createNftDaoMsg(
    //   "Grape",
    //   "https://app.realms.today/realms/Grape/img/grape.png",
    //   lorem.generateWords(1),
    //   createRandomTicker(3),
    //   walletAddresses[0],
    //   "@grapeprotocol"
    // ),
    // createTokenDaoMsg(
    //   "SOCEAN",
    //   "https://socean-git-enhancement-orca-price-feed-lieuzhenghong.vercel.app/static/media/socnRound.c466b499.png",
    //   lorem.generateWords(1),
    //   createRandomTicker(3),
    //   walletAddresses
    // ),
    // createTokenDaoMsg(
    //   "UXDProtocol",
    //   "https://app.realms.today/realms/UXP/img/UXP-Black.png",
    //   lorem.generateWords(1),
    //   createRandomTicker(3),
    //   walletAddresses,
    //   "@UXDProtocol"
    // ),
    // createTokenDaoMsg(
    //   "Media DAO",
    //   "https://media.network/images/token/media.svg",
    //   lorem.generateWords(1),
    //   createRandomTicker(3),
    //   walletAddresses,
    //   "@Media_FDN"
    // ),
    // createTokenDaoMsg(
    //   "GoblinGold",
    //   "https://app.realms.today/realms/GoblinGold/img/logo.png",
    //   lorem.generateWords(1),
    //   createRandomTicker(3),
    //   walletAddresses,
    //   "@goblingold_fi"
    // ),
    createNftDaoMsg(
      "Space Peanuts NFT",
      "https://app.realms.today/realms/Grape/img/grape.png",
      lorem.generateWords(1),
      createRandomTicker(3),
      walletAddresses[0]
    ),
  ];
};

const createRandomTicker = (length: number) => {
  const characters = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

  return [...Array(length)]
    .map(() => characters.charAt(Math.floor(Math.random() * characters.length)))
    .join("");
};

const mintRandomNFTs = async (
  executor: Executor,
  nftAddress: string,
  owners: string[]
) => {
  let tokenId = 1;

  for (let owner of owners) {
    const msg = {
      mint: {
        owner: owner,
        token_id: tokenId.toString(),
        token_uri: null,
        extension: null,
      },
    };

    try {
      await executor.execute(nftAddress, msg);
    } catch (e) {
      console.log(e);
    }

    await sleep();

    tokenId++;
  }
};

const createDaoData = async (
  executor: Executor,
  signer: Signer,
  address: string
) => {
  const walletAddresses = [signer.key.accAddress, ...DEV_WALLETS];

  const info = await signer.lcd.wasm.contractQuery<any>(address, {
    dao_info: {},
  });
  const count = Math.floor(Math.random() * 9) + 1;

  switch (info.dao_type) {
    case "token":
      await createRandomTextProposals(executor, address, count);
      break;

    case "nft":
      await mintRandomNFTs(
        executor,
        info.dao_membership_contract,
        walletAddresses
      );
      break;
  }
};

task(async ({ executor, signer, refs, network }) => {
  const walletAddresses = [signer.key.accAddress, ...DEV_WALLETS];

  for (let msg of createDaos(walletAddresses)) {
    console.log(`Creating "${msg.create_dao.dao_metadata.name}"`);
    try {
      await executor.execute("enterprise-factory", msg);
    } catch (e) {
      console.log(e);
    }

    await sleep();
  }

  const response = await signer.lcd.wasm.contractQuery<{
    daos: string[];
  }>(refs.getAddress(network, "enterprise-factory"), { all_daos: {} });

  for (let address of response.daos) {
    await createDaoData(executor, signer, address);
  }
});
