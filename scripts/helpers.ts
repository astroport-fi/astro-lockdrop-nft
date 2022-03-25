import * as fs from "fs";
import * as promptly from "promptly";
import dotenv from "dotenv";
import {
  isTxError,
  LCDClient,
  LocalTerra,
  MnemonicKey,
  Msg,
  MsgInstantiateContract,
  MsgStoreCode,
  Wallet,
} from "@terra-money/terra.js";

const DEFAULT_GAS_SETTINGS = {
  gasPrices: "0.15uusd",
  gasAdjustment: 1.4,
};

/**
 * @notice Create an `LCDClient` instance based on provided network identifier
 */
export function createLCDClient(network: string): LCDClient {
  if (network === "mainnet") {
    return new LCDClient({
      chainID: "columbus-5",
      URL: "https://lcd.terra.dev",
    });
  } else if (network === "testnet") {
    return new LCDClient({
      chainID: "bombay-12",
      URL: "https://bombay-lcd.terra.dev",
    });
  } else if (network === "localterra") {
    return new LocalTerra();
  } else {
    throw new Error(`invalid network: ${network}, must be mainnet|testnet|localterra`);
  }
}

/**
 * @notice Create a `Wallet` instance by loading the mnemonic phrase stored in `.env`
 */
export function createWallet(terra: LCDClient): Wallet {
  dotenv.config();
  if (!process.env.MNEMONIC) {
    throw new Error("mnemonic not provided");
  }
  return terra.wallet(
    new MnemonicKey({
      mnemonic: process.env.MNEMONIC,
    })
  );
}

/**
 * @notice Send a transaction. Return result if successful, throw error if failed
 *
 * Use uusd for gas payment and mainnet gas prices for default. We could customize it to make the
 * function more flexible, but I'm too lazy for that
 */
export async function sendTransaction(signer: Wallet, msgs: Msg[]) {
  const tx = await signer.createAndSignTx({ msgs, ...DEFAULT_GAS_SETTINGS });
  const result = await signer.lcd.tx.broadcast(tx);
  if (isTxError(result)) {
    throw new Error("tx failed! raw log: " + result.raw_log);
  }
  return result;
}

/**
 * @notice Same with `sendTransaction`, but requires confirmation for CLI before broadcasting
 */
export async function sendTxWithConfirm(signer: Wallet, msgs: Msg[]) {
  const tx = await signer.createAndSignTx({ msgs, ...DEFAULT_GAS_SETTINGS });
  console.log("\n" + JSON.stringify(tx).replace(/\\/g, "") + "\n");

  const proceed = await promptly.confirm("Confirm transaction before broadcasting [y/N]:");
  if (!proceed) {
    console.log("User aborted!");
    process.exit(1);
  }

  const result = await signer.lcd.tx.broadcast(tx);
  if (isTxError(result)) {
    throw new Error(`tx failed! raw log: ${result.raw_log}`);
  }
  return result;
}

/**
 * @notice Same with `storeCode`, but requires confirmation for CLI before broadcasting
 */
export async function storeCodeWithConfirm(signer: Wallet, filePath: string) {
  const code = fs.readFileSync(filePath).toString("base64");
  const result = await sendTxWithConfirm(signer, [new MsgStoreCode(signer.key.accAddress, code)]);
  return parseInt(result.logs[0].eventsByType.store_code.code_id[0]);
}

/**
 * @notice Same with `instantiateContract`, but requires confirmation for CLI before broadcasting
 */
export async function instantiateWithConfirm(signer: Wallet, codeId: number, initMsg: object) {
  const result = await sendTxWithConfirm(signer, [
    new MsgInstantiateContract(signer.key.accAddress, signer.key.accAddress, codeId, initMsg),
  ]);
  return result;
}
