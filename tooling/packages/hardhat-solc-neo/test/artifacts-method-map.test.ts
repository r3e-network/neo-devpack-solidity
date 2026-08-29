import { describe, expect, it } from "vitest";
import { mkdtempSync, rmSync } from "fs";
import { tmpdir } from "os";
import { join } from "path";
import { ArtifactManager } from "../src/artifacts";
import type { BuildArtifact } from "@neo-devpack-solidity/types";

function buildArtifact(methodMap: Record<string, string>): BuildArtifact {
  return {
    contractName: "OverloadedApi",
    sourceName: "contracts/OverloadedApi.sol",
    metadata: {
      compiler: { version: "0.1.0", settings: {} },
      buildTime: new Date().toISOString(),
      environment: {
        nodeVersion: process.version,
        platform: process.platform,
        architecture: process.arch,
      },
      dependencies: {},
    },
    buildInfo: "build-info.json",
    contract: {
      abi: [],
      metadata: "{}",
      evm: { bytecode: { object: "" } },
      neo: {
        nef: {
          magic: "NEF3",
          compiler: "neo-devpack-solidity",
          source: "OverloadedApi.sol",
          tokens: [],
          script: "aa",
          image: "aa",
          checksum: "deadbeef",
        },
        manifest: {
          name: "OverloadedApi",
          groups: [],
          features: {},
          supportedstandards: [],
          abi: {
            methods: [
              {
                name: "foo(uint256)",
                offset: 0,
                parameters: [{ name: "value", type: "Integer" }],
                returntype: "Integer",
                safe: true,
              },
            ],
            events: [],
          },
          permissions: [],
          trusts: [],
          extra: {},
        },
        methodMap,
        storageMap: {},
        gasEstimates: {
          creation: { gas: "0", systemFee: "0", networkFee: "0" },
          functions: {},
        },
      },
    } as any,
  };
}

describe("ArtifactManager methodMap compatibility", () => {
  it("treats Neo method map changes as breaking", async () => {
    const dir = mkdtempSync(join(tmpdir(), "neo-artifacts-"));
    try {
      const manager = new ArtifactManager(dir);
      const before = buildArtifact({ "foo(uint256)": "foo(uint256)" });
      const after = buildArtifact({ "foo(uint256)": "foo_renamed(uint256)" });

      const comparison = await manager.compareArtifacts(before, after);

      expect(comparison.identical).toBe(false);
      expect(
        comparison.differences.some((diff) => diff.path === "contract.neo.methodMap")
      ).toBe(true);
      expect(
        comparison.compatibility.breakingChanges.some((message) =>
          message.includes("Neo method map")
        )
      ).toBe(true);
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });
});
