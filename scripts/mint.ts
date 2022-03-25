import * as fs from "fs";
import * as path from "path";
import yargs from "yargs/yargs";
import { Wallet, Msg, MsgExecuteContract } from "@terra-money/terra.js";
import { createLCDClient, createWallet, sendTransaction } from "./helpers";

const MAX_OWNERS_PER_MSG = 50;
const MAX_MSGS_PER_TX = 10;

function createMintMessages(
  minter: Wallet,
  contractAddress: string,
  level: number,
  owners: string[]
) {
  const msgs: MsgExecuteContract[] = [];
  let start = 0;
  while (start < owners.length) {
    const end = Math.min(start + MAX_OWNERS_PER_MSG, owners.length);
    msgs.push(
      new MsgExecuteContract(minter.key.accAddress, contractAddress, {
        mint: {
          level,
          owners: owners.slice(start, end),
        },
      })
    );
    start = end;
  }
  return msgs;
}

function createMsgBatches(msgs: Msg[]) {
  const batches: Msg[][] = [];
  let start = 0;
  while (start < msgs.length) {
    const end = Math.min(start + MAX_MSGS_PER_TX, msgs.length);
    batches.push(msgs.slice(start, end));
    start = end;
  }
  return batches;
}

const argv = yargs(process.argv)
  .options({
    network: {
      type: "string",
      demandOption: true,
    },
    "contract-address": {
      type: "string",
      demandOption: true,
    },
    level: {
      type: "number",
      demandOption: true,
    },
  })
  .parseSync();

(async function () {
  const terra = createLCDClient(argv["network"]);
  const minter = createWallet(terra);

  const filePath = path.resolve(`../data/level_${argv["level"]}_owners.txt`);
  const owners = fs.readFileSync(filePath, "utf8").split("\n");

  const msgs = createMintMessages(minter, argv["contract-address"], argv["level"], owners);
  console.log(`Created ${msgs.length} messages`);

  const batches = createMsgBatches(msgs);
  console.log(`Created ${batches.length} batches`);

  for (let i = 0; i < batches.length; i++) {
    process.stdout.write(`Broadcasting tx ${i + 1}/${batches.length}... `);
    const { txhash } = await sendTransaction(minter, batches[i]);
    console.log(`Success! Txhash: ${txhash}`);
  }
})();
