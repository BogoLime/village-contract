import { Secp256k1HdWallet, StdFee } from "@cosmjs/amino";
import * as cosm from "@cosmjs/cosmwasm-stargate"
import * as msgs from "./msgs"

// Mnemonic from local chain, scaffolded with wasmd. Check: https://github.com/CosmWasm/wasmd

const mnemonic1 = 'mercy runway manual insect flip maple flip grit analyst person obey sand marriage genuine avocado neglect decade seed anchor innocent place roof armor leg'
const address = "wasm19zaf9fapfm6dk4vppg58qkv2ha0s4tzm2tjzv7";
const mnemonic2 = "knock real play denial muffin slight shaft obscure vacuum wedding truth deny tissue input tilt unique sure under need junior critic cost soap ethics";
const address2 = "wasm1ya7p6e53qr20pwl8x334csegp65zjpu6shuct7"
const cntr_addr = "wasm1lxp2mwpskymsm4aptq2l3pxlcflqg0qer3e3pnhkuql20phejmdqtd9g50";

async function interacting(){
    let wallet = await Secp256k1HdWallet.fromMnemonic(mnemonic2,{prefix:"wasm"})
    const signingC = await cosm.SigningCosmWasmClient.connectWithSigner("http://localhost:26657/",wallet)


    const defaultFee: StdFee = { amount: [{amount: "200000", denom: "ucosm",},], gas: "200000",};

    const token = {amount:"160", denom:"ucosm"}
    
    // const executeResponse = await signingC.execute(address, cntr_addr, msgs.buy, defaultFee,"", [token]);
    const executeResponse = await signingC.execute(address2, cntr_addr, msgs.rateStore, defaultFee);

    console.log(executeResponse)
    for(let e of executeResponse.logs[0].events ){
        console.log("INIT EVENT",e)
    }

}

interacting()