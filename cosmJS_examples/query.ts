import { Secp256k1HdWallet, StdFee } from "@cosmjs/amino";
import * as cosm from "@cosmjs/cosmwasm-stargate"
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import * as msgs from "./msgs"

// Mnemonic from local chain, scaffolded with wasmd. Check: https://github.com/CosmWasm/wasmd


const mnemonic1 = 'mercy runway manual insect flip maple flip grit analyst person obey sand marriage genuine avocado neglect decade seed anchor innocent place roof armor leg'
const address = "wasm19zaf9fapfm6dk4vppg58qkv2ha0s4tzm2tjzv7";
const cntr_addr = "wasm1lxp2mwpskymsm4aptq2l3pxlcflqg0qer3e3pnhkuql20phejmdqtd9g50";

async function query(){
    const qclient = await cosm.CosmWasmClient.connect("http://localhost:26657/")
    let wallet = await Secp256k1HdWallet.fromMnemonic(mnemonic1,{prefix:"wasm"})
    const signingC = await cosm.SigningCosmWasmClient.connectWithSigner("http://localhost:26657/",wallet)

    const codeId= 4

    // const qRes = await signingC.queryContractSmart(cntr_addr,{query_listings:{limit:1}})
    const qRes = await signingC.queryContractSmart(cntr_addr,{query_store_info:{id:address}})
    console.log(qRes)
}

query()