import axios, { AxiosInstance } from "axios";
import type { rpcTypes } from "@neo-devpack-solidity/types";
import { NeoNetworkConfig, base64ToHex, hexToBase64 } from "@neo-devpack-solidity/types";
import { HardhatPluginError } from "hardhat/plugins";
import Debug from "debug";

const debug = Debug("hardhat:neo-deployer:rpc");

/**
 * Neo RPC client implementation
 */
export class NeoRpcClient implements rpcTypes.NeoRpcProvider {
  public url: string;
  public readonly magic: number;
  private client: AxiosInstance;
  private requestId = 1;
  private readonly rpcUrls: string[];
  private rpcUrlIndex = 0;

  constructor(config: NeoNetworkConfig) {
    this.rpcUrls = Array.isArray(config.rpcUrls) && config.rpcUrls.length > 0 ? config.rpcUrls : [];
    if (this.rpcUrls.length === 0) {
      throw new HardhatPluginError(
        "@neo-devpack-solidity/hardhat-neo-deployer",
        "Neo RPC configuration must include at least one URL (neoNetworks.<name>.rpcUrls)"
      );
    }

    this.url = this.rpcUrls[0]; // Default to first RPC URL
    this.magic = config.magic;
    
    this.client = axios.create({
      baseURL: this.url,
      timeout: 30000,
      proxy: false,
      headers: {
        'Content-Type': 'application/json'
      }
    });
  }

  /**
   * Make RPC call
   */
  async call<T = any>(method: string, params: any[] = []): Promise<T> {
    debug(`RPC call: ${method}`, params);
    
    const request = {
      jsonrpc: '2.0',
      method,
      params,
      id: this.requestId++
    };

    // Prefer the last known-good URL, but fail over across configured RPC URLs on network failures.
    const urls = this.rpcUrls;
    let lastError: unknown = null;

    try {
      for (let attempt = 0; attempt < urls.length; attempt++) {
        const idx = (this.rpcUrlIndex + attempt) % urls.length;
        const baseURL = urls[idx];

        try {
          if (this.client.defaults.baseURL !== baseURL) {
            this.client.defaults.baseURL = baseURL;
          }

          const response = await this.client.post('/', request);

          if (response.data?.error) {
            // JSON-RPC-level error is not a transport issue; don't failover.
            throw new Error(`RPC error: ${response.data.error.message}`);
          }

          // Mark this URL as healthy for subsequent calls.
          this.url = baseURL;
          this.rpcUrlIndex = idx;

          return response.data.result;
        } catch (error) {
          lastError = error;

          // Don't retry JSON-RPC errors (contract/validation issues).
          const message = error instanceof Error ? error.message : String(error);
          if (message.startsWith("RPC error:")) {
            throw error;
          }
        }
      }

      throw lastError;
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      throw new HardhatPluginError(
        "@neo-devpack-solidity/hardhat-neo-deployer",
        `RPC call failed: ${message}`
      );
    }
  }

  private static strip0x(value: string): string {
    return value.startsWith("0x") ? value.slice(2) : value;
  }

  private static isHexString(value: string): boolean {
    const stripped = NeoRpcClient.strip0x(value);
    return stripped.length > 0 && /^[0-9a-fA-F]+$/.test(stripped) && stripped.length % 2 === 0;
  }

  /**
   * Get block by hash or index
   */
  async getBlock(hashOrIndex: string | number): Promise<rpcTypes.NeoBlock> {
    return this.call('getblock', [hashOrIndex, 1]); // verbose = 1
  }

  /**
   * Get transaction by hash
   */
  async getTransaction(hash: string): Promise<rpcTypes.NeoTransaction> {
    return this.call('getrawtransaction', [hash, 1]); // verbose = 1
  }

  /**
   * Get contract state
   */
  async getContractState(scriptHash: string): Promise<rpcTypes.ContractState> {
    return this.call('getcontractstate', [scriptHash]);
  }

  /**
   * Invoke contract function (read-only)
   */
  async invokeFunction(
    scriptHash: string,
    method: string,
    params: any[] = []
  ): Promise<rpcTypes.InvokeResult> {
    return this.call('invokefunction', [scriptHash, method, params]);
  }

  /**
   * Invoke an arbitrary script (read-only)
   *
   * Neo nodes differ on whether they expect hex or base64 in this RPC.
   * neo-go expects base64; neo-cli expects hex.
   */
  async invokeScript(scriptHex: string, signers?: any[]): Promise<any> {
    const normalizedHex = NeoRpcClient.strip0x(scriptHex);
    const base64 = hexToBase64(normalizedHex);

    try {
      if (signers && signers.length > 0) {
        return await this.call("invokescript", [base64, signers]);
      }
      return await this.call("invokescript", [base64]);
    } catch (_error) {
      // Fallback to hex if node rejects base64 payloads.
      if (signers && signers.length > 0) {
        return this.call("invokescript", [normalizedHex, signers]);
      }
      return this.call("invokescript", [normalizedHex]);
    }
  }

  /**
   * Send raw transaction
   */
  async sendRawTransaction(signedTransaction: string): Promise<rpcTypes.SendResult> {
    const normalize = (result: any): rpcTypes.SendResult => {
      if (typeof result === "string") return { hash: result };
      if (result && typeof result === "object" && typeof result.hash === "string") return { hash: result.hash };
      return result as rpcTypes.SendResult;
    };

    // neo-cli expects hex; neo-go expects base64. Try hex first when we have hex.
    if (NeoRpcClient.isHexString(signedTransaction)) {
      const hex = NeoRpcClient.strip0x(signedTransaction);
      try {
        return normalize(await this.call<any>("sendrawtransaction", [hex]));
      } catch (firstError) {
        const base64 = hexToBase64(hex);
        try {
          return normalize(await this.call<any>("sendrawtransaction", [base64]));
        } catch {
          throw firstError;
        }
      }
    }

    // For non-hex inputs, assume base64 first then fall back to hex.
    try {
      return normalize(await this.call<any>("sendrawtransaction", [signedTransaction]));
    } catch (firstError) {
      try {
        const hex = base64ToHex(signedTransaction);
        return normalize(await this.call<any>("sendrawtransaction", [NeoRpcClient.strip0x(hex)]));
      } catch {
        throw firstError;
      }
    }
  }

  /**
   * Send a transaction object. Prefer serializing and calling `sendrawtransaction`
   * since `sendtransaction` is not universally supported by Neo nodes.
   */
  async sendTransaction(transaction: any): Promise<rpcTypes.SendResult> {
    if (typeof transaction === "string") {
      return this.sendRawTransaction(transaction);
    }

    if (transaction && typeof transaction.serialize === "function") {
      return this.sendRawTransaction(transaction.serialize(true));
    }

    return this.call("sendtransaction", [transaction]);
  }

  /**
   * Get balance for address
   */
  async getBalance(address: string): Promise<rpcTypes.Balance[]> {
    const nep17Balances = await this.call('getnep17balances', [address]);
    
    return nep17Balances.balance.map((balance: any) => ({
      assetHash: balance.assethash,
      symbol: balance.symbol || 'UNKNOWN',
      decimals: balance.decimals || 0,
      amount: balance.amount
    }));
  }

  /**
   * Get storage item
   */
  async getStorage(scriptHash: string, key: string): Promise<string | null> {
    try {
      return await this.call('getstorage', [scriptHash, key]);
    } catch (_error) {
      // Storage item not found
      return null;
    }
  }

  /**
   * Get block count
   */
  async getBlockCount(): Promise<number> {
    return this.call('getblockcount');
  }

  /**
   * Get best block hash
   */
  async getBestBlockHash(): Promise<string> {
    return this.call('getbestblockhash');
  }

  /**
   * Get transaction height
   */
  async getTransactionHeight(hash: string): Promise<number> {
    return this.call('gettransactionheight', [hash]);
  }

  /**
   * Get application log
   */
  async getApplicationLog(hash: string): Promise<rpcTypes.ApplicationLog> {
    return this.call('getapplicationlog', [hash]);
  }

  /**
   * Calculate network fee
   */
  async calculateNetworkFee(tx: string): Promise<string> {
    const parseNetworkFee = (result: any): string => {
      if (typeof result === "string") return result;
      if (result && typeof result === "object") {
        if (typeof result.networkfee === "string") return result.networkfee;
        if (typeof result.networkFee === "string") return result.networkFee;
      }
      return "0";
    };

    // neo-cli expects hex; neo-go expects base64. Try hex first when we have hex.
    if (NeoRpcClient.isHexString(tx)) {
      const hex = NeoRpcClient.strip0x(tx);
      try {
        return parseNetworkFee(await this.call<any>("calculatenetworkfee", [hex]));
      } catch (firstError) {
        const base64 = hexToBase64(hex);
        try {
          return parseNetworkFee(await this.call<any>("calculatenetworkfee", [base64]));
        } catch {
          throw firstError;
        }
      }
    }

    // For non-hex inputs, assume base64 first then fall back to hex.
    try {
      return parseNetworkFee(await this.call<any>("calculatenetworkfee", [tx]));
    } catch (firstError) {
      try {
        const hex = base64ToHex(tx);
        return parseNetworkFee(await this.call<any>("calculatenetworkfee", [NeoRpcClient.strip0x(hex)]));
      } catch {
        throw firstError;
      }
    }
  }

  /**
   * Get peers
   */
  async getPeers(): Promise<any> {
    return this.call('getpeers');
  }

  /**
   * Get version
   */
  async getVersion(): Promise<any> {
    return this.call('getversion');
  }

  /**
   * Submit block
   */
  async submitBlock(block: string): Promise<boolean> {
    return this.call('submitblock', [block]);
  }

  /**
   * Get mempool
   */
  async getRawMemPool(): Promise<string[]> {
    return this.call('getrawmempool');
  }

  /**
   * Validate address
   */
  async validateAddress(address: string): Promise<any> {
    return this.call('validateaddress', [address]);
  }

  /**
   * List plugins
   */
  async listPlugins(): Promise<any[]> {
    return this.call('listplugins');
  }

  /**
   * Get committee
   */
  async getCommittee(): Promise<string[]> {
    return this.call('getcommittee');
  }

  /**
   * Get next block validators
   */
  async getNextBlockValidators(): Promise<string[]> {
    return this.call('getnextblockvalidators');
  }

  /**
   * Get candidates
   */
  async getCandidates(): Promise<any[]> {
    return this.call('getcandidates');
  }

  /**
   * Get state root
   */
  async getStateRoot(index: number): Promise<any> {
    return this.call('getstateroot', [index]);
  }

  /**
   * Get proof
   */
  async getProof(rootHash: string, contractHash: string, key: string): Promise<string> {
    return this.call('getproof', [rootHash, contractHash, key]);
  }

  /**
   * Verify proof
   */
  async verifyProof(rootHash: string, proof: string): Promise<string> {
    return this.call('verifyproof', [rootHash, proof]);
  }

  /**
   * Find states
   */
  async findStates(rootHash: string, contractHash: string, prefix: string): Promise<any> {
    return this.call('findstates', [rootHash, contractHash, prefix]);
  }

  /**
   * Get NEP-11 balances
   */
  async getNep11Balances(address: string): Promise<any> {
    return this.call('getnep11balances', [address]);
  }

  /**
   * Get NEP-11 transfers
   */
  async getNep11Transfers(address: string, startTime?: number, endTime?: number): Promise<any> {
    const params: Array<string | number> = [address];
    if (startTime !== undefined) params.push(startTime);
    if (endTime !== undefined) params.push(endTime);
    return this.call('getnep11transfers', params);
  }

  /**
   * Get NEP-17 transfers
   */
  async getNep17Transfers(address: string, startTime?: number, endTime?: number): Promise<any> {
    const params: Array<string | number> = [address];
    if (startTime !== undefined) params.push(startTime);
    if (endTime !== undefined) params.push(endTime);
    return this.call('getnep17transfers', params);
  }

  /**
   * Test connection
   */
  async testConnection(): Promise<boolean> {
    try {
      await this.getVersion();
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Get network information
   */
  async getNetworkInfo(): Promise<{
    magic: number;
    version: any;
    blockCount: number;
    peers: number;
  }> {
    const [version, blockCount, peers] = await Promise.all([
      this.getVersion(),
      this.getBlockCount(),
      this.getPeers().then(p => p.connected?.length || 0).catch(() => 0)
    ]);

    return {
      magic: this.magic,
      version,
      blockCount,
      peers
    };
  }
}
