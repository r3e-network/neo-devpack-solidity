import type { NeoDevpackSolidityConfig, CompilationError } from './compiler';

export type TaskArguments = Record<string, any>;
export interface HardhatRuntimeEnvironment {
  [key: string]: any;
}

export interface NeoHardhatConfig {
  solidity: {
    version: string;
    settings: {
      optimizer: {
        enabled: boolean;
        runs: number;
      };
      outputSelection: {
        [key: string]: {
          [key: string]: string[];
        };
      };
      neo: {
        generateNef: boolean;
        generateManifest: boolean;
        optimizeGas: boolean;
        debugInfo: boolean;
        /** Emit CALLT + method tokens for native calls (neo-solc: --callt) */
        callt?: boolean;
        /** Fail if manifest contains full wildcard permissions (neo-solc: --deny-wildcard-permissions) */
        denyWildcardPermissions?: boolean;
        /** Fail if manifest permissions include contract="*" (neo-solc: --deny-wildcard-contracts) */
        denyWildcardContracts?: boolean;
        /** Fail if manifest permissions include methods="*" (neo-solc: --deny-wildcard-methods) */
        denyWildcardMethods?: boolean;
        /** Path to explicit permissions allowlist JSON (neo-solc: --manifest-permissions) */
        manifestPermissions?: string;
        /** How to apply allowlist permissions (neo-solc: --manifest-permissions-mode) */
        manifestPermissionsMode?: 'merge' | 'replace-wildcards';
        /** Override NEF source field (neo-solc: --nef-source) */
        nefSource?: string;
      };
    };
  };
  networks: { [networkName: string]: NeoNetworkConfig };
  paths: {
    sources: string;
    artifacts: string;
    cache: string;
    tests: string;
  };
  neo: {
    rpcUrl: string;
    privateKey: string;
    addressVersion: number;
    magic: number;
    gasLimit: string;
    gasPrice: string;
  };
}

type CompilerConfig = NeoDevpackSolidityConfig;

export interface NeoNetworkConfig {
  magic: number;
  addressVersion: number;
  rpc: {
    url: string;
    timeout?: number;
    headers?: { [key: string]: string };
  };
  accounts: string[] | {
    mnemonic: string;
    path: string;
    initialIndex: number;
    count: number;
  };
  gasLimit: string;
  gasPrice: string;
  blockGasLimit: string;
  hardfork?: string;
  chainId?: number;
}

export interface NeoHardhatTask {
  name: string;
  description: string;
  action: (args: TaskArguments, hre: HardhatRuntimeEnvironment) => Promise<any>;
  paramDefinitions: {
    [paramName: string]: {
      name: string;
      description: string;
      type: string;
      defaultValue?: any;
      isOptional: boolean;
      isFlag: boolean;
      isVariadic: boolean;
    };
  };
}

export interface CompilationJob {
  contractName: string;
  sourceName: string;
  solidityVersion: string;
  input: {
    language: string;
    sources: {
      [fileName: string]: {
        content: string;
        urls?: string[];
      };
    };
    settings: CompilerConfig;
  };
}

export interface CompilationResult {
  success: boolean;
  contractName: string;
  bytecode: string;
  deployedBytecode: string;
  abi: any[];
  metadata: string;
  nef?: string;
  manifest?: string;
  errors: CompilationError[];
  warnings: CompilationError[];
  gasEstimates?: {
    creation: {
      codeDepositCost: string;
      executionCost: string;
      totalCost: string;
    };
    external: {
      [methodName: string]: string;
    };
  };
  sourceMaps: {
    bytecode: string;
    deployedBytecode: string;
  };
}

export interface DeploymentOptions {
  gasLimit?: string;
  gasPrice?: string;
  value?: string;
  from?: string;
  nonce?: number;
  args?: any[];
  libraries?: { [libraryName: string]: string };
  skipDryRun?: boolean;
  confirmations?: number;
  timeout?: number;
}

export interface DeploymentResult {
  contractName: string;
  contractAddress: string;
  transactionHash: string;
  gasUsed: string;
  gasPrice: string;
  deploymentData: {
    bytecode: string;
    constructorArgs: any[];
    libraries: { [libraryName: string]: string };
  };
  receipt: any;
  deployedAt: Date;
  network: string;
  block: {
    number: number;
    hash: string;
    timestamp: number;
  };
}

export interface TestResult {
  contractName: string;
  testName: string;
  status: 'passed' | 'failed' | 'skipped';
  duration: number;
  gasUsed: string;
  error?: {
    message: string;
    stack?: string;
  };
  events: any[];
  coverage?: {
    statements: number;
    branches: number;
    functions: number;
    lines: number;
  };
}

export interface HardhatPlugin {
  name: string;
  version: string;
  tasks: NeoHardhatTask[];
  extendEnvironment: (hre: HardhatRuntimeEnvironment) => void;
  extendConfig: (config: NeoHardhatConfig) => NeoHardhatConfig;
}

export interface DebugSession {
  contractName: string;
  transactionHash: string;
  breakpoints: Breakpoint[];
  currentLine: number;
  stack: StackFrame[];
  memory: { [address: string]: string };
  storage: { [slot: string]: string };
  trace: TraceStep[];
}

export interface Breakpoint {
  file: string;
  line: number;
  column?: number;
  condition?: string;
  enabled: boolean;
}

export interface StackFrame {
  contractName: string;
  functionName: string;
  file: string;
  line: number;
  column: number;
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
}

export interface VerificationData {
  contractName: string;
  contractAddress: string;
  sourceCode: string;
  abi: any[];
  bytecode: string;
  constructorArguments: string;
  compilerVersion: string;
  optimizationUsed: boolean;
  runs: number;
  libraries?: { [libraryName: string]: string };
}

export interface GasReport {
  contractName: string;
  deploymentGas: {
    min: string;
    max: string;
    avg: string;
    total: string;
  };
  methods: {
    [methodName: string]: {
      min: string;
      max: string;
      avg: string;
      calls: number;
      totalGas: string;
    };
  };
  totalCost: string;
  currency?: {
    symbol: string;
    price: number;
    totalUSD: number;
  };
}
