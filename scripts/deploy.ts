import * as fs from "fs";
import * as path from "path";
import yargs from "yargs/yargs";
import {
  createLCDClient,
  createWallet,
  storeCodeWithConfirm,
  instantiateWithConfirm,
} from "./helpers";

const argv = yargs(process.argv)
  .options({
    network: {
      type: "string",
      demandOption: true,
    },
    msg: {
      type: "string",
      demandOption: true,
    },
    admin: {
      type: "string",
      demandOption: true,
    },
    binary: {
      type: "string",
      demandOption: false,
      default: "../contract/artifacts/lockdrop_nft.wasm",
    },
    "code-id": {
      type: "number",
      demandOption: false,
    },
  })
  .parseSync();

(async function () {
  const terra = createLCDClient(argv["network"]);
  const deployer = createWallet(terra);
  const msg = JSON.parse(fs.readFileSync(path.resolve(argv["msg"]), "utf8"));

  let codeId = argv["code-id"];
  if (!codeId) {
    codeId = await storeCodeWithConfirm(deployer, path.resolve(argv["binary"]));
    console.log(`Code uploaded! codeId: ${codeId}`);
  }

  const result = await instantiateWithConfirm(deployer, argv["admin"], codeId, msg);
  const address = result.logs[0].eventsByType.instantiate_contract.contract_address[0];
  console.log(`Contract instantiated! address: ${address}`);
})();
