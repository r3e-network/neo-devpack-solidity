import { describe, expect, it } from "vitest";
import { NeoDevpackSolidityCompiler } from "../src/compiler";
import { isSupportedSolidityVersion } from "@neo-devpack-solidity/types";
import type { CompilationOutput, NeoHardhatConfig } from "@neo-devpack-solidity/types";

const MINIMAL_CONFIG: NeoHardhatConfig = {
  solidity: {
    version: "0.8.19",
    settings: {
      optimizer: { enabled: true, runs: 200 },
      outputSelection: { "*": { "*": ["abi", "evm.bytecode", "metadata"] } },
      neo: {},
    },
  },
  networks: {},
  paths: {
    sources: "./contracts",
    artifacts: "./artifacts",
    cache: "./cache",
    tests: "./test",
  },
  neo: {
    rpcUrl: "http://127.0.0.1:10332",
    privateKey: "",
    addressVersion: 53,
    magic: 860833102,
    gasLimit: "9007199254740991",
    gasPrice: "0",
  },
};

describe("@neo-devpack-solidity/hardhat-solc-neo", () => {
  it("extracts versions from neo-solc output", () => {
    const compiler = new NeoDevpackSolidityCompiler(MINIMAL_CONFIG, { cache: "/tmp" });
    const parsed = (compiler as any).parseVersionOutput(
      "neo-devpack-solidity: 0.9.10\nsolidity: 0.8.19\n"
    ) as string[];

    expect(parsed).toContain("0.9.10");
    expect(parsed).toContain("solidity-0.8.19");
  });

  it("validates version strings", () => {
    const compiler = new NeoDevpackSolidityCompiler(MINIMAL_CONFIG, { cache: "/tmp" });

    expect((compiler as any).isValidVersion("0.1.0")).toBe(true);
    expect((compiler as any).isValidVersion("0.1.0-alpha.1")).toBe(true);
    expect((compiler as any).isValidVersion("latest")).toBe(true);
    expect((compiler as any).isValidVersion("nope")).toBe(false);
  });

  it("accepts only supported Solidity versions", () => {
    expect(isSupportedSolidityVersion(MINIMAL_CONFIG.solidity.version)).toBe(true);

    const invalidConfig: NeoHardhatConfig = {
      ...MINIMAL_CONFIG,
      solidity: { ...MINIMAL_CONFIG.solidity, version: "0.8.34" },
    };
    expect(() => new NeoDevpackSolidityCompiler(invalidConfig, { cache: "/tmp" })).toThrow(
      /Supported range is/
    );
  });

  it("computes compilation stats from standard-json output", () => {
    const compiler = new NeoDevpackSolidityCompiler(MINIMAL_CONFIG, { cache: "/tmp" });

    const output = {
      sources: {},
      contracts: {
        "Counter.sol": {
          Counter: {
            abi: [],
            metadata: "{}",
            evm: { bytecode: { object: "" } },
            neo: {
              nef: {
                magic: "NEF3",
                compiler: "neo-devpack-solidity",
                source: "Counter.sol",
                tokens: [],
                script: "aa".repeat(10),
                image: "bb".repeat(20),
                checksum: "deadbeef",
              },
              manifest: {
                name: "Counter",
                groups: [],
                features: {},
                supportedstandards: [],
                abi: { methods: [], events: [] },
                permissions: [],
                trusts: [],
                extra: {},
              },
              storageMap: {},
              gasEstimates: {
                creation: { gas: "0", systemFee: "0", networkFee: "0" },
                functions: {},
              },
            },
          },
        },
      },
      errors: [
        { severity: "warning", message: "warn" },
        { severity: "error", message: "err" },
      ],
    } as unknown as CompilationOutput;

    const stats = compiler.getCompilationStats(output);
    expect(stats.contractCount).toBe(1);
    expect(stats.warningCount).toBe(1);
    expect(stats.errorCount).toBe(1);
    expect(stats.totalSize).toBe(10);
  });
});
