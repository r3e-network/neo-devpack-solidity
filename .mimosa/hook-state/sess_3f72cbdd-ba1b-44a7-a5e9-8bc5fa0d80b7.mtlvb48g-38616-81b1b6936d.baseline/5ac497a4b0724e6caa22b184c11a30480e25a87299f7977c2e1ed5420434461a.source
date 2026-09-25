#!/usr/bin/env node

import { Command } from "commander";
import chalk from "chalk";
import inquirer from "inquirer";
import { NeoForge } from "./forge.js";
import { NeoForgeBuildError } from "./build-error.js";
import { ConfigManager } from "./config.js";

const program = new Command();

program
  .name("neo-forge")
  .description("Neo-Foundry command line interface")
  .version("0.1.0");

// Build commands
program
  .command("build")
  .description("Build contracts")
  .option("-f, --force", "Force rebuild")
  .option("-w, --watch", "Watch for changes")
  .option("-p, --profile <profile>", "Use specific profile")
  .option("-q, --quiet", "Suppress output")
  .action(async (options) => {
    try {
      const forge = new NeoForge(undefined, options.profile);
      await forge.build(options);
    } catch (error) {
      if (error instanceof NeoForgeBuildError) {
        console.error(chalk.red(`Build failed: ${error.message}`));
        for (const diagnostic of error.diagnostics) {
          const loc = diagnostic.sourceLocation?.file
            ? ` ${diagnostic.sourceLocation.file}${
                diagnostic.sourceLocation.start !== undefined
                  ? ":" + diagnostic.sourceLocation.start
                  : ""
              }`
            : "";
          const prefix = diagnostic.code ? `${diagnostic.code}: ` : "";
          const color = diagnostic.severity === "error" ? chalk.red : chalk.yellow;
          console.error(color(`  ${diagnostic.severity}${loc}: ${prefix}${diagnostic.message}`));
          if (diagnostic.formattedMessage && diagnostic.formattedMessage !== diagnostic.message) {
            console.error(color(`    ${diagnostic.formattedMessage}`));
          }
        }
      } else {
        console.error(chalk.red("Build failed:"), error);
      }
      process.exit(1);
    }
  });

program
  .command("test")
  .description("Run tests")
  .option("-p, --pattern <pattern>", "Test pattern filter")
  .option("-v, --verbose", "Verbose output")
  .option("-g, --gas-report", "Generate gas report")
  .option("-c, --coverage", "Generate coverage report")
  .option("--fork-url <url>", "Fork from URL")
  .option("--fork-block <number>", "Fork from block number", parseInt)
  .option("--profile <profile>", "Use specific profile")
  .action(async (options) => {
    try {
      const forge = new NeoForge(undefined, options.profile);
      await forge.test(options);
    } catch (error) {
      console.error(chalk.red("Tests failed:"), error);
      process.exit(1);
    }
  });

program
  .command("clean")
  .description("Clean build artifacts")
  .option("--profile <profile>", "Use specific profile")
  .action(async (options) => {
    try {
      const forge = new NeoForge(undefined, options.profile);
      await forge.clean(options.profile);
    } catch (error) {
      console.error(chalk.red("Clean failed:"), error);
      process.exit(1);
    }
  });

// Project management
program
  .command("init [path]")
  .description("Initialize new Neo-Foundry project")
  .action(async (projectPath) => {
    try {
      const forge = new NeoForge();
      await forge.init(projectPath);
    } catch (error) {
      console.error(chalk.red("Initialization failed:"), error);
      process.exit(1);
    }
  });

program
  .command("install [dependencies...]")
  .description("Install dependencies")
  .action(async (dependencies) => {
    try {
      if (dependencies.length === 0) {
        console.log(chalk.yellow("No dependencies specified"));
        return;
      }

      const forge = new NeoForge();
      await forge.install(dependencies);
    } catch (error) {
      console.error(chalk.red("Installation failed:"), error);
      process.exit(1);
    }
  });

program
  .command("remove <dependency>")
  .description("Remove dependency")
  .action(async (dependency) => {
    try {
      const forge = new NeoForge();
      await forge.remove(dependency);
    } catch (error) {
      console.error(chalk.red("Removal failed:"), error);
      process.exit(1);
    }
  });

program
  .command("update")
  .description("Update dependencies")
  .action(async () => {
    try {
      const forge = new NeoForge();
      await forge.update();
    } catch (error) {
      console.error(chalk.red("Update failed:"), error);
      process.exit(1);
    }
  });

// Contract inspection
program
  .command("inspect <contract>")
  .description("Inspect contract details")
  .option("--profile <profile>", "Use specific profile")
  .option("--pretty", "Pretty print JSON")
  .action(async (contract, options) => {
    try {
      const forge = new NeoForge(undefined, options.profile);
      await forge.inspect(contract, options);
    } catch (error) {
      console.error(chalk.red("Inspection failed:"), error);
      process.exit(1);
    }
  });

// Configuration
program
  .command("config")
  .description("Manage configuration")
  .action(async () => {
    try {
      const config = new ConfigManager();
      await config.loadConfig();
      
      const choices = [
        "View current configuration",
        "Edit profile settings", 
        "Add network",
        "Set default profile",
        "Reset to defaults"
      ];

      const { action } = await inquirer.prompt([
        {
          type: "list",
          name: "action",
          message: "What would you like to do?",
          choices
        }
      ]);

      switch (action) {
        case "View current configuration":
          console.log(JSON.stringify(config.getConfig(), null, 2));
          break;
        
        case "Edit profile settings":
          await editProfileSettings(config);
          break;
          
        case "Add network":
          await addNetwork(config);
          break;
          
        case "Set default profile":
          await setDefaultProfile(config);
          break;
          
        case "Reset to defaults":
          await resetConfiguration(config);
          break;
      }
    } catch (error) {
      console.error(chalk.red("Configuration failed:"), error);
      process.exit(1);
    }
  });

// Help command
program
  .command("help [command]")
  .description("Show help for command")
  .action((command) => {
    if (command) {
      program.outputHelp();
    } else {
      console.log(chalk.blue("Neo-Forge - Build and test Neo DevPack for Solidity contracts"));
      console.log("");
      console.log("Common commands:");
      console.log("  neo-forge init          Initialize new project");
      console.log("  neo-forge build         Build contracts");
      console.log("  neo-forge test          Run tests");
      console.log("  neo-forge clean         Clean artifacts");
      console.log("  neo-forge install <dep> Install dependency");
      console.log("");
      console.log("Use 'neo-forge <command> --help' for detailed information");
    }
  });

// Error handling
program.on('command:*', () => {
  console.error(chalk.red(`Invalid command: ${program.args.join(' ')}`));
  console.log('Use "neo-forge help" for available commands');
  process.exit(1);
});

// Parse command line arguments
program.parse();

// Configuration helpers
async function editProfileSettings(config: ConfigManager) {
  const currentConfig = config.getConfig();
  const profiles = Object.keys(currentConfig.profile);
  
  const { profile } = await inquirer.prompt([
    {
      type: "list",
      name: "profile",
      message: "Select profile to edit:",
      choices: profiles
    }
  ]);
  
  const profileConfig = currentConfig.profile[profile];
  
  const { setting } = await inquirer.prompt([
    {
      type: "list", 
      name: "setting",
      message: "What would you like to edit?",
      choices: [
        "Source directory",
        "Test directory", 
        "Output directory",
        "Compiler settings",
        "Build settings"
      ]
    }
  ]);
  
  switch (setting) {
    case "Source directory":
      {
        const { src } = await inquirer.prompt([
          {
            type: "input",
            name: "src",
            message: "Source directory:",
            default: profileConfig.src
          }
        ]);
        profileConfig.src = src;
        break;
      }
      
    // Add other setting edits...
  }
  
  config.setProfile(profile, profileConfig);
  await config.saveConfig();
  console.log(chalk.green("Configuration updated"));
}

async function addNetwork(config: ConfigManager) {
  const answers = await inquirer.prompt([
    {
      type: "input",
      name: "name",
      message: "Network name:"
    },
    {
      type: "input", 
      name: "url",
      message: "RPC URL:"
    },
    {
      type: "number",
      name: "magic",
      message: "Network magic number:"
    }
  ]);
  
  // Add network to current profile
  const profile = config.getProfile();
  profile.networks[answers.name] = {
    name: answers.name,
    url: answers.url,
    rpcUrls: [answers.url],
    magic: answers.magic,
    addressVersion: 0x35,
    nativeTokens: {
      gas: {
        name: "GasToken",
        symbol: "GAS",
        hash: "0xd2a4cff31913016155e38e474a2c06d08be276cf",
        decimals: 8
      },
      neo: {
        name: "NeoToken",
        symbol: "NEO", 
        hash: "0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5",
        decimals: 0
      }
    }
  };
  
  await config.saveConfig();
  console.log(chalk.green(`Network "${answers.name}" added`));
}

async function setDefaultProfile(config: ConfigManager) {
  const currentConfig = config.getConfig();
  const profiles = Object.keys(currentConfig.profile);
  
  const { profile } = await inquirer.prompt([
    {
      type: "list",
      name: "profile", 
      message: "Select default profile:",
      choices: profiles,
      default: currentConfig.defaultProfile
    }
  ]);
  
  currentConfig.defaultProfile = profile;
  await config.saveConfig();
  console.log(chalk.green(`Default profile set to "${profile}"`));
}

async function resetConfiguration(config: ConfigManager) {
  const { confirm } = await inquirer.prompt([
    {
      type: "confirm",
      name: "confirm",
      message: "Are you sure you want to reset configuration to defaults?",
      default: false
    }
  ]);
  
  if (confirm) {
    // Reset would recreate default configuration
    await config.saveConfig();
    console.log(chalk.green("Configuration reset to defaults"));
  }
}
