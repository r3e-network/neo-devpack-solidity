/**
 * Supported Solidity version range for the Neo DevPack for Solidity compiler.
 * Keep this in sync with the Rust compiler (`src/frontend/frontend_parse/semver.rs`).
 */
export const SUPPORTED_SOLIDITY_RANGE = ">=0.8.19 <0.8.28";

/** Minimum supported Solidity version (inclusive). */
export const MIN_SUPPORTED_SOLIDITY_VERSION = "0.8.19";

/** Maximum excluded Solidity version (exclusive). */
export const MAX_EXCLUDED_SOLIDITY_VERSION = "0.8.28";

/**
 * Validate that a Solidity version falls within the supported range.
 * Accepts plain versions ("0.8.19"), caret ranges ("^0.8.19"), or full range strings.
 */
export function isSupportedSolidityVersion(version: string): boolean {
  const match = version.match(/(\d+\.\d+\.\d+)/);
  if (!match) {
    return false;
  }
  const v = match[1];
  return (
    compareSemver(v, MIN_SUPPORTED_SOLIDITY_VERSION) >= 0 &&
    compareSemver(v, MAX_EXCLUDED_SOLIDITY_VERSION) < 0
  );
}

function compareSemver(a: string, b: string): number {
  const parse = (v: string) => v.split(".").map((n) => parseInt(n, 10));
  const pa = parse(a);
  const pb = parse(b);
  for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
    const na = pa[i] ?? 0;
    const nb = pb[i] ?? 0;
    if (na !== nb) {
      return na - nb;
    }
  }
  return 0;
}

/**
 * Configuration for the Neo DevPack for Solidity compiler
 */
export interface NeoDevpackSolidityConfig {
  /** Compiler version to use */
  version?: string;
  /** Optimization settings */
  optimizer?: {
    enabled: boolean;
    runs: number;
  };
  /** Output selection for compilation artifacts */
  outputSelection?: {
    [file: string]: {
      [contract: string]: string[];
    };
  };
  /** Libraries to link */
  libraries?: {
    [libraryName: string]: string;
  };
  /** Metadata settings */
  metadata?: {
    useLiteralContent?: boolean;
    bytecodeHash?: 'none' | 'ipfs' | 'bzzr1';
  };
  /** Neo-specific settings */
  neo?: {
    /** Gas cost model configuration */
    gasCostModel?: 'ethereum' | 'neo' | 'hybrid';
    /** Storage cost optimization */
    storageOptimization?: boolean;
    /** Event optimization for Neo notifications */
    eventOptimization?: boolean;
  };
}

/**
 * Compilation input format
 */
export interface CompilationInput {
  language: 'Solidity';
  sources: {
    [fileName: string]: {
      content?: string;
      keccak256?: string;
      urls?: string[];
    };
  };
  settings: NeoDevpackSolidityConfig;
}

/**
 * Compilation output format
 */
export interface CompilationOutput {
  errors?: CompilationError[];
  sources: {
    [fileName: string]: {
      id: number;
      keccak256?: string;
      ast?: any;
    };
  };
  contracts: {
    [fileName: string]: {
      [contractName: string]: CompiledContract;
    };
  };
}

/**
 * Individual compiled contract
 */
export interface CompiledContract {
  abi: any[];
  metadata: string;
  userdoc?: any;
  devdoc?: any;
  ir?: string;
  irOptimized?: string;
  storageLayout?: StorageLayout;
  evm: {
    assembly?: string;
    legacyAssembly?: any;
    bytecode: Bytecode;
    deployedBytecode?: Bytecode;
    methodIdentifiers?: {
      [signature: string]: string;
    };
    gasEstimates?: GasEstimates;
  };
  /** Neo-specific compilation outputs */
  neo: {
    /** Neo VM bytecode */
    nef: {
      /** NEF magic; for Neo N3 this is typically "NEF3" */
      magic: string;
      compiler: string;
      source: string;
      tokens: NeoMethodToken[];
      /** Script hex without 0x prefix */
      script: string;
      /** Full NEF file bytes as hex (no 0x prefix) */
      image: string;
      /** NEF checksum as 8 hex chars (no 0x prefix) */
      checksum: string;
    };
    /** Neo manifest */
    manifest: {
      name: string;
      groups: any[];
      features: any;
      supportedstandards: string[];
      abi: NeoAbi;
      permissions: any[];
      trusts: any[];
      extra: any;
    };
    /** Solidity signature -> Neo invocation name map for exported methods */
    methodMap?: {
      [signature: string]: string;
    };
    /** Storage layout for Neo storage */
    storageMap: {
      [key: string]: {
        slot: number;
        type: string;
        description: string;
      };
    };
    /** Gas cost estimates for Neo */
    gasEstimates: {
      creation: NeoGasEstimate;
      functions: {
        [methodName: string]: NeoGasEstimate;
      };
    };
  };
}

export interface NeoMethodToken {
  /** Contract hash as 0x-prefixed big-endian hex */
  hash: string;
  method: string;
  paramcount: number;
  hasreturnvalue: boolean;
  callflags: number;
}

/**
 * Neo-specific ABI format
 */
export interface NeoAbi {
  methods: NeoMethod[];
  events: NeoEvent[];
}

export interface NeoMethod {
  name: string;
  offset: number;
  parameters: NeoParameter[];
  returntype: string;
  safe: boolean;
}

export interface NeoEvent {
  name: string;
  parameters: NeoParameter[];
}

export interface NeoParameter {
  name: string;
  type: string;
}

/**
 * Neo-specific gas estimation
 */
export interface NeoGasEstimate {
  /** Stringified integer (GAS in fractions) as emitted by standard-json output */
  gas: string;
  systemFee: string;
  networkFee: string;
}

/**
 * Bytecode information
 */
export interface Bytecode {
  object: string;
  opcodes?: string;
  sourceMap?: string;
  linkReferences?: {
    [fileName: string]: {
      [libraryName: string]: Array<{
        start: number;
        length: number;
      }>;
    };
  };
  generatedSources?: GeneratedSource[];
}

/**
 * Generated source information
 */
export interface GeneratedSource {
  ast: any;
  contents: string;
  id: number;
  language: string;
  name: string;
}

/**
 * Storage layout information
 */
export interface StorageLayout {
  storage: StorageItem[];
  types: {
    [typeName: string]: StorageType;
  };
}

export interface StorageItem {
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
  base?: string;
  key?: string;
  value?: string;
  members?: StorageItem[];
}

/**
 * Gas estimation information
 */
export interface GasEstimates {
  creation?: {
    codeDepositCost: string;
    executionCost: string;
    totalCost: string;
  };
  external?: {
    [methodName: string]: string;
  };
  internal?: {
    [methodName: string]: string;
  };
}

/**
 * Compilation error
 */
export interface CompilationError {
  sourceLocation?: {
    file: string;
    start?: number;
    end?: number;
  };
  secondarySourceLocations?: Array<{
    file: string;
    start?: number;
    end?: number;
    message: string;
  }>;
  type: string;
  component: string;
  severity: 'error' | 'warning' | 'info';
  message: string;
  code?: string;
  formattedMessage?: string;
}

/**
 * Compiler options for CLI tools
 */
export interface CompilerOptions {
  /** Input files or directories */
  input: string[];
  /** Output directory */
  output?: string;
  /** Compiler configuration */
  config?: NeoDevpackSolidityConfig;
  /** Include remappings */
  remappings?: string[];
  /** Base path for imports */
  basePath?: string;
  /** Include paths for imports */
  includePaths?: string[];
  /** Allow paths for imports */
  allowPaths?: string[];
  /** Overwrite output files */
  overwrite?: boolean;
  /** Verbose output */
  verbose?: boolean;
  /** Emit only specific outputs */
  emit?: string[];
}
