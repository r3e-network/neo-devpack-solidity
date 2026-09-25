import {
  GasProfiler as IGasProfiler,
  GasProfile,
  TransactionProfile,
  ContractProfile,
  OptimizationSuggestion,
  GasBreakdown,
  ExecutionTrace,
  ReportFormat
} from '@neo-devpack-solidity/types';
import Debug from "debug";

const debug = Debug("hardhat:neo-solc:profiler");

/**
 * Neo N3 Gas Profiler.
 *
 * Analyzes gas consumption using Neo N3 RPC methods:
 * - `getapplicationlog` — get gas consumed after execution
 * - `invokefunction` — estimate gas before execution
 *
 * NOTE: Neo N3 nodes do NOT expose per-opcode gas traces by default,
 * so detailed gas breakdowns are approximated from the total consumed
 * GAS amount.
 */
export class GasProfiler implements IGasProfiler {
  private activeProfiles: Map<string, GasProfile> = new Map();
  private contractProfiles: Map<string, ContractProfile> = new Map();
  private rpcUrl: string;

  constructor(config: any) {
    this.rpcUrl = config?.networks?.hardhat?.rpc?.url ?? "http://127.0.0.1:10332";
  }

  async startProfiling(): Promise<void> {
    const profileId = `profile_${Date.now()}`;

    const profile: GasProfile = {
      id: profileId,
      startTime: new Date(),
      endTime: new Date(),
      duration: 0,
      totalGasUsed: "0",
      totalCost: "0",
      averageGasPrice: "0",
      transactions: [],
      contracts: [],
      summary: {
        totalTransactions: 0,
        successfulTransactions: 0,
        failedTransactions: 0,
        totalGasUsed: "0",
        totalCost: "0",
        averageTransactionCost: "0",
        gasEfficiency: 0,
        topGasConsumers: [],
        patterns: []
      },
      optimizations: []
    };

    this.activeProfiles.set(profileId, profile);
  }

  async stopProfiling(): Promise<GasProfile> {
    const profileId = Array.from(this.activeProfiles.keys())[0];
    if (!profileId) {
      throw new Error("No active profiling session");
    }

    const profile = this.activeProfiles.get(profileId)!;
    profile.endTime = new Date();
    profile.duration = profile.endTime.getTime() - profile.startTime.getTime();
    profile.optimizations = await this.generateOptimizations(profile);
    this.activeProfiles.delete(profileId);
    return profile;
  }

  async profileTransaction(txHash: string): Promise<TransactionProfile> {
    debug(`profileTransaction: ${txHash}`);

    // Fetch application log from Neo N3 node
    const appLog = await this.neoCall("getapplicationlog", [txHash]);
    const execution = appLog?.executions?.[0];

    const vmState: string = execution?.vmstate ?? execution?.vmState ?? "HALT";
    const gasConsumed: string = execution?.gasconsumed ?? execution?.gasConsumed ?? "0";
    const notifications: Array<{ eventname?: string; state?: any[] }> = execution?.notifications ?? [];

    const profile: TransactionProfile = {
      hash: txHash,
      from: "",
      to: "",
      value: "0",
      gasUsed: gasConsumed,
      gasPrice: "0",
      gasLimit: "0",
      cost: gasConsumed,
      status: vmState === "HALT" ? "success" : "failure",
      blockNumber: 0,
      timestamp: new Date(),
      events: notifications.map((n) => ({
        name: n.eventname ?? "unknown",
        args: n.state ?? [],
      })),
      gasBreakdown: {
        total: gasConsumed,
        byCategory: {},
        summary: `VM state: ${vmState}`,
      },
    };

    // Add to active profile if one exists
    for (const profileEntry of this.activeProfiles.values()) {
      profileEntry.transactions.push(profile);
      profileEntry.summary.totalTransactions++;
      if (vmState === "HALT") {
        profileEntry.summary.successfulTransactions++;
      } else {
        profileEntry.summary.failedTransactions++;
      }
    }

    return profile;
  }

  async profileContract(address: string): Promise<ContractProfile> {
    debug(`profileContract: ${address}`);
    const profile: ContractProfile = {
      address,
      name: "",
      totalGasUsed: "0",
      transactionCount: 0,
      averageGasPerTransaction: "0",
      functionProfiles: [],
    };
    this.contractProfiles.set(address, profile);
    return profile;
  }

  async getTransactionTrace(_txHash: string): Promise<ExecutionTrace> {
    // Neo N3 nodes do not expose per-opcode traces by default
    return { steps: [], gasBreakdown: { total: "0", byCategory: {}, summary: "Not available on Neo N3" } };
  }

  async analyzeGasBreakdown(_trace: ExecutionTrace): Promise<GasBreakdown> {
    return { total: "0", byCategory: {}, summary: "Neo N3 does not expose per-opcode gas breakdown" };
  }

  async generateReport(profile: GasProfile, format: ReportFormat = "json"): Promise<string> {
    if (format === "json") {
      return JSON.stringify(profile, null, 2);
    }
    // Compact text summary for the non-JSON formats; Neo N3's application-log
    // data does not support the richer HTML/CSV/PDF breakdowns.
    const s = profile.summary;
    return [
      `Gas profile ${profile.id}`,
      `transactions: ${s.totalTransactions} (${s.successfulTransactions} ok, ${s.failedTransactions} faulted)`,
      `total GAS: ${profile.totalGasUsed}`,
    ].join("\n");
  }

  private async generateOptimizations(profile: GasProfile): Promise<OptimizationSuggestion[]> {
    const suggestions: OptimizationSuggestion[] = [];

    if (profile.summary.failedTransactions > 0) {
      suggestions.push({
        type: "execution",
        severity: "high",
        title: "Transaction failures detected",
        description: `${profile.summary.failedTransactions} transaction(s) FAULTed`,
        location: {},
        currentCost: "Unknown",
        potentialSavings: "Unknown",
        implementation: "Check execution exceptions via getapplicationlog",
      });
    }

    return suggestions;
  }

  /**
   * Make a Neo N3 JSON-RPC call
   */
  private async neoCall(method: string, params: any[] = []): Promise<any> {
    const response = await fetch(this.rpcUrl, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        jsonrpc: "2.0",
        method,
        params,
        id: 1,
      }),
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const data: any = await response.json();
    if (data.error) {
      throw new Error(`Neo RPC error: ${data.error.message}`);
    }
    return data.result;
  }
}
