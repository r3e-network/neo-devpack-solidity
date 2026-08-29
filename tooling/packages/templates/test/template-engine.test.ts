import fs from "fs-extra";
import os from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { isSupportedSolidityVersion } from "@neo-devpack-solidity/types";
import { ProjectScaffolder, TemplateEngine, TemplateGenerator } from "../src/template-engine";

describe("@neo-devpack-solidity/templates", () => {
  it("renders each-loop @index placeholders", () => {
    const engine = new TemplateEngine();

    const output = engine.render("{{#each items}}{{@index}}={{this}};{{/each}}", {
      projectName: "demo",
      author: "tester",
      description: "demo",
      license: "MIT",
      version: "1.0.0",
      packageManager: "npm",
      gitInit: false,
      installDependencies: false,
      items: ["alpha", "beta"],
    });

    expect(output).toBe("0=alpha;1=beta;");
  });

  it("generates basic hardhat templates with Neo-native package and task names", () => {
    const generator = new TemplateGenerator();
    const template = generator.generateBasic({
      name: "basic",
      author: "Neo DevPack for Solidity Team",
      description: "Basic Neo DevPack for Solidity project",
      license: "MIT",
      solcVersion: "0.8.19",
      includeTests: true,
      includeDocs: false,
      framework: "hardhat",
    });

    expect(template.dependencies).toEqual({});
    expect(template.devDependencies).toMatchObject({
      hardhat: "^2.28.6",
      "@neo-devpack-solidity/hardhat-solc-neo": "^0.14.0",
      "@neo-devpack-solidity/hardhat-neo-deployer": "^0.14.0",
      chai: "^4.5.0",
    });
    expect(template.scripts).toMatchObject({
      compile: "npx hardhat neo-compile",
      test: "npx hardhat test --no-compile",
      deploy: "npx hardhat neo-deploy",
      verify: "npx hardhat neo-verify",
    });

    const hardhatConfig = String(template.files.find((file) => file.path === "hardhat.config.js")?.content);
    const contractTemplate = String(template.files.find((file) => file.path === "contracts/{{contractName}}.sol")?.content);
    expect(hardhatConfig).toContain('@neo-devpack-solidity/hardhat-solc-neo');
    expect(hardhatConfig).toContain('@neo-devpack-solidity/hardhat-neo-deployer');
    expect(hardhatConfig).not.toContain('@neo-devpack-solidity/hardhat-plugin');
    expect(contractTemplate).toContain("pragma solidity ^0.8.19;");

    const testTemplate = String(template.files.find((file) => file.path === "test/{{contractName}}.test.js")?.content);
    expect(testTemplate).toContain("hre.neoSolc.artifacts.getBuildArtifact");
    expect(testTemplate).not.toContain("ethers.getContractFactory");
  });

  it("scaffolds a basic project and writes package.json", async () => {
    const tempDir = await fs.mkdtemp(path.join(os.tmpdir(), "neo-template-test-"));
    const projectPath = path.join(tempDir, "basic-project");
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
    expect(await fs.readFile(path.join(projectPath, "contracts", "BasicProject.sol"), "utf8")).toContain(
      "pragma solidity ^0.8.19;",
    );
  });

  it("defaults to a supported Solidity version and rejects out-of-range versions", () => {
    const generator = new TemplateGenerator();

    const defaultTemplate = generator.generateBasic({
      name: "basic",
      author: "Neo DevPack for Solidity Team",
      description: "Basic Neo DevPack for Solidity project",
      license: "MIT",
      includeTests: false,
      includeDocs: false,
      framework: "hardhat",
    });

    const contractTemplate = String(
      defaultTemplate.files.find((file) => file.path === "contracts/{{contractName}}.sol")?.content
    );
    const pragmaMatch = contractTemplate.match(/pragma solidity \^(\d+\.\d+\.\d+);/);
    expect(pragmaMatch).toBeTruthy();
    expect(isSupportedSolidityVersion(pragmaMatch![1])).toBe(true);

    expect(() =>
      generator.generateBasic({
        name: "basic",
        author: "Neo DevPack for Solidity Team",
        description: "Basic Neo DevPack for Solidity project",
        license: "MIT",
        solcVersion: "0.8.34",
        includeTests: false,
        includeDocs: false,
        framework: "hardhat",
      })
    ).toThrow(/Supported range is/);
  });
});
