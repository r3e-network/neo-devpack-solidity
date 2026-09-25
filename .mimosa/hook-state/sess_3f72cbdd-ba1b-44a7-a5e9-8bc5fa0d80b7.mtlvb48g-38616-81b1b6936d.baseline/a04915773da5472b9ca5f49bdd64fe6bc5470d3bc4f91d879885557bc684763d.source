#!/usr/bin/env node

import { Command } from "commander";
import chalk from "chalk";
import ora from "ora";
import { CompilerCLI } from "./compiler-cli.js";

const program = new Command();

program
  .name("neo-sol")
  .description("Neo DevPack for Solidity CLI (experimental)")
  .version("0.1.0");

program
  .command("compile [files...]")
  .alias("c")
  .description("Compile Solidity files to NeoVM bytecode (alias for solc-neo)")
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

program.on("command:*", () => {
  console.error(chalk.red(`Invalid command: ${program.args.join(" ")}`));
  console.log('Use "neo-sol --help" for available commands');
  process.exit(1);
});

if (process.argv.length <= 2) {
  program.outputHelp();
}

program.parse();
