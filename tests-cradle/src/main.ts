import {
    LCDClient,
    LCDClientConfig,
    MnemonicKey,
    Wallet
} from '@terra-money/feather.js';
import dotenv from 'dotenv';
import {env} from 'process';
import {ComponentContractsResponse} from "./types/enterprise";
import {ExecuteCtx} from "./txUtils";
import {verifyUpgradeCorrectness} from "./tests";

dotenv.config();

export type DaoContracts = ComponentContractsResponse & { enterprise: string };

// TODO: move this somewhere else
const LION_DAO_ENTERPRISE: string = "terra1htmjh7ka8andv354m5j3kleffqpkcam6qhjty7ryanw6xgwnz5zs0ckd9u";
const PIXELIONS_DAO_ENTERPRISE: string = "terra1ruv2e3l32pwfmn8l2chay9njymaleccx9frgnkys4g6q7mkap8vs363z9e";

const mainnetConfig: Record<string, LCDClientConfig> = {
    'phoenix-1': {
        chainID: env.CHAIN_ID,
        lcd: env.LCD_ENDPOINT,
        gasAdjustment: 1.75,
        gasPrices: {uluna: 0.15},
        prefix: 'terra',
    },
};

const lcd = new LCDClient(mainnetConfig);

const wallet = new Wallet(lcd, new MnemonicKey({mnemonic: env.MNEMONIC_KEY, coinType: 330}));

const ctx: ExecuteCtx = {
    lcd,
    wallet,
    chainId: env.CHAIN_ID,
}

const run = async () => {

    const newVersion = {
        major: 1,
        minor: 2,
        patch: 0,
    };

    try {
        for (const dao of [LION_DAO_ENTERPRISE, PIXELIONS_DAO_ENTERPRISE]) {
            await verifyUpgradeCorrectness(
                ctx,
                dao,
                newVersion,
                [],
                {
                    major: newVersion.major,
                    minor: newVersion.minor,
                    patch: newVersion.patch + 1,
                },
                {
                    major: newVersion.major,
                    minor: newVersion.minor,
                    patch: newVersion.patch + 2,
                },
            );
            console.log('\n\n');
        }
    } catch (e) {
        console.log(e);
        throw e;
    }
}

run();