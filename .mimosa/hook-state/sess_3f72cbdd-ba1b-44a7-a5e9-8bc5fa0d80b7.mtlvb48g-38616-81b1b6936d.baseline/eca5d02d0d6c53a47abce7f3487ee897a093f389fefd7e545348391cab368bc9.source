import type { NeoHardhatNetworkConfig } from '@neo-devpack-solidity/types';
import Debug from "debug";

const debug = Debug("hardhat:neo-solc:network");

/**
 * Network manager for Neo N3 — uses Neo JSON-RPC methods, NOT Ethereum
 * JsonRpcProvider. This was previously implemented with ethers.js which
 * calls eth_* methods incompatible with Neo N3 nodes.
 */
export class NetworkManager {
  private networks: Map<string, NeoHardhatNetworkConfig> = new Map();
  private rpcClients: Map<string, NeoRpcClient> = new Map();
  private currentNetwork?: string;

  constructor(networks: { [networkName: string]: NeoHardhatNetworkConfig }) {
    for (const [name, config] of Object.entries(networks)) {
      this.addNetwork(name, config);
    }
  }

  // Network Management
  addNetwork(name: string, config: NeoHardhatNetworkConfig): void {
    this.validateNetworkConfig(config);
    this.networks.set(name, config);
  }

  removeNetwork(name: string): boolean {
    const removed = this.networks.delete(name);
    this.rpcClients.delete(name);
    return removed;
  }

  getNetwork(name: string): NeoHardhatNetworkConfig | undefined {
    return this.networks.get(name);
  }

  listNetworks(): string[] {
    return Array.from(this.networks.keys());
  }

  getCurrentNetwork(): string | undefined {
    return this.currentNetwork;
  }

  // RPC Client Management
  private getRpcClient(networkName: string): NeoRpcClient {
    let client = this.rpcClients.get(networkName);
    if (!client) {
      const network = this.networks.get(networkName);
      if (!network) {
        throw new Error(`Network "${networkName}" not found`);
      }
      client = new NeoRpcClient(network);
      this.rpcClients.set(networkName, client);
    }
    return client;
  }

  // Neo N3 RPC Methods
  /**
   * Get current block count (Neo RPC: getblockcount)
   */
  async getBlockCount(networkName: string): Promise<number> {
    const client = this.getRpcClient(networkName);
    return client.call<number>("getblockcount");
  }

  /**
   * Get best block hash (Neo RPC: getbestblockhash)
   */
  async getBestBlockHash(networkName: string): Promise<string> {
    const client = this.getRpcClient(networkName);
    return client.call<string>("getbestblockhash");
  }

  /**
   * Get NEP-17 balances for an address (Neo RPC: getnep17balances)
   */
  async getBalance(networkName: string, address: string): Promise<Array<{ assethash: string; symbol: string; amount: string }>> {
    const client = this.getRpcClient(networkName);
    const result = await client.call<{ balance: Array<{ assethash: string; symbol: string; decimals: number; amount: string }> }>("getnep17balances", [address]);
    return result.balance || [];
  }

  /**
   * Get GAS balance for an address
   */
  async getGasBalance(networkName: string, address: string): Promise<string> {
    const balances = await this.getBalance(networkName, address);
    // GAS native contract hash on Neo N3
    const gasBalance = balances.find(
      b => b.assethash.toLowerCase() === "0xd2a4cff31913016155e38e474a2c06d08be276cf"
    );
    return gasBalance?.amount || "0";
  }

  /**
   * Send raw transaction (Neo RPC: sendrawtransaction)
   */
  async sendRawTransaction(networkName: string, signedTx: string): Promise<{ hash: string }> {
    const client = this.getRpcClient(networkName);
    const result = await client.call<any>("sendrawtransaction", [signedTx]);
    if (typeof result === "string") return { hash: result };
    if (result?.hash) return { hash: result.hash };
    throw new Error(`sendrawtransaction returned unexpected result: ${JSON.stringify(result)}`);
  }

  /**
   * Invoke function (read-only, Neo RPC: invokefunction)
   */
  async invokeFunction(
    networkName: string,
    scriptHash: string,
    method: string,
    params: any[] = []
  ): Promise<any> {
    const client = this.getRpcClient(networkName);
    return client.call("invokefunction", [scriptHash, method, params]);
  }

  /**
   * Get transaction height (Neo RPC: gettransactionheight)
   */
  async getTransactionHeight(networkName: string, txHash: string): Promise<number> {
    const client = this.getRpcClient(networkName);
    return client.call<number>("gettransactionheight", [txHash]);
  }

  /**
   * Get application log (Neo RPC: getapplicationlog)
   */
  async getApplicationLog(networkName: string, txHash: string): Promise<any> {
    const client = this.getRpcClient(networkName);
    return client.call("getapplicationlog", [txHash]);
  }

  // Network Connection
  async connectToNetwork(networkName: string): Promise<void> {
    const network = this.networks.get(networkName);
    if (!network) {
      throw new Error(`Network "${networkName}" not found`);
    }

    try {
      const blockCount = await this.getBlockCount(networkName);
      this.currentNetwork = networkName;
      debug(`Connected to ${networkName} (block #${blockCount})`);
    } catch (error) {
      throw new Error(`Failed to connect to "${networkName}": ${error}`);
    }
  }

  async disconnectFromNetwork(): Promise<void> {
    this.currentNetwork = undefined;
  }

  // Network Status
  async getNetworkStatus(networkName: string): Promise<{
    connected: boolean;
    blockCount: number;
    peerCount?: number;
  }> {
    try {
      const client = this.getRpcClient(networkName);
      const [blockCount, peers] = await Promise.all([
        client.call<number>("getblockcount"),
        client.call<{ connected?: any[] }>("getpeers").catch(() => ({} as { connected?: any[] })),
      ]);
      return {
        connected: true,
        blockCount,
        peerCount: peers?.connected?.length || 0,
      };
    } catch {
      return { connected: false, blockCount: 0 };
    }
  }

  async healthCheck(networkName: string): Promise<{
    healthy: boolean;
    checks: Array<{ name: string; status: "pass" | "fail" | "warn"; message: string }>;
  }> {
    const checks: Array<{ name: string; status: "pass" | "fail" | "warn"; message: string }> = [];

    try {
      const blockCount = await this.getBlockCount(networkName);
      checks.push({ name: "connection", status: "pass", message: `Connected, block #${blockCount}` });
    } catch (error) {
      checks.push({ name: "connection", status: "fail", message: `Connection failed: ${error}` });
    }

    return { healthy: checks.every(c => c.status !== "fail"), checks };
  }

  // Configuration helpers
  getNeoAddressVersion(networkName: string): number {
    return this.networks.get(networkName)?.addressVersion || 53;
  }

  getMagicNumber(networkName: string): number {
    return this.networks.get(networkName)?.magic || 860833102;
  }

  private validateNetworkConfig(config: NeoHardhatNetworkConfig): void {
    if (!config.rpc?.url) {
      throw new Error("Network RPC URL is required");
    }
    if (!config.magic) {
      throw new Error("Network magic number is required");
    }
    if (!config.addressVersion) {
      throw new Error("Network address version is required");
    }
    try {
      new URL(config.rpc.url);
    } catch {
      throw new Error("Invalid RPC URL format");
    }
  }
}

/**
 * Minimal Neo JSON-RPC client used by NetworkManager.
 * Uses raw fetch() to avoid requiring @neo-devpack-solidity/hardhat-neo-deployer
 * as a dependency of @neo-devpack-solidity/hardhat-solc-neo.
 */
class NeoRpcClient {
  private url: string;
  private requestId = 1;

  constructor(config: NeoHardhatNetworkConfig) {
    this.url = config.rpc.url;
  }

  async call<T = any>(method: string, params: any[] = []): Promise<T> {
    const response = await fetch(this.url, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        jsonrpc: "2.0",
        method,
        params,
        id: this.requestId++,
      }),
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const data = await response.json() as { error?: { message?: string; code?: number }; result?: T };
    if (data.error) {
      throw new Error(`Neo RPC error: ${data.error.message} (code ${data.error.code})`);
    }

    return data.result as T;
  }
}
