export interface SourceMap {
  mappings: string;
  sources: string[];
  sourcesContent: string[];
  names: string[];
  version: number;
  file?: string;
  sourceRoot?: string;
}

export interface SourceMapGenerator {
  addMapping(mapping: MappingInfo): void;
  setSourceContent(source: string, content: string): void;
  generate(): SourceMap;
  toString(): string;
}

export interface MappingInfo {
  generated: Position;
  source?: string;
  original?: Position;
  name?: string;
}

export interface Position {
  line: number;
  column: number;
}

export interface SourceMapConsumer {
  originalPositionFor(position: Position): OriginalPosition | null;
  generatedPositionFor(source: string, position: Position): Position | null;
  sourceContentFor(source: string): string | null;
  allGeneratedPositionsFor(source: string, original: Position): Position[];
  eachMapping(callback: (mapping: Mapping) => void): void;
  computeColumnSpans(): void;
}

export interface OriginalPosition {
  source: string;
  line: number;
  column: number;
  name?: string;
}

export interface Mapping {
  generated: Position;
  original?: Position;
  source?: string;
  name?: string;
}

export interface DebugInfo {
  sourceMap: SourceMap;
  pcToSourceMap: Map<number, SourceLocation>;
  sourceToByteMap: Map<string, ByteRange[]>;
  functionDebugData: Map<string, FunctionDebugData>;
  contractDebugData: ContractDebugData;
}

export interface SourceLocation {
  source: string;
  start: number;
  end: number;
  line: number;
  column: number;
  endLine: number;
  endColumn: number;
}

export interface ByteRange {
  start: number;
  end: number;
  pc: number;
  op: string;
}

export interface FunctionDebugData {
  name: string;
  selector: string;
  signature: string;
  entryPoint: number;
  exitPoint: number;
  parameters: ParameterDebugData[];
  localVariables: VariableDebugData[];
  sourceRange: SourceLocation;
}

export interface ParameterDebugData {
  name: string;
  type: string;
  location: 'stack' | 'memory' | 'storage' | 'calldata';
  offset: number;
  size: number;
}

export interface VariableDebugData {
  name: string;
  type: string;
  location: 'stack' | 'memory' | 'storage';
  offset: number;
  size: number;
  scope: SourceLocation;
}

export interface ContractDebugData {
  name: string;
  sourceFiles: string[];
  functions: Map<string, FunctionDebugData>;
  events: Map<string, EventDebugData>;
  stateVariables: Map<string, StateVariableDebugData>;
  constructorDebugData?: FunctionDebugData;
  fallbackDebugData?: FunctionDebugData;
  receiveDebugData?: FunctionDebugData;
}

export interface EventDebugData {
  name: string;
  signature: string;
  topic0: string;
  parameters: EventParameterDebugData[];
  sourceRange: SourceLocation;
}

export interface EventParameterDebugData {
  name: string;
  type: string;
  indexed: boolean;
  size: number;
}

export interface StateVariableDebugData {
  name: string;
  type: string;
  slot: number;
  offset: number;
  size: number;
  constant: boolean;
  immutable: boolean;
  sourceRange: SourceLocation;
}

export interface Debugger {
  loadContract(address: string, debugInfo: DebugInfo): Promise<void>;
  setBreakpoint(source: string, line: number): Promise<Breakpoint>;
  removeBreakpoint(breakpoint: Breakpoint): Promise<void>;
  startDebugging(transactionHash: string): Promise<DebugSession>;
  stepOver(): Promise<DebugState>;
  stepInto(): Promise<DebugState>;
  stepOut(): Promise<DebugState>;
  continue(): Promise<DebugState>;
  getStackTrace(): Promise<StackFrame[]>;
  getLocalVariables(): Promise<Variable[]>;
  getMemory(): Promise<MemoryState>;
  getStorage(): Promise<StorageState>;
  evaluateExpression(expression: string): Promise<any>;
}

export interface Breakpoint {
  id: string;
  source: string;
  line: number;
  column?: number;
  condition?: string;
  enabled: boolean;
  hitCount: number;
}

export interface DebugSession {
  id: string;
  transactionHash: string;
  contractAddress: string;
  debugInfo: DebugInfo;
  currentState: DebugState;
  breakpoints: Breakpoint[];
  callStack: StackFrame[];
}

export interface DebugState {
  pc: number;
  op: string;
  gas: string;
  gasUsed: string;
  depth: number;
  sourceLocation?: SourceLocation;
  stack: string[];
  memory: string[];
  storage: { [key: string]: string };
  calldata: string;
  returndata: string;
  error?: string;
}

export interface StackFrame {
  contractAddress: string;
  contractName: string;
  functionName: string;
  pc: number;
  sourceLocation?: SourceLocation;
}

export interface Variable {
  name: string;
  type: string;
  value: any;
  location: 'stack' | 'memory' | 'storage' | 'calldata';
  offset: number;
  size: number;
}

export interface MemoryState {
  size: number;
  data: string;
  words: string[];
  allocatedSize: number;
}

export interface StorageState {
  slots: { [slot: string]: string };
  layout: StorageLayout;
}

export interface StorageLayout {
  storage: StorageSlot[];
  types: { [typeName: string]: StorageType };
}

export interface StorageSlot {
  astId: number;
  contract: string;
  label: string;
  offset: number;
  slot: string;
  type: string;
}

export interface StorageType {
  encoding: string;
  label: string;
  numberOfBytes: string;
  key?: string;
  value?: string;
  base?: string;
  members?: StorageSlot[];
}

export interface TraceAnalyzer {
  analyzeTransaction(txHash: string): Promise<TransactionTrace>;
  generateCallGraph(trace: TransactionTrace): Promise<CallGraph>;
  findPerformanceBottlenecks(trace: TransactionTrace): Promise<PerformanceIssue[]>;
  detectPatterns(trace: TransactionTrace): Promise<Pattern[]>;
}

export interface TransactionTrace {
  hash: string;
  from: string;
  to: string;
  value: string;
  gasUsed: string;
  gasPrice: string;
  status: 'success' | 'failure' | 'reverted';
  steps: TraceStep[];
  calls: InternalCall[];
  events: TraceEvent[];
  stateChanges: StateChange[];
}

export interface TraceStep {
  pc: number;
  op: string;
  gas: string;
  gasCost: string;
  depth: number;
  stack: string[];
  memory: string[];
  storage: { [key: string]: string };
  sourceLocation?: SourceLocation;
}

export interface InternalCall {
  type: 'call' | 'delegatecall' | 'staticcall' | 'create' | 'create2';
  from: string;
  to: string;
  value: string;
  gas: string;
  gasUsed: string;
  input: string;
  output: string;
  success: boolean;
  error?: string;
  calls: InternalCall[];
}

export interface TraceEvent {
  address: string;
  topics: string[];
  data: string;
  decoded?: DecodedEvent;
}

export interface DecodedEvent {
  name: string;
  args: { [key: string]: any };
  signature: string;
}

export interface StateChange {
  address: string;
  slot: string;
  before: string;
  after: string;
  variable?: string;
}

export interface CallGraph {
  nodes: CallGraphNode[];
  edges: CallGraphEdge[];
  root: string;
}

export interface CallGraphNode {
  id: string;
  address: string;
  contractName?: string;
  functionName?: string;
  gasUsed: string;
  calls: number;
}

export interface CallGraphEdge {
  from: string;
  to: string;
  type: 'call' | 'delegatecall' | 'staticcall';
  gasUsed: string;
  weight: number;
}

export interface PerformanceIssue {
  type: 'gas_inefficiency' | 'infinite_loop' | 'memory_expansion' | 'storage_access';
  severity: 'low' | 'medium' | 'high' | 'critical';
  description: string;
  location: SourceLocation;
  gasImpact: string;
  recommendation: string;
}

export interface Pattern {
  type: 'reentrancy' | 'access_control' | 'state_manipulation' | 'gas_optimization';
  confidence: number;
  description: string;
  locations: SourceLocation[];
  risk: 'low' | 'medium' | 'high';
  mitigation: string;
}