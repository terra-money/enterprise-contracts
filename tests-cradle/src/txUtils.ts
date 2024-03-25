import {
    Coin,
    CreateTxOptions,
    isTxError,
    LCDClient,
    LCDClientConfig,
    Msg,
    MsgSend,
    TxLog, TxSuccess,
    Wallet
} from "@terra-money/feather.js";
import {WaitTxBroadcastResult} from "@terra-money/feather.js/dist/client/lcd/api/TxAPI";

export const LUNA_WHALE: string = "terra18vnrzlzm2c4xfsx382pj2xndqtt00rvhu24sqe";

export interface ExecuteCtx {
    lcd: LCDClient,
    wallet: Wallet,
    chainId: string,
}

export const findAttribute = (logs: TxLog[], key: string): string | undefined => {
    for (const log of logs) {
        for (const event of log.events) {
            for (const attr of event.attributes) {
                if (attr.key === key) {
                    return attr.value
                }
            }
        }
    }
    return undefined
}

export const executeTx = async (ctx: ExecuteCtx, msgs: Msg[]): Promise<TxSuccess> => {
    const txOptions: CreateTxOptions = {
        msgs: msgs,
        chainID: ctx.chainId,
    };

    const tx = await ctx.wallet.createTx(txOptions);

    let result = await ctx.wallet.lcd.tx.broadcast(tx, ctx.chainId);

    if (isTxError(result)) {
        console.log(result);
        throw new Error(`Transaction error, code: ${result.code}, codespace: ${result.codespace}`);
    }

    return result as TxSuccess
}

export const sendCoinsMsg = (ctx: ExecuteCtx, from: string, to: string, denom: string, amount: number): MsgSend => {
    return new MsgSend(
        from,
        to,
        [new Coin(denom, amount.toString())]
    );
}

export const fundAddressWithLuna = async (ctx: ExecuteCtx, address: string) => {
    const amount = 100_000_000;

    await executeTx(ctx, [sendCoinsMsg(ctx, LUNA_WHALE, address, 'uluna', amount)]);
}

export const fundAddressWithLunaMsg = (ctx: ExecuteCtx, address: string): Msg => {
    const amount = 100_000_000;

    return sendCoinsMsg(ctx, LUNA_WHALE, address, 'uluna', amount);
}