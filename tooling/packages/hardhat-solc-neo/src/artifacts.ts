import { promises as fs, Dirent } from "fs";
import path from "path";
import { 
  BuildArtifact, 
  DeploymentArtifact,
  ArtifactManager as IArtifactManager,
  ArtifactStorageConfig,
  ArtifactValidationResult,
  ArtifactComparison,
  ArtifactDifference,
  ArtifactSearchCriteria,
  ArtifactStatistics
} from "@neo-devpack-solidity/types";
import { HardhatPluginError } from "hardhat/plugins";
import Debug from "debug";

const debug = Debug("hardhat:neo-devpack-solidity:artifacts");

/**
 * Artifact manager implementation for Neo DevPack for Solidity
 */
export class ArtifactManager implements IArtifactManager {
  private config: ArtifactStorageConfig;

  constructor(baseDir: string) {
    this.config = {
      baseDir,
      buildDir: path.join(baseDir, "contracts"),
      deploymentDir: path.join(baseDir, "deployments"),
      buildInfoDir: path.join(baseDir, "neo-build-info"),
      cacheDir: path.join(baseDir, ".cache"),
      compression: {
        enabled: false,
        algorithm: "gzip"
      },
      cleanup: {
        maxAge: 30, // days
        maxSize: 100 // MB
      }
    };
  }

  /**
   * Get build artifact by contract name
   */
  async getBuildArtifact(contractName: string): Promise<BuildArtifact | null> {
    debug(`Getting build artifact for ${contractName}`);
    
    try {
      const artifactPath = await this.resolveBuildArtifactPath(contractName);
      if (!artifactPath) {
        return null;
      }
      const content = await fs.readFile(artifactPath, "utf-8");
      return JSON.parse(content);
    } catch (error) {
      if ((error as any).code === "ENOENT") {
        return null;
      }
      throw new HardhatPluginError(
        "@neo-devpack-solidity/hardhat-solc-neo",
        `Failed to read build artifact: ${error}`
      );
    }
  }

  /**
   * Save build artifact
   */
  async saveBuildArtifact(artifact: BuildArtifact): Promise<void> {
    debug(`Saving build artifact for ${artifact.contractName}`);
    
    try {
      const sourceDir = this.getSourceArtifactDir(artifact.sourceName, artifact.contractName);
      await fs.mkdir(sourceDir, { recursive: true });
      
      const artifactPath = path.join(sourceDir, `${artifact.contractName}.json`);
      await fs.writeFile(artifactPath, JSON.stringify(artifact, null, 2));
      
      debug(`Build artifact saved to ${artifactPath}`);
    } catch (error) {
      throw new HardhatPluginError(
        "@neo-devpack-solidity/hardhat-solc-neo",
        `Failed to save build artifact: ${error}`
      );
    }
  }

  /**
   * Get all build artifacts
   */
  async getAllBuildArtifacts(): Promise<BuildArtifact[]> {
    debug("Getting all build artifacts");
    
    try {
      const files = await this.getArtifactFiles(this.config.buildDir);
      const artifacts: BuildArtifact[] = [];
      
      for (const relativePath of files) {
        if (!relativePath.endsWith(".json")) {
          continue;
        }

        const fullPath = path.join(this.config.buildDir, relativePath);
        const content = await fs.readFile(fullPath, "utf-8");
        artifacts.push(JSON.parse(content));
      }
      
      return artifacts;
    } catch (error) {
      throw new HardhatPluginError(
        "@neo-devpack-solidity/hardhat-solc-neo",
        `Failed to get build artifacts: ${error}`
      );
    }
  }

  /**
   * Get deployment artifact
   */
  async getDeploymentArtifact(contractName: string, networkName: string): Promise<DeploymentArtifact | null> {
    debug(`Getting deployment artifact for ${contractName} on ${networkName}`);
    
    try {
      const deploymentPath = path.join(this.config.deploymentDir, networkName, `${contractName}.json`);
      const content = await fs.readFile(deploymentPath, "utf-8");
      return JSON.parse(content);
    } catch (error) {
      if ((error as any).code === "ENOENT") {
        return null;
      }
      throw new HardhatPluginError(
        "@neo-devpack-solidity/hardhat-solc-neo",
        `Failed to read deployment artifact: ${error}`
      );
    }
  }

  /**
   * Save deployment artifact
   */
  async saveDeploymentArtifact(artifact: DeploymentArtifact): Promise<void> {
    debug(`Saving deployment artifact for ${artifact.contractName} on ${artifact.networkName}`);
    
    try {
      const networkDir = path.join(this.config.deploymentDir, artifact.networkName);
      await fs.mkdir(networkDir, { recursive: true });
      
      const deploymentPath = path.join(networkDir, `${artifact.contractName}.json`);
      await fs.writeFile(deploymentPath, JSON.stringify(artifact, null, 2));
      
      debug(`Deployment artifact saved to ${deploymentPath}`);
    } catch (error) {
      throw new HardhatPluginError(
        "@neo-devpack-solidity/hardhat-solc-neo",
        `Failed to save deployment artifact: ${error}`
      );
    }
  }

  /**
   * Get all deployments for a network
   */
  async getNetworkDeployments(networkName: string): Promise<DeploymentArtifact[]> {
    debug(`Getting deployments for network ${networkName}`);
    
    try {
      const networkDir = path.join(this.config.deploymentDir, networkName);
      const files = await this.getArtifactFiles(networkDir);
      const deployments: DeploymentArtifact[] = [];
      
      for (const file of files) {
        if (file.endsWith(".json")) {
          const content = await fs.readFile(path.join(networkDir, file), "utf-8");
          deployments.push(JSON.parse(content));
        }
      }
      
      return deployments;
    } catch (error) {
      if ((error as any).code === "ENOENT") {
        return [];
      }
      throw new HardhatPluginError(
        "@neo-devpack-solidity/hardhat-solc-neo",
        `Failed to get network deployments: ${error}`
      );
    }
  }

  /**
   * Clear all artifacts
   */
  async clearArtifacts(): Promise<void> {
    debug("Clearing all artifacts");
    
    try {
      await this.removeDirectory(this.config.buildDir);
      await this.removeDirectory(this.config.deploymentDir);
      await this.removeDirectory(this.config.buildInfoDir);
      await this.removeDirectory(this.config.cacheDir);
    } catch (error) {
      throw new HardhatPluginError(
        "@neo-devpack-solidity/hardhat-solc-neo",
        `Failed to clear artifacts: ${error}`
      );
    }
  }

  /**
   * Export artifacts to directory
   */
  async exportArtifacts(outputDir: string): Promise<void> {
    debug(`Exporting artifacts to ${outputDir}`);
    
    try {
      await fs.mkdir(outputDir, { recursive: true });
      
      // Export build artifacts
      const buildArtifacts = await this.getAllBuildArtifacts();
      for (const artifact of buildArtifacts) {
        const exportPath = path.join(outputDir, "contracts", `${artifact.contractName}.json`);
        await fs.mkdir(path.dirname(exportPath), { recursive: true });
        await fs.writeFile(exportPath, JSON.stringify(artifact, null, 2));
      }
      
      // Export deployment artifacts
      const networkDirs = await this.getDirectories(this.config.deploymentDir);
      for (const networkName of networkDirs) {
        const deployments = await this.getNetworkDeployments(networkName);
        for (const deployment of deployments) {
          const exportPath = path.join(outputDir, "deployments", networkName, `${deployment.contractName}.json`);
          await fs.mkdir(path.dirname(exportPath), { recursive: true });
          await fs.writeFile(exportPath, JSON.stringify(deployment, null, 2));
        }
      }
      
      debug("Artifacts exported successfully");
    } catch (error) {
      throw new HardhatPluginError(
        "@neo-devpack-solidity/hardhat-solc-neo",
        `Failed to export artifacts: ${error}`
      );
    }
  }

  /**
   * Import artifacts from directory
   */
  async importArtifacts(inputDir: string): Promise<void> {
    debug(`Importing artifacts from ${inputDir}`);
    
    try {
      // Import build artifacts
      const contractsDir = path.join(inputDir, "contracts");
      if (await this.directoryExists(contractsDir)) {
        const files = await this.getArtifactFiles(contractsDir);
        for (const file of files) {
          if (file.endsWith(".json")) {
            const content = await fs.readFile(path.join(contractsDir, file), "utf-8");
            const artifact: BuildArtifact = JSON.parse(content);
            await this.saveBuildArtifact(artifact);
          }
        }
      }
      
      // Import deployment artifacts
      const deploymentsDir = path.join(inputDir, "deployments");
      if (await this.directoryExists(deploymentsDir)) {
        const networks = await this.getDirectories(deploymentsDir);
        for (const networkName of networks) {
          const networkDir = path.join(deploymentsDir, networkName);
          const files = await this.getArtifactFiles(networkDir);
          for (const file of files) {
            if (file.endsWith(".json")) {
              const content = await fs.readFile(path.join(networkDir, file), "utf-8");
              const artifact: DeploymentArtifact = JSON.parse(content);
              await this.saveDeploymentArtifact(artifact);
            }
          }
        }
      }
      
      debug("Artifacts imported successfully");
    } catch (error) {
      throw new HardhatPluginError(
        "@neo-devpack-solidity/hardhat-solc-neo",
        `Failed to import artifacts: ${error}`
      );
    }
  }

  /**
   * Validate artifact integrity
   */
  async validateArtifact(artifact: BuildArtifact): Promise<ArtifactValidationResult> {
    const result: ArtifactValidationResult = {
      valid: true,
      errors: [],
      warnings: [],
      integrity: {
        checksumValid: true,
        signatureValid: true,
        timestampValid: true
      }
    };

    // Validate required fields
    if (!artifact.contractName) {
      result.errors.push("Missing contract name");
      result.valid = false;
    }

    if (!artifact.sourceName) {
      result.errors.push("Missing source name");
      result.valid = false;
    }

    if (!artifact.contract) {
      result.errors.push("Missing contract data");
      result.valid = false;
    } else {
      // Validate Neo-specific fields
      if (!artifact.contract.neo) {
        result.errors.push("Missing Neo-specific compilation output");
        result.valid = false;
      } else {
        if (!artifact.contract.neo.nef) {
          result.errors.push("Missing NEF data");
          result.valid = false;
        }
        
        if (!artifact.contract.neo.manifest) {
          result.errors.push("Missing manifest data");
          result.valid = false;
        }
      }
    }

    return result;
  }

  /**
   * Compare two artifacts
   */
  async compareArtifacts(artifact1: BuildArtifact, artifact2: BuildArtifact): Promise<ArtifactComparison> {
    const differences: ArtifactDifference[] = [];
    const breakingChanges: string[] = [];
    const nonBreakingChanges: string[] = [];

    const recordModified = (
      path: string,
      oldValue: any,
      newValue: any,
      impact: ArtifactDifference["impact"],
      message: string
    ) => {
      differences.push({
        path,
        type: "modified",
        oldValue,
        newValue,
        impact
      });

      if (impact === "breaking" || impact === "high") {
        breakingChanges.push(message);
      } else {
        nonBreakingChanges.push(message);
      }
    };

    // Compare basic properties
    if (artifact1.contractName !== artifact2.contractName) {
      recordModified(
        "contractName",
        artifact1.contractName,
        artifact2.contractName,
        "breaking",
        "Contract name differs"
      );
    }

    if (artifact1.sourceName !== artifact2.sourceName) {
      recordModified(
        "sourceName",
        artifact1.sourceName,
        artifact2.sourceName,
        "medium",
        "Source name differs"
      );
    }

    // Compare bytecode
    const bytecode1 = artifact1.contract?.evm?.bytecode?.object ?? "";
    const bytecode2 = artifact2.contract?.evm?.bytecode?.object ?? "";
    if (bytecode1 !== bytecode2) {
      recordModified(
        "contract.evm.bytecode.object",
        bytecode1,
        bytecode2,
        "breaking",
        "Bytecode change detected"
      );
    }

    // Compare ABI
    const abiComparison = this.compareABI(artifact1.contract.abi, artifact2.contract.abi);
    if (abiComparison.differences.length > 0) {
      differences.push({
        path: "contract.abi",
        type: "modified",
        oldValue: artifact1.contract.abi,
        newValue: artifact2.contract.abi,
        impact: abiComparison.breakingChanges.length > 0 ? "breaking" : "medium"
      });
      breakingChanges.push(...abiComparison.breakingChanges);
      nonBreakingChanges.push(...abiComparison.nonBreakingChanges);
    }

    // Compare Neo-specific data
    if (artifact1.contract.neo && artifact2.contract.neo) {
      const nef1 = artifact1.contract.neo.nef?.image ?? artifact1.contract.neo.nef?.script ?? "";
      const nef2 = artifact2.contract.neo.nef?.image ?? artifact2.contract.neo.nef?.script ?? "";
      if (nef1 !== nef2) {
        recordModified("contract.neo.nef", nef1, nef2, "breaking", "NEF change detected");
      }

      if (JSON.stringify(artifact1.contract.neo.manifest) !== JSON.stringify(artifact2.contract.neo.manifest)) {
        differences.push({
          path: "contract.neo.manifest",
          type: "modified",
          oldValue: artifact1.contract.neo.manifest,
          newValue: artifact2.contract.neo.manifest,
          impact: "high"
        });
        const manifestComparison = this.compareManifests(
          artifact1.contract.neo.manifest,
          artifact2.contract.neo.manifest
        );
        breakingChanges.push(...manifestComparison.breaking);
        nonBreakingChanges.push(...manifestComparison.nonBreaking);
      }

      if (JSON.stringify(artifact1.contract.neo.methodMap ?? {}) !== JSON.stringify(artifact2.contract.neo.methodMap ?? {})) {
        recordModified(
          "contract.neo.methodMap",
          artifact1.contract.neo.methodMap ?? {},
          artifact2.contract.neo.methodMap ?? {},
          "breaking",
          "Neo method map changed"
        );
      }
    }

    const identical = differences.length === 0;
    const upgradeable = breakingChanges.length === 0;

    return {
      identical,
      differences,
      compatibility: {
        upgradeable,
        breakingChanges,
        nonBreakingChanges
      }
    };
  }

  /**
   * Search artifacts by criteria
   */
  async searchArtifacts(criteria: ArtifactSearchCriteria): Promise<BuildArtifact[]> {
    const allArtifacts = await this.getAllBuildArtifacts();
    
    return allArtifacts.filter(artifact => {
      if (criteria.contractName) {
        if (typeof criteria.contractName === "string") {
          if (!artifact.contractName.includes(criteria.contractName)) {
            return false;
          }
        } else {
          if (!criteria.contractName.test(artifact.contractName)) {
            return false;
          }
        }
      }
      
      // Add more filtering logic as needed
      return true;
    });
  }

  /**
   * Get artifact statistics
   */
  async getStatistics(): Promise<ArtifactStatistics> {
    const buildArtifacts = await this.getAllBuildArtifacts();
    const allNetworks = await this.getDirectories(this.config.deploymentDir);
    
    let totalDeployments = 0;
    const networkDistribution: { [key: string]: number } = {};
    
    for (const network of allNetworks) {
      const deployments = await this.getNetworkDeployments(network);
      totalDeployments += deployments.length;
      networkDistribution[network] = deployments.length;
    }

    return {
      totalBuildArtifacts: buildArtifacts.length,
      totalDeploymentArtifacts: totalDeployments,
      storageUsage: {
        total: 0, // Would calculate actual storage usage
        buildArtifacts: 0,
        deploymentArtifacts: 0,
        buildInfo: 0,
        cache: 0
      },
      compilerVersions: {},
      networkDistribution,
      recentActivity: {
        buildsLastWeek: 0,
        deploymentsLastWeek: 0,
        verificationsLastWeek: 0
      }
    };
  }

  // Helper methods

  private async getArtifactFiles(dir: string): Promise<string[]> {
    return this.collectArtifactFiles(dir, "");
  }

  private async getDirectories(dir: string): Promise<string[]> {
    try {
      const entries = await fs.readdir(dir, { withFileTypes: true });
      return entries
        .filter(entry => entry.isDirectory())
        .map(entry => entry.name);
    } catch (error) {
      if ((error as any).code === "ENOENT") {
        return [];
      }
      throw error;
    }
  }

  private async directoryExists(dir: string): Promise<boolean> {
    try {
      const stat = await fs.stat(dir);
      return stat.isDirectory();
    } catch {
      return false;
    }
  }

  private async fileExists(filePath: string): Promise<boolean> {
    try {
      const stat = await fs.stat(filePath);
      return stat.isFile();
    } catch {
      return false;
    }
  }

  private async removeDirectory(dir: string): Promise<void> {
    try {
      await fs.rm(dir, { recursive: true, force: true });
    } catch (error) {
      if ((error as any).code !== "ENOENT") {
        throw error;
      }
    }
  }

  private async resolveBuildArtifactPath(contractIdentifier: string): Promise<string | null> {
    let sourceQualifier: string | null = null;
    let simpleName = contractIdentifier;

    const colonIndex = contractIdentifier.indexOf(":");
    if (colonIndex !== -1) {
      sourceQualifier = this.normalizeSourceIdentifier(contractIdentifier.slice(0, colonIndex));
      simpleName = contractIdentifier.slice(colonIndex + 1);
    }

    if (sourceQualifier) {
      const sourceDir = this.getSourceArtifactDir(sourceQualifier, simpleName);
      const fqPath = path.join(sourceDir, `${simpleName}.json`);
      if (await this.fileExists(fqPath)) {
        return fqPath;
      }
      return null;
    }

    const flatPath = path.join(this.config.buildDir, `${simpleName}.json`);
    if (await this.fileExists(flatPath)) {
      return flatPath;
    }

    const files = await this.getArtifactFiles(this.config.buildDir);
    const matches = files.filter(file => path.basename(file) === `${simpleName}.json`);

    if (matches.length === 0) {
      return null;
    }

    if (matches.length > 1) {
      const matchList = matches.map(match => ` • ${match}`).join("\n");
      throw new HardhatPluginError(
        "@neo-devpack-solidity/hardhat-solc-neo",
        `Multiple artifacts match '${simpleName}'. Use fully qualified name (source:contract).\n${matchList}`
      );
    }

    return path.join(this.config.buildDir, matches[0]);
  }

  private getSourceArtifactDir(sourceName: string | undefined, fallback: string): string {
    const normalized = sourceName ? this.normalizeSourceIdentifier(sourceName) : fallback;
    const segments = this.normalizeSourceSegments(normalized);
    if (segments.length === 0) {
      return this.config.buildDir;
    }
    return path.join(this.config.buildDir, ...segments);
  }

  private normalizeSourceIdentifier(sourceName: string): string {
    let normalized = sourceName.replace(/\\/g, "/");
    if (normalized.startsWith("./")) {
      normalized = normalized.slice(2);
    }
    if (normalized.toLowerCase().startsWith("contracts/")) {
      normalized = normalized.slice("contracts/".length);
    }
    return normalized;
  }

  private normalizeSourceSegments(sourceName: string): string[] {
    return sourceName
      .replace(/\\/g, "/")
      .split("/")
      .map(segment => segment.trim())
      .filter(segment => segment.length > 0 && segment !== "." && segment !== "..");
  }

  private async collectArtifactFiles(root: string, relative: string): Promise<string[]> {
    const fullPath = relative ? path.join(root, relative) : root;
    let entries: Dirent[];

    try {
      entries = await fs.readdir(fullPath, { withFileTypes: true });
    } catch (error) {
      if ((error as any).code === "ENOENT") {
        return [];
      }
      throw error;
    }

    const files: string[] = [];
    for (const entry of entries) {
      const entryRelative = relative ? path.join(relative, entry.name) : entry.name;
      if (entry.isDirectory()) {
        const nested = await this.collectArtifactFiles(root, entryRelative);
        files.push(...nested);
      } else {
        files.push(entryRelative);
      }
    }

    return files;
  }

  /**
   * Compare ABI interfaces
   */
  private compareABI(abi1: any[], abi2: any[]): {
    differences: string[];
    breakingChanges: string[];
    nonBreakingChanges: string[];
  } {
    const differences: string[] = [];
    const breakingChanges: string[] = [];
    const nonBreakingChanges: string[] = [];

    // Create maps for easier comparison
    const functions1 = new Map(abi1.filter(item => item.type === 'function').map(item => [item.name, item]));
    const functions2 = new Map(abi2.filter(item => item.type === 'function').map(item => [item.name, item]));
    
    const events1 = new Map(abi1.filter(item => item.type === 'event').map(item => [item.name, item]));
    const events2 = new Map(abi2.filter(item => item.type === 'event').map(item => [item.name, item]));

    // Check for removed functions (breaking)
    for (const [name, func] of functions1) {
      if (!functions2.has(name)) {
        differences.push(`Function '${name}' removed`);
        breakingChanges.push(`Function '${name}' removed`);
      } else {
        // Compare function signatures
        const func2 = functions2.get(name)!;
        const sigDiff = this.compareFunctionSignatures(func, func2);
        if (sigDiff.length > 0) {
          differences.push(`Function '${name}' signature changed: ${sigDiff.join(', ')}`);
          breakingChanges.push(`Function '${name}' signature changed`);
        }
      }
    }

    // Check for added functions (non-breaking)
    for (const [name] of functions2) {
      if (!functions1.has(name)) {
        differences.push(`Function '${name}' added`);
        nonBreakingChanges.push(`Function '${name}' added`);
      }
    }

    // Check for removed events (potentially breaking)
    for (const [name] of events1) {
      if (!events2.has(name)) {
        differences.push(`Event '${name}' removed`);
        breakingChanges.push(`Event '${name}' removed`);
      }
    }

    // Check for added events (non-breaking)
    for (const [name] of events2) {
      if (!events1.has(name)) {
        differences.push(`Event '${name}' added`);
        nonBreakingChanges.push(`Event '${name}' added`);
      }
    }

    return { differences, breakingChanges, nonBreakingChanges };
  }

  /**
   * Compare function signatures
   */
  private compareFunctionSignatures(func1: any, func2: any): string[] {
    const differences: string[] = [];

    // Compare inputs
    if (JSON.stringify(func1.inputs) !== JSON.stringify(func2.inputs)) {
      differences.push('inputs changed');
    }

    // Compare outputs
    if (JSON.stringify(func1.outputs) !== JSON.stringify(func2.outputs)) {
      differences.push('outputs changed');
    }

    // Compare state mutability
    if (func1.stateMutability !== func2.stateMutability) {
      differences.push(`state mutability changed from '${func1.stateMutability}' to '${func2.stateMutability}'`);
    }

    return differences;
  }

  /**
   * Compare Neo manifests
   */
  private compareManifests(manifest1: any, manifest2: any): {
    breaking: string[];
    nonBreaking: string[];
  } {
    const breaking: string[] = [];
    const nonBreaking: string[] = [];

    // Compare basic properties
    if (manifest1.name !== manifest2.name) {
      breaking.push('Contract name changed');
    }

    if (manifest1.abi?.hash !== manifest2.abi?.hash) {
      breaking.push('ABI hash changed');
    }

    // Compare permissions
    if (JSON.stringify(manifest1.permissions) !== JSON.stringify(manifest2.permissions)) {
      const oldPerms = manifest1.permissions?.length || 0;
      const newPerms = manifest2.permissions?.length || 0;
      
      if (newPerms > oldPerms) {
        nonBreaking.push('Permissions added');
      } else if (newPerms < oldPerms) {
        breaking.push('Permissions removed');
      } else {
        breaking.push('Permissions modified');
      }
    }

    // Compare trusts
    if (JSON.stringify(manifest1.trusts) !== JSON.stringify(manifest2.trusts)) {
      nonBreaking.push('Trusts modified');
    }

    // Compare groups
    if (JSON.stringify(manifest1.groups) !== JSON.stringify(manifest2.groups)) {
      nonBreaking.push('Groups modified');
    }

    return { breaking, nonBreaking };
  }
}
