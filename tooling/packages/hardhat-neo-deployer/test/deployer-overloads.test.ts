import { describe, expect, it, vi } from "vitest";
import { NeoDeployer } from "../src/deployer";

function createDeployer() {
  const rpc = {
    invokeScript: vi.fn(async () => ({ stack: [] })),
  } as any;

  const accounts = {
    getDefaultAccount: vi.fn(() => undefined),
    getDefaultSigner: vi.fn(() => undefined),
    getAccount: vi.fn(() => undefined),
  } as any;

  const deployer = new NeoDeployer(rpc, accounts, {}, "test", 0x35);

  const artifact = {
    contractName: "OverloadedApi",
    sourceName: "OverloadedApi.sol",
    metadata: {} as any,
    buildInfo: "build-info.json",
    contract: {
      abi: [],
      neo: {
        manifest: {
          name: "OverloadedApi",
          abi: {
            methods: [
              {
                name: "foo(uint256)",
                parameters: [{ name: "value", type: "Integer" }],
                returntype: "Integer",
                safe: true,
              },
              {
                name: "foo(uint256,uint256)",
                parameters: [
                  { name: "value", type: "Integer" },
                  { name: "extra", type: "Integer" },
                ],
                returntype: "Integer",
                safe: true,
              },
              {
                name: "bar",
                parameters: [{ name: "value", type: "Integer" }],
                returntype: "Integer",
                safe: true,
              },
            ],
            events: [],
          },
        },
        methodMap: {
          "foo(uint256)": "foo(uint256)",
          "foo(uint256,uint256)": "foo(uint256,uint256)",
          "bar(uint256)": "bar",
        },
      },
    },
  } as any;

  const contract = (deployer as any).createContractInstance(
    artifact,
    "0xcf76e28bd0062c4a478ee35561011319f3cfa4d2"
  );

  return { contract, rpc };
}

describe("hardhat neo deployer overload handling", () => {
  it("exposes signature-keyed methods for overloads", async () => {
    const { contract, rpc } = createDeployer();

    await contract.methods["foo(uint256)"].call(7);
    await contract.methods["foo(uint256,uint256)"].call(7, 8);

    expect(rpc.invokeScript).toHaveBeenCalledTimes(2);
  });

  it("rejects ambiguous bare overload names", async () => {
    const { contract } = createDeployer();

    await expect(contract.methods["foo"].call(7)).rejects.toThrow(/overloaded|signature/i);
  });

  it("keeps bare names for non-overloaded methods", async () => {
    const { contract, rpc } = createDeployer();

    await contract.methods["bar"].call(9);

    expect(rpc.invokeScript).toHaveBeenCalledTimes(1);
  });
});
