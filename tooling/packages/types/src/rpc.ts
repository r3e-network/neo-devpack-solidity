/**
 * Neo RPC provider interface
 */
export interface NeoRpcProvider {
  /** RPC endpoint URL */
  url: string;
  
  /** Network magic number */
  magic: number;
  
  /** Make RPC call */
  call<T = any>(method: string, params?: any[]): Promise<T>;
  
  /** Get block by hash or index */
  getBlock(hashOrIndex: string | number): Promise<NeoBlock>;
  
  /** Get transaction by hash */
  getTransaction(hash: string): Promise<NeoTransaction>;
  
  /** Get contract state */
  getContractState(scriptHash: string): Promise<ContractState>;
  
  /** Invoke contract method (read-only) */
  invokeFunction(scriptHash: string, method: string, params?: any[]): Promise<InvokeResult>;
  
  /** Send raw transaction */
  sendRawTransaction(signedTransaction: string): Promise<SendResult>;
  
  /** Send structured transaction */
  sendTransaction(transaction: any): Promise<SendResult>;
  
  /** Get balance for address */
  getBalance(address: string): Promise<Balance[]>;
  
  /** Get storage item */
  getStorage(scriptHash: string, key: string): Promise<string | null>;
  
  /** Get block count */
  getBlockCount(): Promise<number>;
  
  /** Get current block hash */
  getBestBlockHash(): Promise<string>;
  
  /** Get transaction height */
  getTransactionHeight(hash: string): Promise<number>;
  
  /** Get application log */
  getApplicationLog(hash: string): Promise<ApplicationLog>;
}

/**
 * Neo block
 */
export interface NeoBlock {
  /** Block hash */
  hash: string;
  
  /** Block size */
  size: number;
  
  /** Block version */
  version: number;
  
  /** Previous block hash */
  previousBlockHash: string;
  
  /** Merkle root */
  merkleRoot: string;
  
  /** Timestamp */
  time: number;
  
  /** Block index */
  index: number;
  
  /** Next consensus */
  nextConsensus: string;
  
  /** Witnesses */
  witnesses: TransactionWitness[];
  
  /** Transactions */
  tx: NeoTransaction[];
  
  /** Confirmations */
  confirmations: number;
}

/**
 * Neo transaction
 */
export interface NeoTransaction {
  /** Transaction hash */
  hash: string;
  
  /** Transaction size */
  size: number;
  
  /** Version */
  version: number;
  
  /** Nonce */
  nonce: number;
  
  /** Sender */
  sender: string;
  
  /** System fee */
  sysFee: string;
  
  /** Network fee */
  netFee: string;
  
  /** Valid until block */
  validUntilBlock: number;
  
  /** Signers */
  signers: TransactionSigner[];
  
  /** Attributes */
  attributes: TransactionAttribute[];
  
  /** Script */
  script: string;
  
  /** Witnesses */
  witnesses: TransactionWitness[];
  
  /** Block hash (if confirmed) */
  blockHash?: string;
  
  /** Block index (if confirmed) */
  blockTime?: number;
  
  /** Confirmations */
  confirmations?: number;
}

/**
 * Transaction signer
 */
export interface TransactionSigner {
  /** Account script hash */
  account: string;
  
  /** Scopes */
  scopes: 'None' | 'CalledByEntry' | 'CustomContracts' | 'CustomGroups' | 'WitnessRules' | 'Global';
  
  /** Allowed contracts */
  allowedContracts?: string[];
  
  /** Allowed groups */
  allowedGroups?: string[];
  
  /** Rules */
  rules?: WitnessRule[];
}

/**
 * Witness rule
 */
export interface WitnessRule {
  /** Action */
  action: 'Allow' | 'Deny';
  
  /** Condition */
  condition: WitnessCondition;
}

/**
 * Witness condition
 */
export interface WitnessCondition {
  /** Condition type */
  type: 'Boolean' | 'Not' | 'And' | 'Or' | 'ScriptHash' | 'Group' | 'CalledByEntry' | 'CalledByContract' | 'CalledByGroup';
  
  /** Boolean value (for Boolean condition) */
  booleanExpression?: boolean;
  
  /** Expressions (for And/Or) */
  expressions?: WitnessCondition[];
  
  /** Nested condition (for Not) */
  nestedExpression?: WitnessCondition;
  
  /** Hash (for ScriptHash/Group/CalledByContract/CalledByGroup) */
  hash?: string;
}

/**
 * Transaction attribute
 */
export interface TransactionAttribute {
  /** Attribute type */
  type: 'HighPriority' | 'OracleResponse' | 'NotValidBefore' | 'Conflicts' | 'NotaryAssisted';
  
  /** Height (for NotValidBefore) */
  height?: number;
  
  /** Hash (for Conflicts) */
  hash?: string;
  
  /** Keys count (for NotaryAssisted) */
  nKeys?: number;
}

/**
 * Transaction witness
 */
export interface TransactionWitness {
  /** Invocation script */
  invocation: string;
  
  /** Verification script */
  verification: string;
}

/**
 * Contract state
 */
export interface ContractState {
  /** Contract ID */
  id: number;
  
  /** Update counter */
  updateCounter: number;
  
  /** Contract hash */
  hash: string;
  
  /** NEF (Neo Executable Format) */
  nef: {
    magic: number;
    compiler: string;
    source: string;
    tokens: any[];
    script: string;
    checksum: number;
  };
  
  /** Contract manifest */
  manifest: ContractManifest;
}

/**
 * Contract manifest
 */
export interface ContractManifest {
  /** Contract name */
  name: string;
  
  /** Groups */
  groups: ContractGroup[];
  
  /** Features */
  features: any;
  
  /** Supported standards */
  supportedStandards: string[];
  
  /** ABI */
  abi: ContractAbi;
  
  /** Permissions */
  permissions: ContractPermission[];
  
  /** Trusts */
  trusts: string[];
  
  /** Extra metadata */
  extra: any;
}

/**
 * Contract group
 */
export interface ContractGroup {
  /** Public key */
  pubKey: string;
  
  /** Signature */
  signature: string;
}

/**
 * Contract ABI
 */
export interface ContractAbi {
  /** Methods */
  methods: ContractMethod[];
  
  /** Events */
  events: ContractEvent[];
}

/**
 * Contract method
 */
export interface ContractMethod {
  /** Method name */
  name: string;
  
  /** Parameters */
  parameters: ContractParameter[];
  
  /** Return type */
  returnType: string;
  
  /** Offset in contract script */
  offset: number;
  
  /** Whether method is safe */
  safe: boolean;
}

/**
 * Contract event
 */
export interface ContractEvent {
  /** Event name */
  name: string;
  
  /** Parameters */
  parameters: ContractParameter[];
}

/**
 * Contract parameter
 */
export interface ContractParameter {
  /** Parameter name */
  name: string;
  
  /** Parameter type */
  type: ParameterType;
}

/**
 * Parameter types
 */
export type ParameterType = 
  | 'Any'
  | 'Boolean' 
  | 'Integer'
  | 'ByteArray'
  | 'String'
  | 'Hash160'
  | 'Hash256'
  | 'PublicKey'
  | 'Signature'
  | 'Array'
  | 'Map'
  | 'InteropInterface'
  | 'Void';

/**
 * Contract permission
 */
export interface ContractPermission {
  /** Contract hash or wildcard */
  contract: string;
  
  /** Allowed methods */
  methods: string[];
}

/**
 * Invoke result
 */
export interface InvokeResult {
  /** Script */
  script: string;
  
  /** VM state */
  state: 'HALT' | 'FAULT';
  
  /** Gas consumed */
  gasConsumed: string;
  
  /** Exception (if any) */
  exception?: string;
  
  /** Stack result */
  stack: StackItem[];
  
  /** Notifications */
  notifications: Notification[];
  
  /** Session */
  session?: string;
}

/**
 * Stack item
 */
export interface StackItem {
  /** Item type */
  type: StackItemType;
  
  /** Value */
  value: any;
}

/**
 * Stack item types
 */
export type StackItemType =
  | 'Any'
  | 'Pointer'
  | 'Boolean'
  | 'Integer'
  | 'ByteString'
  | 'Buffer'
  | 'Array'
  | 'Struct'
  | 'Map'
  | 'InteropInterface';

/**
 * Notification
 */
export interface Notification {
  /** Contract hash */
  contract: string;
  
  /** Event name */
  eventName: string;
  
  /** State */
  state: StackItem[];
}

/**
 * Send result
 */
export interface SendResult {
  /** Transaction hash */
  hash: string;
}

/**
 * Balance
 */
export interface Balance {
  /** Asset hash */
  assetHash: string;
  
  /** Asset symbol */
  symbol: string;
  
  /** Asset decimals */
  decimals: number;
  
  /** Balance amount */
  amount: string;
}

/**
 * Application log
 */
export interface ApplicationLog {
  /** Transaction hash */
  txId: string;
  
  /** Executions */
  executions: Execution[];
}

/**
 * Execution
 */
export interface Execution {
  /** Trigger */
  trigger: 'OnPersist' | 'PostPersist' | 'Verification' | 'Application';
  
  /** VM state */
  vmState: 'HALT' | 'FAULT';
  
  /** Exception (if any) */
  exception?: string;
  
  /** Gas consumed */
  gasConsumed: string;
  
  /** Stack */
  stack: StackItem[];
  
  /** Notifications */
  notifications: Notification[];
}

/**
 * RPC batch request
 */
export interface RpcBatchRequest {
  /** Requests */
  requests: RpcRequest[];
  
  /** Execute batch */
  execute: () => Promise<any[]>;
}

/**
 * Individual RPC request
 */
export interface RpcRequest {
  /** Method name */
  method: string;
  
  /** Parameters */
  params?: any[];
  
  /** Request ID */
  id?: number;
}

/**
 * RPC configuration
 */
export interface RpcConfig {
  /** RPC URLs */
  urls: string[];
  
  /** Timeout (ms) */
  timeout: number;
  
  /** Retry configuration */
  retry: {
    attempts: number;
    delay: number;
    exponentialBackoff: boolean;
  };
  
  /** Load balancing strategy */
  loadBalancing: 'round-robin' | 'random' | 'priority';
  
  /** Authentication */
  auth?: {
    username: string;
    password: string;
  };
  
  /** Headers */
  headers?: {
    [key: string]: string;
  };
}
