import { 
  Interface, 
  FunctionFragment, 
  EventFragment,
  Result,
  LogDescription
} from "ethers";
import { 
  NeoRpcProvider,
  InvokeResult
} from "@neo-devpack-solidity/types";
import { RpcAdapter } from "./rpc-adapter.js";
import { TransactionBuilder } from "./transaction-builder.js";
import { EventDecoder } from "./event-decoder.js";
import { ContractWrapper } from "./contract-wrapper.js";
import { NeoDeploymentArtifacts } from "./types.js";
import Debug from "debug";
import {
  decodeNeoBytes,
  evmAddressToNeoHash160,
  neoBytesToEvmAddress,
  neoHash160ToEvmAddress,
  parseNeoBoolean,
  parseNeoInteger,
  stackItemArrayValue,
} from "./neo-utils.js";

const debug = Debug("neo-devpack-solidity:abi-router");

/**
 * ABI Router - Provides Ethereum-compatible interface for Neo contracts
 */
export class AbiRouter {
  private rpcAdapter: RpcAdapter;
  private transactionBuilder: TransactionBuilder;
  private eventDecoder: EventDecoder;

  constructor(rpcProvider: NeoRpcProvider) {
    this.rpcAdapter = new RpcAdapter(rpcProvider);
    this.transactionBuilder = new TransactionBuilder(rpcProvider);
    this.eventDecoder = new EventDecoder();
  }

  /**
   * Create contract wrapper with Ethereum-compatible interface
   */
  createContract(
    address: string,
    abi: any[],
    signerOrProvider?: any
  ): ContractWrapper {
    debug(`Creating contract wrapper for ${address}`);
    
    return new ContractWrapper(
      address,
      abi,
      this.rpcAdapter,
      this.transactionBuilder,
      this.eventDecoder,
      signerOrProvider
    );
  }

  /**
   * Get contract at address with ABI
   */
  getContract(
    address: string,
    abi: any[],
    signerOrProvider?: any
  ): ContractWrapper {
    return this.createContract(address, abi, signerOrProvider);
  }

  /**
   * Deploy contract with Ethereum-compatible interface
   */
  async deployContract(
    artifacts: NeoDeploymentArtifacts,
    abi: any[],
    constructorArgs: any[] = [],
    options: {
      gasLimit?: string;
      gasPrice?: string;
      value?: string;
    } = {}
  ): Promise<ContractWrapper> {
    debug("Deploying contract with ABI compatibility");

    try {
      // Deploy contract using transaction builder
      const deployment = await this.transactionBuilder.deployContract(
        artifacts,
        constructorArgs,
        options
      );

      // Create and return contract wrapper
      return this.createContract(deployment.address, abi);
    } catch (error) {
      debug(`Contract deployment failed: ${error}`);
      throw error;
    }
  }

  /**
   * Encode function call data
   */
  encodeFunctionData(
    functionFragment: string | FunctionFragment,
    args: any[] = []
  ): string {
    debug(`Encoding function data for ${functionFragment}`);

    const iface = new Interface([
      typeof functionFragment === "string" 
        ? `function ${functionFragment}` 
        : functionFragment
    ]);

    const fragment = typeof functionFragment === "string"
      ? iface.getFunction(functionFragment)!
      : functionFragment;

    return iface.encodeFunctionData(fragment, args);
  }

  /**
   * Decode function result data
   */
  decodeFunctionResult(
    functionFragment: string | FunctionFragment,
    data: string
  ): Result {
    debug(`Decoding function result for ${functionFragment}`);

    const iface = new Interface([
      typeof functionFragment === "string" 
        ? `function ${functionFragment}` 
        : functionFragment
    ]);

    const fragment = typeof functionFragment === "string"
      ? iface.getFunction(functionFragment)!
      : functionFragment;

    return iface.decodeFunctionResult(fragment, data);
  }

  /**
   * Encode event filter topics
   */
  encodeFilterTopics(
    eventFragment: string | EventFragment,
    values: any[] = []
  ): Array<string | string[] | null> {
    debug(`Encoding filter topics for ${eventFragment}`);

    const iface = new Interface([
      typeof eventFragment === "string" 
        ? `event ${eventFragment}` 
        : eventFragment
    ]);

    const fragment = typeof eventFragment === "string"
      ? iface.getEvent(eventFragment)!
      : eventFragment;

    return iface.encodeFilterTopics(fragment, values);
  }

  /**
   * Decode event log
   */
  decodeEventLog(
    eventFragment: string | EventFragment,
    data: string,
    topics: string[]
  ): LogDescription {
    debug(`Decoding event log for ${eventFragment}`);

    const iface = new Interface([
      typeof eventFragment === "string" 
        ? `event ${eventFragment}` 
        : eventFragment
    ]);

    const parsed = iface.parseLog({ data, topics });
    if (!parsed) {
      throw new Error("Failed to parse event log");
    }
    return parsed;
  }

  /**
   * Convert Neo stack result to Ethereum format
   */
  convertNeoResultToEthereum(
    neoResult: InvokeResult,
    returnTypes: string[]
  ): any {
    debug("Converting Neo result to Ethereum format");

    if (!neoResult.stack || neoResult.stack.length === 0) {
      return null;
    }

    // Single return value
    if (returnTypes.length === 1) {
      return this.convertStackItem(neoResult.stack[0], returnTypes[0]);
    }

    // Multiple return values
    const result: any = {};
    for (let i = 0; i < Math.min(neoResult.stack.length, returnTypes.length); i++) {
      result[i] = this.convertStackItem(neoResult.stack[i], returnTypes[i]);
    }

    return result;
  }

  /**
   * Convert Ethereum arguments to Neo format
   */
  convertEthereumArgsToNeo(args: any[], paramTypes: string[]): any[] {
    debug("Converting Ethereum args to Neo format");

    const neoArgs: any[] = [];

    for (let i = 0; i < args.length; i++) {
      const arg = args[i];
      const paramType = paramTypes[i] || "string";
      
      neoArgs.push(this.convertEthereumArgToNeo(arg, paramType));
    }

    return neoArgs;
  }

  /**
   * Get function selector from signature
   */
  getFunctionSelector(signature: string): string {
    const iface = new Interface([`function ${signature}`]);
    const fragment = iface.getFunction(signature)!;
    return fragment.selector;
  }

  /**
   * Get event topic hash from signature
   */
  getEventTopicHash(signature: string): string {
    const iface = new Interface([`event ${signature}`]);
    const fragment = iface.getEvent(signature)!;
    return fragment.topicHash;
  }

  // Private methods

  private convertStackItem(item: any, targetType: string): any {
    if (!item) return null;

    switch (item.type) {
      case 'Boolean':
        return parseNeoBoolean(item.value);

      case 'Integer':
        if (targetType.startsWith('uint') || targetType.startsWith('int')) {
          return parseNeoInteger(item.value);
        }
        return Number(item.value);

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
        return children.map((subItem: any) =>
          this.convertStackItem(subItem, this.getArrayElementType(targetType))
        );
      }

      case 'Map':
        // Convert Neo map to object
        {
          const obj: any = {};
          for (const [key, value] of Object.entries(item.value || {})) {
            obj[key] = this.convertStackItem(value, 'string');
          }
          return obj;
        }

      default:
        return item.value;
    }
  }

  private convertEthereumArgToNeo(arg: any, paramType: string): any {
    switch (paramType) {
      case 'bool':
        return {
          type: 'Boolean',
          value: Boolean(arg)
        };

      case 'address':
        return {
          type: 'Hash160',
          value: evmAddressToNeoHash160(String(arg))
        };

      case 'string':
        return {
          type: 'String',
          value: String(arg)
        };

      default:
        if (paramType.startsWith('uint') || paramType.startsWith('int')) {
          return {
            type: 'Integer',
            value: String(arg)
          };
        }
        
        if (paramType.startsWith('bytes')) {
          const asString = String(arg);
          return {
            type: 'ByteArray',
            value: asString.startsWith('0x') ? asString.slice(2) : asString
          };
        }

        // Default to string representation
        return {
          type: 'String',
          value: String(arg)
        };
    }
  }

  private getArrayElementType(arrayType: string): string {
    // Extract element type from array type (e.g., "uint256[]" -> "uint256")
    return arrayType.replace(/\[\]$/, '');
  }
}
