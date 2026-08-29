export interface ABIEntry {
  type: 'function' | 'constructor' | 'event' | 'error' | 'fallback' | 'receive';
  name?: string;
  inputs: ABIInput[];
  outputs?: ABIOutput[];
  stateMutability?: 'pure' | 'view' | 'nonpayable' | 'payable';
  constant?: boolean;
  payable?: boolean;
  anonymous?: boolean;
}

export interface ABIInput {
  name: string;
  type: string;
  indexed?: boolean | null;
  internalType?: string;
  components?: ABIInput[];
}

export interface ABIOutput {
  name: string;
  type: string;
  internalType?: string;
  components?: ABIOutput[];
}

export type ContractABI = ABIEntry[];

export interface ABICompatibilityLayer {
  // Ethers.js integration
  toEthersInterface(): any;
  fromEthersInterface(iface: any): ContractABI;
  
  // Web3.js integration
  toWeb3ABI(): any[];
  fromWeb3ABI(abi: any[]): ContractABI;
  
  // Encoding/Decoding
  encodeFunction(name: string, params: any[]): string;
  decodeFunction(name: string, data: string): any[];
  encodeConstructor(params: any[]): string;
  decodeConstructor(data: string): any[];
  encodeEvent(name: string, values: any[]): string;
  decodeEvent(name: string, data: string, topics: string[]): any;
  
  // Type conversions
  convertToNeoVM(type: string, value: any): any;
  convertFromNeoVM(type: string, value: any): any;
  
  // Validation
  validateInput(functionName: string, inputs: any[]): boolean;
  validateOutput(functionName: string, outputs: any[]): boolean;
}

export interface EncodedData {
  functionSignature: string;
  encodedParams: string;
  decodedParams: any[];
  gasEstimate?: string;
}

export interface DecodedEvent {
  eventName: string;
  args: { [key: string]: any };
  signature: string;
  topics: string[];
  data: string;
  blockNumber: number;
  transactionHash: string;
  logIndex: number;
}

export interface TypeDefinition {
  name: string;
  type: 'elementary' | 'array' | 'mapping' | 'struct' | 'enum' | 'contract';
  baseType?: string;
  keyType?: string;
  valueType?: string;
  length?: number;
  members?: { [key: string]: TypeDefinition };
  values?: string[];
}

export interface FunctionSelector {
  signature: string;
  selector: string;
  inputs: ABIInput[];
  outputs: ABIOutput[];
  stateMutability?: 'pure' | 'view' | 'payable' | 'nonpayable';
}

export interface EventSelector {
  signature: string;
  topic0: string;
  inputs: ABIInput[];
  indexed: ABIInput[];
  nonIndexed: ABIInput[];
}

export interface ABIRegistry {
  functions: Map<string, FunctionSelector>;
  events: Map<string, EventSelector>;
  errors: Map<string, ABIEntry>;
  
  addFunction(abi: ABIEntry): void;
  addEvent(abi: ABIEntry): void;
  addError(abi: ABIEntry): void;
  
  getFunctionBySelector(selector: string): FunctionSelector | undefined;
  getFunctionByName(name: string): FunctionSelector | undefined;
  getEventByTopic(topic: string): EventSelector | undefined;
  getEventByName(name: string): EventSelector | undefined;
  getErrorBySelector(selector: string): ABIEntry | undefined;

  getFunctionSelectors(): FunctionSelector[];
  getEventSelectors(): EventSelector[];
  getConstructors(): ABIEntry[];
  
  validateABI(abi: ContractABI): ABIValidationResult;
}

export interface ABIValidationResult {
  valid: boolean;
  errors: ABIValidationError[];
  warnings: ABIValidationWarning[];
}

export interface ABIValidationError {
  type: 'duplicate_selector' | 'invalid_type' | 'missing_name' | 'invalid_signature';
  message: string;
  entry?: ABIEntry;
}

export interface ABIValidationWarning {
  type: 'naming_convention' | 'gas_optimization' | 'deprecated_feature';
  message: string;
  entry?: ABIEntry;
}

export interface ContractWrapper {
  address: string;
  abi: ContractABI;
  provider: any;
  signer?: any;
  
  // Call functions
  call<T = any>(functionName: string, ...params: any[]): Promise<T>;
  callStatic<T = any>(functionName: string, ...params: any[]): Promise<T>;
  
  // Send transactions
  send(functionName: string, options?: TransactionOptions, ...params: any[]): Promise<TransactionResponse>;
  
  // Events
  on(eventName: string, listener: EventListener): void;
  off(eventName: string, listener?: EventListener): void;
  queryFilter(eventName: string, fromBlock?: number, toBlock?: number): Promise<DecodedEvent[]>;
  
  // Utility
  estimateGas(functionName: string, ...params: any[]): Promise<string>;
  interface: ABICompatibilityLayer;
}

export interface TransactionOptions {
  gasLimit?: string;
  gasPrice?: string;
  value?: string;
  nonce?: number;
  from?: string;
}

export interface TransactionResponse {
  hash: string;
  blockNumber?: number;
  blockHash?: string;
  transactionIndex?: number;
  confirmations: number;
  from: string;
  to: string;
  gasUsed?: string;
  gasPrice: string;
  status?: number;
  logs: any[];
  events?: DecodedEvent[];
  wait(confirmations?: number): Promise<TransactionReceipt>;
}

export interface TransactionReceipt {
  transactionHash: string;
  transactionIndex: number;
  blockHash: string;
  blockNumber: number;
  from: string;
  to: string;
  gasUsed: string;
  gasPrice: string;
  status: number;
  logs: any[];
  events: DecodedEvent[];
  contractAddress?: string;
}

export type EventListener = (...args: any[]) => void;

export interface ABICodec {
  // Function encoding/decoding
  encodeFunctionData(fragment: ABIEntry, values: any[]): string;
  decodeFunctionData(fragment: ABIEntry, data: string): any[];
  encodeFunctionResult(fragment: ABIEntry, values: any[]): string;
  decodeFunctionResult(fragment: ABIEntry, data: string): any[];
  
  // Event encoding/decoding
  encodeEventLog(fragment: ABIEntry, values: any[]): { topics: string[], data: string };
  decodeEventLog(fragment: ABIEntry, data: string, topics: string[]): any[];
  
  // Error encoding/decoding
  encodeErrorResult(fragment: ABIEntry, values: any[]): string;
  decodeErrorResult(fragment: ABIEntry, data: string): any[];
  
  // Type utilities
  getTypeName(type: string): string;
  getCanonicalType(type: string): string;
  isValidType(type: string): boolean;
}

export interface ABIOptimizer {
  // Function selector optimization
  optimizeFunctionSelectors(abi: ContractABI): ContractABI;
  minimizeCollisions(functions: ABIEntry[]): ABIEntry[];
  
  // Gas optimization
  optimizeForGas(abi: ContractABI): ContractABI;
  suggestPackedStructs(abi: ContractABI): PackingRecommendation[];
  
  // Size optimization
  optimizeForSize(abi: ContractABI): ContractABI;
  removeUnusedFunctions(abi: ContractABI, usageData: FunctionUsage[]): ContractABI;
}

export interface PackingRecommendation {
  structName: string;
  originalOrder: string[];
  optimizedOrder: string[];
  gasSavings: string;
}

export interface FunctionUsage {
  functionName: string;
  callCount: number;
  gasUsed: string;
  lastUsed: Date;
}
