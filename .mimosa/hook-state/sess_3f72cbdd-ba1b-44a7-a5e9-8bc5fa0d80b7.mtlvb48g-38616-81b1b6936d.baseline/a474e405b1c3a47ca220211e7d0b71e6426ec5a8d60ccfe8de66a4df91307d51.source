import { describe, expect, it } from "vitest";
import { createContractCallScript } from "../src/script-builder";

describe("script-builder", () => {
  it("matches neon-js for contract calls without arguments", () => {
    expect(
      createContractCallScript({
        scriptHash: "fffdc93764dbaddd97c48f252a53ea4643faa3fd",
        operation: "ping",
        args: [],
      })
    ).toBe("c21f0c0470696e670c14fda3fa4346ea532a258fc497ddaddb6437c9fdff41627d5b52");
  });

  it("matches neon-js for mixed contract parameters", () => {
    const args = [
      { type: "String", value: "hello" },
      { type: "Integer", value: "42" },
      { type: "Boolean", value: true },
      { type: "ByteArray", value: { toLittleEndian: () => "01020304" } },
      { type: "Array", value: [{ type: "String", value: "nested" }] },
    ];

    expect(
      createContractCallScript({
        scriptHash: "d2a4cff31913016155e38e474a2c06d08be276cf",
        operation: "mixed",
        args,
      })
    ).toBe(
      "0c066e657374656411c00c040102030408002a0c0568656c6c6f15c01f0c056d697865640c14cf76e28bd0062c4a478ee35561011319f3cfa4d241627d5b52"
    );
  });
});
