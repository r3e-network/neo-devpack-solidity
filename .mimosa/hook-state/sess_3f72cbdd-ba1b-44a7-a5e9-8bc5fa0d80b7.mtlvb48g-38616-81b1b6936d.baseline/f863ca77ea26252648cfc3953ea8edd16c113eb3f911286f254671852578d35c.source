import fs from "fs-extra";
import os from "node:os";
import path from "node:path";
import { afterEach, beforeAll, describe, expect, it } from "vitest";
import { NeoForge } from "../src/forge";
import { CompilerInvoker } from "../src/compiler-invoker";
import { ArtifactCollector } from "../src/artifact-collector";

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
  return path.resolve(process.cwd(), "..", "..", "..", "target", "debug", "neo-solc");
}

describe("NeoForge.build", () => {
  it("discovers sources, compiles, and emits .nef/.manifest.json artifacts", async () => {
    const projectPath = await makeTempDir("neo-forge-build-");
    const srcDir = path.join(projectPath, "src");
    await fs.ensureDir(srcDir);
    await fs.writeFile(
      path.join(srcDir, "Counter.sol"),
      `// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract Counter {
    uint256 public number;

    function setNumber(uint256 newNumber) public {
        number = newNumber;
    }

    function increment() public {
        number++;
    }
}`
    );

    const forge = new NeoForge(path.join(projectPath, "neo-foundry.toml"));
    const result = await forge.build({ quiet: true });

    expect(result.artifacts).toHaveLength(1);
    const artifact = result.artifacts[0];
    expect(artifact.contractName).toBe("Counter");
    expect(await fs.pathExists(artifact.nefPath)).toBe(true);
    expect(await fs.pathExists(artifact.manifestPath)).toBe(true);

    const manifest = await fs.readJson(artifact.manifestPath);
    expect(manifest.name).toBe("Counter");
    expect(manifest.abi.methods).toBeInstanceOf(Array);
  });

  it("rejects Solidity source with compilation errors", async () => {
    const projectPath = await makeTempDir("neo-forge-build-error-");
    const srcDir = path.join(projectPath, "src");
    await fs.ensureDir(srcDir);
    await fs.writeFile(
      path.join(srcDir, "Bad.sol"),
      `// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract Bad {
    uint256 public number
}`
    );

    const forge = new NeoForge(path.join(projectPath, "neo-foundry.toml"));
    await expect(forge.build({ quiet: true })).rejects.toThrow(/neo-solc compilation failed/);
  });

  it("returns cached results when sources are unchanged", async () => {
    const projectPath = await makeTempDir("neo-forge-build-cache-");
    const srcDir = path.join(projectPath, "src");
    await fs.ensureDir(srcDir);
    await fs.writeFile(
      path.join(srcDir, "Counter.sol"),
      `// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract Counter {
    uint256 public number;
}`
    );

    const forge = new NeoForge(path.join(projectPath, "neo-foundry.toml"));
    const first = await forge.build({ quiet: true });
    expect(first.cached).toBe(false);

    const second = await forge.build({ quiet: true });
    expect(second.cached).toBe(true);
    expect(second.artifacts).toHaveLength(1);
  });

  it("bypasses cache with force", async () => {
    const projectPath = await makeTempDir("neo-forge-build-force-");
    const srcDir = path.join(projectPath, "src");
    await fs.ensureDir(srcDir);
    await fs.writeFile(
      path.join(srcDir, "Counter.sol"),
      `// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract Counter {
    uint256 public number;
}`
    );

    const forge = new NeoForge(path.join(projectPath, "neo-foundry.toml"));
    await forge.build({ quiet: true });

    const forced = await forge.build({ quiet: true, force: true });
    expect(forced.cached).toBe(false);
  });
});

describe("CompilerInvoker", () => {
  it("invokes neo-solc and returns compilation output", async () => {
    const invoker = new CompilerInvoker();
    const neoSolcPath = getNeoSolcPath();
    if (!(await fs.pathExists(neoSolcPath))) {
      return;
    }

    const output = await invoker.invoke(
      {
        language: "Solidity",
        sources: {
          "Counter.sol": {
            content: `// SPDX-License-Identifier: MIT\npragma solidity ^0.8.19;\n\ncontract Counter { uint256 public number; }`,
          },
        },
        settings: { optimizer: { enabled: true, runs: 200 } },
      },
      { neoSolcPath, cwd: await makeTempDir("neo-forge-invoker-") }
    );

    expect(output.contracts["Counter.sol"]).toBeDefined();
    expect(output.contracts["Counter.sol"].Counter).toBeDefined();
    expect(output.contracts["Counter.sol"].Counter.neo).toBeDefined();
  });

  it("throws NeoForgeBuildError on compilation errors", async () => {
    const invoker = new CompilerInvoker();
    const neoSolcPath = getNeoSolcPath();
    if (!(await fs.pathExists(neoSolcPath))) {
      return;
    }

    await expect(
      invoker.invoke(
        {
          language: "Solidity",
          sources: {
            "Bad.sol": {
              content: `// SPDX-License-Identifier: MIT\npragma solidity ^0.8.19;\n\ncontract Bad { uint256 public number }`,
            },
          },
          settings: {},
        },
        { neoSolcPath, cwd: await makeTempDir("neo-forge-invoker-error-") }
      )
    ).rejects.toThrow(/neo-solc compilation failed/);
  });
});

describe("ArtifactCollector", () => {
  it("writes .nef and .manifest.json from standard-json output", async () => {
    const outDir = await makeTempDir("neo-forge-collector-");
    const collector = new ArtifactCollector();

    const output = {
      sources: {},
      contracts: {
        "Counter.sol": {
          Counter: {
            abi: [],
            metadata: "{}",
            evm: { bytecode: { object: "0x" } },
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
    } as any;

    const artifacts = await collector.collect(output, outDir);
    expect(artifacts).toHaveLength(1);
    expect(await fs.pathExists(artifacts[0].nefPath)).toBe(true);
    expect(await fs.pathExists(artifacts[0].manifestPath)).toBe(true);
  });
});
