import * as fs from "fs";
import * as path from "path";
import yargs from "yargs/yargs";
import { Wallet, Msg, MsgExecuteContract } from "@terra-money/terra.js";
import { createLCDClient, createWallet, sendTxWithConfirm } from "./helpers";

const MAX_OWNERS_PER_MSG = 50;
const MAX_MSGS_PER_TX = 10;
const DEFAULT_GAS = (10_000_000).toString();

// https://stackoverflow.com/questions/2450954/how-to-randomize-shuffle-a-javascript-array
function shuffle(array: string[]) {
  for (let i = array.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [array[i], array[j]] = [array[j], array[i]];
  }
}

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

  // We shuffle the list of owners so that the `token_id`s they get are randomized
  shuffle(owners);
  fs.writeFileSync(`.level_${argv["level"]}_owners_shuffled.json`, JSON.stringify(owners, null, 2));

  const msgs = createMintMessages(minter, argv["contract-address"], argv["level"], owners);
  console.log(`Created ${msgs.length} messages`);

  const batches = createMsgBatches(msgs);
  console.log(`Created ${batches.length} batches`);

  const sequence = await minter.sequence();
  console.log(`Queried minter sequence: ${sequence}`);

  for (let i = 0; i < batches.length; i++) {
    process.stdout.write(`Broadcasting tx ${i + 1}/${batches.length}... `);
    try {
      const { txhash } = await sendTxWithConfirm(minter, batches[i], sequence + i, DEFAULT_GAS);
      console.log(`Success! Txhash: ${txhash}`);
    } catch (err) {
      console.log("Tx result or unable to confirm result!");
    }
  }
})();
