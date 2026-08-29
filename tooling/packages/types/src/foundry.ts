export interface FoundryConfig {
  src: string;
  out: string;
  libs: string[];
  remappings: string[];
  auto_detect_solc: boolean;
  solc_version: string;
  optimizer: boolean;
  optimizer_runs: number;
  via_ir: boolean;
  verbosity: number;
  evm_version: string;
  neo?: {
    rpc_url: string;
    private_key: string;
    chain_id: number;
    gas_limit: string;
    gas_price: string;
  };
}

export interface ForgeProject {
  name: string;
  root: string;
  src: string;
  out: string;
  libs: string[];
  test: string;
  cache: string;
  artifacts: string;
  config: FoundryConfig;
}

export interface ForgeBuildOptions {
  root?: string;
  contracts?: string[];
  compiler_version?: string;
  evm_version?: string;
  optimizer?: boolean;
  optimizer_runs?: number;
  via_ir?: boolean;
  out?: string;
  cache?: boolean;
  force?: boolean;
  watch?: boolean;
  names?: boolean;
  sizes?: boolean;
  offline?: boolean;
}

export interface ForgeBuildResult {
  success: boolean;
  contracts: {
    [contractPath: string]: {
      [contractName: string]: {
        abi: any[];
        bytecode: {
          object: string;
          sourceMap: string;
          linkReferences: any;
        };
        deployedBytecode: {
          object: string;
          sourceMap: string;
          linkReferences: any;
        };
        metadata: string;
        storageLayout: {
          storage: StorageSlot[];
          types: { [typeName: string]: StorageType };
        };
      };
    };
  };
  compilation_time: number;
  compiler_version: string;
  errors: CompilationMessage[];
  warnings: CompilationMessage[];
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

export interface CompilationMessage {
  severity: 'error' | 'warning' | 'info';
  type: string;
  component: string;
  message: string;
  formattedMessage: string;
  sourceLocation?: {
    file: string;
    start: number;
    end: number;
  };
}

export interface ForgeTestOptions {
  root?: string;
  contracts?: string[];
  match_contract?: string;
  no_match_contract?: string;
  match_path?: string;
  no_match_path?: string;
  match_test?: string;
  no_match_test?: string;
  fork_url?: string;
  fork_block_number?: number;
  verbosity?: number;
  gas_report?: boolean;
  coverage?: boolean;
  json?: boolean;
  etherscan_api_key?: string;
}

export interface ForgeTestResult {
  success: boolean;
  test_results: {
    [testSuite: string]: {
      [testName: string]: {
        success: boolean;
        reason?: string;
        counterexample?: any;
        gas_used: number;
        duration: number;
        logs: string[];
      };
    };
  };
  gas_report?: GasUsageReport;
  coverage?: CoverageReport;
  summary: {
    total_tests: number;
    passed: number;
    failed: number;
    skipped: number;
    duration: number;
    gas_used: number;
  };
}

export interface GasUsageReport {
  contracts: {
    [contractName: string]: {
      functions: {
        [functionName: string]: {
          min: number;
          max: number;
          avg: number;
          calls: number;
        };
      };
      deployments: {
        min: number;
        max: number;
        avg: number;
        calls: number;
      };
    };
  };
}

export interface CoverageReport {
  files: {
    [fileName: string]: {
      lines: {
        [lineNumber: string]: number;
      };
      functions: {
        [functionName: string]: number;
      };
      branches: {
        [branchId: string]: number;
      };
      statements: {
        [statementId: string]: number;
      };
    };
  };
  summary: {
    lines: CoverageSummary;
    functions: CoverageSummary;
    branches: CoverageSummary;
    statements: CoverageSummary;
  };
}

export interface CoverageSummary {
  total: number;
  covered: number;
  skipped: number;
  pct: number;
}

export interface CastCallOptions {
  rpc_url?: string;
  block?: string | number;
  from?: string;
  gas?: string;
  gas_price?: string;
  value?: string;
  private_key?: string;
  mnemonic?: string;
  mnemonic_path?: string;
  keystore?: string;
  password?: string;
  interactive?: boolean;
  ledger?: boolean;
}

export interface CastCallResult {
  success: boolean;
  result?: string;
  error?: {
    code: number;
    message: string;
    data?: any;
  };
  gas_used?: string;
  gas_limit?: string;
  transaction_hash?: string;
}

export interface CastSendOptions extends CastCallOptions {
  nonce?: number;
  confirmations?: number;
  timeout?: number;
  json?: boolean;
  async?: boolean;
}

export interface CastSendResult {
  success: boolean;
  transaction_hash?: string;
  block_number?: number;
  gas_used?: string;
  gas_price?: string;
  status?: string;
  error?: {
    code: number;
    message: string;
    data?: any;
  };
  logs?: any[];
  receipt?: any;
}

export interface AnvilOptions {
  port?: number;
  host?: string;
  accounts?: number;
  mnemonic?: string;
  derivation_path?: string;
  balance?: string;
  timestamp?: number;
  gas_limit?: string;
  gas_price?: string;
  block_time?: number;
  fork_url?: string;
  fork_block_number?: number;
  chain_id?: number;
  hardfork?: string;
  steps_tracing?: boolean;
  tracing?: boolean;
  debug?: boolean;
  silent?: boolean;
  dump_state?: string;
  load_state?: string;
}

export interface AnvilInstance {
  pid: number;
  port: number;
  host: string;
  rpc_url: string;
  accounts: AnvilAccount[];
  private_keys: string[];
  mnemonic?: string;
  base_fee: string;
  gas_price: string;
  gas_limit: string;
  chain_id: number;
  block_time?: number;
}

export interface AnvilAccount {
  address: string;
  private_key: string;
  balance: string;
  index: number;
}

export interface ScriptOptions {
  target_contract?: string;
  sig?: string;
  args?: string[];
  rpc_url?: string;
  private_key?: string;
  broadcast?: boolean;
  verify?: boolean;
  etherscan_api_key?: string;
  resume?: boolean;
  multi?: boolean;
  json?: boolean;
  debug?: boolean;
  gas_estimate_multiplier?: number;
  legacy?: boolean;
}

export interface ScriptResult {
  success: boolean;
  transactions: ScriptTransaction[];
  receipts: any[];
  gas_used: string;
  logs: string[];
  error?: {
    message: string;
    stack?: string;
  };
}

export interface ScriptTransaction {
  type: string;
  to: string;
  value: string;
  gas: string;
  gas_price: string;
  data: string;
  nonce: number;
  transaction_hash?: string;
  block_number?: number;
  status?: string;
}

export interface VerifyOptions {
  contract?: string;
  address: string;
  constructor_args?: string;
  constructor_args_path?: string;
  compiler_version?: string;
  num_of_optimizations?: number;
  etherscan_api_key?: string;
  chain_id?: number;
  verifier?: string;
  verifier_url?: string;
  flatten?: boolean;
  show_standard_json_input?: boolean;
}

export interface VerifyResult {
  success: boolean;
  guid?: string;
  status?: string;
  message?: string;
  error?: {
    message: string;
    details?: any;
  };
}

export interface FlattenOptions {
  root?: string;
  output?: string;
  wrap?: boolean;
  remove_version_pragma?: boolean;
  remove_assert?: boolean;
}

export interface FlattenResult {
  success: boolean;
  flattened_source: string;
  contracts_found: string[];
  imports_resolved: string[];
  errors: string[];
  warnings: string[];
}