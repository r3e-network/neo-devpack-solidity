import { describe, expect, it } from "vitest";
import {
  base64ScriptToHex,
  decimalToIntegerString,
  normalizeByteArrayInput,
  reverseHexBytes,
} from "../src/neo-primitives";

describe("neo-primitives", () => {
  it("converts fixed decimal strings to integer strings", () => {
    expect(decimalToIntegerString("1.23", 8)).toBe("123000000");
    expect(decimalToIntegerString("42", 0)).toBe("42");
  });

  it("converts base64 scripts to hex", () => {
    expect(base64ScriptToHex("dGVzdA==")).toBe("74657374");
  });

  it("reverses hex byte order", () => {
    expect(reverseHexBytes("0x010203")).toBe("030201");
  });

  it("normalizes byte array inputs to little-endian hex payloads", () => {
    expect(normalizeByteArrayInput("0x01020304")).toBe("01020304");
    expect(normalizeByteArrayInput("AQIDBA==")).toBe("01020304");
    expect(normalizeByteArrayInput(Buffer.from([1, 2, 3, 4]))).toBe("01020304");
  });
});
