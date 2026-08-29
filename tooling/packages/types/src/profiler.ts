export interface GasProfiler {
  startProfiling(): Promise<void>;
  stopProfiling(): Promise<GasProfile>;
  profileTransaction(txHash: string): Promise<TransactionProfile>;
  profileContract(address: string): Promise<ContractProfile>;
  generateReport(profile: GasProfile, format?: ReportFormat): Promise<string>;
}

export interface GasProfile {
  id: string;
  startTime: Date;
  endTime: Date;
  duration: number;
  totalGasUsed: string;
  totalCost: string;
  averageGasPrice: string;
  transactions: TransactionProfile[];
  contracts: ContractProfile[];
  summary: ProfileSummary;
  optimizations: OptimizationSuggestion[];
}

export interface TransactionProfile {
  hash: string;
  from: string;
  to: string;
  value: string;
  gasUsed: string;
  gasPrice: string;
  gasLimit: string;
  cost: string;
  status: 'success' | 'failure' | 'reverted';
  blockNumber: number;
  timestamp: Date;
  functionCalled?: string;
  inputs?: any[];
  outputs?: any[];
  events: EventProfile[];
  gasBreakdown: GasBreakdown;
  trace?: ExecutionTrace;
}

// Neo N3 shape. A Neo node exposes per-transaction GAS consumed
// (getapplicationlog) but not EVM-style deployment-cost / code-size /
// storage-slot accounting, so the contract profile is aggregated from the
// transactions attributed to the contract.
export interface ContractProfile {
  address: string;
  name?: string;
  totalGasUsed: string;
  transactionCount: number;
  averageGasPerTransaction: string;
  functionProfiles: FunctionCallProfile[];
}

export interface FunctionCallProfile {
  functionName: string;
  selector: string;
  callCount: number;
  totalGasUsed: string;
  averageGasUsed: string;
  minGasUsed: string;
  maxGasUsed: string;
  gasDistribution: GasDistribution;
  parameters: ParameterProfile[];
  optimizations: FunctionOptimization[];
}

export interface StorageProfile {
  slotsUsed: number;
  totalReads: number;
  totalWrites: number;
  coldReads: number;
  warmReads: number;
  coldWrites: number;
  warmWrites: number;
  storageLayout: StorageLayoutProfile[];
}

export interface StorageLayoutProfile {
  slot: number;
  offset: number;
  type: string;
  name: string;
  size: number;
  reads: number;
  writes: number;
  gasPerRead: string;
  gasPerWrite: string;
}

// Neo N3 shape: a notification is an event name plus a NeoVM stack-item
// state array (no EVM indexed-topics / data / logIndex split).
export interface EventProfile {
  name: string;
  args: any[];
}

// Neo N3 shape: a Neo node reports only the total GAS consumed per
// execution (getapplicationlog), not the EVM per-category split
// (intrinsic / storage / refund / …). `byCategory` is kept open for any
// coarse attribution a caller can derive.
export interface GasBreakdown {
  total: string;
  byCategory: Record<string, string>;
  summary: string;
}

// Neo N3 shape: Neo nodes do not expose per-opcode traces by default, so a
// trace carries whatever steps are available plus the aggregate breakdown.
export interface ExecutionTrace {
  steps: TraceStep[];
  gasBreakdown: GasBreakdown;
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
  refund: string;
  error?: string;
}

export interface GasDistribution {
  p10: string;
  p25: string;
  p50: string;
  p75: string;
  p90: string;
  p95: string;
  p99: string;
  stddev: string;
}

export interface ParameterProfile {
  name: string;
  type: string;
  gasImpact: string;
  frequency: number;
  values: any[];
}

export interface ProfileSummary {
  totalTransactions: number;
  successfulTransactions: number;
  failedTransactions: number;
  totalGasUsed: string;
  totalCost: string;
  averageTransactionCost: string;
  gasEfficiency: number;
  topGasConsumers: GasConsumer[];
  patterns: UsagePattern[];
}

export interface GasConsumer {
  type: 'contract' | 'function' | 'operation';
  name: string;
  address?: string;
  gasUsed: string;
  percentage: number;
  callCount: number;
}

export interface UsagePattern {
  pattern: string;
  frequency: number;
  gasUsed: string;
  description: string;
}

export interface OptimizationSuggestion {
  type: 'storage' | 'computation' | 'memory' | 'call' | 'loop' | 'execution';
  severity: 'low' | 'medium' | 'high' | 'critical';
  title: string;
  description: string;
  location: {
    contract?: string;
    function?: string;
    line?: number;
  };
  currentCost: string;
  potentialSavings: string;
  implementation: string;
  codeExample?: string;
}

export interface FunctionOptimization {
  type: 'parameter_packing' | 'storage_layout' | 'loop_unroll' | 'inline' | 'cache';
  description: string;
  gasImpact: string;
  difficulty: 'easy' | 'medium' | 'hard';
  risk: 'low' | 'medium' | 'high';
}

export interface PerformanceProfiler {
  startSession(): Promise<ProfileSession>;
  endSession(sessionId: string): Promise<PerformanceProfile>;
  measureFunction(functionName: string): Promise<FunctionMeasurement>;
  measureBlock(blockRange: [number, number]): Promise<BlockMeasurement>;
  compareBenchmarks(benchmark1: string, benchmark2: string): Promise<BenchmarkComparison>;
}

export interface ProfileSession {
  id: string;
  startTime: Date;
  network: string;
  blockNumber: number;
  gasPrice: string;
}

export interface PerformanceProfile {
  sessionId: string;
  duration: number;
  measurements: FunctionMeasurement[];
  blockMeasurements: BlockMeasurement[];
  networkMetrics: NetworkMetrics;
  systemMetrics: SystemMetrics;
  bottlenecks: PerformanceBottleneck[];
  recommendations: PerformanceRecommendation[];
}

export interface FunctionMeasurement {
  functionName: string;
  contractAddress: string;
  executionTime: number;
  gasUsed: string;
  gasEfficiency: number;
  memoryUsage: number;
  cpuUsage: number;
  callCount: number;
  errorRate: number;
  throughput: number;
}

export interface BlockMeasurement {
  blockNumber: number;
  blockTime: number;
  transactionCount: number;
  gasUsed: string;
  gasLimit: string;
  utilization: number;
  averageGasPrice: string;
  processingTime: number;
}

export interface NetworkMetrics {
  latency: number;
  throughput: number;
  errorRate: number;
  connectionPool: number;
  requestsPerSecond: number;
  responseTime: {
    average: number;
    p95: number;
    p99: number;
  };
}

export interface SystemMetrics {
  cpuUsage: number;
  memoryUsage: number;
  diskUsage: number;
  networkIO: number;
  processCount: number;
  threadCount: number;
}

export interface PerformanceBottleneck {
  type: 'gas' | 'memory' | 'network' | 'computation' | 'storage';
  location: string;
  severity: number;
  description: string;
  impact: string;
  suggestion: string;
}

export interface PerformanceRecommendation {
  category: 'optimization' | 'scaling' | 'caching' | 'architecture';
  priority: 'low' | 'medium' | 'high' | 'critical';
  title: string;
  description: string;
  implementation: string;
  expectedImprovement: string;
  effort: 'low' | 'medium' | 'high';
}

export interface BenchmarkComparison {
  benchmark1: string;
  benchmark2: string;
  metrics: {
    gasUsed: ComparisonMetric;
    executionTime: ComparisonMetric;
    cost: ComparisonMetric;
    efficiency: ComparisonMetric;
  };
  regression: boolean;
  improvement: boolean;
  significantChanges: SignificantChange[];
}

export interface ComparisonMetric {
  before: number;
  after: number;
  change: number;
  changePercent: number;
  significant: boolean;
}

export interface SignificantChange {
  metric: string;
  change: number;
  impact: 'positive' | 'negative' | 'neutral';
  severity: 'minor' | 'moderate' | 'major';
  description: string;
}

export type ReportFormat = 'json' | 'html' | 'csv' | 'pdf' | 'markdown';

export interface ReportGenerator {
  generateGasReport(profile: GasProfile, format: ReportFormat): Promise<string>;
  generatePerformanceReport(profile: PerformanceProfile, format: ReportFormat): Promise<string>;
  generateComparisonReport(comparison: BenchmarkComparison, format: ReportFormat): Promise<string>;
  generateCustomReport(data: any, template: string, format: ReportFormat): Promise<string>;
}