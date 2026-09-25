// Re-export types for convenience
export * from "@neo-devpack-solidity/types";

/**
 * Hardhat deployer plugin-specific types
 */

/**
 * Deployment task arguments
 */
export interface DeployTaskArgs {
  contract: string;
  args: string;
  from?: string;
  gasLimit?: string;
  verify: boolean;
}

/**
 * Batch deployment task arguments
 */
export interface BatchDeployTaskArgs {
  config: string;
  verify: boolean;
}

/**
 * Account task arguments
 */
export interface AccountTaskArgs {
  balances: boolean;
  private: boolean;
}

/**
 * Balance task arguments
 */
export interface BalanceTaskArgs {
  address: string;
}

/**
 * Import account task arguments
 */
export interface ImportAccountTaskArgs {
  privateKey: string;
  label?: string;
}

/**
 * Export accounts task arguments
 */
export interface ExportAccountsTaskArgs {
  file: string;
  includePrivateKeys: boolean;
}

/**
 * Generate account task arguments
 */
export interface GenerateAccountTaskArgs {
  label?: string;
  save: boolean;
}

/**
 * Set default account task arguments
 */
export interface SetDefaultAccountTaskArgs {
  account: string;
}

/**
 * Deployment configuration file format
 */
export interface DeploymentConfigFile {
  /** Network name */
  network: string;
  
  /** Global deployment settings */
  settings?: {
    gasLimit?: string;
    gasPrice?: string;
    from?: string;
    verify?: boolean;
  };
  
  /** Contracts to deploy */
  contracts: Array<{
    name: string;
    args?: any[];
    from?: string;
    gasLimit?: string;
    libraries?: { [name: string]: string };
    metadata?: {
      tags?: string[];
      description?: string;
    };
  }>;
  
  /** Post-deployment scripts */
  scripts?: Array<{
    name: string;
    script: string;
    args?: any[];
  }>;
}