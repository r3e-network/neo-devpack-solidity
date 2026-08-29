import axios from "axios";
import chalk from "chalk";
import { createHash, randomBytes } from "crypto";
import Debug from "debug";
import type { rpcTypes } from "@neo-devpack-solidity/types";

type NeoRpcProvider = rpcTypes.NeoRpcProvider;
type InvokeResult = rpcTypes.InvokeResult;
type Balance = rpcTypes.Balance;

const debug = Debug("neo-foundry:cast");

export class NeoCast {
  private rpcProvider?: NeoRpcProvider;

  constructor(_configPath?: string, rpcUrl?: string) {
    if (rpcUrl) {
      this.setRpc(rpcUrl);
    }
  }

  setRpc(rpcUrl: string): void {
    this.rpcProvider = this.createRpcProvider(rpcUrl);
  }

  async call(
    contractAddress: string,
    method: string,
    args: unknown[] = [],
    _options: {
      blockTag?: string | number;
      from?: string;
    } = {}
  ): Promise<unknown> {
    const rpc = this.ensureRpc();

    console.log(
      chalk.blue(`📞 Calling ${contractAddress}.${method}(${args.map(String).join(", ")})...`)
    );

    const scriptHash = this.addressToScriptHash(contractAddress);
    const result = await rpc.invokeFunction(scriptHash, method, this.encodeArgs(args));

    if (result.state === "FAULT") {
      throw new Error(`Contract call failed: ${result.exception ?? "Unknown error"}`);
    }

    console.log(chalk.green("✅ Call successful"));
    console.log("Result:", this.formatStackResult(result.stack));
    return result.stack;
  }

  async send(
    contractAddress: string,
    method: string,
    args: unknown[] = [],
    options: {
      from?: string;
      gasLimit?: string;
      gasPrice?: string;
      value?: string;
    } = {}
  ): Promise<string> {
    void contractAddress;
    void method;
    void args;
    void options;

    throw new Error(
      "neo-cast send is not implemented yet (requires transaction building + signing)."
    );
  }

  async estimateGas(
    contractAddress: string,
    method: string,
    args: unknown[] = [],
    _options: {
      from?: string;
      value?: string;
    } = {}
  ): Promise<void> {
    const rpc = this.ensureRpc();

    console.log(
      chalk.blue(`⛽ Estimating gas for ${contractAddress}.${method}(${args.map(String).join(", ")})...`)
    );

    const scriptHash = this.addressToScriptHash(contractAddress);
    const result = await rpc.invokeFunction(scriptHash, method, this.encodeArgs(args));

    console.log(chalk.green("Gas estimation:"));
    console.log(`  Gas Consumed: ${result.gasConsumed}`);
    console.log(`  VM State: ${result.state}`);
  }

  async deployContract(
    bytecode: string,
    constructorArgs: unknown[] = [],
    options: {
      from?: string;
      gasLimit?: string;
      gasPrice?: string;
      value?: string;
    } = {}
  ): Promise<string> {
    void bytecode;
    void constructorArgs;
    void options;

    throw new Error(
      "neo-cast deploy is not implemented yet (requires NEF+manifest deployment and signing)."
    );
  }

  async balance(address: string): Promise<void> {
    const rpc = this.ensureRpc();
    console.log(chalk.blue(`💰 Getting balance for ${address}...`));

    const balances = await rpc.getBalance(address);

    if (balances.length === 0) {
      console.log(chalk.yellow("No tokens found"));
      return;
    }

    console.log(chalk.green("Balances:"));
    for (const balance of balances) {
      const amount = Number(balance.amount) / Math.pow(10, balance.decimals);
      console.log(`  ${amount.toFixed(balance.decimals)} ${balance.symbol}`);
    }
  }

  async transaction(txHash: string): Promise<void> {
    const rpc = this.ensureRpc();
    console.log(chalk.blue(`🔍 Getting transaction ${txHash}...`));

    const tx = await rpc.getTransaction(txHash);
    console.log(chalk.green("Transaction details:"));
    console.log(`  Hash: ${tx.hash}`);
    console.log(`  Sender: ${tx.sender}`);
    console.log(`  System Fee: ${tx.sysFee} GAS`);
    console.log(`  Network Fee: ${tx.netFee} GAS`);
    console.log(`  Valid Until Block: ${tx.validUntilBlock}`);
  }

  async block(blockHashOrNumber: string | number): Promise<void> {
    const rpc = this.ensureRpc();
    console.log(chalk.blue(`🧱 Getting block ${blockHashOrNumber}...`));

    const block = await rpc.getBlock(blockHashOrNumber);
    console.log(chalk.green("Block details:"));
    console.log(`  Hash: ${block.hash}`);
    console.log(`  Index: ${block.index}`);
    console.log(`  Time: ${new Date(block.time * 1000).toISOString()}`);
    console.log(`  Transactions: ${block.tx.length}`);
  }

  async storage(contractAddress: string, key: string): Promise<void> {
    const rpc = this.ensureRpc();
    console.log(chalk.blue(`🗄️ Getting storage ${key} from ${contractAddress}...`));

    const scriptHash = this.addressToScriptHash(contractAddress);
    const value = await rpc.getStorage(scriptHash, key);

    if (value == null) {
      console.log(chalk.yellow("Storage key not found"));
      return;
    }

    console.log(chalk.green("Storage value:"));
    console.log(`  Key: ${key}`);
    console.log(`  Value: ${value}`);
    console.log(`  Decoded: ${this.tryDecodeStorage(value)}`);
  }

  async networkInfo(): Promise<void> {
    const rpc = this.ensureRpc();
    console.log(chalk.blue("🌐 Getting network information..."));

    const [version, blockCount, peers] = await Promise.all([
      rpc.call<any>("getversion"),
      rpc.getBlockCount(),
      rpc.call<any>("getpeers").catch(() => ({ connected: [] })),
    ]);

    console.log(chalk.green("Network information:"));
    console.log(`  Version: ${version?.useragent ?? "Unknown"}`);
    console.log(`  Block Height: ${blockCount}`);
    console.log(`  Connected Peers: ${peers?.connected?.length ?? 0}`);
    console.log(`  Network Magic: ${version?.protocol?.network ?? "Unknown"}`);
  }

  async convert(value: string, from: string, to: string): Promise<void> {
    console.log(chalk.blue(`🔄 Converting ${value} from ${from} to ${to}...`));

    let result: string;

    if (from === "hex" && to === "decimal") {
      result = parseInt(value, 16).toString();
    } else if (from === "decimal" && to === "hex") {
      result = "0x" + parseInt(value, 10).toString(16);
    } else if (from === "hex" && to === "ascii") {
      result = Buffer.from(value.replace("0x", ""), "hex").toString("ascii");
    } else if (from === "ascii" && to === "hex") {
      result = "0x" + Buffer.from(value, "ascii").toString("hex");
    } else {
      throw new Error(`Unsupported conversion: ${from} to ${to}`);
    }

    console.log(chalk.green("Conversion result:"));
    console.log(`  ${from}: ${value}`);
    console.log(`  ${to}: ${result}`);
  }

  async generateAccount(): Promise<void> {
    // Intentionally minimal in the scaffold (no wallet + signing pipeline yet).
    const privateKey = randomBytes(32).toString("hex");

    console.log(chalk.yellow("⚠️  Account generation is a placeholder (no signing support yet)."));
    console.log(chalk.green("New account generated:"));
    console.log(chalk.red(`  Private Key: 0x${privateKey}`));
  }

  private ensureRpc(): NeoRpcProvider {
    if (!this.rpcProvider) {
      throw new Error("RPC provider not configured. Use --rpc-url or configure a network.");
    }
    return this.rpcProvider;
  }

  private createRpcProvider(rpcUrl: string): NeoRpcProvider {
    const call = async <T = any>(method: string, params: any[] = []): Promise<T> => {
      debug(`RPC call: ${method}`);

      const response = await axios.post(rpcUrl, {
        jsonrpc: "2.0",
        method,
        params,
        id: 1,
      });

      if (response.data?.error) {
        throw new Error(response.data.error.message ?? "RPC error");
      }

      return response.data.result as T;
    };

    return {
      url: rpcUrl,
      magic: 0,
      call,
      getBlock: async (hashOrIndex) => call("getblock", [hashOrIndex, 1]),
      getTransaction: async (hash) => call("getrawtransaction", [hash, 1]),
      getContractState: async (scriptHash) => call("getcontractstate", [scriptHash]),
      invokeFunction: async (scriptHash, method, params) =>
        call<InvokeResult>("invokefunction", [scriptHash, method, params ?? []]),
      sendRawTransaction: async (signedTransaction) => call("sendrawtransaction", [signedTransaction]),
      sendTransaction: async (transaction) => call("sendtransaction", [transaction]),
      getBalance: async (address) => {
        const nep17Balances = await call<any>("getnep17balances", [address]);
        const balances: Balance[] = Array.isArray(nep17Balances?.balance)
          ? nep17Balances.balance.map((b: any) => ({
              assetHash: b.assethash,
              symbol: b.symbol ?? "UNKNOWN",
              decimals: b.decimals ?? 0,
              amount: b.amount,
            }))
          : [];
        return balances;
      },
      getStorage: async (scriptHash, key) => {
        try {
          return await call<string>("getstorage", [scriptHash, key]);
        } catch {
          return null;
        }
      },
      getBlockCount: async () => call("getblockcount"),
      getBestBlockHash: async () => call("getbestblockhash"),
      getTransactionHeight: async (hash) => call("gettransactionheight", [hash]),
      getApplicationLog: async (hash) => call("getapplicationlog", [hash]),
    };
  }

  private addressToScriptHash(address: string): string {
    const decoded = this.base58CheckDecode(address);
    if (decoded.length !== 21) {
      throw new Error("Invalid address length");
    }
    const scriptHash = decoded.slice(1);
    return "0x" + Buffer.from(scriptHash).reverse().toString("hex");
  }

  private base58CheckDecode(address: string): Buffer {
    const alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let num = 0n;

    for (const char of address) {
      const index = alphabet.indexOf(char);
      if (index === -1) {
        throw new Error(`Invalid character in address: ${char}`);
      }
      num = num * 58n + BigInt(index);
    }

    const hex = num.toString(16);
    const decoded = Buffer.from(hex.length % 2 ? "0" + hex : hex, "hex");

    const leadingOnes = address.match(/^1*/)?.[0]?.length ?? 0;
    const payloadWithChecksum = Buffer.concat([Buffer.alloc(leadingOnes), decoded]);

    if (payloadWithChecksum.length < 5) {
      throw new Error("Invalid address");
    }

    const payload = payloadWithChecksum.subarray(0, payloadWithChecksum.length - 4);
    const checksum = payloadWithChecksum.subarray(payloadWithChecksum.length - 4);

    const checksum1 = createHash("sha256").update(payload).digest();
    const checksum2 = createHash("sha256").update(checksum1).digest();
    const expected = checksum2.subarray(0, 4);

    if (!expected.equals(checksum)) {
      throw new Error("Invalid address checksum");
    }

    return payload;
  }

  private encodeArgs(args: unknown[]): any[] {
    return args.map((arg) => {
      if (typeof arg === "string") {
        return { type: "String", value: arg };
      }
      if (typeof arg === "number" || typeof arg === "bigint") {
        return { type: "Integer", value: arg.toString() };
      }
      if (typeof arg === "boolean") {
        return { type: "Boolean", value: arg };
      }
      if (Array.isArray(arg)) {
        return { type: "Array", value: this.encodeArgs(arg) };
      }
      return { type: "ByteString", value: Buffer.from(JSON.stringify(arg)).toString("hex") };
    });
  }

  private formatStackResult(stack: any[]): any {
    if (!stack || stack.length === 0) return null;
    if (stack.length === 1) return this.formatStackItem(stack[0]);
    return stack.map((item) => this.formatStackItem(item));
  }

  private formatStackItem(item: any): any {
    if (!item) return null;
    switch (item.type) {
      case "Boolean":
        return item.value;
      case "Integer":
        return item.value;
      case "ByteString":
        return item.value;
      case "Array":
        return item.value?.map((sub: any) => this.formatStackItem(sub));
      case "Map":
        return item.value;
      default:
        return item.value;
    }
  }

  private tryDecodeStorage(value: string): string {
    try {
      const decoded = Buffer.from(value, "hex").toString("ascii");
      if (/^[\x20-\x7E]*$/.test(decoded)) {
        return `"${decoded}"`;
      }
      const intValue = parseInt(value, 16);
      if (!Number.isNaN(intValue)) {
        return intValue.toString();
      }
      return value;
    } catch {
      return value;
    }
  }
}
