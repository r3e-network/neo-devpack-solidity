/**
 * Contract deployment result
 */
export interface ContractDeployment {
  /** Contract address */
  address: string;
  /** Script hash */
  scriptHash: string;
  /** Transaction hash */
  transactionHash: string;
  /** Block number */
  blockNumber: number;
  /** Gas used */
  gasUsed: bigint;
  /** Deployment receipt */
  receipt: DeploymentReceipt;
  /** Contract instance */
  contract: NeoContract;
}

/**
 * Deployment receipt
 */
export interface DeploymentReceipt {
  /** Transaction hash */
  transactionHash: string;
  /** Block number */
  blockNumber: number;
  /** Block hash */
  blockHash: string;
  /** Transaction index */
  transactionIndex: number;
  /** From address */
  from: string;
  /** Contract address */
  contractAddress: string;
  /** Gas used */
  gasUsed: bigint;
  /** Effective gas price */
  effectiveGasPrice: bigint;
  /** Status */
  status: 'success' | 'failed';
  /** Events emitted */
  events: ContractEvent[];
  /** Neo-specific receipt data */
  neo: {
    /** VM state */
    vmState: 'HALT' | 'FAULT';
    /** Exception message (if any) */
    exception?: string;
    /** Stack result */
    stack: any[];
    /** Notifications */
    notifications: NeoNotification[];
    /** System fee */
    systemFee: bigint;
    /** Network fee */
    networkFee: bigint;
  };
}

/**
 * Neo contract instance
 */
export interface NeoContract {
  /** Contract address */
  address: string;
  /** Script hash */
  scriptHash: string;
  /** Contract ABI */
  abi: any[];
  /** Neo manifest */
  manifest: any;
  /** Interface for calling contract methods */
  methods: {
    [methodName: string]: ContractMethod;
  };
  /** Interface for contract events */
  events: {
    [eventName: string]: ContractEvent;
  };
  /** Network provider */
  provider: any;
  /** Signer (if available) */
  signer?: any;
}

/**
 * Contract method interface
 */
export interface ContractMethod {
  /** Method name */
  name: string;
  /** Method parameters */
  parameters: MethodParameter[];
  /** Return type */
  returnType: string;
  /** Whether method is safe (read-only) */
  safe: boolean;
  /** Call the method */
  call: (...args: any[]) => Promise<any>;
  /** Invoke the method (write) */
  invoke: (...args: any[]) => Promise<TransactionResult>;
  /** Estimate gas for the method */
  estimateGas: (...args: any[]) => Promise<bigint>;
}

/**
 * Method parameter
 */
export interface MethodParameter {
  /** Parameter name */
  name: string;
  /** Parameter type */
  type: string;
  /** Parameter description */
  description?: string;
}

/**
 * Contract event
 */
export interface ContractEvent {
  /** Event name */
  name: string;
  /** Event parameters */
  parameters: EventParameter[];
  /** Event signature */
  signature: string;
  /** Event topic hash */
  topicHash: string;
  /** Event data */
  data?: any;
  /** Block number */
  blockNumber?: number;
  /** Transaction hash */
  transactionHash?: string;
}

/**
 * Event parameter
 */
export interface EventParameter {
  /** Parameter name */
  name: string;
  /** Parameter type */
  type: string;
  /** Whether parameter is indexed */
  indexed: boolean;
  /** Parameter value */
  value?: any;
}

/**
 * Neo notification (event equivalent)
 */
export interface NeoNotification {
  /** Contract script hash */
  contract: string;
  /** Event name */
  eventName: string;
  /** State data */
  state: any[];
}

/**
 * Transaction result
 */
export interface TransactionResult {
  /** Transaction hash */
  hash: string;
  /** Transaction object */
  transaction: NeoTransaction;
  /** Wait for confirmation */
  wait: (confirmations?: number) => Promise<DeploymentReceipt>;
}

/**
 * Neo transaction
 */
export interface NeoTransaction {
  /** Transaction hash */
  hash: string;
  /** Version */
  version: number;
  /** Nonce */
  nonce: number;
  /** System fee */
  systemFee: bigint;
  /** Network fee */
  networkFee: bigint;
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
}

/**
 * Transaction signer
 */
export interface TransactionSigner {
  /** Account script hash */
  account: string;
  /** Scopes */
  scopes: 'None' | 'CalledByEntry' | 'CustomContracts' | 'CustomGroups' | 'WitnessRules' | 'Global';
  /** Allowed contracts (if CustomContracts scope) */
  allowedContracts?: string[];
  /** Allowed groups (if CustomGroups scope) */
  allowedGroups?: string[];
  /** Witness rules (if WitnessRules scope) */
  rules?: any[];
}

/**
 * Transaction attribute
 */
export interface TransactionAttribute {
  /** Attribute type */
  type: string;
  /** Attribute data */
  data?: any;
}

/**
 * Transaction witness
 */
export interface TransactionWitness {
  /** Invocation script */
  invocationScript: string;
  /** Verification script */
  verificationScript: string;
}

/**
 * Contract call options
 */
export interface CallOptions {
  /** From address */
  from?: string;
  /** Gas limit */
  gasLimit?: bigint;
  /** Gas price */
  gasPrice?: bigint;
  /** Value to send */
  value?: bigint;
  /** Block number for historical calls */
  blockTag?: number | string;
}

/**
 * Contract factory for deployment
 */
export interface ContractFactory {
  /** Contract bytecode */
  bytecode: string;
  /** Contract ABI */
  abi: any[];
  /** Neo manifest */
  manifest: any;
  /** Deploy contract */
  deploy: (...args: any[]) => Promise<ContractDeployment>;
  /** Estimate deployment gas */
  estimateDeployGas: (...args: any[]) => Promise<bigint>;
  /** Get deployment data */
  getDeploymentData: (...args: any[]) => string;
}

/**
 * Contract verification result
 */
export interface VerificationResult {
  /** Whether verification was successful */
  success: boolean;
  /** Verification message */
  message: string;
  /** Explorer URL (if verified) */
  explorerUrl?: string;
  /** Source code URL */
  sourceUrl?: string;
}

/**
 * Contract interaction options
 */
export interface InteractionOptions {
  /** Network to interact with */
  network?: string;
  /** Account to interact from */
  from?: string;
  /** Gas settings */
  gas?: {
    limit?: bigint;
    price?: bigint;
  };
  /** Retry settings */
  retry?: {
    attempts: number;
    delay: number;
  };
  /** Timeout settings */
  timeout?: number;
}
