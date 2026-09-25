#!/usr/bin/env node

import { Command } from "commander";
import chalk from "chalk";
import ora from "ora";
import { CompilerCLI } from "./compiler-cli.js";

const program = new Command();

program
  .name("solc-neo")
  .description("Neo DevPack for Solidity Compiler Command Line Interface")
  .version("0.1.0");

// Compilation commands
program
  .command("compile [files...]")
  .alias("c")
  .description("Compile Solidity files to NeoVM bytecode")
  .option("-o, --output <dir>", "Output directory", "build")
  .option("-O, --optimize", "Enable optimizations")
  .option("--optimize-runs <runs>", "Number of optimization runs", "200")
  .option("--gas-model <model>", "Gas cost model (ethereum|neo|hybrid)", "hybrid")
  .option("--storage-opt", "Enable storage optimization")
  .option("--event-opt", "Enable event optimization")
  .option("--include-paths <paths>", "Include paths (comma-separated)")
  .option("--libraries <libs>", "Library addresses (format: LibName:address)")
  .option("--metadata", "Include metadata in output")
  .option("--combined-json <items>", "Output combined JSON (abi,bin,metadata)")
  .option("--standard-json", "Use standard JSON input/output")
  .option("-v, --verbose", "Verbose output")
  .option("-q, --quiet", "Suppress output")
  .action(async (files, options) => {
    const spinner = ora("Compiling contracts...").start();

    try {
      const compiler = new CompilerCLI();
      const result = await compiler.compile(files, options);

      spinner.succeed(chalk.green(`Successfully compiled ${result.contractCount} contracts`));
      
      if (!options.quiet) {
        console.log(chalk.blue("\n📋 Compilation Summary:"));
        console.log(`  Contracts: ${result.contractCount}`);
        console.log(`  Duration: ${result.duration}ms`);
        console.log(`  Output: ${result.outputDir}`);
        
        if (result.warnings > 0) {
          console.log(chalk.yellow(`  Warnings: ${result.warnings}`));
        }
      }
    } catch (error) {
      spinner.fail(chalk.red("Compilation failed"));
      console.error(chalk.red(error instanceof Error ? error.message : String(error)));
      process.exit(1);
    }
  });

program
  .command("version")
  .description("Show compiler version information")
  .action(async () => {
    try {
      const compiler = new CompilerCLI();
      const version = await compiler.getVersion();
      
      console.log(chalk.blue("Neo DevPack for Solidity Compiler"));
      console.log(`Version: ${version.compiler}`);
      console.log(`Solidity: ${version.solidity}`);
      console.log(`Neo VM: ${version.neovm}`);
    } catch (_error) {
      console.error(chalk.red("Failed to get version information"));
      process.exit(1);
    }
  });

program
  .command("install [version]")
  .description("Install specific compiler version")
  .action(async (version) => {
    const spinner = ora(`Installing compiler version ${version || 'latest'}...`).start();

    try {
      const compiler = new CompilerCLI();
      await compiler.install(version);
      
      spinner.succeed(chalk.green("Compiler installed successfully"));
    } catch (error) {
      spinner.fail(chalk.red("Installation failed"));
      console.error(chalk.red(error instanceof Error ? error.message : String(error)));
      process.exit(1);
    }
  });

program
  .command("list")
  .description("List available compiler versions")
  .action(async () => {
    try {
      const compiler = new CompilerCLI();
      const versions = await compiler.listVersions();
      
      console.log(chalk.blue("Available Neo DevPack for Solidity Compiler Versions:"));
      versions.forEach(v => {
        const marker = v.current ? chalk.green("* ") : "  ";
        console.log(`${marker}${v.version}${v.prerelease ? chalk.yellow(" (prerelease)") : ""}`);
      });
    } catch (_error) {
      console.error(chalk.red("Failed to list versions"));
      process.exit(1);
    }
  });

// Standard JSON mode
program
  .command("standard-json")
  .description("Compile using standard JSON input/output")
  .option("-i, --input <file>", "Input JSON file (default: stdin)")
  .option("-o, --output <file>", "Output JSON file (default: stdout)")
  .action(async (options) => {
    try {
      const compiler = new CompilerCLI();
      await compiler.standardJson(options.input, options.output);
    } catch (error) {
      console.error(chalk.red("Standard JSON compilation failed"));
      console.error(chalk.red(error instanceof Error ? error.message : String(error)));
      process.exit(1);
    }
  });

// Optimization commands
program
  .command("analyze [files...]")
  .description("Analyze contracts for optimization opportunities")
  .option("--gas-report", "Generate gas usage report")
  .option("--size-report", "Generate contract size report")
  .option("--output <format>", "Output format (table|json|csv)", "table")
  .action(async (files, options) => {
    const spinner = ora("Analyzing contracts...").start();

    try {
      const compiler = new CompilerCLI();
      const analysis = await compiler.analyze(files, options);
      
      spinner.succeed(chalk.green("Analysis completed"));
      
      console.log(chalk.blue("\n📊 Analysis Results:"));
      compiler.displayAnalysis(analysis, options.output);
    } catch (error) {
      spinner.fail(chalk.red("Analysis failed"));
      console.error(chalk.red(error instanceof Error ? error.message : String(error)));
      process.exit(1);
    }
  });

// Utility commands
program
  .command("flatten <file>")
  .description("Flatten Solidity file (resolve imports)")
  .option("-o, --output <file>", "Output file")
  .action(async (file, options) => {
    try {
      const compiler = new CompilerCLI();
      const flattened = await compiler.flatten(file);
      
      if (options.output) {
        const fs = await import('fs/promises');
        await fs.writeFile(options.output, flattened);
        console.log(chalk.green(`Flattened code saved to ${options.output}`));
      } else {
        console.log(flattened);
      }
    } catch (error) {
      console.error(chalk.red("Flattening failed"));
      console.error(chalk.red(error instanceof Error ? error.message : String(error)));
      process.exit(1);
    }
  });

program
  .command("verify-contract")
  .description("Verify contract source code")
  .requiredOption("-a, --address <address>", "Contract address")
  .requiredOption("-s, --source <file>", "Source file")
  .option("-c, --contract <name>", "Contract name (if the source contains multiple contracts)")
  .option("-n, --network <network>", "Network name", "testnet")
  .option("--constructor-args <args>", "Constructor arguments (JSON)")
  .option("--optimize", "Enable optimizer (must match deployed build settings)")
  .option("--optimize-runs <runs>", "Optimizer runs (default: 200)")
  .option("--gas-model <model>", "Neo gas cost model (hybrid|neo)", "hybrid")
  .action(async (options) => {
    const spinner = ora("Verifying contract...").start();

    try {
      const compiler = new CompilerCLI();
      const result = await compiler.verifyContract(options);
      
      if (result.success) {
        spinner.succeed(chalk.green("Contract verified successfully"));
        if (result.explorerUrl) {
          console.log(chalk.blue(`Explorer: ${result.explorerUrl}`));
        }
      } else {
        spinner.fail(chalk.red("Verification failed"));
        console.error(chalk.red(result.error));
      }
    } catch (error) {
      spinner.fail(chalk.red("Verification failed"));
      console.error(chalk.red(error instanceof Error ? error.message : String(error)));
      process.exit(1);
    }
  });

// Help and error handling
program.on('command:*', () => {
  console.error(chalk.red(`Invalid command: ${program.args.join(' ')}`));
  console.log('Use "solc-neo --help" for available commands');
  process.exit(1);
});

if (process.argv.length <= 2) {
  program.outputHelp();
}

program.parse();
