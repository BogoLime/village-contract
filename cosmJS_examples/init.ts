import { Secp256k1HdWallet, StdFee } from "@cosmjs/amino";
import * as cosm from "@cosmjs/cosmwasm-stargate"


// Mnemonic from local chain, scaffolded with wasmd. Check: https://github.com/CosmWasm/wasmd

const mnemonic1 = 'mercy runway manual insect flip maple flip grit analyst person obey sand marriage genuine avocado neglect decade seed anchor innocent place roof armor leg'
const address = "wasm19zaf9fapfm6dk4vppg58qkv2ha0s4tzm2tjzv7";

async function initing(){
    let wallet = await Secp256k1HdWallet.fromMnemonic(mnemonic1,{prefix:"wasm"})
    const signingC = await cosm.SigningCosmWasmClient.connectWithSigner("http://localhost:26657/",wallet)


    const defaultFee: StdFee = { amount: [{amount: "200000", denom: "ucosm",},], gas: "200000",};

    const codeId= 14
    const instantiateMsg = {"storeName":"village-store-one","refundPeriodPolicy":50}

    const instantiateResponse = await signingC.instantiate(address, codeId, instantiateMsg, "My First Village Contract", defaultFee);

    console.log(instantiateResponse)
    for(let e of instantiateResponse.logs[0].events ){
        console.log("INIT EVENT",e)
    }

}

initing()