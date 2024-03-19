import {LCDClient} from "@terra-money/feather.js";

interface BalanceResponse {
    balance: string,
}

export const queryCw20Balance = async (lcd: LCDClient, cw20Contract: string, address: string): Promise<number> => {
    const response = await lcd.wasm.contractQuery<BalanceResponse>(cw20Contract, {
        balance: {
            address: address,
        }
    });

    return parseInt(response.balance, 10)
}