import {LCDClient, MsgExecuteContract} from "@terra-money/feather.js";
import {ExecuteCtx, executeTx} from "./txUtils";

export interface TokensResponse {
    tokens: string[]
}

export interface OwnerOfResponse {
    owner: string,
}

export const queryOwnerOf = async (lcd: LCDClient, cw721Contract: string, token: string): Promise<string> => {
    const response = await lcd.wasm.contractQuery<OwnerOfResponse>(cw721Contract, {
        owner_of: {
            token_id: token,
        }
    });

    return response.owner
}

export const queryAllTokens = async (lcd: LCDClient, cw721Contract: string, startAfter: string | null = null, limit: number | null = null): Promise<TokensResponse> => {
    return await lcd.wasm.contractQuery<TokensResponse>(cw721Contract, {
        all_tokens: {
            start_after: startAfter,
            limit: limit ?? 100,
        }
    });
}

export const queryAllOwnerTokens = async (lcd: LCDClient, cw721Contract: string, owner: string): Promise<string[]> => {
    let tokensOwned: string[] = [];

    let response: TokensResponse;
    let startAfter: string | null = null;
    do {
        response = await lcd.wasm.contractQuery<TokensResponse>(cw721Contract, {
            tokens: {
                owner,
                start_after: startAfter,
                limit: 100,
            }
        })

        tokensOwned = [...tokensOwned, ...response.tokens];

        if (response.tokens.length > 0) {
            startAfter = response.tokens[response.tokens.length - 1]
        }
    } while (response.tokens.length > 0);

    return tokensOwned
}

export const approveTokens = async (ctx: ExecuteCtx, cw721Contract: string, owner: string, tokens: string[], spender: string)=> {
    const msgs = tokens.map((token) => new MsgExecuteContract(
        owner,
        cw721Contract,
        {
            approve: {
                spender,
                token_id: token,
            }
        }
    ));

    await executeTx(ctx, msgs);
}