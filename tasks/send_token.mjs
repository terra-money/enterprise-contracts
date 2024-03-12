import {encodePubkey, Registry, makeAuthInfoBytes} from "@cosmjs/proto-signing"
import {StargateClient} from "@cosmjs/stargate"
import {TxRaw} from "cosmjs-types/cosmos/tx/v1beta1/tx.js";

//------------------------------------------------
// Please fill in the information needed to send a transaction.
const sender = "terra1x5zsfdfxj6xg5pqm0999lagmccmrwk54495e9v";
const recipient = "terra16d6pzemu64ckee4l9t2ryrpjdgdrmxzd5jaajp";
const denom = "uluna";
const amount = "10000000";
// const amount = "1000000";
const endpoints = "https://cradle-manager.ec1-prod.newmetric.xyz/cradle/proxy/33373285-dcdc-4782-83fc-81938ef57d56";
//------------------------------------------------

sendToken(sender, recipient, denom, amount, endpoints);

// ---- implementation ----
async function sendToken(sender, recipient, denom, amount, rpcAddress) {
    const signingClient = await StargateClient.connect(rpcAddress);
    const registry = new Registry();

    const account = await signingClient.getAccount(sender);
    console.log(JSON.stringify(account.pubkey));

    const pubkey = encodePubkey({
        type: "tendermint/PubKeySecp256k1",
        value: account.pubkey.value,
    });

    const txBodyFields = {
        typeUrl: "/cosmos.tx.v1beta1.TxBody",
        value: {
            messages: [
                {
                    typeUrl: "/cosmos.bank.v1beta1.MsgSend",
                    value: {
                        fromAddress: sender,
                        toAddress: recipient,
                        amount: [{
                            amount,
                            denom
                        }],
                    },
                },
            ],
        },
    };

    const feeAmount = [
        {
            amount: "1",
            denom,
        },
    ];

    const txBodyBytes = registry.encode(txBodyFields);

    const gasLimit = 200000;
    const feeGranter = undefined;
    const feePayer = undefined;
    const authInfoBytes = makeAuthInfoBytes([{pubkey, sequence: 0}], feeAmount, gasLimit, feeGranter, feePayer);

    const txRaw = TxRaw.fromPartial({
        bodyBytes: txBodyBytes,
        authInfoBytes: authInfoBytes,
        signatures: [],
    });
    const txRawBytes = Uint8Array.from(TxRaw.encode(txRaw).finish());


    const result = await signingClient.broadcastTx(txRawBytes,);

    console.log(result)
}
