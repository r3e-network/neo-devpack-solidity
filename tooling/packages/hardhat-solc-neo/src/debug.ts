import { sourcemapTypes } from '@neo-devpack-solidity/types';
import Debug from "debug";

const debug = Debug("hardhat:neo-solc:debug");

type DebugSession = sourcemapTypes.DebugSession;
type DebugState = sourcemapTypes.DebugState;
type Breakpoint = sourcemapTypes.Breakpoint;
type StackFrame = sourcemapTypes.StackFrame;
type Variable = sourcemapTypes.Variable;
type MemoryState = sourcemapTypes.MemoryState;
type StorageState = sourcemapTypes.StorageState;

/**
 * Minimal Neo N3 JSON-RPC client for debug operations.
 */
export interface DebugRpcClient {
  /** Get application log for a transaction */
  getApplicationLog(txHash: string): Promise<any>;
  /** Invoke contract function (read-only simulation) */
  invokeFunction(scriptHash: string, method: string, params?: any[]): Promise<any>;
  /** Get storage for a contract key */
  getStorage(scriptHash: string, key: string): Promise<string | null>;
}

/**
 * Neo N3 Debug Manager.
 *
 * Neo N3 nodes expose post-execution application logs via `getapplicationlog`
 * (VM state, stack, gas consumed, notifications) but do NOT expose
 * per-opcode stepping, breakpoints, or source-level debugging.
 *
 * For step-by-step / interactive debugging of the in-tree NeoVM simulator,
 * build and use `neo-test` (the Foundry-style test runner) which runs the
 * embedded runtime with full trace output via `--trace` or `-vvvv`:
 *
 *     cargo build --release --bin neo-test
 *     neo-test --trace path/to/Test.sol
 */
export class DebugManager {
  private sessions: Map<string, DebugSession> = new Map();
  private rpcClient: DebugRpcClient;

  constructor(rpcClient: DebugRpcClient) {
    this.rpcClient = rpcClient;
  }

  // ===================================================================
  // Session lifecycle
  // ===================================================================

  /**
   * Start a debug session by fetching the application log for a
   * previously-executed transaction.
   */
  async startDebugSession(transactionHash: string): Promise<DebugSession> {
    debug(`startDebugSession: ${transactionHash}`);

    let appLog: any;
    try {
      appLog = await this.rpcClient.getApplicationLog(transactionHash);
    } catch (err) {
      throw new Error(
        `Failed to fetch application log for tx ${transactionHash}: ${err}. ` +
        `Ensure the transaction has been persisted.`
      );
    }

    const execution = appLog?.executions?.[0];
    if (!execution) {
      throw new Error(`No execution found for tx ${transactionHash}`);
    }

    const vmState: string = execution.vmstate ?? execution.vmState ?? "HALT";
    const gasConsumed: string = execution.gasconsumed ?? execution.gasConsumed ?? "0";

    const sessionId = `debug_${transactionHash}_${Date.now()}`;

    const session: DebugSession = {
      id: sessionId,
      transactionHash,
      contractAddress: "",
      debugInfo: {
        sourceMap: { mappings: "", sources: [], sourcesContent: [], names: [], version: 3 },
        pcToSourceMap: new Map(),
        sourceToByteMap: new Map(),
        functionDebugData: new Map(),
        contractDebugData: {
          name: "",
          sourceFiles: [],
          functions: new Map(),
          events: new Map(),
          stateVariables: new Map(),
        },
      },
      currentState: {
        pc: 0,
        op: vmState,
        gas: gasConsumed,
        gasUsed: gasConsumed,
        depth: 0,
        stack: (execution.stack ?? []).map((s: any) =>
          typeof s === "object" ? JSON.stringify(s) : String(s)
        ),
        memory: [],
        storage: {},
        calldata: "",
        returndata: "",
      },
      breakpoints: [],
      callStack: [],
    };

    this.sessions.set(sessionId, session);
    debug(`Session ${sessionId} started, state=${vmState}`);
    return session;
  }

  /**
   * Fetch the debug trace (application log) for a transaction.
   *
   * Neo N3 nodes do not expose per-opcode traces by default; the
   * `getapplicationlog` RPC provides the final execution result only.
   */
  async getDebugTrace(transactionHash: string): Promise<Array<{
    pc: number;
    op: string;
    gas: string;
    gasCost: string;
    depth: number;
    stack: string[];
    memory: string[];
    storage: { [key: string]: string };
  }>> {
    debug(`getDebugTrace: ${transactionHash}`);
    const session = await this.startDebugSession(transactionHash);

    return [
      {
        pc: 0,
        op: "APPLICATION_LOG",
        gas: session.currentState.gas,
        gasCost: session.currentState.gasUsed,
        depth: 0,
        stack: session.currentState.stack,
        memory: session.currentState.memory,
        storage: session.currentState.storage,
      },
    ];
  }

  // ===================================================================
  // Contract state inspection via RPC
  // ===================================================================

  /**
   * Simulate a read-only contract call via `invokefunction`.
   */
  async simulateCall(
    scriptHash: string,
    method: string,
    params: any[] = []
  ): Promise<{ state: string; gasConsumed: string; stack: any[] }> {
    debug(`simulateCall: ${method} on ${scriptHash}`);
    try {
      const result = await this.rpcClient.invokeFunction(scriptHash, method, params);
      return {
        state: result.state ?? "FAULT",
        gasConsumed: result.gasconsumed ?? result.gasConsumed ?? "0",
        stack: result.stack ?? [],
      };
    } catch (err) {
      throw new Error(`Failed to simulate ${method} on ${scriptHash}: ${err}`);
    }
  }

  /**
   * Read a storage key from a contract.
   */
  async getContractStorage(scriptHash: string, key: string): Promise<string | null> {
    debug(`getStorage: ${scriptHash}/${key}`);
    return this.rpcClient.getStorage(scriptHash, key);
  }

  // ===================================================================
  // Gas analysis
  // ===================================================================

  /**
   * Analyze gas usage from a transaction's application log.
   */
  async analyzeGasUsage(transactionHash: string): Promise<{
    totalGas: string;
    gasPerOperation: Array<{ op: string; gas: string; percentage: number }>;
    recommendations: string[];
  }> {
    debug(`analyzeGasUsage: ${transactionHash}`);
    const session = await this.startDebugSession(transactionHash);
    const totalGas = session.currentState.gas;

    const recommendations: string[] = [];
    if (session.currentState.op === "FAULT") {
      recommendations.push(
        `Transaction FAULTed: ${session.currentState.error ?? "unknown reason"}`
      );
    }

    return {
      totalGas,
      gasPerOperation: [{ op: "total", gas: totalGas, percentage: 100 }],
      recommendations,
    };
  }

  /**
   * Generate an execution report from a transaction's application log.
   */
  async generateExecutionReport(transactionHash: string): Promise<{
    executionTime: number;
    operationCounts: { [op: string]: number };
    memoryUsage: { peak: number; average: number };
    storageAccess: { reads: number; writes: number };
    callGraph: Array<{ name: string }>;
  }> {
    debug(`generateExecutionReport: ${transactionHash}`);
    const session = await this.startDebugSession(transactionHash);

    return {
      executionTime: 0,
      operationCounts: {
        stack_items: session.currentState.stack.length,
      },
      memoryUsage: { peak: 0, average: 0 },
      storageAccess: { reads: 0, writes: 0 },
      callGraph: [],
    };
  }

  /**
   * Export a debug session to JSON or CSV.
   */
  async exportDebugSession(
    sessionId: string,
    format: "json" | "csv"
  ): Promise<string> {
    const session = this.sessions.get(sessionId);
    if (!session) {
      throw new Error(`Debug session ${sessionId} not found`);
    }
    if (format === "json") {
      return JSON.stringify(session, null, 2);
    }
    return `id,transactionHash,gas,state\n${session.id},${session.transactionHash},${session.currentState.gas},${session.currentState.op}\n`;
  }

  // ===================================================================
  // Unsupported features
  // ===================================================================

  async stepOver(_sessionId: string): Promise<DebugState> {
    throw new Error(
      "Neo N3 nodes do not expose per-opcode stepping. " +
      "Use `neo-test --trace` (in-tree NeoVM simulator) for step-by-step debugging."
    );
  }

  async stepInto(_sessionId: string): Promise<DebugState> {
    throw new Error(
      "Neo N3 nodes do not expose per-opcode stepping. " +
      "Use `neo-test --trace` (in-tree NeoVM simulator) for step-by-step debugging."
    );
  }

  async stepOut(_sessionId: string): Promise<DebugState> {
    throw new Error(
      "Neo N3 nodes do not expose per-opcode stepping. " +
      "Use `neo-test --trace` (in-tree NeoVM simulator) for step-by-step debugging."
    );
  }

  async continue(_sessionId: string): Promise<DebugState> {
    throw new Error(
      "Neo N3 nodes do not expose per-opcode stepping. " +
      "Use `neo-test --trace` (in-tree NeoVM simulator) for step-by-step debugging."
    );
  }

  async evaluateExpression(
    _sessionId: string,
    _expression: string
  ): Promise<unknown> {
    throw new Error(
      "Expression evaluation is not supported on Neo N3 nodes. " +
      "Use `neo-test --trace` for expression-level debugging."
    );
  }

  async setBreakpoint(
    _source: string,
    _line: number,
    _column?: number,
    _condition?: string
  ): Promise<Breakpoint> {
    throw new Error(
      "Breakpoints are not supported on Neo N3 nodes. " +
      "Use `neo-test` (in-tree NeoVM simulator) for Solidity source-level debugging."
    );
  }

  async removeBreakpoint(_breakpointId: string): Promise<boolean> {
    throw new Error("Breakpoints are not supported on Neo N3 nodes.");
  }

  async getStackTrace(_sessionId: string): Promise<StackFrame[]> {
    return [];
  }

  async getLocalVariables(_sessionId: string): Promise<Variable[]> {
    return [];
  }

  async getMemory(_sessionId: string): Promise<MemoryState> {
    return { size: 0, data: "", words: [], allocatedSize: 0 };
  }

  async getStorageState(_sessionId: string): Promise<StorageState> {
    return { slots: {}, layout: { storage: [], types: {} } };
  }
}
