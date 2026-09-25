// Core Types for Neo DevPack for Solidity Tooling
export * from './compiler';
export * from './contracts';
export * from './networks';
export * from './artifacts';
export * from './abi';
export * from './cli';
export * from './hardhat-compat';
export * from './templates';
export * from './neo';

// Namespaced modules expose specialized type groups without polluting the global namespace
export * as debuggerTypes from './debugger';
export * as foundryTypes from './foundry';
export * as hardhatTypes from './hardhat';
export * as profilerTypes from './profiler';
export * as rpcTypes from './rpc';
export * as sourcemapTypes from './sourcemap';
export * as verificationTypes from './verification';

// Commonly consumed submodules (safe named re-exports).
export type { NeoHardhatConfig, DeploymentOptions, DeploymentResult } from './hardhat';
export type { NeoNetworkConfig as NeoHardhatNetworkConfig } from './hardhat';
export type { NeoRpcProvider, InvokeResult, ContractManifest } from './rpc';

export type {
  GasProfiler,
  GasProfile,
  TransactionProfile,
  ContractProfile,
  FunctionCallProfile,
  StorageProfile,
  OptimizationSuggestion,
  GasBreakdown,
  ExecutionTrace,
  ReportFormat,
  PerformanceProfiler,
  PerformanceProfile,
} from './profiler';

// Verification types are exported with aliases to avoid clashing with `contracts.VerificationResult`.
export type {
  ContractVerifier,
  MultiVerifier,
  VerificationRequest,
  VerificationStatus,
  SourceCodeResult,
  SimilarContract,
  VerificationCache,
  BytecodeAnalyzer,
  BytecodeAnalysis,
  ComparisonResult,
  DecompiledCode,
  VerificationPipeline,
  VerificationStep,
  VerificationContext,
  StepResult,
  BytecodeMetadata,
  SecurityAnalysis,
} from './verification';
export type { VerificationResult as ExplorerVerificationResult } from './verification';
