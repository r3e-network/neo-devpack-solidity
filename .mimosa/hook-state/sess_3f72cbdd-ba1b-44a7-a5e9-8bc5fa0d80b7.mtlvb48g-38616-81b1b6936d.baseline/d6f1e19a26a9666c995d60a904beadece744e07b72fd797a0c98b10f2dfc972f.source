import { describe, expect, it } from "vitest";
import {
  decodeNeoBytes,
  evmAddressToNeoHash160,
  neoHash160ToEvmAddress,
  neoBytesToEvmAddress,
  neoScriptHashToAddress,
} from "../src/neo-utils";

describe("neo-utils", () => {
  it("decodes ByteString as base64 or hex", () => {
    expect(decodeNeoBytes("dGVzdA==").toString("utf8")).toBe("test");
    expect(decodeNeoBytes("74657374").toString("utf8")).toBe("test");
    expect(decodeNeoBytes("0x74657374").toString("utf8")).toBe("test");
  });

  it("converts EVM address hex to Neo Hash160 (little-endian)", () => {
    const evm = "0x11223344556677889900aabbccddeeff00112233";
    expect(evmAddressToNeoHash160(evm)).toBe("0x33221100ffeeddccbbaa00998877665544332211");
  });

  it("accepts base58 addresses and returns Neo Hash160", () => {
    const scriptHash = "d2a4cff31913016155e38e474a2c06d08be276cf";
    const address = neoScriptHashToAddress("0x" + scriptHash);
    expect(evmAddressToNeoHash160(address)).toBe("0x" + scriptHash);
  });

  it("converts Neo Hash160 to EVM address hex (big-endian)", () => {
    const scriptHash = "d2a4cff31913016155e38e474a2c06d08be276cf";
    expect(neoHash160ToEvmAddress("0x" + scriptHash)).toBe(
      "0xcf76e28bd0062c4a478ee35561011319f3cfa4d2"
    );
  });

  it("converts 20-byte ByteString to EVM address", () => {
    const bytesLeHex = "d2a4cff31913016155e38e474a2c06d08be276cf";
    expect(neoBytesToEvmAddress(bytesLeHex)).toBe(
      "0xcf76e28bd0062c4a478ee35561011319f3cfa4d2"
    );
  });
});
