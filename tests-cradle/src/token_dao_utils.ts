import {LCDClient} from "@terra-money/feather.js";
import {DaoContracts} from "./main";
import {Duration} from "./types/enterprise_facade";
import {queryCw20Balance} from "./cw20_utils";
import {isNotContract, isNotDaoContract} from "./dao_queries";

export interface TokenConfigResponse {
    enterprise_contract: string,
    token_contract: string,
    unlocking_period: Duration,
}

export interface AllAccountsResponse {
    accounts: string[]
}

export const queryTokenConfig = async (lcd: LCDClient, dao: DaoContracts): Promise<TokenConfigResponse> => {
    return await lcd.wasm.contractQuery<TokenConfigResponse>(dao.membership_contract, {
        token_config: {}
    })
}

export const findRandomTokenHolder = async (lcd: LCDClient, dao: DaoContracts, minimumBalance: number | undefined = undefined): Promise<string> => {
    const tokenConfig = await queryTokenConfig(lcd, dao);

    const requiredBalance = minimumBalance ?? 1;

    const allAccounts = await lcd.wasm.contractQuery<AllAccountsResponse>(tokenConfig.token_contract, {
        all_accounts: {
            limit: 100,
        }
    });

    for (const account of allAccounts.accounts) {
        if (isNotDaoContract(dao, account) && await isNotContract(lcd, account)) {
            const balance = await queryCw20Balance(lcd, tokenConfig.token_contract, account);
            if (balance >= requiredBalance) {
                return account
            }
        }
    }

    throw new Error("No token holders with positive balance found");
}