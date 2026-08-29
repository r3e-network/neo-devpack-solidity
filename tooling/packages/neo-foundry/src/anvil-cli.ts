#!/usr/bin/env node

import { Command } from "commander";
import chalk from "chalk";
import { NeoAnvil } from "./anvil.js";

const program = new Command();

program
  .name("neo-anvil")
  .description("Neo-Anvil - Local Neo blockchain simulation")
  .version("0.1.0");

// Start command (default)
program
  .command("start")
  .alias("run")
  .description("Start local Neo blockchain")
  .option("-p, --port <port>", "Port number", parseInt, 40332)
  .option("-c, --chain-id <id>", "Chain ID", parseInt, 12345)
  .option("-a, --accounts <count>", "Number of accounts", parseInt, 10)
  .option("-b, --balance <balance>", "Account balance in GAS", "100000000000000")
  .option("-g, --gas-limit <limit>", "Gas limit", "50000000")
  .option("--gas-price <price>", "Gas price", "1000")
  .option("-t, --block-time <seconds>", "Block time in seconds", parseInt, 15)
  .option("--fork-url <url>", "Fork from URL")
  .option("--fork-block <number>", "Fork block number", parseInt)
  .option("-q, --quiet", "Suppress output")
  .action(async (options) => {
    try {
      const anvil = new NeoAnvil();
      
      await anvil.start({
        port: options.port,
        chainId: options.chainId,
        accounts: options.accounts,
        balance: options.balance,
        gasLimit: options.gasLimit,
        gasPrice: options.gasPrice,
        blockTime: options.blockTime,
        fork: options.forkUrl,
        forkBlockNumber: options.forkBlock,
        quiet: options.quiet
      });

      // Keep process running
      process.on('SIGINT', async () => {
        console.log(chalk.blue("\n🛑 Shutting down Neo-Anvil..."));
        await anvil.stop();
        process.exit(0);
      });

      // Prevent process from exiting
      setInterval(() => {}, 1000);
      
    } catch (error) {
      console.error(chalk.red("Failed to start Neo-Anvil:"), error);
      process.exit(1);
    }
  });

// Stop command
program
  .command("stop")
  .description("Stop local Neo blockchain")
  .action(async () => {
    try {
      const anvil = new NeoAnvil();
      await anvil.stop();
    } catch (error) {
      console.error(chalk.red("Failed to stop Neo-Anvil:"), error);
      process.exit(1);
    }
  });

// Reset command
program
  .command("reset")
  .description("Reset blockchain state")
  .action(async () => {
    try {
      const anvil = new NeoAnvil();
      await anvil.reset();
    } catch (error) {
      console.error(chalk.red("Failed to reset Neo-Anvil:"), error);
      process.exit(1);
    }
  });

// Mine command
program
  .command("mine [blocks]")
  .description("Mine new block(s)")
  .action(async (blocks) => {
    try {
      const blockCount = blocks ? parseInt(blocks) : 1;
      const anvil = new NeoAnvil();
      await anvil.mine(blockCount);
    } catch (error) {
      console.error(chalk.red("Mining failed:"), error);
      process.exit(1);
    }
  });

// Time manipulation commands
program
  .command("set-time <timestamp>")
  .description("Set time for next block")
  .action(async (timestamp) => {
    try {
      const anvil = new NeoAnvil();
      await anvil.setTime(parseInt(timestamp));
    } catch (error) {
      console.error(chalk.red("Set time failed:"), error);
      process.exit(1);
    }
  });

program
  .command("increase-time <seconds>")
  .description("Increase time by seconds")
  .action(async (seconds) => {
    try {
      const anvil = new NeoAnvil();
      await anvil.increaseTime(parseInt(seconds));
    } catch (error) {
      console.error(chalk.red("Increase time failed:"), error);
      process.exit(1);
    }
  });

// Snapshot commands
program
  .command("snapshot")
  .description("Take blockchain snapshot")
  .action(async () => {
    try {
      const anvil = new NeoAnvil();
      const snapshotId = await anvil.snapshot();
      console.log(chalk.green(`Snapshot ID: ${snapshotId}`));
    } catch (error) {
      console.error(chalk.red("Snapshot failed:"), error);
      process.exit(1);
    }
  });

program
  .command("restore <snapshot-id>")
  .description("Restore from snapshot")
  .action(async (snapshotId) => {
    try {
      const anvil = new NeoAnvil();
      await anvil.restore(snapshotId);
    } catch (error) {
      console.error(chalk.red("Restore failed:"), error);
      process.exit(1);
    }
  });

// Status command
program
  .command("status")
  .description("Get Neo-Anvil status")
  .action(() => {
    try {
      const anvil = new NeoAnvil();
      const status = anvil.getStatus();
      
      console.log(chalk.blue("Neo-Anvil Status:"));
      console.log(`  Running: ${status.running ? chalk.green("Yes") : chalk.red("No")}`);
      
      if (status.port) {
        console.log(`  Port: ${status.port}`);
      }
      if (status.chainId) {
        console.log(`  Chain ID: ${status.chainId}`);
      }
      if (status.accounts) {
        console.log(`  Accounts: ${status.accounts}`);
      }
    } catch (error) {
      console.error(chalk.red("Status check failed:"), error);
      process.exit(1);
    }
  });

// Version command
program
  .command("version")
  .description("Show version information")
  .action(() => {
    console.log(chalk.blue("Neo-Anvil version 0.1.0"));
    console.log("Local Neo blockchain simulation for development and testing");
  });

// Help command
program
  .command("help [command]")
  .description("Show help for command")
  .action((command) => {
    if (command) {
      program.outputHelp();
    } else {
      console.log(chalk.blue("Neo-Anvil - Local Neo blockchain simulation"));
      console.log("");
      console.log("Blockchain control:");
      console.log("  neo-anvil start                Start local blockchain");
      console.log("  neo-anvil stop                 Stop local blockchain");
      console.log("  neo-anvil reset                Reset blockchain state");
      console.log("  neo-anvil status               Show status");
      console.log("");
      console.log("Block manipulation:");
      console.log("  neo-anvil mine [blocks]        Mine new blocks");
      console.log("  neo-anvil set-time <timestamp> Set next block time");
      console.log("  neo-anvil increase-time <sec>  Increase time");
      console.log("");
      console.log("Snapshots:");
      console.log("  neo-anvil snapshot             Take snapshot");
      console.log("  neo-anvil restore <id>         Restore snapshot");
      console.log("");
      console.log("Options for start command:");
      console.log("  -p, --port <port>              Port number (default: 40332)");
      console.log("  -c, --chain-id <id>            Chain ID (default: 12345)");
      console.log("  -a, --accounts <count>         Number of accounts (default: 10)");
      console.log("  -b, --balance <balance>        Account balance (default: 10000000 GAS)");
      console.log("  --fork-url <url>               Fork from URL");
      console.log("  --fork-block <number>          Fork block number");
      console.log("");
      console.log("Use 'neo-anvil <command> --help' for detailed information");
    }
  });

// Make start the default command if no command is specified
if (process.argv.length === 2) {
  process.argv.push('start');
}

// Error handling
program.on('command:*', () => {
  console.error(chalk.red(`Invalid command: ${program.args.join(' ')}`));
  console.log('Use "neo-anvil help" for available commands');
  process.exit(1);
});

// Parse command line arguments
program.parse();
