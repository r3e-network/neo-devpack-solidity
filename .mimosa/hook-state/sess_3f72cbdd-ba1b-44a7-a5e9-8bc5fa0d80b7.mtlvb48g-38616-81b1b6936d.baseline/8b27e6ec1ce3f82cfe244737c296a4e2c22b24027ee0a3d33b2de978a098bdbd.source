/**
 * Hardhat plugin-specific types
 */

/**
 * Plugin configuration
 */
export interface HardhatNeoSolcConfig {
  /** Compiler settings */
  compiler: {
    version: string;
    settings: any;
  };
  
  /** Artifact storage settings */
  artifacts: {
    path: string;
    clear: boolean;
  };
  
  /** Network-specific settings */
  networks: {
    [networkName: string]: {
      url: string;
      chainId: number;
    };
  };
}

/**
 * Task arguments for compilation
 */
export interface CompileTaskArgs {
  force: boolean;
  quiet: boolean;
}

/**
 * Task arguments for cleaning
 */
export interface CleanTaskArgs {
  deployments: boolean;
  cache: boolean;
  all: boolean;
}

/**
 * Task arguments for verification
 */
export interface VerifyTaskArgs {
  contract: string;
  address: string;
  network?: string;
  constructorArgs?: string;
}
