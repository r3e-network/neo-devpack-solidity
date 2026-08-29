export {
  DEFAULT_NEO_ADDRESS_VERSION as DEFAULT_ADDRESS_VERSION,
  decodeNeoBytes,
  evmAddressToNeoHash160,
  isHexString,
  neoBytesToEvmAddress,
  neoHash160ToEvmAddress,
  neoScriptHashToAddress,
  normalizeNeoHash160,
  strip0x,
} from "@neo-devpack-solidity/types";

export function stackItemArrayValue(item: any): any[] {
  if (!item) return [];
  if (Array.isArray(item)) return item;
  if (typeof item === "object" && item.type === "Array" && Array.isArray(item.value)) return item.value;
  if (typeof item === "object" && item.type === "Struct" && Array.isArray(item.value)) return item.value;
  return [];
}

export function parseNeoBoolean(value: unknown): boolean {
  if (typeof value === "boolean") return value;
  if (typeof value === "number") return value !== 0;
  if (typeof value === "bigint") return value !== 0n;
  if (typeof value === "string") {
    const trimmed = value.trim().toLowerCase();
    if (trimmed === "true" || trimmed === "1") return true;
    if (trimmed === "false" || trimmed === "0") return false;
  }
  return Boolean(value);
}

export function parseNeoInteger(value: unknown): bigint {
  if (typeof value === "bigint") return value;
  if (typeof value === "number") return BigInt(Math.trunc(value));
  if (typeof value === "string") {
    const trimmed = value.trim();
    if (!trimmed) return 0n;
    return BigInt(trimmed);
  }
  return 0n;
}
