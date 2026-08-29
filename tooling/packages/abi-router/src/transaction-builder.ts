import { NeoRpcProvider } from "@neo-devpack-solidity/types";
import { NeoDeploymentArtifacts } from "./types.js";
import Debug from "debug";

const debug = Debug("neo-devpack-solidity:transaction-builder");

/**
 * Transaction builder for Neo N3.
 *
 * Provides read-only contract invocation (invokeFunction) and prepares
 * transactions for signing and sending. For full transaction signing and
 * deployment, use @neo-devpack-solidity/hardhat-neo-deployer which has
 * a complete Neo N3 transaction pipeline (account management, script
 * building, witness construction, RPC broadcasting).
 */
export class TransactionBuilder {
  constructor(private readonly rpcProvider: NeoRpcProvider) {}

  /**
   * Read-only contract call via Neo N3 invokefunction RPC.
   * Does not require signing or gas.
   */
  async invokeFunction(
    scriptHash: string,
    method: string,
    args: any[] = []
  ): Promise<{ state: string; gasConsumed: string; stack: any[] }> {
    debug(`invokeFunction: ${method} on ${scriptHash}`);
    return this.rpcProvider.invokeFunction(scriptHash, method, this.encodeParams(args));
  }

  /**
   * Send a signed transaction to the Neo N3 network.
   *
   * @param signedTransaction - A hex-encoded signed transaction string.
   * @returns The transaction hash.
   */
  async sendRawTransaction(signedTransaction: string): Promise<{ hash: string }> {
    debug("sendRawTransaction");
    const result = await this.rpcProvider.sendRawTransaction(signedTransaction);
    return result;
  }

  /**
   * Send a transaction (Neo N3).
   *
   * Requires a pre-signed transaction, typically obtained via
   * @neo-devpack-solidity/hardhat-neo-deployer's transaction building + signing pipeline.
   *
   * @deprecated Use sendRawTransaction directly with a fully signed transaction.
   *   Full transaction building (script construction, fee estimation, signing, witness
   *   creation) requires private key management and is available in the
   *   @neo-devpack-solidity/hardhat-neo-deployer package.
   */
  async sendTransaction(
    contractAddress: string,
    methodName: string,
    args: any[] = [],
    options: {
      gasLimit?: string;
      gasPrice?: string;
      value?: string;
    } = {}
  ): Promise<{ hash: string }> {
    debug(`sendTransaction requested for ${methodName} on ${contractAddress}`);
    void args;
    void options;

    // Delegate to invokefunction for now.
    // Full transaction construction + signing requires private key management
    // and is provided by @neo-devpack-solidity/hardhat-neo-deployer.
    //
    // To send a transaction:
    // 1. Build the script with @cityofzion/neon-js
    // 2. Create and sign the transaction
    // 3. Call sendRawTransaction(signedTxHex)
    //
    // Example with neon-js:
    //   import { sc, tx, wallet, u } from "@cityofzion/neon-js";
    //   const script = sc.createScript({ scriptHash, operation: methodName, args });
    //   const transaction = new tx.Transaction();
    //   transaction.script = script;
    //   // ... set fees, signers, sign ...
    //   const signed = transaction.serialize(true);
    //   await txBuilder.sendRawTransaction(signed);
    throw new Error(
      "TransactionBuilder.sendTransaction requires a pre-signed transaction. " +
      "Use invokeFunction for read-only calls, or sign a Neo N3 transaction first " +
      "(e.g. with @cityofzion/neon-js) and pass it to sendRawTransaction."
    );
  }

  /**
   * Deploy a Neo N3 contract.
   *
   * Full deployment (NEF + manifest construction, fee estimation, signing,
   * RPC broadcasting) is implemented in @neo-devpack-solidity/hardhat-neo-deployer.
   *
   * This method requires NEF bytes and manifest JSON that the neo-solc compiler produces.
   */
  async deployContract(
    artifacts: NeoDeploymentArtifacts,
    constructorArgs: any[] = [],
    options: {
      gasLimit?: string;
      gasPrice?: string;
      value?: string;
    } = {}
  ): Promise<{ address: string; hash: string }> {
    debug("deployContract requested");
    void artifacts;
    void constructorArgs;
    void options;

    throw new Error(
      "TransactionBuilder.deployContract is not implemented. " +
      "Use @neo-devpack-solidity/hardhat-neo-deployer (Hardhat plugin) for contract deployment, " +
      "or use neo-solc + neo-cli / Neo-Express directly."
    );
  }

  /**
   * Encode parameters for Neo N3 invokefunction RPC
   */
  private encodeParams(args: any[]): Array<{ type: string; value: any }> {
    return args.map((arg) => this.encodeParam(arg));
  }

  /**
   * Encode a single parameter for Neo N3 invokefunction RPC
   */
  private encodeParam(arg: any): { type: string; value: any } {
    if (arg === null || arg === undefined) {
      return { type: "Any", value: null };
    }
    if (typeof arg === "boolean") {
      return { type: "Boolean", value: arg };
    }
    if (typeof arg === "number") {
      return { type: "Integer", value: arg.toString() };
    }
    if (typeof arg === "bigint") {
      return { type: "Integer", value: arg.toString() };
    }
    if (typeof arg === "string") {
      // Check for hex address
      if (arg.startsWith("0x") && arg.length === 42) {
        return { type: "Hash160", value: arg };
      }
      // Check for plain hex
      if (/^[0-9a-fA-F]{40}$/.test(arg)) {
        return { type: "Hash160", value: "0x" + arg };
      }
      return { type: "String", value: arg };
    }
    if (Array.isArray(arg)) {
      return {
        type: "Array",
        value: arg.map((item) => this.encodeParam(item)),
      };
    }
    if (typeof arg === "object") {
      return {
        type: "Map",
        value: Object.entries(arg).map(([key, val]) => ({
          key: this.encodeParam(key),
          value: this.encodeParam(val),
        })),
      };
    }
    return { type: "String", value: String(arg) };
  }
}
