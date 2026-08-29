import { describe, expect, it, vi } from "vitest";
import { ContractWrapper } from "../src/contract-wrapper";

function createWrapper() {
  const invokeFunction = vi.fn(async (_address: string, methodName: string) => ({
    stack: [{ type: "Integer", value: methodName.includes(",") ? "11" : "7" }],
  }));

  const estimateGas = vi.fn(async () => ({
    systemFee: "1",
    networkFee: "1",
    totalGas: "2",
  }));

  const rpcAdapter = {
    invokeFunction,
    estimateGas,
    getTransactionReceipt: vi.fn(),
    getBlockNumber: vi.fn(),
    getEvents: vi.fn(),
  } as any;

  const transactionBuilder = {
    sendTransaction: vi.fn(async () => ({ hash: "0x1234" })),
  } as any;

  const eventDecoder = {
    decodeEvent: vi.fn(),
  } as any;

  const abi = [
    "function foo(uint256 value) view returns (uint256)",
    "function foo(uint256 value, uint256 extra) view returns (uint256)",
    "function bar(uint256 value) view returns (uint256)",
  ];

  return {
    wrapper: new ContractWrapper(
      "0xcf76e28bd0062c4a478ee35561011319f3cfa4d2",
      abi,
      rpcAdapter,
      transactionBuilder,
      eventDecoder
    ),
    invokeFunction,
    estimateGas,
  };
}

describe("contract-wrapper overload handling", () => {
  it("invokes overloaded functions by signature", async () => {
    const { wrapper, invokeFunction } = createWrapper();

    const result = await wrapper.getFunction("foo(uint256)").call(7);

    expect(result).toBe(7n);
    expect(invokeFunction).toHaveBeenCalledWith(
      "0xcf76e28bd0062c4a478ee35561011319f3cfa4d2",
      "foo(uint256)",
      [{ type: "Integer", value: "7" }],
      undefined
    );
  });

  it("creates signature-keyed proxies for overloaded functions", async () => {
    const { wrapper, invokeFunction } = createWrapper();

    const result = await (wrapper as any)["foo(uint256,uint256)"](5, 6);

    expect(result).toBe(11n);
    expect(invokeFunction).toHaveBeenCalledWith(
      "0xcf76e28bd0062c4a478ee35561011319f3cfa4d2",
      "foo(uint256,uint256)",
      [
        { type: "Integer", value: "5" },
        { type: "Integer", value: "6" },
      ],
      undefined
    );
  });

  it("rejects bare overloaded proxy names with a helpful error", async () => {
    const { wrapper } = createWrapper();

    await expect((wrapper as any).foo(1)).rejects.toThrow(/overloaded/i);
  });

  it("keeps bare-name proxies for non-overloaded functions", async () => {
    const { wrapper, invokeFunction } = createWrapper();

    const result = await (wrapper as any).bar(7);

    expect(result).toBe(7n);
    expect(invokeFunction).toHaveBeenCalledWith(
      "0xcf76e28bd0062c4a478ee35561011319f3cfa4d2",
      "bar",
      [{ type: "Integer", value: "7" }],
      undefined
    );
  });
});
