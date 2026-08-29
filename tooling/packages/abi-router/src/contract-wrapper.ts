import { Interface, FunctionFragment, EventFragment } from "ethers";
import { EventEmitter } from "events";
import { RpcAdapter } from "./rpc-adapter.js";
import { TransactionBuilder } from "./transaction-builder.js";
import { EventDecoder } from "./event-decoder.js";
import Debug from "debug";
import {
  decodeNeoBytes,
  evmAddressToNeoHash160,
  isHexString,
  neoBytesToEvmAddress,
  neoHash160ToEvmAddress,
  parseNeoBoolean,
  parseNeoInteger,
  stackItemArrayValue,
  strip0x,
} from "./neo-utils.js";

const debug = Debug("neo-devpack-solidity:contract-wrapper");

/**
 * Contract wrapper that provides Ethereum-compatible interface for Neo contracts
 */
export class ContractWrapper extends EventEmitter {
  public readonly address: string;
  public readonly interface: Interface;
  
  private rpcAdapter: RpcAdapter;
  private transactionBuilder: TransactionBuilder;
  private eventDecoder: EventDecoder;
  private signer?: any;
  private overloadedFunctionNames: Set<string>;

  constructor(
    address: string,
    abi: any[],
    rpcAdapter: RpcAdapter,
    transactionBuilder: TransactionBuilder,
    eventDecoder: EventDecoder,
    signerOrProvider?: any
  ) {
    super();
    
    this.address = address;
    this.interface = new Interface(abi);
    this.rpcAdapter = rpcAdapter;
    this.transactionBuilder = transactionBuilder;
    this.eventDecoder = eventDecoder;
    this.signer = signerOrProvider;
    this.overloadedFunctionNames = this.collectOverloadedFunctionNames();

    // Create method proxies
    this.createMethodProxies();
    this.createEventFilters();
  }

  /**
   * Call contract method (read-only)
   */
  async call(
    methodName: string,
    args: any[] = [],
    options: {
      blockTag?: string | number;
    } = {}
  ): Promise<any> {
    debug(`Calling ${methodName} with args:`, args);

    try {
      const fragment = this.interface.getFunction(methodName);
      if (!fragment) {
        throw new Error(`Method ${methodName} not found in ABI`);
      }
      const invocationName = this.getInvocationName(fragment);

      // Convert Ethereum args to Neo format
      const neoArgs = this.convertArgsToNeo(args, fragment);

      // Call via RPC adapter
      const result = await this.rpcAdapter.invokeFunction(
        this.address,
        invocationName,
        neoArgs,
        options.blockTag
      );

      // Convert result back to Ethereum format
      return this.convertResultToEthereum(result, fragment);
    } catch (error) {
      debug(`Call to ${methodName} failed:`, error);
      throw error;
    }
  }

  /**
   * Send transaction to contract method
   */
  async send(
    methodName: string,
    args: any[] = [],
    options: {
      gasLimit?: string;
      gasPrice?: string;
      value?: string;
    } = {}
  ): Promise<any> {
    debug(`Sending transaction to ${methodName} with args:`, args);

    if (!this.signer) {
      throw new Error("No signer available for transaction");
    }

    try {
      const fragment = this.interface.getFunction(methodName);
      if (!fragment) {
        throw new Error(`Method ${methodName} not found in ABI`);
      }
      const invocationName = this.getInvocationName(fragment);

      // Convert Ethereum args to Neo format
      const neoArgs = this.convertArgsToNeo(args, fragment);

      // Send transaction via transaction builder
      const txResult = await this.transactionBuilder.sendTransaction(
        this.address,
        invocationName,
        neoArgs,
        options
      );

      return {
        hash: txResult.hash,
        wait: async (confirmations = 1) => {
          return this.waitForTransaction(txResult.hash, confirmations);
        },
        from: this.signer.address,
        to: this.address,
        value: options.value || "0",
        gasLimit: options.gasLimit,
        gasPrice: options.gasPrice
      };
    } catch (error) {
      debug(`Transaction to ${methodName} failed:`, error);
      throw error;
    }
  }

  /**
   * Estimate gas for method call
   */
  async estimateGas(
    methodName: string,
    args: any[] = [],
    options: {
      from?: string;
      value?: string;
    } = {}
  ): Promise<bigint> {
    debug(`Estimating gas for ${methodName} with args:`, args);

    try {
      const fragment = this.interface.getFunction(methodName);
      if (!fragment) {
        throw new Error(`Method ${methodName} not found in ABI`);
      }
      const invocationName = this.getInvocationName(fragment);

      // Convert Ethereum args to Neo format
      const neoArgs = this.convertArgsToNeo(args, fragment);

      // Estimate gas via RPC adapter
      const gasEstimate = await this.rpcAdapter.estimateGas(
        this.address,
        invocationName,
        neoArgs,
        options
      );

      return BigInt(gasEstimate.totalGas);
    } catch (error) {
      debug(`Gas estimation for ${methodName} failed:`, error);
      throw error;
    }
  }

  /**
   * Get past events
   */
  async queryFilter(
    eventName: string,
    fromBlockOrFilter?: any,
    toBlock?: number
  ): Promise<any[]> {
    debug(`Querying filter for event ${eventName}`);

    try {
      const fragment = this.interface.getEvent(eventName);
      if (!fragment) {
        throw new Error(`Event ${eventName} not found in ABI`);
      }

      // Get events from RPC adapter
      const events = await this.rpcAdapter.getEvents(
        this.address,
        eventName,
        fromBlockOrFilter,
        toBlock
      );

      // Decode and return events in Ethereum format
      return events.map(event => this.eventDecoder.decodeEvent(event, fragment));
    } catch (error) {
      debug(`Query filter for ${eventName} failed:`, error);
      throw error;
    }
  }

  /**
   * Create event listener
   */
  on(eventName: string, listener: (...args: any[]) => void): this {
    debug(`Adding listener for event ${eventName}`);
    
    // Start listening for this event type if not already
    this.startEventListener(eventName);
    
    return super.on(eventName, listener);
  }

  /**
   * Create one-time event listener
   */
  once(eventName: string, listener: (...args: any[]) => void): this {
    debug(`Adding one-time listener for event ${eventName}`);
    
    // Start listening for this event type if not already
    this.startEventListener(eventName);
    
    return super.once(eventName, listener);
  }

  /**
   * Remove event listener
   */
  off(eventName: string, listener: (...args: any[]) => void): this {
    debug(`Removing listener for event ${eventName}`);
    return super.off(eventName, listener);
  }

  /**
   * Connect to different signer
   */
  connect(signer: any): ContractWrapper {
    return new ContractWrapper(
      this.address,
      this.interface.fragments.map(f => f.format()),
      this.rpcAdapter,
      this.transactionBuilder,
      this.eventDecoder,
      signer
    );
  }

  /**
   * Get function by name
   */
  getFunction(nameOrSignature: string): any {
    const fragment = this.interface.getFunction(nameOrSignature);
    if (!fragment) {
      throw new Error(`Function ${nameOrSignature} not found`);
    }
    const invocationName = this.getInvocationName(fragment);

    return {
      fragment,
      call: (...args: any[]) => this.call(invocationName, args),
      send: (...args: any[]) => this.send(invocationName, args),
      estimateGas: (...args: any[]) => this.estimateGas(invocationName, args)
    };
  }

  /**
   * Get event by name
   */
  getEvent(nameOrSignature: string): any {
    const fragment = this.interface.getEvent(nameOrSignature);
    if (!fragment) {
      throw new Error(`Event ${nameOrSignature} not found`);
    }

    return {
      fragment,
      queryFilter: (filter?: any, _fromBlock?: number, toBlock?: number) =>
        this.queryFilter(fragment.name, filter, toBlock)
    };
  }

  // Private methods

  private createMethodProxies(): void {
    const configuredOverloadGuards = new Set<string>();

    // Create proxy methods for all functions in the ABI
    for (const fragment of this.interface.fragments) {
      if (fragment.type === 'function') {
        const func = fragment as FunctionFragment;
        const invocationName = this.getInvocationName(func);
        const proxyKey = this.isOverloadedFunction(func.name)
          ? invocationName
          : func.name;

        if (this.isOverloadedFunction(func.name) && !configuredOverloadGuards.has(func.name)) {
          const overloadError = () => {
            throw new Error(
              `Function ${func.name} is overloaded; use a full signature such as ${invocationName}`
            );
          };
          (this as any)[func.name] = async (..._args: any[]) => overloadError();
          (this as any)[func.name].staticCall = (..._args: any[]) => overloadError();
          (this as any)[func.name].estimateGas = (..._args: any[]) => overloadError();
          configuredOverloadGuards.add(func.name);
        }

        // Create method proxy
        (this as any)[proxyKey] = async (...args: any[]) => {
          const options = args.length > func.inputs.length ? args.pop() : {};
          
          if (func.stateMutability === 'view' || func.stateMutability === 'pure') {
            return this.call(invocationName, args, options);
          } else {
            return this.send(invocationName, args, options);
          }
        };

        // Create static call method
        (this as any)[proxyKey].staticCall = (...args: any[]) => {
          const options = args.length > func.inputs.length ? args.pop() : {};
          return this.call(invocationName, args, options);
        };

        // Create estimate gas method
        (this as any)[proxyKey].estimateGas = (...args: any[]) => {
          const options = args.length > func.inputs.length ? args.pop() : {};
          return this.estimateGas(invocationName, args, options);
        };
      }
    }
  }

  private collectOverloadedFunctionNames(): Set<string> {
    const counts = new Map<string, number>();

    for (const fragment of this.interface.fragments) {
      if (fragment.type !== "function") continue;
      const name = (fragment as FunctionFragment).name;
      counts.set(name, (counts.get(name) ?? 0) + 1);
    }

    return new Set(
      [...counts.entries()]
        .filter(([, count]) => count > 1)
        .map(([name]) => name)
    );
  }

  private isOverloadedFunction(name: string): boolean {
    return this.overloadedFunctionNames.has(name);
  }

  private getInvocationName(fragment: FunctionFragment): string {
    if (!this.isOverloadedFunction(fragment.name)) {
      return fragment.name;
    }

    return `${fragment.name}(${fragment.inputs.map(input => input.type).join(",")})`;
  }

  private createEventFilters(): void {
    // Create event filter methods
    for (const fragment of this.interface.fragments) {
      if (fragment.type === 'event') {
        const event = fragment as EventFragment;
        
        // Create filter method
        (this as any)[`${event.name}Filter`] = (...args: any[]) => {
          return {
            address: this.address,
            topics: this.interface.encodeFilterTopics(event, args)
          };
        };
      }
    }
  }

  private startEventListener(eventName: string): void {
    // This would start polling for events or set up WebSocket listeners
    // For now, this is a placeholder
    debug(`Starting event listener for ${eventName}`);
  }

  private async waitForTransaction(
    txHash: string,
    confirmations: number
  ): Promise<any> {
    debug(`Waiting for transaction ${txHash} with ${confirmations} confirmations`);

    // Poll for transaction receipt
    let receipt;
    let attempts = 0;
    const maxAttempts = 60; // 5 minutes with 5s intervals

    while (!receipt && attempts < maxAttempts) {
      try {
        receipt = await this.rpcAdapter.getTransactionReceipt(txHash);
        if (receipt) break;
      } catch {
        // Transaction not yet confirmed
      }

      await new Promise(resolve => setTimeout(resolve, 5000));
      attempts++;
    }

    if (!receipt) {
      throw new Error(`Transaction ${txHash} not confirmed after ${maxAttempts} attempts`);
    }

    // Wait for additional confirmations if needed
    if (confirmations > 1) {
      const targetBlock = receipt.blockNumber + confirmations - 1;
      await this.waitForBlock(targetBlock);
    }

    return receipt;
  }

  private async waitForBlock(blockNumber: number): Promise<void> {
    let currentBlock;
    
    do {
      currentBlock = await this.rpcAdapter.getBlockNumber();
      if (currentBlock >= blockNumber) break;
      
      await new Promise(resolve => setTimeout(resolve, 5000));
    } while (currentBlock < blockNumber);
  }

  private convertArgsToNeo(args: any[], fragment: FunctionFragment): any[] {
    const neoArgs: any[] = [];
    
    for (let i = 0; i < args.length; i++) {
      const arg = args[i];
      const paramType = fragment.inputs[i]?.type || "string";
      
      neoArgs.push(this.convertArgToNeo(arg, paramType));
    }
    
    return neoArgs;
  }

  private convertArgToNeo(arg: any, paramType: string): any {
    // Convert Ethereum argument to Neo parameter format
    switch (paramType) {
      case 'bool':
        return { type: 'Boolean', value: Boolean(arg) };
      case 'address':
        return { type: 'Hash160', value: evmAddressToNeoHash160(String(arg)) };
      case 'string':
        return { type: 'String', value: String(arg) };
      default:
        if (paramType.startsWith('uint') || paramType.startsWith('int')) {
          return { type: 'Integer', value: String(arg) };
        }
        if (paramType.startsWith('bytes')) {
          if (Buffer.isBuffer(arg) || arg instanceof Uint8Array) {
            return { type: 'ByteArray', value: Buffer.from(arg).toString('hex') };
          }
          const asString = String(arg);
          if (asString.startsWith('0x')) {
            return { type: 'ByteArray', value: strip0x(asString) };
          }
          if (isHexString(asString)) {
            return { type: 'ByteArray', value: asString };
          }
          return { type: 'ByteArray', value: Buffer.from(asString, 'utf8').toString('hex') };
        }
        return { type: 'String', value: String(arg) };
    }
  }

  private convertResultToEthereum(result: any, fragment: FunctionFragment): any {
    // Convert Neo result to Ethereum format based on function return types
    if (!result.stack || result.stack.length === 0) {
      return null;
    }

    if (fragment.outputs.length === 1) {
      return this.convertStackItemToEthereum(result.stack[0], fragment.outputs[0].type);
    }

    // Multiple return values
    const returnValue: any = {};
    for (let i = 0; i < Math.min(result.stack.length, fragment.outputs.length); i++) {
      const output = fragment.outputs[i];
      returnValue[i] = this.convertStackItemToEthereum(result.stack[i], output.type);
      if (output.name) {
        returnValue[output.name] = returnValue[i];
      }
    }

    return returnValue;
  }

  private convertStackItemToEthereum(item: any, targetType: string): any {
    if (!item) return null;

    switch (item.type) {
      case 'Boolean':
        return parseNeoBoolean(item.value);
      case 'Integer':
        return targetType.startsWith('uint') || targetType.startsWith('int') 
          ? parseNeoInteger(item.value)
          : Number(item.value);
      case 'ByteString':
      case 'ByteArray':
      case 'Buffer': {
        const bytes = decodeNeoBytes(item.value);
        if (targetType === 'address') {
          return neoBytesToEvmAddress(bytes);
        }
        if (targetType === 'string') {
          return bytes.toString('utf8');
        }
        if (targetType.startsWith('bytes') || targetType === 'bytes') {
          return '0x' + bytes.toString('hex');
        }
        return bytes.toString('hex');
      }
      case 'Hash160':
        if (targetType === 'address') {
          return neoHash160ToEvmAddress(String(item.value));
        }
        return String(item.value);
      case 'Array':
      case 'Struct': {
        const children = stackItemArrayValue(item);
        const elementType = targetType.endsWith('[]') ? targetType.slice(0, -2) : targetType;
        return children.map((subItem: any) => this.convertStackItemToEthereum(subItem, elementType));
      }
      default:
        return item.value;
    }
  }
}
