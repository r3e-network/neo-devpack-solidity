import { Dirent, promises as fs } from "fs";
import path from "path";
import { spawn } from "child_process";
import chalk from "chalk";
import type { CompilationInput } from "@neo-devpack-solidity/types";
import { ConfigManager } from "./config.js";
import { CompilerInvoker } from "./compiler-invoker.js";
import { ArtifactCollector, NeoForgeBuildArtifact } from "./artifact-collector.js";
import { BuildCache } from "./build-cache.js";
import { NeoForgeBuildError } from "./build-error.js";

/**
 * Resolve the `neo-test` runner binary: the `NEO_TEST` env var wins, otherwise
 * `neo-test` is expected on `PATH`. (Build it from this repo with
 * `cargo build --release --bin neo-test`.)
 */
function resolveNeoTestBin(): string {
  return process.env.NEO_TEST || "neo-test";
}

function runNeoTest(bin: string, args: string[]): Promise<void> {
  return new Promise((resolve, reject) => {
    const child = spawn(bin, args, { stdio: "inherit" });
    child.on("error", (err: NodeJS.ErrnoException) => {
      if (err.code === "ENOENT") {
        reject(
          new Error(
            `neo-test binary not found ('${bin}'). Build it with ` +
              "`cargo build --release --bin neo-test` and put it on PATH, " +
              "or set the NEO_TEST environment variable to its path."
          )
        );
      } else {
        reject(err);
      }
    });
    child.on("close", (code) => {
      if (code === 0) resolve();
      else reject(new Error(`neo-test exited with code ${code ?? "unknown"}`));
    });
  });
}

export interface BuildOptions {
  force?: boolean;
  watch?: boolean;
  profile?: string;
  quiet?: boolean;
}

export interface BuildResult {
  artifacts: NeoForgeBuildArtifact[];
  cached: boolean;
}

/**
 * Neo-Forge - Foundry-like UX for Neo DevPack for Solidity projects.
 *
 * `build()` is the primary entry point: it discovers sources, invokes
 * `neo-solc --standard-json`, and writes `.nef`/`.manifest.json` artifacts.
 */
export class NeoForge {
  private readonly config: ConfigManager;
  private readonly profileName: string;

  constructor(configPath?: string, profileName = "default") {
    this.config = new ConfigManager(configPath);
    this.profileName = profileName;
  }

  async build(options: BuildOptions = {}): Promise<BuildResult> {
    await this.config.loadConfig();
    const profile = this.config.getProfile(options.profile || this.profileName);

    if (!options.quiet) {
      console.log(chalk.blue("🔧 neo-forge build"));
      console.log(`  profile: ${options.profile || this.profileName}`);
      console.log(`  src: ${profile.src}`);
      console.log(`  out: ${profile.out}`);
      if (options.watch) {
        console.log(chalk.yellow("  watch mode is not implemented yet"));
      }
    }

    const projectRoot = this.config.getConfigDir();
    const srcDir = path.resolve(projectRoot, profile.src);
    const outDir = path.resolve(projectRoot, profile.out);
    const cacheDir = path.resolve(projectRoot, profile.build.cacheDir);

    const sourceFiles = await this.discoverSources(srcDir);
    if (sourceFiles.length === 0) {
      throw new NeoForgeBuildError(
        `NSH-7010: no Solidity source files found in ${profile.src}.`,
        { code: "NSH-7010" }
      );
    }

    if (!options.quiet) {
      console.log(chalk.blue(`📝 Compiling ${sourceFiles.length} files...`));
    }

    const cache = new BuildCache(cacheDir);
    const expectedArtifacts = await this.estimateArtifactPaths(sourceFiles, srcDir, outDir);

    if (!options.force && profile.build.incremental) {
      const upToDate = await cache.isUpToDate(sourceFiles, expectedArtifacts);
      if (upToDate) {
        if (!options.quiet) {
          console.log(chalk.green("✅ Build is up to date"));
        }
        return {
          artifacts: sourceFiles.map((sourceFile) => {
            const contractName = path.basename(sourceFile, ".sol");
            const sourceName = path.relative(srcDir, sourceFile).split(path.sep).join("/");
            const contractOutDir = path.join(outDir, path.dirname(sourceName), contractName);
            return {
              contractName,
              sourceName,
              nefPath: path.join(contractOutDir, `${contractName}.nef`),
              manifestPath: path.join(contractOutDir, `${contractName}.manifest.json`),
            };
          }),
          cached: true,
        };
      }
    }

    const sources = await this.readSources(sourceFiles, srcDir);
    const input: CompilationInput = {
      language: "Solidity",
      sources,
      settings: {
        optimizer: profile.neoSolc.optimizer,
      },
    };

    const invoker = new CompilerInvoker();
    const output = await invoker.invoke(input, { cwd: projectRoot, verbose: !options.quiet });

    const collector = new ArtifactCollector();
    const artifacts = await collector.collect(output, outDir);

    if (profile.build.incremental) {
      const artifactPaths = artifacts.map((a) => [a.nefPath, a.manifestPath]).flat();
      await cache.update(sourceFiles, artifactPaths);
    }

    if (!options.quiet) {
      console.log(chalk.green(`✅ Successfully compiled ${artifacts.length} contract(s)`));
      for (const artifact of artifacts) {
        console.log(`  ${artifact.contractName}: ${path.relative(projectRoot, artifact.nefPath)}`);
      }
    }

    return { artifacts, cached: false };
  }

  async test(options: {
    pattern?: string;
    verbose?: boolean;
    gasReport?: boolean;
    coverage?: boolean;
    forkUrl?: string;
    forkBlockNumber?: number;
    profile?: string;
  } = {}): Promise<void> {
    await this.config.loadConfig();
    const profile = this.config.getProfile(options.profile || this.profileName);
    const testDir = profile.test || "test";

    // Delegate to the native `neo-test` runner: it compiles each *.t.sol with
    // neo-solc and executes every test*()/setUp() on the in-tree NeoVM.
    const args: string[] = [testDir];
    if (options.pattern) args.push("--match-test", options.pattern);
    if (options.gasReport) args.push("--gas");
    if (options.verbose) args.push("-v");
    if (options.coverage) {
      console.log(chalk.yellow("  (coverage is not yet supported by neo-test — ignoring --coverage)"));
    }
    if (options.forkUrl || typeof options.forkBlockNumber === "number") {
      console.log(chalk.yellow("  (forking is not yet supported by neo-test — ignoring --fork-*)"));
    }

    await runNeoTest(resolveNeoTestBin(), args);
  }

  async clean(profileName?: string): Promise<void> {
    await this.config.loadConfig();
    const profile = this.config.getProfile(profileName || this.profileName);

    console.log(chalk.blue("🧹 Cleaning build artifacts..."));

    await this.rmrf(profile.out);
    await this.rmrf(profile.build.cacheDir);

    console.log(chalk.green("✅ Cleaning completed"));
  }

  async init(projectPath = "."): Promise<void> {
    console.log(chalk.blue("🚀 Initializing Neo-Foundry project..."));
    await this.config.initProject(projectPath);
    console.log(chalk.green("✅ Project initialized successfully!"));
  }

  async install(dependencies: string[]): Promise<void> {
    console.log(chalk.blue(`📦 neo-forge install (scaffold) (${dependencies.length} deps)`));
    void dependencies;
    throw new Error("neo-forge install is not implemented yet.");
  }

  async remove(dependency: string): Promise<void> {
    console.log(chalk.blue(`🗑️  neo-forge remove (scaffold): ${dependency}`));
    throw new Error("neo-forge remove is not implemented yet.");
  }

  async update(): Promise<void> {
    console.log(chalk.blue("🔄 neo-forge update (scaffold)"));
    throw new Error("neo-forge update is not implemented yet.");
  }

  async inspect(contract: string, options: { pretty?: boolean } = {}): Promise<void> {
    await this.config.loadConfig();
    void options;

    console.log(chalk.blue(`🔎 neo-forge inspect (scaffold): ${contract}`));
    throw new Error("neo-forge inspect is not implemented yet.");
  }

  private async discoverSources(srcDir: string): Promise<string[]> {
    const files: string[] = [];

    async function scan(dir: string) {
      let entries: Dirent[];
      try {
        entries = await fs.readdir(dir, { withFileTypes: true });
      } catch (error) {
        if ((error as NodeJS.ErrnoException).code === "ENOENT") {
          return;
        }
        throw error;
      }

      for (const entry of entries) {
        const fullPath = path.join(dir, entry.name);
        if (entry.isDirectory()) {
          if (entry.name === "node_modules" || entry.name === ".git") {
            continue;
          }
          await scan(fullPath);
        } else if (entry.name.endsWith(".sol")) {
          files.push(fullPath);
        }
      }
    }

    await scan(srcDir);
    return files.sort((a, b) => a.localeCompare(b));
  }

  private async readSources(
    sourceFiles: string[],
    srcDir: string
  ): Promise<Record<string, { content: string }>> {
    const sources: Record<string, { content: string }> = {};
    for (const filePath of sourceFiles) {
      const content = await fs.readFile(filePath, "utf-8");
      const relativePath = path.relative(srcDir, filePath).split(path.sep).join("/");
      sources[relativePath] = { content };
    }
    return sources;
  }

  private async estimateArtifactPaths(
    sourceFiles: string[],
    srcDir: string,
    outDir: string
  ): Promise<string[]> {
    const paths: string[] = [];
    for (const sourceFile of sourceFiles) {
      const contractName = path.basename(sourceFile, ".sol");
      const sourceName = path.relative(srcDir, sourceFile).split(path.sep).join("/");
      const contractOutDir = path.join(outDir, path.dirname(sourceName), contractName);
      paths.push(path.join(contractOutDir, `${contractName}.nef`));
      paths.push(path.join(contractOutDir, `${contractName}.manifest.json`));
    }
    return paths;
  }

  private async rmrf(dir: string): Promise<void> {
    const full = path.isAbsolute(dir) ? dir : path.join(process.cwd(), dir);
    await fs.rm(full, { recursive: true, force: true });
  }
}
