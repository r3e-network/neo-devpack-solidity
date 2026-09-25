import fs from "fs-extra";
import os from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { NeoABICompatibilityLayer } from "@neo-devpack-solidity/abi-router";
import { NeoDevpackSolidityCLI } from "@neo-devpack-solidity/cli-tools";
import { NeoForge } from "@neo-devpack-solidity/neo-foundry";
import { ProjectScaffolder } from "@neo-devpack-solidity/templates";
import { isSupportedSolidityVersion } from "@neo-devpack-solidity/types";

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

describe("Integration Test Scenarios", () => {
  it("scaffolds a basic hardhat project with Neo-native tasks", async () => {
    const root = await makeTempDir("neo-devpack-solidity-basic-");
    const projectPath = path.join(root, "basic-project");
    const scaffolder = new ProjectScaffolder();

    const result = await scaffolder.scaffold({
      template: "basic",
      name: "BasicProject",
      directory: projectPath,
      context: {
        contractName: "BasicProject",
        author: "Neo DevPack for Solidity Team",
        description: "Basic Neo DevPack for Solidity project",
        license: "MIT",
        solcVersion: "0.8.19",
      },
      dryRun: false,
      interactive: false,
      gitInit: false,
      install: false,
    });

    expect(result.success).toBe(true);
    expect(await fs.pathExists(path.join(projectPath, "package.json"))).toBe(true);
    expect(await fs.pathExists(path.join(projectPath, "contracts", "BasicProject.sol"))).toBe(true);

    const packageJson = await fs.readJson(path.join(projectPath, "package.json"));
    expect(packageJson.devDependencies).toMatchObject({
      hardhat: "^2.28.6",
      "@neo-devpack-solidity/hardhat-solc-neo": "^0.14.0",
      "@neo-devpack-solidity/hardhat-neo-deployer": "^0.14.0",
      chai: "^4.5.0",
    });
    expect(packageJson.scripts).toMatchObject({
      compile: "npx hardhat neo-compile",
      test: "npx hardhat test --no-compile",
      deploy: "npx hardhat neo-deploy",
      verify: "npx hardhat neo-verify",
    });

    const hardhatConfig = await fs.readFile(path.join(projectPath, "hardhat.config.js"), "utf8");
    const contractSource = await fs.readFile(path.join(projectPath, "contracts", "BasicProject.sol"), "utf8");
    expect(hardhatConfig).toContain('@neo-devpack-solidity/hardhat-solc-neo');
    expect(hardhatConfig).toContain('@neo-devpack-solidity/hardhat-neo-deployer');
    expect(hardhatConfig).not.toContain('@neo-devpack-solidity/hardhat-plugin');
    expect(hardhatConfig).toContain('version: "0.8.19"');
    expect(contractSource).toContain("pragma solidity ^0.8.19;");
  });

  it("scaffolds the ERC20 template without stale OpenZeppelin imports", async () => {
    const root = await makeTempDir("neo-devpack-solidity-erc20-");
    const projectPath = path.join(root, "erc20-project");
    const scaffolder = new ProjectScaffolder();

    const result = await scaffolder.scaffold({
      template: "erc20",
      name: "MyToken",
      directory: projectPath,
      context: {
        contractName: "MyToken",
        tokenName: "MyToken",
        tokenSymbol: "MTK",
        totalSupply: "1000000",
        author: "Neo DevPack for Solidity Team",
        description: "ERC20-style starter",
        license: "MIT",
        solcVersion: "0.8.19",
        ownable: true,
        mintable: true,
        burnable: false,
        pausable: false,
        accessControl: false,
      },
      dryRun: false,
      interactive: false,
      gitInit: false,
      install: false,
    });

    expect(result.success).toBe(true);

    const contract = await fs.readFile(path.join(projectPath, "contracts", "MyToken.sol"), "utf8");
    const hardhatConfig = await fs.readFile(path.join(projectPath, "hardhat.config.js"), "utf8");
    expect(contract).toContain("pragma solidity ^0.8.19;");
    expect(hardhatConfig).toContain('version: "0.8.19"');
    expect(contract).toContain('import "@openzeppelin/contracts/token/ERC20/ERC20.sol";');
    expect(contract).toContain("contract MyToken is ERC20, Ownable");
    expect(contract).toContain("function mint(address to, uint256 amount) public onlyOwner");
    expect(contract).not.toContain("ERC20Mintable");
  });

  it("executes registered CLI commands and returns structured failures", async () => {
    const root = await makeTempDir("neo-devpack-solidity-cli-");

    const successCli = new NeoDevpackSolidityCLI({
      defaults: {},
      profiles: {},
      plugins: [],
      aliases: {},
    });

    successCli.register({
      name: "init",
      description: "Initialize a project",
      options: [
        { name: "template", description: "template", type: "string", required: true },
        { name: "name", description: "name", type: "string", required: true },
      ],
      action: async (args) => {
        await fs.ensureDir(path.join(root, String(args.name)));
      },
    });

    const success = await successCli.execute(["node", "cli", "init", "erc20", "CLITestToken"]);
    expect(success.success).toBe(true);
    expect(await fs.pathExists(path.join(root, "CLITestToken"))).toBe(true);

    const failingCli = new NeoDevpackSolidityCLI({
      defaults: {},
      profiles: {},
      plugins: [],
      aliases: {},
    });

    failingCli.register({
      name: "failing-command",
      description: "Fails intentionally",
      options: [],
      action: async () => {
        throw new Error("Intentional failure");
      },
    });

    const failure = await failingCli.execute(["node", "cli", "failing-command"]);
    expect(failure.success).toBe(false);
    expect(failure.error?.message).toContain("Intentional failure");
  });

  it("initializes a neo-foundry project and reports scaffold status explicitly", async () => {
    const root = await makeTempDir("neo-devpack-solidity-forge-");
    const projectPath = path.join(root, "forge-project");
    const forge = new NeoForge(path.join(projectPath, "neo-foundry.toml"));

    await forge.init(projectPath);

    expect(await fs.pathExists(path.join(projectPath, "neo-foundry.toml"))).toBe(true);
    expect(await fs.pathExists(path.join(projectPath, "src", "Counter.sol"))).toBe(true);
    expect(await fs.pathExists(path.join(projectPath, "test", "Counter.t.sol"))).toBe(true);

    const counterContract = await fs.readFile(path.join(projectPath, "src", "Counter.sol"), "utf8");
    const counterTest = await fs.readFile(path.join(projectPath, "test", "Counter.t.sol"), "utf8");
    expect(counterContract).toContain("pragma solidity ^0.8.19;");
    expect(counterTest).toContain("pragma solidity ^0.8.19;");

    const buildResult = await forge.build({ quiet: true });
    expect(buildResult.artifacts.length).toBeGreaterThan(0);
    const counterArtifact = buildResult.artifacts.find((a) => a.contractName === "Counter");
    expect(counterArtifact).toBeDefined();
    expect(await fs.pathExists(counterArtifact!.nefPath)).toBe(true);
    expect(await fs.pathExists(counterArtifact!.manifestPath)).toBe(true);
  });

  it("enforces the supported Solidity version range in neo-foundry configuration", async () => {
    const root = await makeTempDir("neo-devpack-solidity-version-");
    const projectPath = path.join(root, "version-project");
    const forge = new NeoForge(path.join(projectPath, "neo-foundry.toml"));

    await forge.init(projectPath);
    const config = await (forge as any).config.loadConfig();
    expect(isSupportedSolidityVersion(config.profile.default.neoSolc.version)).toBe(true);

    const invalidForge = new NeoForge(path.join(projectPath, "neo-foundry-invalid.toml"));
    expect(() =>
      (invalidForge as any).config.setProfile("default", {
        neoSolc: { version: "0.8.34" },
      })
    ).toThrow(/Supported range is/);
  });

  it("keeps ABI compatibility exports aligned across package boundaries", () => {
    const layer = new NeoABICompatibilityLayer();
    layer.registerABI([
      {
        type: "function",
        name: "balanceOf",
        inputs: [{ name: "owner", type: "address" }],
        outputs: [{ name: "balance", type: "uint256" }],
        stateMutability: "view",
      },
    ]);

    const web3ABI = layer.toWeb3ABI();
    const fn = web3ABI.find((entry) => entry.type === "function" && entry.name === "balanceOf");

    expect(fn?.constant).toBe(true);
    expect(fn?.stateMutability).toBe("view");

    const arrayValue = layer.convertToNeoVM("uint256[]", ["100", "200"]);
    expect(arrayValue.type).toBe("Array");
    expect(arrayValue.value).toHaveLength(2);
  });
});
