import { CompiledContract } from './compiler';

/**
 * Build artifact for a compiled contract
 */
export interface BuildArtifact {
  /** Contract name */
  contractName: string;
  /** Source file name */
  sourceName: string;
  /** Compilation metadata */
  metadata: ArtifactMetadata;
  /** Compiled contract data */
  contract: CompiledContract;
  /** Build info reference */
  buildInfo: string;
}

/**
 * Artifact metadata
 */
export interface ArtifactMetadata {
  /** Compiler version used */
  compiler: {
    version: string;
    settings: any;
  };
  /** Build timestamp */
  buildTime: string;
  /** Git commit hash (if available) */
  gitCommit?: string;
  /** Build environment */
  environment: {
    nodeVersion: string;
    platform: string;
    architecture: string;
  };
  /** Dependencies used during compilation */
  dependencies: {
    [packageName: string]: string;
  };
}

/**
 * Build info containing compilation details
 */
export interface BuildInfo {
  /** Build ID */
  id: string;
  /** Solidity version used */
  solcVersion: string;
  /** Neo DevPack for Solidity compiler version */
  neoSolcVersion: string;
  /** Input to compiler */
  input: any;
  /** Output from compiler */
  output: any;
  /** Build metadata */
  metadata: BuildMetadata;
}

/**
 * Build metadata
 */
export interface BuildMetadata {
  /** Build timestamp */
  timestamp: string;
  /** Build duration (ms) */
  duration: number;
  /** Source files involved */
  sourceFiles: string[];
  /** Optimization settings */
  optimization: {
    enabled: boolean;
    runs: number;
  };
  /** Neo-specific settings */
  neo: {
    gasCostModel: string;
    storageOptimization: boolean;
    eventOptimization: boolean;
  };
}

/**
 * Deployment artifact
 */
export interface DeploymentArtifact {
  /** Contract name */
  contractName: string;
  /** Network name */
  networkName: string;
  /** Deployment address */
  address: string;
  /** Script hash */
  scriptHash: string;
  /** Transaction hash */
  transactionHash: string;
  /** Block number */
  blockNumber: number;
  /** Deployment timestamp */
  deployedAt: string;
  /** Deployer address */
  deployedBy: string;
  /** Constructor arguments */
  constructorArgs: any[];
  /** Build artifact reference */
  buildArtifact: string;
  /** Verification status */
  verified: boolean;
  /** Verification details */
  verification?: {
    status: 'pending' | 'verified' | 'failed';
    explorerUrl?: string;
    sourceCodeUrl?: string;
  };
}

/**
 * Artifact manager interface
 */
export interface ArtifactManager {
  /** Get build artifact by name */
  getBuildArtifact(contractName: string): Promise<BuildArtifact | null>;
  
  /** Save build artifact */
  saveBuildArtifact(artifact: BuildArtifact): Promise<void>;
  
  /** Get all build artifacts */
  getAllBuildArtifacts(): Promise<BuildArtifact[]>;
  
  /** Get deployment artifact */
  getDeploymentArtifact(contractName: string, networkName: string): Promise<DeploymentArtifact | null>;
  
  /** Save deployment artifact */
  saveDeploymentArtifact(artifact: DeploymentArtifact): Promise<void>;
  
  /** Get all deployments for a network */
  getNetworkDeployments(networkName: string): Promise<DeploymentArtifact[]>;
  
  /** Clear artifacts */
  clearArtifacts(): Promise<void>;
  
  /** Export artifacts */
  exportArtifacts(outputDir: string): Promise<void>;
  
  /** Import artifacts */
  importArtifacts(inputDir: string): Promise<void>;
}

/**
 * Artifact storage configuration
 */
export interface ArtifactStorageConfig {
  /** Base directory for artifacts */
  baseDir: string;
  /** Build artifacts subdirectory */
  buildDir: string;
  /** Deployment artifacts subdirectory */
  deploymentDir: string;
  /** Build info subdirectory */
  buildInfoDir: string;
  /** Cache directory */
  cacheDir: string;
  /** Compression settings */
  compression: {
    enabled: boolean;
    algorithm: 'gzip' | 'brotli' | 'lz4';
  };
  /** Cleanup settings */
  cleanup: {
    maxAge: number; // days
    maxSize: number; // MB
  };
}

/**
 * Artifact validation result
 */
export interface ArtifactValidationResult {
  /** Whether artifact is valid */
  valid: boolean;
  /** Validation errors */
  errors: string[];
  /** Validation warnings */
  warnings: string[];
  /** Artifact integrity */
  integrity: {
    checksumValid: boolean;
    signatureValid: boolean;
    timestampValid: boolean;
  };
}

/**
 * Artifact comparison result
 */
export interface ArtifactComparison {
  /** Whether artifacts are identical */
  identical: boolean;
  /** Differences found */
  differences: ArtifactDifference[];
  /** Compatibility assessment */
  compatibility: {
    /** Whether upgradeable */
    upgradeable: boolean;
    /** Breaking changes */
    breakingChanges: string[];
    /** Non-breaking changes */
    nonBreakingChanges: string[];
  };
}

/**
 * Individual artifact difference
 */
export interface ArtifactDifference {
  /** Path to differing property */
  path: string;
  /** Type of difference */
  type: 'added' | 'removed' | 'modified';
  /** Old value */
  oldValue?: any;
  /** New value */
  newValue?: any;
  /** Impact level */
  impact: 'low' | 'medium' | 'high' | 'breaking';
}

/**
 * Artifact search criteria
 */
export interface ArtifactSearchCriteria {
  /** Contract name pattern */
  contractName?: string | RegExp;
  /** Network name */
  networkName?: string;
  /** Date range */
  dateRange?: {
    from: Date;
    to: Date;
  };
  /** Compiler version */
  compilerVersion?: string;
  /** Deployment status */
  deploymentStatus?: 'deployed' | 'undeployed' | 'failed';
  /** Verification status */
  verificationStatus?: 'verified' | 'unverified' | 'pending';
  /** Tags */
  tags?: string[];
}

/**
 * Artifact statistics
 */
export interface ArtifactStatistics {
  /** Total number of build artifacts */
  totalBuildArtifacts: number;
  /** Total number of deployment artifacts */
  totalDeploymentArtifacts: number;
  /** Storage usage */
  storageUsage: {
    total: number; // bytes
    buildArtifacts: number;
    deploymentArtifacts: number;
    buildInfo: number;
    cache: number;
  };
  /** Compiler version distribution */
  compilerVersions: {
    [version: string]: number;
  };
  /** Network deployment distribution */
  networkDistribution: {
    [networkName: string]: number;
  };
  /** Recent activity */
  recentActivity: {
    buildsLastWeek: number;
    deploymentsLastWeek: number;
    verificationsLastWeek: number;
  };
}