import { describe, expect, it } from "vitest";
import {
  base64ToHex,
  evmAddressToNeoHash160,
  hexToBase64,
  isNeoAddress,
  neoAddressToScriptHash,
  neoHash160ToEvmAddress,
  neoScriptHashToAddress,
} from "../src/neo";

describe("neo helpers", () => {
  const scriptHashLe = "0xd2a4cff31913016155e38e474a2c06d08be276cf";
  const neoAddress = "NepwUjd9GhqgNkrfXaxj9mmsFhFzGoFuWM";

  it("converts Neo N3 addresses to script hashes", () => {
    expect(isNeoAddress(neoAddress)).toBe(true);
    expect(neoAddressToScriptHash(neoAddress)).toBe(scriptHashLe);
  });

  it("converts script hashes back to Neo N3 addresses", () => {
    expect(neoScriptHashToAddress(scriptHashLe)).toBe(neoAddress);
  });

  it("converts between Neo Hash160 and EVM address endianness", () => {
    expect(neoHash160ToEvmAddress(scriptHashLe)).toBe(
      "0xcf76e28bd0062c4a478ee35561011319f3cfa4d2"
    );
    expect(evmAddressToNeoHash160("0xcf76e28bd0062c4a478ee35561011319f3cfa4d2")).toBe(scriptHashLe);
  });

  it("round-trips hex and base64 encoding", () => {
    const hex = "0x74657374";
    const base64 = hexToBase64(hex);
    expect(base64).toBe("dGVzdA==");
    expect(base64ToHex(base64)).toBe(hex);
  });

  it("rejects invalid Neo address checksums", () => {
    expect(() => neoAddressToScriptHash("NepwUjd9GhqgNkrfXaxj9mmsFhFzGoFuWN")).toThrow(
      /checksum|invalid/i
    );
  });
});
