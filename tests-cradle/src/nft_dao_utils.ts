import {LCDClient} from "@terra-money/feather.js";
import {DaoContracts} from "./main";
import {Duration} from "./types/enterprise_facade";
import {
    queryAllOwnerTokens,
    queryAllTokens,
    queryOwnerOf,
    TokensResponse
} from "./cw721_utils";
import {isNotContract, isNotDaoContract} from "./dao_queries";

export interface NftConfigResponse {
    enterprise_contract: string,
    nft_contract: string,
    unlocking_period: Duration,
}

export const queryNftConfig = async (lcd: LCDClient, dao: DaoContracts): Promise<NftConfigResponse> => {
    return await lcd.wasm.contractQuery<NftConfigResponse>(dao.membership_contract, {
        nft_config: {}
    })
}

export const findRandomNftHolder = async (lcd: LCDClient, dao: DaoContracts, minimumBalance: number | undefined = undefined): Promise<string> => {
    const nftConfig = await queryNftConfig(lcd, dao);
    const nft_contract = nftConfig.nft_contract;

    const requiredBalance = minimumBalance ?? 1;

    let response: TokensResponse;
    let startAfter: string | null = null;
    do {
        response = await queryAllTokens(lcd, nft_contract, startAfter);

        for (const token of response.tokens) {
            const owner = await queryOwnerOf(lcd, nft_contract, token);
            if (isNotDaoContract(dao, owner) && await isNotContract(lcd, owner)) {

                const tokensOwned = (await queryAllOwnerTokens(lcd, nft_contract, owner)).length;
                if (tokensOwned >= requiredBalance) {
                    return owner
                }
            }
        }

        if (response.tokens.length > 0) {
            startAfter = response.tokens[response.tokens.length - 1]
        }
    } while (response.tokens.length > 0);

    throw new Error("No token holders with positive balance found");
}