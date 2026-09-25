import { EventFragment, LogDescription, Result } from "ethers";
import Debug from "debug";
import {
  decodeNeoBytes,
  neoBytesToEvmAddress,
  neoHash160ToEvmAddress,
  parseNeoBoolean,
  parseNeoInteger,
  stackItemArrayValue,
} from "./neo-utils.js";

const debug = Debug("neo-devpack-solidity:event-decoder");

/**
 * Event decoder for Neo events to Ethereum format
 */
export class EventDecoder {

  /**
   * Decode Neo event to Ethereum log format
   */
  decodeEvent(neoEvent: any, eventFragment: EventFragment): LogDescription {
    debug(`Decoding event ${eventFragment.name}`);

    const stateItems = this.extractStateItems(neoEvent?.state);
    const values = eventFragment.inputs.map((input, index) =>
      this.convertStackItem(stateItems[index], input.type)
    );
    const keys = eventFragment.inputs.map((input) => (input.name ? input.name : null));
    const args = Result.fromItems(values, keys);

    return new LogDescription(eventFragment, eventFragment.topicHash, args);
  }

  /**
   * Decode multiple events
   */
  decodeEvents(neoEvents: any[], eventFragment: EventFragment): LogDescription[] {
    return neoEvents.map(event => this.decodeEvent(event, eventFragment));
  }

  // Private methods

  private extractStateItems(state: any): any[] {
    if (!state) return [];
    if (Array.isArray(state)) return state;
    if (typeof state === "object" && (state.type === "Array" || state.type === "Struct")) {
      return Array.isArray(state.value) ? state.value : [];
    }
    return [];
  }

  private convertStackItem(item: any, targetType: string): any {
    if (targetType.endsWith("[]")) {
      const elementType = targetType.slice(0, -2);
      const children = stackItemArrayValue(item);
      return children.map((child) => this.convertStackItem(child, elementType));
    }

    if (!item) {
      if (targetType === "bool") return false;
      if (targetType.startsWith("uint") || targetType.startsWith("int")) return 0n;
      if (targetType === "address") return "0x" + "0".repeat(40);
      if (targetType === "string") return "";
      if (targetType.startsWith("bytes")) return "0x";
      return null;
    }

    if (targetType === "address") {
      if (item.type === "Hash160") {
        return neoHash160ToEvmAddress(String(item.value));
      }
      return neoBytesToEvmAddress(item.value);
    }

    if (targetType === "string") {
      if (item.type === "ByteString" || item.type === "ByteArray" || item.type === "Buffer") {
        return decodeNeoBytes(item.value).toString("utf8");
      }
      return String(item.value ?? "");
    }

    if (targetType === "bool") {
      return parseNeoBoolean(item.value);
    }

    if (targetType.startsWith("uint") || targetType.startsWith("int")) {
      if (item.type === "Integer") return parseNeoInteger(item.value);
      if (typeof item.value === "number" || typeof item.value === "bigint" || typeof item.value === "string") {
        return parseNeoInteger(item.value);
      }
      return 0n;
    }

    if (targetType.startsWith("bytes")) {
      const bytes = item.type === "ByteString" || item.type === "ByteArray" || item.type === "Buffer"
        ? decodeNeoBytes(item.value)
        : Buffer.from([]);
      return "0x" + bytes.toString("hex");
    }

    return item.value;
  }
}
