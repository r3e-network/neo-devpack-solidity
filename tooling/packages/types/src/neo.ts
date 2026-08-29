import { createHash } from "crypto";

export const DEFAULT_NEO_ADDRESS_VERSION = 0x35;

const BASE58_ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
const BASE64_RE = /^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/;

export function strip0x(value: string): string {
  return value.startsWith("0x") ? value.slice(2) : value;
}

export function normalize0xHex(value: string): string {
  return "0x" + strip0x(value).toLowerCase();
}

export function isHexString(value: string): boolean {
  const hex = strip0x(value);
  return hex.length > 0 && hex.length % 2 === 0 && /^[0-9a-fA-F]+$/.test(hex);
}

export function hexToBase64(value: string): string {
  const hex = strip0x(value);
  if (!isHexString(hex)) {
    throw new Error(`Expected even-length hex string, got: ${value}`);
  }
  return Buffer.from(hex, "hex").toString("base64");
}

export function base64ToHex(value: string): string {
  const trimmed = value.trim();
  if (!trimmed || !BASE64_RE.test(trimmed)) {
    throw new Error(`Invalid base64 string: ${value}`);
  }

  const bytes = Buffer.from(trimmed, "base64");
  const normalizedInput = trimmed.replace(/=+$/u, "");
  const normalizedRoundTrip = bytes.toString("base64").replace(/=+$/u, "");
  if (normalizedInput !== normalizedRoundTrip) {
    throw new Error(`Invalid base64 string: ${value}`);
  }

  return "0x" + bytes.toString("hex");
}

export function decodeNeoBytes(value: unknown): Buffer {
  if (value == null) return Buffer.alloc(0);
  if (Buffer.isBuffer(value)) return value;
  if (value instanceof Uint8Array) return Buffer.from(value);

  if (typeof value === "string") {
    const trimmed = value.trim();
    if (!trimmed) return Buffer.alloc(0);

    if (trimmed.startsWith("0x")) {
      return Buffer.from(strip0x(trimmed), "hex");
    }

    if (isHexString(trimmed)) {
      return Buffer.from(trimmed, "hex");
    }

    return Buffer.from(trimmed, "base64");
  }

  if (Array.isArray(value) && value.every((item) => typeof item === "number")) {
    return Buffer.from(value);
  }

  throw new Error(`Unsupported ByteString/ByteArray value: ${typeof value}`);
}

function sha256(data: Buffer): Buffer {
  return createHash("sha256").update(data).digest();
}

function base58CheckEncode(payload: Buffer): string {
  const checksum = sha256(sha256(payload)).subarray(0, 4);
  const full = Buffer.concat([payload, checksum]);

  let value = BigInt("0x" + (full.toString("hex") || "00"));
  let encoded = "";
  while (value > 0n) {
    const remainder = Number(value % 58n);
    encoded = BASE58_ALPHABET[remainder] + encoded;
    value /= 58n;
  }

  const leadingZeros = full.findIndex((byte) => byte !== 0);
  const zeroCount = leadingZeros === -1 ? full.length : leadingZeros;
  return "1".repeat(zeroCount) + (encoded || "");
}

function base58CheckDecode(value: string): Buffer {
  let num = 0n;
  for (const char of value) {
    const index = BASE58_ALPHABET.indexOf(char);
    if (index === -1) {
      throw new Error(`Invalid Neo address character: ${char}`);
    }
    num = num * 58n + BigInt(index);
  }

  const hex = num.toString(16);
  const decoded = Buffer.from(hex.length % 2 === 0 ? hex : "0" + hex, "hex");
  const leadingOnes = value.match(/^1*/u)?.[0]?.length ?? 0;
  const payloadWithChecksum = Buffer.concat([Buffer.alloc(leadingOnes), decoded]);

  if (payloadWithChecksum.length < 5) {
    throw new Error("Invalid Neo address length");
  }

  const payload = payloadWithChecksum.subarray(0, payloadWithChecksum.length - 4);
  const checksum = payloadWithChecksum.subarray(payloadWithChecksum.length - 4);
  const expected = sha256(sha256(payload)).subarray(0, 4);
  if (!expected.equals(checksum)) {
    throw new Error("Invalid Neo address checksum");
  }

  return payload;
}

function decodeAddressPayload(address: string): Buffer {
  const payload = base58CheckDecode(address.trim());
  if (payload.length !== 21) {
    throw new Error(`Invalid Neo address payload length: ${payload.length}`);
  }
  return payload;
}

export function getNeoAddressVersion(address: string): number {
  return decodeAddressPayload(address)[0];
}

export function isNeoAddress(address: string, expectedVersion?: number): boolean {
  try {
    const version = getNeoAddressVersion(address);
    return expectedVersion == null || version === expectedVersion;
  } catch {
    return false;
  }
}

export function neoAddressToScriptHash(address: string, expectedVersion?: number): string {
  const payload = decodeAddressPayload(address);
  const version = payload[0];
  if (expectedVersion != null && version !== expectedVersion) {
    throw new Error(
      `Address version mismatch: expected 0x${expectedVersion.toString(16)}, got 0x${version.toString(16)}`
    );
  }
  return "0x" + Buffer.from(payload.subarray(1)).reverse().toString("hex");
}

export function neoScriptHashToAddress(
  scriptHash: string,
  addressVersion = DEFAULT_NEO_ADDRESS_VERSION
): string {
  const hex = strip0x(scriptHash);
  if (!/^[0-9a-fA-F]{40}$/u.test(hex)) {
    throw new Error(`Expected 20-byte script hash, got: ${scriptHash}`);
  }
  const payload = Buffer.concat([
    Buffer.from([addressVersion]),
    Buffer.from(Buffer.from(hex, "hex")).reverse(),
  ]);
  return base58CheckEncode(payload);
}

export function normalizeNeoHash160(value: string, expectedVersion?: number): string {
  const trimmed = value.trim();

  if (isNeoAddress(trimmed, expectedVersion)) {
    return neoAddressToScriptHash(trimmed, expectedVersion);
  }

  const hex = strip0x(trimmed);
  if (!/^[0-9a-fA-F]{40}$/u.test(hex)) {
    throw new Error(`Expected 20-byte hex (Hash160), got: ${value}`);
  }

  return "0x" + hex.toLowerCase();
}

export function evmAddressToNeoHash160(address: string, expectedVersion?: number): string {
  const trimmed = address.trim();

  if (isNeoAddress(trimmed, expectedVersion)) {
    return neoAddressToScriptHash(trimmed, expectedVersion);
  }

  const hex = strip0x(trimmed);
  if (!/^[0-9a-fA-F]{40}$/u.test(hex)) {
    throw new Error(`Expected address as base58 or 0x + 40 hex chars, got: ${address}`);
  }

  return "0x" + Buffer.from(hex, "hex").reverse().toString("hex");
}

export function neoHash160ToEvmAddress(value: string, expectedVersion?: number): string {
  const trimmed = value.trim();

  if (isNeoAddress(trimmed, expectedVersion)) {
    return neoHash160ToEvmAddress(neoAddressToScriptHash(trimmed, expectedVersion));
  }

  const hexLe = strip0x(trimmed);
  if (!/^[0-9a-fA-F]{40}$/u.test(hexLe)) {
    throw new Error(`Expected Hash160 as base58 or 0x + 40 hex chars, got: ${value}`);
  }

  return "0x" + Buffer.from(hexLe, "hex").reverse().toString("hex");
}

export function neoBytesToEvmAddress(bytesValue: unknown): string {
  const bytes = decodeNeoBytes(bytesValue);
  if (bytes.length !== 20) {
    throw new Error(`Expected 20 bytes for address, got ${bytes.length}`);
  }
  return "0x" + Buffer.from(bytes).reverse().toString("hex");
}
