const { expect } = require("chai");
const fs = require("fs");
const path = require("path");
const hre = require("hardhat");

/**
 * Integration tests for Neo N3 Devpack artifacts.
 * These tests validate neo-devpack-solidity compile outputs instead of EVM deployment flows.
 */

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, "utf-8"));
}

function collectJsonFiles(dirPath) {
  const files = [];

  function walk(currentDir) {
    if (!fs.existsSync(currentDir)) {
      return;
    }

    for (const entry of fs.readdirSync(currentDir, { withFileTypes: true })) {
      const fullPath = path.join(currentDir, entry.name);
      if (entry.isDirectory()) {
        walk(fullPath);
      } else if (entry.isFile() && entry.name.endsWith(".json")) {
        files.push(fullPath);
      }
    }
  }

  walk(dirPath);
  return files;
}

function getMethodSet(artifact) {
  const methods = artifact.contract.neo.manifest.abi.methods || [];
  return new Set(methods.map((method) => method.name));
}

describe("Neo N3 Devpack Integration Tests", function () {
  this.timeout(120000);

  const artifactsDir = path.join(__dirname, "..", "artifacts");
  const contractArtifactsDir = path.join(artifactsDir, "contracts");
  const neoBuildInfoDir = path.join(artifactsDir, "neo-build-info");

  const expectedArtifacts = [
    { contractName: "Framework", sourceName: "contracts/Framework.sol" },
    { contractName: "FrameworkBase", sourceName: "contracts/FrameworkBase.sol" },
    { contractName: "NEP17Rescue", sourceName: "contracts/NEP17Rescue.sol" },
    { contractName: "OracleService", sourceName: "contracts/OracleService.sol" },
    { contractName: "NEP17", sourceName: "standards/NEP17.sol" }
  ];

  let latestBuildInfo;
  let artifactByContract;

  before(async function () {
    expect(hre.neoSolc, "Neo Hardhat plugin did not load").to.exist;

    await hre.run("neo-compile", { force: true, quiet: true });

    expect(fs.existsSync(neoBuildInfoDir), "neo-build-info directory not found").to.be.true;
    const buildInfoFiles = fs
      .readdirSync(neoBuildInfoDir)
      .filter((file) => file.endsWith(".json"))
      .sort();

    expect(buildInfoFiles.length, "No neo build-info files found").to.be.greaterThan(0);
    latestBuildInfo = readJson(path.join(neoBuildInfoDir, buildInfoFiles[buildInfoFiles.length - 1]));

    artifactByContract = new Map();
    for (const artifactPath of collectJsonFiles(contractArtifactsDir)) {
      const artifact = readJson(artifactPath);
      if (artifact && artifact.contractName) {
        artifactByContract.set(artifact.contractName, artifact);
      }
    }
  });

  describe("Build Info Integration", function () {
    it("should write Neo build-info with expected schema", function () {
      expect(latestBuildInfo).to.include.all.keys(
        "id",
        "solcVersion",
        "neoSolcVersion",
        "input",
        "output",
        "metadata"
      );
      expect(latestBuildInfo.input.language).to.equal("Solidity");
      expect(latestBuildInfo.solcVersion).to.equal("0.8.19");
      expect(latestBuildInfo.input.settings).to.have.property("outputSelection");
      expect(latestBuildInfo.metadata).to.have.property("duration");
    });

    it("should include contract sources plus recursive imports", function () {
      const inputSources = Object.keys(latestBuildInfo.input.sources || {});

      expect(inputSources).to.include("contracts/Framework.sol");
      expect(inputSources).to.include("contracts/OracleService.sol");
      expect(inputSources).to.include("libraries/Neo.sol");
      expect(inputSources).to.include("libraries/Runtime.sol");
      expect(inputSources).to.include("libraries/Storage.sol");
      expect(inputSources).to.include("standards/NEP17.sol");
    });

    it("should contain warnings only (no compiler errors)", function () {
      const messages = latestBuildInfo.output.errors || [];
      const errors = messages.filter((entry) => entry.severity === "error");
      expect(errors).to.have.lengthOf(0);
    });
  });

  describe("Artifact Generation", function () {
    it("should emit expected core devpack artifacts", function () {
      for (const expectedArtifact of expectedArtifacts) {
        const artifact = artifactByContract.get(expectedArtifact.contractName);
        expect(artifact, `Missing artifact for ${expectedArtifact.contractName}`).to.exist;
        expect(artifact.sourceName).to.equal(expectedArtifact.sourceName);
        expect(artifact.buildInfo).to.equal(latestBuildInfo.id);
      }
    });

    it("should emit valid Neo and EVM payloads for each artifact", function () {
      for (const { contractName } of expectedArtifacts) {
        const artifact = artifactByContract.get(contractName);

        expect(artifact.contract).to.have.property("neo");
        expect(artifact.contract).to.have.property("evm");
        expect(artifact.contract.abi).to.be.an("array").that.is.not.empty;

        const evmBytecode = artifact.contract.evm.bytecode.object;
        const neoScript = artifact.contract.neo.nef.script;

        expect(evmBytecode).to.be.a("string").and.match(/^0x[0-9a-fA-F]+$/);
        expect(neoScript).to.be.a("string").and.match(/^[0-9a-fA-F]+$/);
      }
    });
  });

  describe("Manifest Correctness", function () {
    it("should keep manifest name aligned with contractName", function () {
      for (const { contractName } of expectedArtifacts) {
        const artifact = artifactByContract.get(contractName);
        const manifest = artifact.contract.neo.manifest;

        expect(manifest.name).to.equal(contractName);
        expect(manifest.abi.methods).to.be.an("array").that.is.not.empty;
      }
    });

    it("should constrain permissions and only allow callback wildcards where needed", function () {
      const allowedWildcardMethodsByContract = {
        Framework: new Set(),
        FrameworkBase: new Set(),
        NEP17Rescue: new Set(["onNEP17Payment"]),
        OracleService: new Set(["onOracleResponse"]),
        NEP17: new Set(["onNEP17Payment"])
      };

      for (const { contractName } of expectedArtifacts) {
        const artifact = artifactByContract.get(contractName);
        const permissions = artifact.contract.neo.manifest.permissions || [];
        const allowedWildcardMethods = allowedWildcardMethodsByContract[contractName];

        for (const permission of permissions) {
          expect(permission.methods).to.not.equal("*");

          if (permission.contract === "*") {
            expect(Array.isArray(permission.methods)).to.equal(true);
            expect(permission.methods.length).to.equal(1);
            expect(
              allowedWildcardMethods.has(permission.methods[0]),
              `${contractName} has unexpected wildcard callback permission: ${permission.methods[0]}`
            ).to.equal(true);
            continue;
          }

          if (Array.isArray(permission.methods)) {
            expect(permission.methods, `${contractName} has wildcard methods list`).to.not.include("*");
          } else {
            expect(permission.methods, `${contractName} has wildcard method`).to.not.equal("*");
          }
        }
      }
    });

    it("should expose expected framework, oracle, and token methods", function () {
      const frameworkMethods = getMethodSet(artifactByContract.get("Framework"));
      ["initialized", "owner", "transferOwnership", "getCurrentBlock", "getDiagnostics", "estimateGas"].forEach(
        (method) => {
          expect(frameworkMethods.has(method), `Framework missing method ${method}`).to.equal(true);
        }
      );

      const oracleMethods = getMethodSet(artifactByContract.get("OracleService"));
      ["request", "oracleCallback", "getRequest"].forEach((method) => {
        expect(oracleMethods.has(method), `OracleService missing method ${method}`).to.equal(true);
      });

      const nep17Methods = getMethodSet(artifactByContract.get("NEP17"));
      ["name", "symbol", "decimals", "totalSupply", "balanceOf", "transfer", "transfer(address,uint256)"]
        .forEach((method) => {
          expect(nep17Methods.has(method), `NEP17 missing method ${method}`).to.equal(true);
        });

      const nep17RescueMethods = getMethodSet(artifactByContract.get("NEP17Rescue"));
      ["emergencyTokenRecovery", "transferWithTimelock", "claimTimelock"].forEach((method) => {
        expect(nep17RescueMethods.has(method), `NEP17Rescue missing method ${method}`).to.equal(true);
      });
    });

    it("should advertise NEP-17 standard for token contracts", function () {
      const nep17 = artifactByContract.get("NEP17");
      const nep17Rescue = artifactByContract.get("NEP17Rescue");

      expect(nep17.contract.neo.manifest.supportedstandards).to.include("NEP-17");
      expect(nep17Rescue.contract.neo.manifest.supportedstandards).to.include("NEP-17");
    });
  });
});
