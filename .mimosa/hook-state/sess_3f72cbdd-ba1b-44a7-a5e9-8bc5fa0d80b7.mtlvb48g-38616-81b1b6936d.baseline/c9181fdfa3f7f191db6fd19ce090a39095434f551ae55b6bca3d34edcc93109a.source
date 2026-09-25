#!/usr/bin/env node

import { Command } from "commander";
import chalk from "chalk";
import { NeoCast } from "./cast.js";
import { ConfigManager } from "./config.js";

const program = new Command();

program
  .name("neo-cast")
  .description("Neo-Cast - Interact with Neo contracts and perform RPC calls")
  .version("0.1.0");

// RPC URL option for all commands
program
  .option("-r, --rpc-url <url>", "RPC endpoint URL")
  .option("-c, --config <path>", "Configuration file path");

// Contract interaction commands
program
  .command("call <address> <method> [args...]")
  .description("Call contract method (read-only)")
  .option("-b, --block <block>", "Block number or tag")
  .option("-f, --from <address>", "From address")
  .action(async (address, method, args, options) => {
    try {
      const cast = await createCast(options, program.opts());
      await cast.call(address, method, args || [], {
        blockTag: options.block,
        from: options.from
      });
    } catch (error) {
      console.error(chalk.red("Call failed:"), error);
      process.exit(1);
    }
  });

program
  .command("send <address> <method> [args...]")
  .description("Send transaction to contract method")
  .option("-f, --from <address>", "From address")
  .option("-g, --gas-limit <limit>", "Gas limit")
  .option("-p, --gas-price <price>", "Gas price")
  .option("-v, --value <value>", "Value to send")
  .action(async (address, method, args, options) => {
    try {
      const cast = await createCast(options, program.opts());
      const txHash = await cast.send(address, method, args || [], {
        from: options.from,
        gasLimit: options.gasLimit,
        gasPrice: options.gasPrice,
        value: options.value
      });
      
      console.log(chalk.blue("Transaction hash:"), txHash);
    } catch (error) {
      console.error(chalk.red("Transaction failed:"), error);
      process.exit(1);
    }
  });

program
  .command("estimate-gas <address> <method> [args...]")
  .description("Estimate gas for contract method")
  .option("-f, --from <address>", "From address")
  .option("-v, --value <value>", "Value to send")
  .action(async (address, method, args, options) => {
    try {
      const cast = await createCast(options, program.opts());
      await cast.estimateGas(address, method, args || [], {
        from: options.from,
        value: options.value
      });
    } catch (error) {
      console.error(chalk.red("Gas estimation failed:"), error);
      process.exit(1);
    }
  });

// Deployment commands
program
  .command("deploy <bytecode> [args...]")
  .description("Deploy contract")
  .option("-f, --from <address>", "From address")
  .option("-g, --gas-limit <limit>", "Gas limit")
  .option("-p, --gas-price <price>", "Gas price")
  .option("-v, --value <value>", "Value to send")
  .action(async (bytecode, args, options) => {
    try {
      const cast = await createCast(options, program.opts());
      const address = await cast.deployContract(bytecode, args || [], {
        from: options.from,
        gasLimit: options.gasLimit,
        gasPrice: options.gasPrice,
        value: options.value
      });
      
      console.log(chalk.blue("Contract address:"), address);
    } catch (error) {
      console.error(chalk.red("Deployment failed:"), error);
      process.exit(1);
    }
  });

// Account and balance commands
program
  .command("balance <address>")
  .description("Get account balance")
  .action(async (address, options) => {
    try {
      const cast = await createCast(options, program.opts());
      await cast.balance(address);
    } catch (error) {
      console.error(chalk.red("Balance check failed:"), error);
      process.exit(1);
    }
  });

program
  .command("generate-account")
  .description("Generate new account")
  .action(async (options) => {
    try {
      const cast = await createCast(options, program.opts());
      await cast.generateAccount();
    } catch (error) {
      console.error(chalk.red("Account generation failed:"), error);
      process.exit(1);
    }
  });

// Blockchain commands
program
  .command("transaction <hash>")
  .alias("tx")
  .description("Get transaction details")
  .action(async (hash, options) => {
    try {
      const cast = await createCast(options, program.opts());
      await cast.transaction(hash);
    } catch (error) {
      console.error(chalk.red("Transaction lookup failed:"), error);
      process.exit(1);
    }
  });

program
  .command("block <hash-or-number>")
  .description("Get block information")
  .action(async (hashOrNumber, options) => {
    try {
      const cast = await createCast(options, program.opts());
      const blockId = isNaN(Number(hashOrNumber)) ? hashOrNumber : Number(hashOrNumber);
      await cast.block(blockId);
    } catch (error) {
      console.error(chalk.red("Block lookup failed:"), error);
      process.exit(1);
    }
  });

program
  .command("storage <address> <key>")
  .description("Get contract storage")
  .action(async (address, key, options) => {
    try {
      const cast = await createCast(options, program.opts());
      await cast.storage(address, key);
    } catch (error) {
      console.error(chalk.red("Storage lookup failed:"), error);
      process.exit(1);
    }
  });

// Network commands
program
  .command("network")
  .description("Get network information")
  .action(async (options) => {
    try {
      const cast = await createCast(options, program.opts());
      await cast.networkInfo();
    } catch (error) {
      console.error(chalk.red("Network info failed:"), error);
      process.exit(1);
    }
  });

// Utility commands
program
  .command("convert <value> <from> <to>")
  .description("Convert between data formats")
  .action(async (value, from, to, options) => {
    try {
      const cast = await createCast(options, program.opts());
      await cast.convert(value, from, to);
    } catch (error) {
      console.error(chalk.red("Conversion failed:"), error);
      process.exit(1);
    }
  });

// ABI encoding/decoding (future feature)
program
  .command("abi-encode <signature> [args...]")
  .description("Encode function call data")
  .action(async (signature, args, options) => {
    void signature;
    void args;
    void options;
    console.log(chalk.yellow("ABI encoding not yet implemented"));
  });

program
  .command("abi-decode <signature> <data>")
  .description("Decode function return data")
  .action(async (signature, data, options) => {
    void signature;
    void data;
    void options;
    console.log(chalk.yellow("ABI decoding not yet implemented"));
  });

// Interactive mode
program
  .command("console")
  .description("Start interactive console")
  .action(async (options) => {
    void options;
    console.log(chalk.blue("🚀 Starting Neo-Cast interactive console..."));
    console.log(chalk.gray("Type 'help' for available commands, 'exit' to quit"));
    
    // This would start an interactive REPL
    console.log(chalk.yellow("Interactive console not yet implemented"));
  });

// Help command
program
  .command("help [command]")
  .description("Show help for command")
  .action((command) => {
    if (command) {
      program.outputHelp();
    } else {
      console.log(chalk.blue("Neo-Cast - Interact with Neo contracts"));
      console.log("");
      console.log("Contract interaction:");
      console.log("  neo-cast call <address> <method> [args]    Call contract method");
      console.log("  neo-cast send <address> <method> [args]    Send transaction");
      console.log("  neo-cast deploy <bytecode> [args]          Deploy contract");
      console.log("");
      console.log("Blockchain queries:");
      console.log("  neo-cast balance <address>                 Get account balance");
      console.log("  neo-cast transaction <hash>                Get transaction details");
      console.log("  neo-cast block <hash-or-number>            Get block information");
      console.log("  neo-cast storage <address> <key>           Get contract storage");
      console.log("");
      console.log("Utilities:");
      console.log("  neo-cast convert <value> <from> <to>       Convert data formats");
      console.log("  neo-cast generate-account                  Generate new account");
      console.log("  neo-cast network                           Get network info");
      console.log("");
      console.log("Use 'neo-cast <command> --help' for detailed information");
    }
  });

// Error handling
program.on('command:*', () => {
  console.error(chalk.red(`Invalid command: ${program.args.join(' ')}`));
  console.log('Use "neo-cast help" for available commands');
  process.exit(1);
});

// Parse command line arguments
program.parse();

// Helper function to create Cast instance
async function createCast(_commandOptions: any, globalOptions: any): Promise<NeoCast> {
  const configPath = globalOptions.config;
  const rpcUrl = globalOptions.rpcUrl;
  
  const cast = new NeoCast(configPath);
  
  if (rpcUrl) {
    cast.setRpc(rpcUrl);
  } else {
    // Try to get RPC URL from config
    const config = new ConfigManager(configPath);
    await config.loadConfig();
    
    const profile = config.getProfile();
    const networks = Object.values(profile.networks);
    
    if (networks.length > 0) {
      const defaultNetwork = networks[0] as any;
      cast.setRpc(defaultNetwork.url);
    } else {
      throw new Error("No RPC URL specified and no networks configured");
    }
  }
  
  return cast;
}
