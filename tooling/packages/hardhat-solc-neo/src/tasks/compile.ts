import { task } from "hardhat/config";
import { HardhatRuntimeEnvironment } from "hardhat/types";
import { promises as fs } from "fs";
import path from "path";
import chalk from "chalk";
import { BuildArtifact, BuildInfo, addFlagOption, setTaskAction } from "@neo-devpack-solidity/types";

const neoCompileTask = task("neo-compile", "Compile contracts using Neo DevPack for Solidity compiler");
addFlagOption(neoCompileTask, "force", "Force recompilation");
addFlagOption(neoCompileTask, "quiet", "Suppress output");
setTaskAction(neoCompileTask, async (taskArgs: any, hre: HardhatRuntimeEnvironment) => {
    const { force, quiet } = taskArgs;
    const projectRoot = hre.config.paths.root || process.cwd();
    
    if (!quiet) {
      console.log(chalk.blue("🔧 Compiling contracts with Neo DevPack for Solidity..."));
    }

    try {
      // Get source files
      const sourceFiles = await getSourceFiles(hre.config.paths.sources);
      
      if (sourceFiles.length === 0) {
        if (!quiet) {
          console.log(chalk.yellow("No Solidity files found"));
        }
        return;
      }

      // Check if compilation is needed
      if (!force && !quiet) {
        const needsCompilation = await checkCompilationNeeded(sourceFiles, hre);
        if (!needsCompilation) {
          console.log(chalk.green("✅ Contracts are up to date"));
          return;
        }
      }

      // Read source content
      const sources: { [fileName: string]: { content: string } } = {};
      for (const filePath of sourceFiles) {
        const content = await fs.readFile(filePath, "utf-8");
        // Use project-root-relative source names so cross-directory imports
        // (e.g. contracts -> ../libraries) resolve consistently.
        const relativePath = path.relative(projectRoot, filePath).split(path.sep).join("/");
        sources[relativePath] = { content };
      }

      if (!quiet) {
        console.log(chalk.blue(`📝 Compiling ${sourceFiles.length} files...`));
      }

      // Compile with Neo DevPack for Solidity
      const compileStartedAt = Date.now();
      const compilationOutput = await hre.neoSolc.compiler.compile(sources);
      const compileDurationMs = Date.now() - compileStartedAt;

      const compilerSettings = {
        optimizer: hre.config.neoSolc.solidity.settings.optimizer,
        outputSelection: hre.config.neoSolc.solidity.settings.outputSelection || {
          "*": {
            "*": [
              "abi",
              "evm.bytecode",
              "evm.deployedBytecode",
              "evm.methodIdentifiers",
              "metadata",
              "storageLayout"
            ]
          }
        }
      };
      const compilerNeoSettings = hre.config.neoSolc.solidity.settings.neo;

      // Save build info
      const buildInfo: BuildInfo = {
        id: generateBuildId(),
        solcVersion: hre.config.neoSolc.solidity.version || "0.8.19",
        neoSolcVersion: "0.1.0", // Would get from Neo compiler
        input: { language: "Solidity", sources, settings: compilerSettings as any },
        output: compilationOutput,
        metadata: {
          timestamp: new Date().toISOString(),
          duration: compileDurationMs,
          sourceFiles: Object.keys(sources),
          optimization: hre.config.neoSolc.solidity.settings.optimizer,
          neo: {
            gasCostModel: "neo",
            storageOptimization: Boolean(compilerNeoSettings?.optimizeGas),
            eventOptimization: Boolean(compilerNeoSettings?.optimizeGas)
          }
        }
      };

      // Save build info for traceability/debugging.
      const buildInfoDir = path.join(hre.config.paths.artifacts, "neo-build-info");
      await fs.mkdir(buildInfoDir, { recursive: true });
      await fs.writeFile(
        path.join(buildInfoDir, `${buildInfo.id}.json`),
        JSON.stringify(buildInfo, null, 2)
      );

      // Save artifacts
      let compiledContracts = 0;
      for (const fileName of Object.keys(compilationOutput.contracts)) {
        for (const contractName of Object.keys(compilationOutput.contracts[fileName])) {
          const contract = compilationOutput.contracts[fileName][contractName];
          
          const artifact: BuildArtifact = {
            contractName,
            sourceName: fileName,
            metadata: {
              compiler: {
                version: buildInfo.neoSolcVersion,
                settings: hre.config.neoSolc.solidity.settings
              },
              buildTime: buildInfo.metadata.timestamp,
              environment: {
                nodeVersion: process.version,
                platform: process.platform,
                architecture: process.arch
              },
              dependencies: {} // Would extract from package.json
            },
            contract,
            buildInfo: buildInfo.id
          };

          await hre.neoSolc.artifacts.saveBuildArtifact(artifact);
          compiledContracts++;
        }
      }

      if (!quiet) {
        console.log(chalk.green(`✅ Successfully compiled ${compiledContracts} contracts`));
        
        // Print contract sizes
        await printContractSizes(compilationOutput);
        
        // Print warnings if any
        if (compilationOutput.errors) {
          const warnings = compilationOutput.errors.filter((e: any) => e.severity === "warning");
          if (warnings.length > 0) {
            const uniqueWarnings = new Set(
              warnings.map((w: any) => w.formattedMessage || w.message),
            );
            console.log(
              chalk.yellow(
                `⚠️  ${warnings.length} warnings (${uniqueWarnings.size} unique)`,
              ),
            );
          }
        }
      }

    } catch (error) {
      console.error(chalk.red("❌ Compilation failed:"));
      console.error(error instanceof Error ? error.message : String(error));
      throw error;
    }
  });

/**
 * Get all Solidity source files
 */
async function getSourceFiles(sourcesPath: string): Promise<string[]> {
  const files: string[] = [];
  const discovered = new Set<string>();
  
  async function scan(dir: string) {
    const entries = await fs.readdir(dir, { withFileTypes: true });
    
    for (const entry of entries) {
      const fullPath = path.join(dir, entry.name);
      
      if (entry.isDirectory()) {
        if (entry.name === "node_modules" || entry.name === "artifacts" || entry.name === "cache") {
          continue;
        }
        await scan(fullPath);
      } else if (entry.name.endsWith(".sol")) {
        files.push(fullPath);
        discovered.add(path.resolve(fullPath));
      }
    }
  }
  
  try {
    await scan(sourcesPath);
  } catch (error) {
    if ((error as any).code !== "ENOENT") {
      throw error;
    }
  }

  // Expand the source set with relative imports that point outside `sourcesPath`
  // (e.g., contracts importing ../libraries or ../standards in monorepos).
  const pending = [...files];
  const importRegex = /^\s*import\s+(?:[^'"]+from\s+)?["']([^"']+)["'];/gm;

  while (pending.length > 0) {
    const current = pending.pop()!;
    const content = await fs.readFile(current, "utf-8");
    const baseDir = path.dirname(current);

    importRegex.lastIndex = 0;
    let match: RegExpExecArray | null;
    while ((match = importRegex.exec(content)) !== null) {
      const specifier = match[1];
      if (!specifier.startsWith(".")) {
        continue;
      }

      let resolved = path.resolve(baseDir, specifier);
      if (!resolved.endsWith(".sol")) {
        resolved += ".sol";
      }

      try {
        await fs.access(resolved);
      } catch {
        continue;
      }

      const normalized = path.resolve(resolved);
      if (discovered.has(normalized)) {
        continue;
      }

      discovered.add(normalized);
      files.push(normalized);
      pending.push(normalized);
    }
  }

  return files.sort((a, b) => a.localeCompare(b));
}

/**
 * Check if compilation is needed
 */
async function checkCompilationNeeded(sourceFiles: string[], hre: HardhatRuntimeEnvironment): Promise<boolean> {
  try {
    // Get modification times of source files
    const sourceStats = await Promise.all(
      sourceFiles.map(async (file) => {
        const stat = await fs.stat(file);
        return { file, mtime: stat.mtime };
      })
    );
    
    const latestSource = sourceStats.reduce((latest, current) => 
      current.mtime > latest.mtime ? current : latest
    );

    // Check if any artifacts exist and their modification time
    const artifacts = await hre.neoSolc.artifacts.getAllBuildArtifacts();
    if (artifacts.length === 0) {
      return true; // No artifacts, need to compile
    }

    // Check if source is newer than any artifact
    for (const artifact of artifacts) {
      const buildTime = new Date(artifact.metadata.buildTime);
      if (latestSource.mtime > buildTime) {
        return true;
      }
    }

    return false;
  } catch (_error) {
    // If we can't determine, assume compilation is needed
    return true;
  }
}

/**
 * Print contract sizes
 */
async function printContractSizes(compilationOutput: any) {
  console.log(chalk.blue("\n📊 Contract sizes:"));
  
  for (const fileName of Object.keys(compilationOutput.contracts)) {
    for (const contractName of Object.keys(compilationOutput.contracts[fileName])) {
      const contract = compilationOutput.contracts[fileName][contractName];
      
      // Calculate sizes
      const evmBytecodeSize = contract.evm.bytecode.object.length / 2; // hex to bytes
      const neoScriptSize = contract.neo.nef.script.length / 2;
      
      console.log(`  ${contractName}:`);
      console.log(`    EVM bytecode: ${chalk.cyan(evmBytecodeSize.toLocaleString())} bytes`);
      console.log(`    Neo script: ${chalk.cyan(neoScriptSize.toLocaleString())} bytes`);
      
      // Size warnings
      if (evmBytecodeSize > 24576) { // Ethereum contract size limit
        console.log(chalk.yellow(`    ⚠️  Large contract size (>${24576} bytes)`));
      }
    }
  }
}

/**
 * Generate unique build ID
 */
function generateBuildId(): string {
  return `build-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
}
