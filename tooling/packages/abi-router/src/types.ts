import type { ContractManifest } from "@neo-devpack-solidity/types";

/**
 * ABI Router specific types
 */

/**
 * Contract call options
 */
export interface CallOptions {
  blockTag?: string | number;
  from?: string;
}

/**
 * Transaction options
 */
export interface TransactionOptions {
  gasLimit?: string;
  gasPrice?: string;
  value?: string;
  from?: string;
}

/**
 * Event filter options
 */
export interface EventFilterOptions {
  fromBlock?: number | string;
  toBlock?: number | string;
  topics?: string[];
  address?: string;
}

/**
 * Gas estimation result
 */
export interface GasEstimateResult {
  systemFee: string;
  networkFee: string;
  totalGas: string;
}

/**
 * Transaction response
 */
export interface TransactionResponse {
  hash: string;
  from?: string;
  to?: string;
  value?: string;
  gasLimit?: string;
  gasPrice?: string;
  wait: (confirmations?: number) => Promise<TransactionReceipt>;
}

/**
 * Transaction receipt
 */
export interface TransactionReceipt {
  transactionHash: string;
  transactionIndex: number;
  blockNumber: number;
  blockHash: string;
  cumulativeGasUsed: string;
  gasUsed: string;
  contractAddress?: string;
  logs: EventLog[];
  status: number;
  from: string;
  to?: string;
}

/**
 * Event log
 */
export interface EventLog {
  address: string;
  topics: string[];
  data: string;
  logIndex: number;
  transactionIndex: number;
  blockNumber: number;
  blockHash: string;
}

/**
 * Decoded event log
 */
export interface DecodedEventLog extends EventLog {
  event: string;
  args: any[];
  signature: string;
}

/**
 * Contract wrapper options
 */
export interface ContractWrapperOptions {
  address: string;
  abi: any[];
  signer?: any;
  provider?: any;
}

/**
 * Method call result
 */
export interface MethodCallResult {
  value: any;
  gasUsed?: string;
  status: 'success' | 'failed';
}

/**
 * ABI encoding options
 */
export interface AbiEncodingOptions {
  packed?: boolean;
  functionFragment?: string;
  eventFragment?: string;
}

/**
 * Network configuration for ABI router
 */
export interface RouterNetworkConfig {
  name: string;
  rpcUrl: string;
  chainId: number;
  blockTime: number;
  gasToken: {
    symbol: string;
    decimals: number;
  };
}

/**
 * Router configuration
 */
export interface RouterConfig {
  network: RouterNetworkConfig;
  defaultGasLimit: string;
  defaultGasPrice: string;
  confirmations: number;
  timeout: number;
}

/**
 * Event subscription options
 */
export interface EventSubscriptionOptions {
  fromBlock?: number;
  toBlock?: number;
  filter?: any;
  polling?: boolean;
  pollingInterval?: number;
}

/**
 * Contract deployment result
 */
export interface DeploymentResult {
  address: string;
  transactionHash: string;
  contract: any;
  receipt: TransactionReceipt;
}

/**
 * Minimal NEF + manifest payload required to deploy a Neo contract.
 */
export interface NeoDeploymentArtifacts {
  nef: string;
  manifest: ContractManifest | Record<string, any>;
}

/**
 * ABI fragment types
 */
export type AbiFragment = {
  type: 'function' | 'event' | 'constructor' | 'fallback' | 'receive';
  name?: string;
  inputs?: AbiParameter[];
  outputs?: AbiParameter[];
  stateMutability?: 'pure' | 'view' | 'nonpayable' | 'payable';
  anonymous?: boolean;
  indexed?: boolean;
};

export type AbiParameter = {
  name: string;
  type: string;
  indexed?: boolean;
  components?: AbiParameter[];
};
