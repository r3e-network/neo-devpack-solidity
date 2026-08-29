import { base64ToHex, isHexString, strip0x } from "@neo-devpack-solidity/types";

export function reverseHexBytes(value: string): string {
  const hex = strip0x(value);
  if (hex.length % 2 !== 0 || !/^[0-9a-fA-F]*$/u.test(hex)) {
    throw new Error(`Expected even-length hex string, got: ${value}`);
  }
  return Buffer.from(hex, "hex").reverse().toString("hex");
}

export function base64ScriptToHex(value: string): string {
  return strip0x(base64ToHex(value));
}

export function normalizeByteArrayInput(value: string | Buffer | Uint8Array): string {
  if (Buffer.isBuffer(value) || value instanceof Uint8Array) {
    return Buffer.from(value).toString("hex");
  }

  const text = value.trim();
  if (isHexString(text)) {
    return strip0x(text);
  }

  return strip0x(base64ToHex(text));
}

export function decimalToIntegerString(value: string | number, decimals: number): string {
  const text = String(value).trim();
  if (!/^[+-]?\d+(?:\.\d+)?$/u.test(text)) {
    throw new Error(`Invalid decimal value: ${value}`);
  }

  const negative = text.startsWith("-");
  const unsigned = negative || text.startsWith("+") ? text.slice(1) : text;
  const [wholePart, fractionPart = ""] = unsigned.split(".");

  const paddedFraction = (fractionPart + "0".repeat(decimals)).slice(0, decimals);
  const combined = `${wholePart}${paddedFraction}`.replace(/^0+(?=\d)/u, "") || "0";
  return negative ? `-${combined}` : combined;
}
