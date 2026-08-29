import {
  ABICompatibilityLayer,
  ContractABI,
  ABIEntry,
  DecodedEvent,
  FunctionSelector,
  EventSelector,
  ABIRegistry,
  ABICodec
} from '@neo-devpack-solidity/types';
import { ethers } from 'ethers';
import { EventEmitter } from 'events';

export class NeoABICompatibilityLayer extends EventEmitter implements ABICompatibilityLayer {
  private registry: ABIRegistry;
  private codec: ABICodec;
  private cache: Map<string, any> = new Map();

  constructor() {
    super();
    this.registry = new NeoABIRegistry();
    this.codec = new NeoABICodec();
  }

  registerABI(abi: ContractABI): void {
    this.registry = new NeoABIRegistry();
    for (const entry of abi) {
      switch (entry.type) {
        case "function":
          this.registry.addFunction(entry);
          break;
        case "event":
          this.registry.addEvent(entry);
          break;
        case "error":
          this.registry.addError(entry);
          break;
        default:
          break;
      }
    }
  }

  // Ethers.js Integration
  toEthersInterface(): ethers.Interface {
    const mapInput = (input: any): any => ({
      name: input.name,
      type: input.type,
      indexed: input.indexed ?? undefined,
      internalType: input.internalType,
      components: Array.isArray(input.components) ? input.components.map(mapInput) : undefined
    });

    const mapOutput = (output: any): any => ({
      name: output.name,
      type: output.type,
      internalType: output.internalType,
      components: Array.isArray(output.components) ? output.components.map(mapOutput) : undefined
    });

    const abi = this.registry.getFunctionSelectors().map(selector => ({
      type: 'function',
      name: this.extractFunctionName(selector.signature),
      inputs: selector.inputs.map(mapInput),
      outputs: selector.outputs.map(mapOutput),
      stateMutability: this.determineStateMutability(selector)
    }));

    // Add events
    const events = this.registry.getEventSelectors().map(selector => ({
      type: 'event',
      name: this.extractEventName(selector.signature),
      inputs: selector.inputs.map(mapInput),
      anonymous: false
    }));

    return new ethers.Interface([...abi, ...events]);
  }

  fromEthersInterface(iface: ethers.Interface): ContractABI {
    const abi: ContractABI = [];

    // Convert functions
    for (const fragment of iface.fragments) {
      if (fragment.type === 'function') {
        const funcFragment = fragment as ethers.FunctionFragment;
        abi.push({
          type: 'function',
          name: funcFragment.name,
          inputs: funcFragment.inputs.map(input => ({
            name: input.name,
            type: input.type,
            internalType: input.baseType
          })),
          outputs: funcFragment.outputs?.map(output => ({
            name: output.name,
            type: output.type,
            internalType: output.baseType
          })) || [],
          stateMutability: funcFragment.stateMutability
        });
      } else if (fragment.type === 'event') {
        const eventFragment = fragment as ethers.EventFragment;
        abi.push({
          type: 'event',
          name: eventFragment.name,
          inputs: eventFragment.inputs.map(input => ({
            name: input.name,
            type: input.type,
            indexed: input.indexed,
            internalType: input.baseType
          })),
          anonymous: eventFragment.anonymous
        });
      }
    }

    return abi;
  }

  // Web3.js Integration
  toWeb3ABI(): any[] {
    const abi: any[] = [];

    // Add functions
    const functions = this.registry.getFunctionSelectors();
    for (const func of functions) {
      abi.push({
        type: 'function',
        name: this.extractFunctionName(func.signature),
        inputs: func.inputs.map(input => ({
          name: input.name,
          type: input.type,
          internalType: input.internalType
        })),
        outputs: func.outputs.map(output => ({
          name: output.name,
          type: output.type,
          internalType: output.internalType
        })),
        stateMutability: this.determineStateMutability(func),
        constant: this.isConstantFunction(func),
        payable: this.isPayableFunction(func)
      });
    }

    // Add events
    const events = this.registry.getEventSelectors();
    for (const event of events) {
      abi.push({
        type: 'event',
        name: this.extractEventName(event.signature),
        inputs: event.inputs.map(input => ({
          name: input.name,
          type: input.type,
          indexed: input.indexed,
          internalType: input.internalType
        })),
        anonymous: false
      });
    }

    return abi;
  }

  fromWeb3ABI(abi: any[]): ContractABI {
    const contractABI: ContractABI = [];

    for (const entry of abi) {
      if (entry.type === 'function') {
        contractABI.push({
          type: 'function',
          name: entry.name,
          inputs: entry.inputs?.map((input: any) => ({
            name: input.name,
            type: input.type,
            internalType: input.internalType
          })) || [],
          outputs: entry.outputs?.map((output: any) => ({
            name: output.name,
            type: output.type,
            internalType: output.internalType
          })) || [],
          stateMutability: entry.stateMutability || (entry.constant ? 'view' : 'nonpayable')
        });
      } else if (entry.type === 'event') {
        contractABI.push({
          type: 'event',
          name: entry.name,
          inputs: entry.inputs?.map((input: any) => ({
            name: input.name,
            type: input.type,
            indexed: input.indexed,
            internalType: input.internalType
          })) || [],
          anonymous: entry.anonymous || false
        });
      } else if (entry.type === 'constructor') {
        contractABI.push({
          type: 'constructor',
          inputs: entry.inputs?.map((input: any) => ({
            name: input.name,
            type: input.type,
            internalType: input.internalType
          })) || [],
          stateMutability: entry.stateMutability || 'nonpayable'
        });
      }
    }

    return contractABI;
  }

  // Encoding/Decoding Functions
  encodeFunction(name: string, params: any[]): string {
    if (!name.includes("(") && this.isAmbiguousFunctionName(name)) {
      throw new Error(`Function ${name} is overloaded; use a full signature`);
    }
    const selector = this.registry.getFunctionByName(name);
    if (!selector) {
      throw new Error(`Function ${name} not found in registry`);
    }

    return this.codec.encodeFunctionData(
      this.selectorToABIEntry(selector),
      params
    );
  }

  decodeFunction(name: string, data: string): any[] {
    if (!name.includes("(") && this.isAmbiguousFunctionName(name)) {
      throw new Error(`Function ${name} is overloaded; use a full signature`);
    }
    const selector = this.registry.getFunctionByName(name);
    if (!selector) {
      throw new Error(`Function ${name} not found in registry`);
    }

    return this.codec.decodeFunctionData(
      this.selectorToABIEntry(selector),
      data
    );
  }

  encodeConstructor(params: any[]): string {
    // Get constructor from registry
    const constructors = this.registry.getConstructors();
    if (constructors.length === 0) {
      throw new Error('No constructor found in registry');
    }

    return this.codec.encodeFunctionData(constructors[0], params);
  }

  decodeConstructor(data: string): any[] {
    const constructors = this.registry.getConstructors();
    if (constructors.length === 0) {
      throw new Error('No constructor found in registry');
    }

    return this.codec.decodeFunctionData(constructors[0], data);
  }

  encodeEvent(name: string, values: any[]): string {
    const selector = this.registry.getEventByName(name);
    if (!selector) {
      throw new Error(`Event ${name} not found in registry`);
    }

    const { topics, data } = this.codec.encodeEventLog(
      this.eventSelectorToABIEntry(selector),
      values
    );

    return JSON.stringify({ topics, data });
  }

  decodeEvent(name: string, data: string, topics: string[]): DecodedEvent {
    const selector = this.registry.getEventByName(name);
    if (!selector) {
      throw new Error(`Event ${name} not found in registry`);
    }

    const decoded = this.codec.decodeEventLog(
      this.eventSelectorToABIEntry(selector),
      data,
      topics
    );

    return {
      eventName: name,
      args: this.arrayToObject(decoded, selector.inputs),
      signature: selector.signature,
      topics,
      data,
      blockNumber: 0, // Would be filled by caller
      transactionHash: '', // Would be filled by caller
      logIndex: 0 // Would be filled by caller
    };
  }

  // Type Conversion for NeoVM
  convertToNeoVM(type: string, value: any): any {
    switch (type) {
      case 'bool':
        return { type: 'Boolean', value: Boolean(value) };
      
      case 'address':
        return { type: 'Hash160', value: this.addressToScriptHash(value) };
      
      case 'string':
        return { type: 'String', value: String(value) };
      
      case 'bytes':
      case 'bytes32':
        return { 
          type: 'ByteArray', 
          value: typeof value === 'string' 
            ? (value.startsWith('0x') ? value.slice(2) : value)
            : Buffer.from(value).toString('hex')
        };
      
      default:
        if (type.endsWith('[]')) {
          const baseType = type.slice(0, -2);
          return {
            type: 'Array',
            value: Array.isArray(value) 
              ? value.map(item => this.convertToNeoVM(baseType, item))
              : []
          };
        }
        if (type.startsWith('uint') || type.startsWith('int')) {
          return { type: 'Integer', value: String(value) };
        }
        return { type: 'String', value: String(value) };
    }
  }

  convertFromNeoVM(type: string, value: any): any {
    if (!value || typeof value !== 'object') {
      return value;
    }

    switch (value.type) {
      case 'Boolean':
        return Boolean(value.value);
      
      case 'Integer':
        if (type.startsWith('uint') || type.startsWith('int')) {
          return BigInt(value.value);
        }
        return Number(value.value);
      
      case 'ByteString':
        if (type === 'address') {
          return this.scriptHashToAddress(value.value);
        }
        if (type === 'string') {
          return Buffer.from(value.value, 'hex').toString('utf8');
        }
        return '0x' + value.value;
      
      case 'Array':
        {
          const baseType = type.endsWith('[]') ? type.slice(0, -2) : 'string';
          return (value.value || []).map((item: any) => this.convertFromNeoVM(baseType, item));
        }
      
      case 'Map':
        {
          const result: { [key: string]: any } = {};
          if (value.value) {
            for (const [k, v] of Object.entries(value.value)) {
              result[k] = this.convertFromNeoVM('string', v);
            }
          }
          return result;
        }
      
      default:
        return value.value;
    }
  }

  // Validation
  validateInput(functionName: string, inputs: any[]): boolean {
    if (!functionName.includes("(") && this.isAmbiguousFunctionName(functionName)) {
      return false;
    }
    const selector = this.registry.getFunctionByName(functionName);
    if (!selector) {
      return false;
    }

    if (inputs.length !== selector.inputs.length) {
      return false;
    }

    for (let i = 0; i < inputs.length; i++) {
      if (!this.validateType(selector.inputs[i].type, inputs[i])) {
        return false;
      }
    }

    return true;
  }

  validateOutput(functionName: string, outputs: any[]): boolean {
    if (!functionName.includes("(") && this.isAmbiguousFunctionName(functionName)) {
      return false;
    }
    const selector = this.registry.getFunctionByName(functionName);
    if (!selector) {
      return false;
    }

    if (outputs.length !== selector.outputs.length) {
      return false;
    }

    for (let i = 0; i < outputs.length; i++) {
      if (!this.validateType(selector.outputs[i].type, outputs[i])) {
        return false;
      }
    }

    return true;
  }

  // Utility Methods
  private extractFunctionName(signature: string): string {
    const match = signature.match(/^([^(]+)/);
    return match ? match[1] : signature;
  }

  private extractEventName(signature: string): string {
    const match = signature.match(/^([^(]+)/);
    return match ? match[1] : signature;
  }

  private determineStateMutability(
    selector: FunctionSelector
  ): 'pure' | 'view' | 'payable' | 'nonpayable' {
    return selector.stateMutability || 'nonpayable';
  }

  private isConstantFunction(selector: FunctionSelector): boolean {
    return this.determineStateMutability(selector) === 'view';
  }

  private isPayableFunction(selector: FunctionSelector): boolean {
    return this.determineStateMutability(selector) === 'payable';
  }

  private selectorToABIEntry(selector: FunctionSelector): ABIEntry {
    return {
      type: 'function',
      name: this.extractFunctionName(selector.signature),
      inputs: selector.inputs,
      outputs: selector.outputs,
      stateMutability: this.determineStateMutability(selector)
    };
  }

  private eventSelectorToABIEntry(selector: EventSelector): ABIEntry {
    return {
      type: 'event',
      name: this.extractEventName(selector.signature),
      inputs: selector.inputs,
      anonymous: false
    };
  }

  private arrayToObject(array: any[], inputs: any[]): { [key: string]: any } {
    const result: { [key: string]: any } = {};
    
    for (let i = 0; i < array.length && i < inputs.length; i++) {
      result[i] = array[i];
      if (inputs[i].name) {
        result[inputs[i].name] = array[i];
      }
    }
    
    return result;
  }

  private validateType(expectedType: string, value: any): boolean {
    switch (expectedType) {
      case 'bool':
        return typeof value === 'boolean';
      
      case 'address':
        return typeof value === 'string' && /^0x[a-fA-F0-9]{40}$/.test(value);
      
      case 'string':
        return typeof value === 'string';
      
      default:
        if (expectedType.startsWith('uint') || expectedType.startsWith('int')) {
          return typeof value === 'number' || typeof value === 'bigint' || 
                 (typeof value === 'string' && !isNaN(Number(value)));
        }
        if (expectedType.startsWith('bytes')) {
          return typeof value === 'string' && /^0x[a-fA-F0-9]*$/.test(value);
        }
        if (expectedType.endsWith('[]')) {
          return Array.isArray(value);
        }
        return true; // Default to valid for unknown types
    }
  }

  private addressToScriptHash(address: string): string {
    // Convert Ethereum address to Neo script hash
    if (!address.startsWith('0x')) {
      return '0x' + address;
    }
    return address.toLowerCase();
  }

  private scriptHashToAddress(scriptHash: string): string {
    // Convert Neo script hash to Ethereum address format
    if (!scriptHash.startsWith('0x')) {
      return '0x' + scriptHash;
    }
    return scriptHash.toLowerCase();
  }

  // Cache Management
  clearCache(): void {
    this.cache.clear();
  }

  getCacheStats(): { size: number; hitRate: number } {
    return {
      size: this.cache.size,
      hitRate: 0 // Would track hits/misses in real implementation
    };
  }

  private isAmbiguousFunctionName(name: string): boolean {
    return this.registry instanceof NeoABIRegistry
      ? this.registry.isAmbiguousFunctionName(name)
      : false;
  }
}

// Registry Implementation
export class NeoABIRegistry implements ABIRegistry {
  functions: Map<string, FunctionSelector> = new Map();
  events: Map<string, EventSelector> = new Map();
  errors: Map<string, ABIEntry> = new Map();
  private ambiguousFunctionNames: Set<string> = new Set();

  addFunction(abi: ABIEntry): void {
    if (abi.type !== 'function') {
      throw new Error('Expected function ABI entry');
    }

    const signature = this.generateFunctionSignature(abi);
    const selector = this.generateFunctionSelector(signature);

    this.functions.set(selector, {
      signature,
      selector,
      inputs: abi.inputs,
      outputs: abi.outputs || [],
      stateMutability: abi.stateMutability
    });
    this.functions.set(signature, this.functions.get(selector)!);

    // Also index by name for easier lookup
    if (abi.name) {
      const existing = this.functions.get(abi.name);
      if (existing && existing.signature !== signature) {
        this.ambiguousFunctionNames.add(abi.name);
        this.functions.delete(abi.name);
      } else if (!this.ambiguousFunctionNames.has(abi.name)) {
        this.functions.set(abi.name, this.functions.get(selector)!);
      }
    }
  }

  addEvent(abi: ABIEntry): void {
    if (abi.type !== 'event') {
      throw new Error('Expected event ABI entry');
    }

    const signature = this.generateEventSignature(abi);
    const topic0 = this.generateEventTopic(signature);

    this.events.set(topic0, {
      signature,
      topic0,
      inputs: abi.inputs,
      indexed: abi.inputs.filter(input => input.indexed),
      nonIndexed: abi.inputs.filter(input => !input.indexed)
    });

    // Also index by name
    if (abi.name) {
      this.events.set(abi.name, this.events.get(topic0)!);
    }
  }

  addError(abi: ABIEntry): void {
    if (abi.type !== 'error') {
      throw new Error('Expected error ABI entry');
    }

    const signature = this.generateFunctionSignature(abi);
    const selector = this.generateFunctionSelector(signature);

    this.errors.set(selector, abi);

    if (abi.name) {
      this.errors.set(abi.name, abi);
    }
  }

  getFunctionBySelector(selector: string): FunctionSelector | undefined {
    return this.functions.get(selector);
  }

  getFunctionByName(name: string): FunctionSelector | undefined {
    if (!name.includes("(") && this.ambiguousFunctionNames.has(name)) {
      return undefined;
    }
    return this.functions.get(name);
  }

  isAmbiguousFunctionName(name: string): boolean {
    return this.ambiguousFunctionNames.has(name);
  }

  getEventByTopic(topic: string): EventSelector | undefined {
    return this.events.get(topic);
  }

  getEventByName(name: string): EventSelector | undefined {
    return this.events.get(name);
  }

  getErrorBySelector(selector: string): ABIEntry | undefined {
    return this.errors.get(selector);
  }

  getFunctionSelectors(): FunctionSelector[] {
    const selectors: FunctionSelector[] = [];
    for (const [key, value] of this.functions.entries()) {
      if (key.startsWith('0x')) { // Only include actual selectors, not names
        selectors.push(value);
      }
    }
    return selectors;
  }

  getEventSelectors(): EventSelector[] {
    const selectors: EventSelector[] = [];
    for (const [key, value] of this.events.entries()) {
      if (key.startsWith('0x')) { // Only include actual topics, not names
        selectors.push(value);
      }
    }
    return selectors;
  }

  getConstructors(): ABIEntry[] {
    // Return constructor entries from registry
    return [];
  }

  validateABI(_abi: ContractABI): any {
    // Validate ABI structure and return results
    return {
      valid: true,
      errors: [],
      warnings: []
    };
  }

  private generateFunctionSignature(abi: ABIEntry): string {
    const inputs = abi.inputs.map(input => input.type).join(',');
    return `${abi.name}(${inputs})`;
  }

  private generateEventSignature(abi: ABIEntry): string {
    const inputs = abi.inputs.map(input => input.type).join(',');
    return `${abi.name}(${inputs})`;
  }

  private generateFunctionSelector(signature: string): string {
    return ethers.id(signature).slice(0, 10);
  }

  private generateEventTopic(signature: string): string {
    return ethers.id(signature);
  }
}

// Codec Implementation
export class NeoABICodec implements ABICodec {
  private abiCoder = ethers.AbiCoder.defaultAbiCoder();

  encodeFunctionData(fragment: ABIEntry, values: any[]): string {
    const signature = `${fragment.name}(${fragment.inputs.map(i => i.type).join(',')})`;
    const selector = ethers.id(signature).slice(0, 10);
    
    if (values.length === 0) {
      return selector;
    }

    const encoded = this.abiCoder.encode(
      fragment.inputs.map(input => input.type),
      values
    );

    return selector + encoded.slice(2);
  }

  decodeFunctionData(fragment: ABIEntry, data: string): any[] {
    if (data.length < 10) {
      throw new Error('Data too short for function call');
    }

    const paramData = '0x' + data.slice(10);
    return this.abiCoder.decode(
      fragment.inputs.map(input => input.type),
      paramData
    );
  }

  encodeFunctionResult(fragment: ABIEntry, values: any[]): string {
    return this.abiCoder.encode(
      (fragment.outputs || []).map(output => output.type),
      values
    );
  }

  decodeFunctionResult(fragment: ABIEntry, data: string): any[] {
    return this.abiCoder.decode(
      (fragment.outputs || []).map(output => output.type),
      data
    );
  }

  encodeEventLog(fragment: ABIEntry, values: any[]): { topics: string[], data: string } {
    const indexed = fragment.inputs.filter(input => input.indexed);
    const nonIndexed = fragment.inputs.filter(input => !input.indexed);
    
    const topics: string[] = [ethers.id(`${fragment.name}(${fragment.inputs.map(i => i.type).join(',')})`)];
    
    // Add indexed parameters as topics
    let indexedValueIndex = 0;
    for (const input of indexed) {
      if (indexedValueIndex < values.length) {
        const value = values[indexedValueIndex];
        topics.push(this.encodeIndexedValue(input.type, value));
        indexedValueIndex++;
      }
    }

    // Encode non-indexed parameters as data
    const nonIndexedValues = values.slice(indexedValueIndex);
    const data = nonIndexed.length > 0 
      ? this.abiCoder.encode(nonIndexed.map(input => input.type), nonIndexedValues)
      : '0x';

    return { topics, data };
  }

  decodeEventLog(fragment: ABIEntry, data: string, topics: string[]): any[] {
    const indexed = fragment.inputs.filter(input => input.indexed);
    const nonIndexed = fragment.inputs.filter(input => !input.indexed);
    
    const result: any[] = [];
    
    // Decode indexed values from topics (skip first topic which is event signature)
    for (let i = 0; i < indexed.length && i + 1 < topics.length; i++) {
      result.push(this.decodeIndexedValue(indexed[i].type, topics[i + 1]));
    }
    
    // Decode non-indexed values from data
    if (nonIndexed.length > 0 && data !== '0x') {
      const decoded = this.abiCoder.decode(
        nonIndexed.map(input => input.type),
        data
      );
      result.push(...decoded);
    }
    
    return result;
  }

  encodeErrorResult(fragment: ABIEntry, values: any[]): string {
    const signature = `${fragment.name}(${fragment.inputs.map(i => i.type).join(',')})`;
    const selector = ethers.id(signature).slice(0, 10);
    
    const encoded = this.abiCoder.encode(
      fragment.inputs.map(input => input.type),
      values
    );

    return selector + encoded.slice(2);
  }

  decodeErrorResult(fragment: ABIEntry, data: string): any[] {
    const paramData = '0x' + data.slice(10);
    return this.abiCoder.decode(
      fragment.inputs.map(input => input.type),
      paramData
    );
  }

  getTypeName(type: string): string {
    return type.split('[')[0]; // Remove array notation
  }

  getCanonicalType(type: string): string {
    // Convert to canonical form (e.g., uint -> uint256)
    if (type === 'uint') return 'uint256';
    if (type === 'int') return 'int256';
    if (type === 'bytes') return 'bytes';
    return type;
  }

  isValidType(type: string): boolean {
    const basicTypes = [
      'bool', 'address', 'string', 'bytes'
    ];
    
    if (basicTypes.includes(type)) return true;
    if (/^uint\d*$/.test(type)) return true;
    if (/^int\d*$/.test(type)) return true;
    if (/^bytes\d*$/.test(type)) return true;
    if (type.endsWith('[]')) return this.isValidType(type.slice(0, -2));
    
    return false;
  }

  private encodeIndexedValue(type: string, value: any): string {
    if (type === 'string' || type === 'bytes' || type.startsWith('bytes[')) {
      return ethers.keccak256(ethers.toUtf8Bytes(String(value)));
    }
    
    return ethers.zeroPadValue(ethers.toBeHex(value), 32);
  }

  private decodeIndexedValue(type: string, topic: string): any {
    if (type === 'string' || type === 'bytes' || type.startsWith('bytes[')) {
      return topic; // Return hash for complex types
    }
    
    return this.abiCoder.decode([type], topic)[0];
  }
}
