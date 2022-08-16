import fs from "fs"
import * as cosm from "@cosmjs/cosmwasm-stargate";
import { Secp256k1HdWallet, StdFee } from "@cosmjs/amino";
import { GasPrice, calculateFee } from "@cosmjs/stargate";

// Mnemonic from local chain, scaffolded with wasmd. Check: https://github.com/CosmWasm/wasmd

const mnemonic1 = 'mercy runway manual insect flip maple flip grit analyst person obey sand marriage genuine avocado neglect decade seed anchor innocent place roof armor leg';

const contractPath = "/Users/bogomiltsvetkov/Desktop/Village/WASM_TEST/village-contracts/target/wasm32-unknown-unknown/release/village_contracts.wasm";
async function deploy(){
   const qclient = await cosm.CosmWasmClient.connect("http://localhost:26657/")
    let wallet = await Secp256k1HdWallet.fromMnemonic(mnemonic1,{prefix:"wasm"})
    let addr = (await wallet.getAccounts())[0].address;
    const signingC = await cosm.SigningCosmWasmClient.connectWithSigner("http://localhost:26657/",wallet)

    const wasm = fs.readFileSync(contractPath);
    const uploadFee = calculateFee(1000000000,"0.002ucosm");
    const result = await signingC.upload(addr, wasm, uploadFee);
    console.log(result);
}

deploy()