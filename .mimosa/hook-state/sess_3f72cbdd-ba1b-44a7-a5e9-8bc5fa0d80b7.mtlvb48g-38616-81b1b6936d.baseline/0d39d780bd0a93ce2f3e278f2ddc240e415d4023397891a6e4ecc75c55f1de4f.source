import fs from "fs-extra";
import os from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { NeoForge } from "@neo-devpack-solidity/neo-foundry";

const tempRoots: string[] = [];

afterEach(async () => {
  while (tempRoots.length > 0) {
    const dir = tempRoots.pop();
    if (dir) {
      await fs.remove(dir);
    }
  }
});

async function makeTempDir(prefix: string): Promise<string> {
  const dir = await fs.mkdtemp(path.join(os.tmpdir(), prefix));
  tempRoots.push(dir);
  return dir;
}

function getNeoSolcPath(): string {
  const fromEnv = process.env.NEO_SOLC;
  if (fromEnv) {
    return fromEnv;
  }
  return path.resolve(process.cwd(), "..", "..", "target", "debug", "neo-solc");
}

describe("neo-forge build end-to-end", () => {
  it("builds a scaffold project and emits Neo artifacts", async () => {
    const neoSolcPath = getNeoSolcPath();
    if (!(await fs.pathExists(neoSolcPath))) {
      return;
    }

    const projectPath = await makeTempDir("neo-forge-e2e-");
    const forge = new NeoForge(path.join(projectPath, "neo-foundry.toml"));

    await forge.init(projectPath);
    expect(await fs.pathExists(path.join(projectPath, "src", "Counter.sol"))).toBe(true);

    const result = await forge.build({ quiet: true });
    expect(result.artifacts.length).toBeGreaterThan(0);

    const counterArtifact = result.artifacts.find((a) => a.contractName === "Counter");
    expect(counterArtifact).toBeDefined();
    expect(await fs.pathExists(counterArtifact!.nefPath)).toBe(true);
    expect(await fs.pathExists(counterArtifact!.manifestPath)).toBe(true);

    const manifest = await fs.readJson(counterArtifact!.manifestPath);
    expect(manifest.name).toBe("Counter");
    expect(manifest.abi.methods).toBeInstanceOf(Array);
  });

  it("reports compilation errors for invalid Solidity", async () => {
    const neoSolcPath = getNeoSolcPath();
    if (!(await fs.pathExists(neoSolcPath))) {
      return;
    }

    const projectPath = await makeTempDir("neo-forge-e2e-error-");
    const forge = new NeoForge(path.join(projectPath, "neo-foundry.toml"));

    await forge.init(projectPath);
    await fs.writeFile(
      path.join(projectPath, "src", "Counter.sol"),
      `// SPDX-License-Identifier: MIT\npragma solidity ^0.8.19;\n\ncontract Counter { uint256 public number }`
    );

    await expect(forge.build({ quiet: true })).rejects.toThrow(/neo-solc compilation failed/);
  });
});
