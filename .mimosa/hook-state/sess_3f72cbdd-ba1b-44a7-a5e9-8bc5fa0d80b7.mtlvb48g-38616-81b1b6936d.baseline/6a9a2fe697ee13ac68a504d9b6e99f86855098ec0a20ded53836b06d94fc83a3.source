import { NeoRpcProvider, InvokeResult } from "@neo-devpack-solidity/types";
import Debug from "debug";
import { id } from "ethers";
import {
  decodeNeoBytes,
  normalizeNeoHash160,
  strip0x,
} from "./neo-utils.js";

const debug = Debug("neo-devpack-solidity:rpc-adapter");

/**
 * RPC adapter that bridges Ethereum-style calls to Neo RPC
 */
export class RpcAdapter {
  private rpcProvider: NeoRpcProvider;

  constructor(rpcProvider: NeoRpcProvider) {
    this.rpcProvider = rpcProvider;
  }

  /**
   * Invoke contract function (read-only)
   */
  async invokeFunction(
    contractAddress: string,
    methodName: string,
    args: any[] = [],
    _blockTag?: string | number
  ): Promise<InvokeResult> {
    debug(`Invoking function ${methodName} on ${contractAddress}`);

    try {
      const scriptHash = this.addressToScriptHash(contractAddress);
      
      // Convert arguments to Neo RPC format
      const neoParams = this.convertArgsToNeoParams(args);
      
      const result = await this.rpcProvider.invokeFunction(
        scriptHash,
        methodName,
        neoParams
      );

      if (result.state === "FAULT") {
        throw new Error(`Function call failed: ${result.exception || "Unknown error"}`);
      }

      return result;
    } catch (error) {
      debug(`Function invocation failed: ${error}`);
      throw error;
    }
  }

  /**
   * Estimate gas for transaction
   */
  async estimateGas(
    contractAddress: string,
    methodName: string,
    args: any[] = [],
    _options: {
      from?: string;
      value?: string;
    } = {}
  ): Promise<{
    systemFee: string;
    networkFee: string;
    totalGas: string;
  }> {
    debug(`Estimating gas for ${methodName} on ${contractAddress}`);

    try {
      // Invoke function to get gas consumption
      const scriptHash = this.addressToScriptHash(contractAddress);
      const neoParams = this.convertArgsToNeoParams(args);
      
      const result = await this.rpcProvider.invokeFunction(
        scriptHash,
        methodName,
        neoParams
      );

      // Calculate fees based on gas consumption
      const systemFee = result.gasConsumed;
      const networkFee = "1000000"; // 0.01 GAS for network fee (rough estimate)
      const totalGas = (BigInt(systemFee) + BigInt(networkFee)).toString();

      return {
        systemFee,
        networkFee,
        totalGas
      };
    } catch (error) {
      debug(`Gas estimation failed: ${error}`);
      throw error;
    }
  }

  /**
   * Get transaction receipt
   */
  async getTransactionReceipt(txHash: string): Promise<any> {
    debug(`Getting transaction receipt for ${txHash}`);

    try {
      // Get transaction details
      const tx = await this.rpcProvider.getTransaction(txHash);
      
      // Get application log for events and execution details
      const appLog = await this.rpcProvider.getApplicationLog(txHash);
      
      // Convert to Ethereum-style receipt
      return this.convertToEthereumReceipt(tx, appLog);
    } catch (error) {
      debug(`Failed to get transaction receipt: ${error}`);
      return null; // Transaction not found or not yet confirmed
    }
  }

  /**
   * Get current block number
   */
  async getBlockNumber(): Promise<number> {
    return this.rpcProvider.getBlockCount();
  }

  /**
   * Get block by number or hash
   */
  async getBlock(blockHashOrNumber: string | number): Promise<any> {
    debug(`Getting block ${blockHashOrNumber}`);

    try {
      const block = await this.rpcProvider.getBlock(blockHashOrNumber);
      return this.convertToEthereumBlock(block);
    } catch (error) {
      debug(`Failed to get block: ${error}`);
      throw error;
    }
  }

  /**
   * Get contract events
   */
  async getEvents(
    contractAddress: string,
    eventName: string,
    fromBlock?: number | string,
    toBlock?: number | string
  ): Promise<any[]> {
    debug(`Getting events ${eventName} from ${contractAddress}`);

    try {
      const filter = { address: contractAddress, eventName, fromBlock, toBlock };

      return await this.fetchEventsFromNeo(filter);
    } catch (error) {
      debug(`Failed to get events: ${error}`);
      throw error;
    }
  }

  /**
   * Get account balance
   */
  async getBalance(address: string, _blockTag?: string | number): Promise<string> {
    debug(`Getting balance for ${address}`);

    try {
      const balances = await this.rpcProvider.getBalance(address);
      
      // Find GAS balance
      const gasBalance = balances.find(b => b.symbol === 'GAS');
      
      if (!gasBalance) {
        return "0";
      }

      // Convert to Wei-like format (GAS has 8 decimals, we need 18 for ETH compatibility)
      const gasAmount = BigInt(gasBalance.amount);
      const weiAmount = gasAmount * BigInt(10 ** 10); // Convert 8 decimals to 18 decimals
      
      return weiAmount.toString();
    } catch (error) {
      debug(`Failed to get balance: ${error}`);
      throw error;
    }
  }

  /**
   * Get transaction count (nonce)
   */
  async getTransactionCount(address: string, _blockTag?: string | number): Promise<number> {
    try {
      // Neo doesn't use nonces like Ethereum, but we can return the number of transactions
      // sent by this address as an approximation
      const scriptHash = this.addressToScriptHash(address);
      const transactions = await this.rpcCall('getaddresshistory', [scriptHash]);
      return transactions?.length || 0;
    } catch (error) {
      debug(`Failed to get transaction count: ${error}`);
      return 0;
    }
  }

  /**
   * Get code at address
   */
  async getCode(address: string, _blockTag?: string | number): Promise<string> {
    debug(`Getting code for ${address}`);

    try {
      const scriptHash = this.addressToScriptHash(address);
      const contractState = await this.rpcProvider.getContractState(scriptHash);
      
      // Return NEF script as bytecode
      return '0x' + contractState.nef.script;
    } catch (error) {
      debug(`Failed to get code: ${error}`);
      return "0x"; // No code found
    }
  }

  /**
   * Get storage at position
   */
  async getStorageAt(
    address: string,
    position: string,
    _blockTag?: string | number
  ): Promise<string> {
    debug(`Getting storage at ${position} for ${address}`);

    try {
      const scriptHash = this.addressToScriptHash(address);
      const value = await this.rpcProvider.getStorage(scriptHash, position);
      
      return value || "0x0000000000000000000000000000000000000000000000000000000000000000";
    } catch (error) {
      debug(`Failed to get storage: ${error}`);
      return "0x0000000000000000000000000000000000000000000000000000000000000000";
    }
  }

  /**
   * Send raw transaction
   */
  async sendRawTransaction(signedTx: string): Promise<string> {
    debug("Sending raw transaction");

    try {
      const result = await this.rpcProvider.sendRawTransaction(signedTx);
      return result.hash;
    } catch (error) {
      debug(`Failed to send raw transaction: ${error}`);
      throw error;
    }
  }

  // Private methods

  private convertArgsToNeoParams(args: any[]): any[] {
    return args.map(arg => {
      if (typeof arg === 'object' && arg !== null && arg.type) {
        // Already in Neo format
        return arg;
      }

      // Convert primitive types to Neo format
      if (typeof arg === 'boolean') {
        return { type: 'Boolean', value: arg };
      }
      
      if (typeof arg === 'number' || typeof arg === 'bigint') {
        return { type: 'Integer', value: String(arg) };
      }
      
      if (typeof arg === 'string') {
        if (arg.startsWith('0x')) {
          return { type: 'ByteArray', value: arg.slice(2) };
        }
        return { type: 'String', value: arg };
      }

      // Default to string representation
      return { type: 'String', value: String(arg) };
    });
  }

  private convertToEthereumReceipt(tx: any, appLog: any): any {
    return {
      transactionHash: tx.hash,
      transactionIndex: 0,
      blockNumber: tx.blockTime ? tx.blockTime : null,
      blockHash: tx.blockHash || null,
      cumulativeGasUsed: tx.sysFee,
      gasUsed: tx.sysFee,
      contractAddress: null, // Would be set for contract creation
      logs: this.convertNeoLogsToEthereumLogs(appLog.executions?.[0]?.notifications || []),
      status: appLog.executions?.[0]?.vmState === 'HALT' ? 1 : 0,
      from: tx.sender,
      to: null, // Would need to extract from transaction
      type: 0,
      effectiveGasPrice: "1000"
    };
  }

  private convertToEthereumBlock(neoBlock: any): any {
    return {
      number: neoBlock.index,
      hash: neoBlock.hash,
      parentHash: neoBlock.previousBlockHash,
      timestamp: neoBlock.time,
      size: neoBlock.size,
      gasLimit: "0xffffffff",
      gasUsed: "0x0", // Would need to calculate from transactions
      transactions: neoBlock.tx?.map((tx: any) => tx.hash) || [],
      miner: neoBlock.nextConsensus,
      difficulty: "0x0",
      totalDifficulty: "0x0"
    };
  }

  private convertNeoLogsToEthereumLogs(notifications: any[]): any[] {
    return notifications.map((notification, index) => ({
      address: notification.contract,
      topics: [
        // Best-effort placeholder; decode via EventFragment in EventDecoder for real topics.
        id(String(notification.eventname || notification.eventName || "UnknownEvent")),
      ],
      data: this.encodeLogData(notification.state),
      logIndex: index,
      transactionIndex: 0,
      blockNumber: 0, // Would be filled from actual block
      blockHash: "0x0000000000000000000000000000000000000000000000000000000000000000"
    }));
  }

  private encodeLogData(state: any[]): string {
    // Best-effort: encode the notification state into hex(JSON) for inspection.
    try {
      const payload = JSON.stringify(state ?? null);
      return "0x" + Buffer.from(payload, "utf8").toString("hex");
    } catch {
      return "0x";
    }
  }

  /**
   * Fetch events from Neo blockchain
   */
  private async fetchEventsFromNeo(filter: {
    address?: string;
    eventName?: string;
    fromBlock?: string | number;
    toBlock?: string | number;
  }): Promise<any[]> {
    const events: any[] = [];
    
    try {
      const fromBlock = await this.resolveBlockTag(filter.fromBlock ?? 'earliest');
      const toBlock = await this.resolveBlockTag(filter.toBlock ?? 'latest');

      const blockRange = toBlock - fromBlock;
      if (blockRange > 2000) {
        throw new Error(
          `Refusing to scan ${blockRange} blocks. Provide a narrower fromBlock/toBlock range.`
        );
      }
      
      // Get application logs for the block range
      for (let blockHeight = fromBlock; blockHeight <= toBlock; blockHeight++) {
        const block = await this.rpcCall('getblock', [blockHeight, 1]);
        if (!block || !block.tx) continue;
        
        for (const tx of block.tx) {
          const appLog = await this.rpcCall('getapplicationlog', [tx.hash]);
          if (appLog && appLog.executions) {
            for (const execution of appLog.executions) {
              if (execution.notifications) {
                const annotated = (execution.notifications as any[]).map((notification) => ({
                  ...notification,
                  blockNumber: blockHeight,
                  transactionHash: tx.hash,
                }));
                events.push(...this.filterNotifications(annotated, filter));
              }
            }
          }
        }
      }
    } catch (error) {
      debug(`Failed to fetch events: ${error}`);
    }
    
    return events;
  }

  /**
   * Filter Neo notifications based on event filter criteria
   */
  private filterNotifications(notifications: any[], filter: any): any[] {
    return notifications.filter(notification => {
      // Filter by contract address
      if (filter.address) {
        const targetScriptHash = strip0x(this.addressToScriptHash(filter.address)).toLowerCase();
        const contractHash = strip0x(String(notification.contract ?? "")).toLowerCase();
        if (!contractHash || contractHash !== targetScriptHash) {
          return false;
        }
      }
      
      if (filter.eventName) {
        const name = String(notification.eventname || notification.eventName || "");
        if (name !== filter.eventName) return false;
      }
      
      return true;
    });
  }

  /**
   * Parse block tag to block number
   */
  private async resolveBlockTag(blockTag: string | number): Promise<number> {
    if (typeof blockTag === 'number') {
      return blockTag;
    }
    
    if (typeof blockTag === 'string') {
      switch (blockTag) {
        case 'earliest':
          return 0;
        case 'latest':
        case 'pending': {
          try {
            const count = await this.rpcProvider.getBlockCount();
            return Math.max(0, count - 1);
          } catch (error) {
            debug(`Failed to resolve ${blockTag} block tag: ${error}`);
            return 0;
          }
        }
        default:
          if (blockTag.startsWith('0x')) {
            return parseInt(blockTag, 16);
          }
          return parseInt(blockTag, 10);
      }
    }

    return 0;
  }

  /**
   * Convert Neo address to script hash
   */
  private addressToScriptHash(address: string): string {
    // Allow direct script hash usage (Neo canonical form)
    try {
      return normalizeNeoHash160(address);
    } catch {
      // Fallback: treat as raw ByteString and attempt to decode to 20 bytes, then normalize.
      const bytes = decodeNeoBytes(address);
      if (bytes.length !== 20) {
        throw new Error(`Invalid Neo address/script hash: ${address}`);
      }
      return "0x" + bytes.toString("hex");
    }
  }

  /**
   * Make RPC call to Neo node
   */
  private async rpcCall(method: string, params: any[] = []): Promise<any> {
    try {
      return await this.rpcProvider.call(method, params);
    } catch (error) {
      debug(`RPC call failed: ${method} - ${error}`);
      throw error;
    }
  }
}
