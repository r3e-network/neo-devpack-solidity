/**
 * Debugging support types for Neo DevPack for Solidity development
 */

/**
 * Debug session configuration
 */
export interface DebugSession {
  /** Session ID */
  id: string;
  
  /** Contract being debugged */
  contract: {
    name: string;
    address: string;
    scriptHash: string;
  };
  
  /** Source maps */
  sourceMaps: {
    [fileName: string]: SourceMap;
  };
  
  /** Breakpoints */
  breakpoints: Breakpoint[];
  
  /** Current execution state */
  state: ExecutionState;
  
  /** Debug configuration */
  config: DebugConfig;
}

/**
 * Source map for debugging
 */
export interface SourceMap {
  /** File name */
  fileName: string;
  
  /** Source code */
  source: string;
  
  /** Mappings from bytecode to source */
  mappings: SourceMapping[];
  
  /** AST nodes */
  ast: AstNode[];
}

/**
 * Source mapping entry
 */
export interface SourceMapping {
  /** Bytecode offset */
  bytecodeOffset: number;
  
  /** Source location */
  sourceLocation: SourceLocation;
  
  /** Instruction info */
  instruction: {
    opcode: string;
    operand?: string;
    gasUsed: number;
  };
}

/**
 * Source location
 */
export interface SourceLocation {
  /** File name */
  file: string;
  
  /** Start position */
  start: {
    line: number;
    column: number;
    offset: number;
  };
  
  /** End position */
  end: {
    line: number;
    column: number;
    offset: number;
  };
}

/**
 * AST node for debugging
 */
export interface AstNode {
  /** Node ID */
  id: number;
  
  /** Node type */
  nodeType: string;
  
  /** Source location */
  src: string;
  
  /** Node name */
  name?: string;
  
  /** Node value */
  value?: any;
  
  /** Child nodes */
  children?: AstNode[];
  
  /** Parent node ID */
  parent?: number;
}

/**
 * Breakpoint configuration
 */
export interface Breakpoint {
  /** Breakpoint ID */
  id: string;
  
  /** File name */
  file: string;
  
  /** Line number */
  line: number;
  
  /** Column (optional) */
  column?: number;
  
  /** Whether breakpoint is enabled */
  enabled: boolean;
  
  /** Condition (optional) */
  condition?: string;
  
  /** Hit count condition */
  hitCondition?: {
    operator: '==' | '>' | '>=' | '<' | '<=' | '%';
    value: number;
  };
  
  /** Log message (logpoint) */
  logMessage?: string;
  
  /** Current hit count */
  hitCount: number;
}

/**
 * Execution state during debugging
 */
export interface ExecutionState {
  /** Whether execution is paused */
  paused: boolean;
  
  /** Current instruction pointer */
  instructionPointer: number;
  
  /** Current source location */
  currentLocation?: SourceLocation;
  
  /** Call stack */
  callStack: StackFrame[];
  
  /** Local variables */
  variables: Variable[];
  
  /** VM state */
  vmState: 'HALT' | 'FAULT' | 'BREAK';
  
  /** Execution result */
  result?: any;
  
  /** Exception (if any) */
  exception?: string;
  
  /** Gas consumed */
  gasConsumed: number;
  
  /** Notifications emitted */
  notifications: DebugNotification[];
}

/**
 * Stack frame for call stack
 */
export interface StackFrame {
  /** Frame ID */
  id: string;
  
  /** Function name */
  name: string;
  
  /** Source location */
  location: SourceLocation;
  
  /** Local variables */
  variables: Variable[];
  
  /** Function parameters */
  parameters: Variable[];
  
  /** Instruction pointer */
  instructionPointer: number;
  
  /** Whether this is the current frame */
  current: boolean;
}

/**
 * Variable information
 */
export interface Variable {
  /** Variable name */
  name: string;
  
  /** Variable type */
  type: string;
  
  /** Variable value */
  value: any;
  
  /** Formatted value for display */
  displayValue: string;
  
  /** Whether variable is readable */
  readable: boolean;
  
  /** Whether variable is writable */
  writable: boolean;
  
  /** Memory location */
  location?: {
    type: 'storage' | 'memory' | 'stack';
    address: string;
  };
  
  /** Child variables (for structs/arrays) */
  children?: Variable[];
}

/**
 * Debug notification
 */
export interface DebugNotification {
  /** Contract hash */
  contract: string;
  
  /** Event name */
  eventName: string;
  
  /** Event parameters */
  parameters: Variable[];
  
  /** Execution context */
  context: {
    instructionPointer: number;
    gasConsumed: number;
    sourceLocation?: SourceLocation;
  };
}

/**
 * Debug configuration
 */
export interface DebugConfig {
  /** Whether to pause on exceptions */
  pauseOnExceptions: boolean;
  
  /** Whether to pause on entry */
  pauseOnEntry: boolean;
  
  /** Whether to step over library calls */
  stepOverLibraryCalls: boolean;
  
  /** Maximum call stack depth */
  maxCallStackDepth: number;
  
  /** Variable inspection depth */
  variableInspectionDepth: number;
  
  /** Gas limit for debugging */
  gasLimit: number;
  
  /** Timeout for debugging operations */
  timeout: number;
}

/**
 * Debug adapter interface
 */
export interface DebugAdapter {
  /** Start debug session */
  startSession(config: DebugConfig): Promise<DebugSession>;
  
  /** Stop debug session */
  stopSession(sessionId: string): Promise<void>;
  
  /** Set breakpoint */
  setBreakpoint(sessionId: string, breakpoint: Breakpoint): Promise<void>;
  
  /** Remove breakpoint */
  removeBreakpoint(sessionId: string, breakpointId: string): Promise<void>;
  
  /** Continue execution */
  continue(sessionId: string): Promise<void>;
  
  /** Step over */
  stepOver(sessionId: string): Promise<void>;
  
  /** Step into */
  stepInto(sessionId: string): Promise<void>;
  
  /** Step out */
  stepOut(sessionId: string): Promise<void>;
  
  /** Pause execution */
  pause(sessionId: string): Promise<void>;
  
  /** Evaluate expression */
  evaluate(sessionId: string, expression: string, frameId?: string): Promise<Variable>;
  
  /** Get variables */
  getVariables(sessionId: string, scope: 'local' | 'global' | 'storage'): Promise<Variable[]>;
  
  /** Get call stack */
  getCallStack(sessionId: string): Promise<StackFrame[]>;
}

/**
 * Debug event types
 */
export type DebugEvent = 
  | 'sessionStarted'
  | 'sessionStopped' 
  | 'paused'
  | 'continued'
  | 'stepCompleted'
  | 'breakpointHit'
  | 'exception'
  | 'output'
  | 'terminated';

/**
 * Debug event handler
 */
export interface DebugEventHandler {
  /** Handle debug event */
  handle(event: DebugEvent, data: any): void;
}

/**
 * Profiling data
 */
export interface ProfilingData {
  /** Total execution time */
  totalTime: number;
  
  /** Gas consumption by operation */
  gasProfile: {
    [operation: string]: {
      count: number;
      totalGas: number;
      averageGas: number;
    };
  };
  
  /** Function call profile */
  functionProfile: {
    [functionName: string]: {
      callCount: number;
      totalTime: number;
      averageTime: number;
      gasUsed: number;
    };
  };
  
  /** Memory usage profile */
  memoryProfile: {
    maxUsage: number;
    allocations: number;
    deallocations: number;
  };
  
  /** Storage access profile */
  storageProfile: {
    reads: number;
    writes: number;
    totalCost: number;
  };
}

/**
 * Test case for debugging
 */
export interface DebugTestCase {
  /** Test name */
  name: string;
  
  /** Test description */
  description: string;
  
  /** Contract to test */
  contract: string;
  
  /** Method to call */
  method: string;
  
  /** Method parameters */
  parameters: any[];
  
  /** Expected result */
  expectedResult?: any;
  
  /** Expected events */
  expectedEvents?: DebugNotification[];
  
  /** Setup operations */
  setup?: DebugOperation[];
  
  /** Teardown operations */
  teardown?: DebugOperation[];
}

/**
 * Debug operation
 */
export interface DebugOperation {
  /** Operation type */
  type: 'call' | 'deploy' | 'transfer' | 'storage';
  
  /** Operation parameters */
  parameters: any;
}