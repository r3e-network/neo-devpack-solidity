import { task } from "hardhat/config";
import { HardhatRuntimeEnvironment } from "hardhat/types";
import chalk from "chalk";
import {
  addFlagOption,
  addOptionalStringOption,
  addRequiredStringOption,
  getHardhatSelectedNetworkName,
  setTaskAction,
} from "@neo-devpack-solidity/types";

const neoDeployTask = task("neo-deploy", "Deploy contracts to Neo blockchain");
addRequiredStringOption(neoDeployTask, "contract", "Contract name to deploy");
addOptionalStringOption(neoDeployTask, "args", "Constructor arguments (JSON array)", "[]");
addOptionalStringOption(neoDeployTask, "from", "Account to deploy from");
addOptionalStringOption(neoDeployTask, "gasLimit", "Gas limit for deployment");
addFlagOption(neoDeployTask, "verify", "Verify contract after deployment");
setTaskAction(neoDeployTask, async (taskArgs: any, hre: HardhatRuntimeEnvironment) => {
    const { contract, args, from, gasLimit, verify } = taskArgs;
    const networkName = getHardhatSelectedNetworkName(hre);

    console.log(chalk.blue(`🚀 Deploying ${contract} to ${networkName}...`));

    try {
      // Parse constructor arguments
      let constructorArgs: any[] = [];
      if (args !== "[]") {
        try {
          constructorArgs = JSON.parse(args);
        } catch (error) {
          throw new Error(`Invalid constructor arguments JSON: ${error}`);
        }
      }

      // Set deployment account if specified
      if (from) {
        hre.neoDeploy.accounts.setDefaultAccount(from);
      }

      // Deploy contract
      const deployment = await hre.neoDeploy.deployer.deploy(contract, constructorArgs, {
        gasLimit,
        from
      });

      console.log(chalk.green("✅ Deployment successful!"));
      console.log(chalk.blue("📋 Deployment Details:"));
      console.log(`   Contract: ${contract}`);
      console.log(`   Address: ${deployment.address}`);
      console.log(`   Script Hash: ${deployment.scriptHash}`);
      console.log(`   Transaction Hash: ${deployment.transactionHash}`);
      console.log(`   Block Number: ${deployment.blockNumber}`);
      console.log(`   Gas Used: ${deployment.gasUsed.toString()}`);

      // Verify contract if requested
      if (verify) {
        console.log(chalk.yellow("🔍 Verifying contract..."));
        
        try {
          await hre.run("neo-verify", {
            contract,
            address: deployment.address,
            constructorArgs: JSON.stringify(constructorArgs)
          });
        } catch (verifyError) {
          console.log(chalk.yellow(`⚠️  Verification failed: ${verifyError}`));
          console.log(chalk.gray("You can verify manually later using: npx hardhat neo-verify"));
        }
      }

      // Show contract interaction example
      console.log(chalk.blue("\n💡 Contract Interaction:"));
      console.log(chalk.gray(`   const contract = await hre.neoDeploy.deployer.getContract("${contract}", "${deployment.address}");`));
      console.log(chalk.gray("   // Call contract methods..."));

      return deployment;
    } catch (error) {
      console.error(chalk.red("❌ Deployment failed:"));
      console.error(error instanceof Error ? error.message : String(error));
      throw error;
    }
  });

const neoDeployBatchTask = task("neo-deploy-batch", "Deploy multiple contracts in batch");
addRequiredStringOption(neoDeployBatchTask, "deployConfig", "Deployment configuration file path");
addFlagOption(neoDeployBatchTask, "verify", "Verify all contracts after deployment");
setTaskAction(neoDeployBatchTask, async (taskArgs: any, hre: HardhatRuntimeEnvironment) => {
    const { deployConfig, verify } = taskArgs;
    
    console.log(chalk.blue(`📦 Batch deploying from ${deployConfig}...`));

    try {
      // Read deployment configuration
      const fs = await import('fs/promises');
      const configContent = await fs.readFile(deployConfig, 'utf-8');
      const deploymentConfig = JSON.parse(configContent);

      if (!Array.isArray(deploymentConfig.contracts)) {
        throw new Error("Configuration must contain a 'contracts' array");
      }

      // Deploy contracts
      const deployments = await hre.neoDeploy.deployer.deployBatch(
        deploymentConfig.contracts
      );

      console.log(chalk.green(`✅ Successfully deployed ${deployments.length} contracts!`));

      // Print summary
      console.log(chalk.blue("\n📋 Deployment Summary:"));
      const deploymentsWithNames = deployments.map((deployment: any, index: number) => ({
        contractName: deploymentConfig.contracts[index]?.name ?? `Contract ${index + 1}`,
        deployment,
      }));

      for (const item of deploymentsWithNames) {
        console.log(`   ${item.contractName}: ${item.deployment.address}`);
      }

      // Verify all contracts if requested
      if (verify) {
        console.log(chalk.yellow("\n🔍 Verifying contracts..."));
        
        for (const item of deploymentsWithNames) {
          try {
            // Find the original config for constructor args
            const contractConfig = deploymentConfig.contracts.find(
              (c: any) => c.name === item.contractName
            );
            
            await hre.run("neo-verify", {
              contract: item.contractName,
              address: item.deployment.address,
              constructorArgs: JSON.stringify(contractConfig?.args || [])
            });
          } catch (verifyError) {
            console.log(chalk.yellow(`⚠️  Verification failed for ${item.contractName}: ${verifyError}`));
          }
        }
      }

      // Save deployment summary
      const networkName = getHardhatSelectedNetworkName(hre);
      const summaryPath = `deployments/${networkName}/summary.json`;
      const fs2 = await import('fs/promises');
      await fs2.mkdir(`deployments/${networkName}`, { recursive: true });
      
      const summary = {
        network: networkName,
        timestamp: new Date().toISOString(),
        deployments: deploymentsWithNames.map((item: any) => ({
          contract: item.contractName,
          address: item.deployment.address,
          scriptHash: item.deployment.scriptHash,
          transactionHash: item.deployment.transactionHash,
          blockNumber: item.deployment.blockNumber,
          gasUsed: item.deployment.gasUsed.toString()
        }))
      };
      
      await fs2.writeFile(summaryPath, JSON.stringify(summary, null, 2));
      console.log(chalk.blue(`📄 Deployment summary saved to ${summaryPath}`));

      return deployments;
    } catch (error) {
      console.error(chalk.red("❌ Batch deployment failed:"));
      console.error(error instanceof Error ? error.message : String(error));
      throw error;
    }
  });

const neoDeployEstimateTask = task("neo-deploy-estimate", "Estimate deployment gas costs");
addRequiredStringOption(neoDeployEstimateTask, "contract", "Contract name to estimate");
addOptionalStringOption(neoDeployEstimateTask, "args", "Constructor arguments (JSON array)", "[]");
setTaskAction(neoDeployEstimateTask, async (taskArgs: any, hre: HardhatRuntimeEnvironment) => {
    const { contract, args } = taskArgs;
    
    console.log(chalk.blue(`⛽ Estimating deployment gas for ${contract}...`));

    try {
      // Parse constructor arguments
      let constructorArgs: any[] = [];
      if (args !== "[]") {
        try {
          constructorArgs = JSON.parse(args);
        } catch (error) {
          throw new Error(`Invalid constructor arguments JSON: ${error}`);
        }
      }

      // Estimate gas
      const estimate = await hre.neoDeploy.deployer.estimateDeploymentGas(
        contract,
        constructorArgs
      );

      console.log(chalk.green("📊 Gas Estimation:"));
      console.log(`   System Fee: ${estimate.systemFee} GAS`);
      console.log(`   Network Fee: ${estimate.networkFee} GAS`);
      
      const totalGas = (BigInt(estimate.systemFee) + BigInt(estimate.networkFee)).toString();
      console.log(`   Total: ${totalGas} GAS`);

      // Convert to approximate USD (would need real price feed)
      const gasPrice = 50; // Mock $50 per GAS
      const estimatedUSD = (Number(totalGas) / 1e8 * gasPrice).toFixed(2);
      console.log(`   Estimated Cost: ~$${estimatedUSD} USD`);

    } catch (error) {
      console.error(chalk.red("❌ Gas estimation failed:"));
      console.error(error instanceof Error ? error.message : String(error));
      throw error;
    }
  });
