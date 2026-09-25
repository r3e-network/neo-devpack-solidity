import { task } from "hardhat/config";
import { HardhatRuntimeEnvironment } from "hardhat/types";
import chalk from "chalk";
import {
  addOptionalIntOption,
  addOptionalStringOption,
  addRequiredStringOption,
  getHardhatSelectedNetworkName,
  setTaskAction,
} from "@neo-devpack-solidity/types";

const neoDeployAndVerifyTask = task("neo-deploy-and-verify", "Deploy and verify contract in one step");
addRequiredStringOption(neoDeployAndVerifyTask, "contract", "Contract name to deploy");
addOptionalStringOption(neoDeployAndVerifyTask, "args", "Constructor arguments (JSON array)", "[]");
addOptionalStringOption(neoDeployAndVerifyTask, "from", "Account to deploy from");
addOptionalStringOption(neoDeployAndVerifyTask, "gasLimit", "Gas limit for deployment");
addOptionalIntOption(neoDeployAndVerifyTask, "waitBlocks", "Blocks to wait before verification", 5);
setTaskAction(neoDeployAndVerifyTask, async (taskArgs: any, hre: HardhatRuntimeEnvironment) => {
    const { contract, args, from, gasLimit, waitBlocks } = taskArgs;
    const networkName = getHardhatSelectedNetworkName(hre);

    console.log(chalk.blue(`🚀 Deploying and verifying ${contract} on ${networkName}...`));

    try {
      // Deploy contract first
      console.log(chalk.yellow("1️⃣ Deploying contract..."));
      
      const deployResult = await hre.run("neo-deploy", {
        contract,
        args,
        from,
        gasLimit,
        verify: false // We'll verify separately
      });

      // Wait for specified number of blocks
      const blocksToWait = Number(waitBlocks ?? 5);
      if (blocksToWait > 0) {
        console.log(chalk.yellow(`2️⃣ Waiting for ${blocksToWait} blocks...`));
        await waitForBlocks(hre, blocksToWait);
      }

      // Verify contract
      console.log(chalk.yellow("3️⃣ Verifying contract..."));
      
      const deploymentAddress: string | undefined = deployResult?.address;
      if (!deploymentAddress) {
        throw new Error("Deployment task did not return a contract address for verification");
      }

      await hre.run("neo-verify", {
        contract,
        address: deploymentAddress,
        constructorArgs: args
      });

      console.log(chalk.green("✅ Deploy and verify completed successfully!"));
      console.log(chalk.blue("🔗 Links:"));
      console.log(`   Contract: ${deploymentAddress}`);
      console.log(`   Explorer: https://explorer.neo.org/contract/${deploymentAddress}`);

      return deployResult;
    } catch (error) {
      console.error(chalk.red("❌ Deploy and verify failed:"));
      console.error(error instanceof Error ? error.message : String(error));
      throw error;
    }
  });

const neoRedeployTask = task("neo-redeploy", "Redeploy contract (useful for development)");
addRequiredStringOption(neoRedeployTask, "contract", "Contract name to redeploy");
addOptionalStringOption(neoRedeployTask, "args", "Constructor arguments (JSON array)", "[]");
addOptionalStringOption(neoRedeployTask, "from", "Account to deploy from");
setTaskAction(neoRedeployTask, async (taskArgs: any, hre: HardhatRuntimeEnvironment) => {
    const { contract, args, from } = taskArgs;
    const networkName = getHardhatSelectedNetworkName(hre);

    console.log(chalk.blue(`🔄 Redeploying ${contract} on ${networkName}...`));

    try {
      console.log(chalk.yellow("🔧 Recompiling contracts..."));
      await hre.run("neo-compile", { force: true, quiet: true });

      await hre.run("neo-deploy-and-verify", {
        contract,
        args,
        from
      });

      console.log(chalk.green("✅ Redeployment completed!"));

    } catch (error) {
      console.error(chalk.red("❌ Redeployment failed:"));
      console.error(error instanceof Error ? error.message : String(error));
      throw error;
    }
  });

const neoDeployUpgradeTask = task("neo-deploy-upgrade", "Deploy contract upgrade");
addRequiredStringOption(neoDeployUpgradeTask, "contract", "Contract name to upgrade");
addRequiredStringOption(neoDeployUpgradeTask, "proxy", "Deployed contract address to upgrade");
addOptionalStringOption(neoDeployUpgradeTask, "args", "Upgrade arguments (JSON array)", "[]");
addOptionalStringOption(neoDeployUpgradeTask, "from", "Account to deploy from");
setTaskAction(neoDeployUpgradeTask, async (taskArgs: any, hre: HardhatRuntimeEnvironment) => {
    const { contract, proxy, args, from } = taskArgs;
    
    console.log(chalk.blue(`🆙 Upgrading ${contract} at ${proxy}...`));

    try {
      // Parse upgrade arguments
      let upgradeArgs: any[] = [];
      if (args !== "[]") {
        try {
          upgradeArgs = JSON.parse(args);
        } catch (error) {
          throw new Error(`Invalid upgrade arguments JSON: ${error}`);
        }
      }

      // Set deployment account if specified
      if (from) {
        hre.neoDeploy.accounts.setDefaultAccount(from);
      }

      // Load build artifact for the new code
      const artifactsAny = (hre as any).neoSolc?.artifacts;
      if (typeof artifactsAny?.getBuildArtifact !== "function") {
        throw new Error(
          "Neo build artifacts not available. Install and use @neo-devpack-solidity/hardhat-solc-neo and run `npx hardhat neo-compile`."
        );
      }

      let buildArtifact = await artifactsAny.getBuildArtifact(contract);
      if (!buildArtifact) {
        console.log(chalk.yellow("🔧 No build artifact found, compiling..."));
        await hre.run("neo-compile", { force: true, quiet: true });
        buildArtifact = await artifactsAny.getBuildArtifact(contract);
      }
      if (!buildArtifact?.contract?.neo?.nef?.image) {
        throw new Error(`Build artifact for ${contract} is missing NEF image data`);
      }
      if (!buildArtifact?.contract?.neo?.manifest) {
        throw new Error(`Build artifact for ${contract} is missing manifest data`);
      }

      const manifestJson = JSON.stringify(buildArtifact.contract.neo.manifest);
      const nefHex = buildArtifact.contract.neo.nef.image;

      // Upgrade is performed by calling the deployed contract's own `update` entry point.
      // Contracts must implement an `update(nef, manifest, data)` method that calls
      // `ContractManagement.update(...)` and enforces authorization (e.g., checkWitness).
      console.log(chalk.yellow("1️⃣ Preparing upgrade transaction (calling update)..."));

      const target = await hre.neoDeploy.deployer.getContract(contract, proxy);
      const updateMethod = (target.methods as any).update;
      if (!updateMethod) {
        throw new Error(
          `Contract ABI for ${contract} does not expose an update method. ` +
            `Implement \`update(bytes nef, string manifest, any data)\` in your contract to enable upgrades.`
        );
      }

      const txResult = await updateMethod.invoke(nefHex, manifestJson, upgradeArgs);

      console.log(chalk.green("✅ Upgrade transaction broadcast"));
      console.log(chalk.blue("📋 Upgrade Details:"));
      console.log(`   Contract: ${proxy}`);
      console.log(`   Tx Hash: ${txResult.hash}`);

      const receipt = await txResult.wait();
      console.log(chalk.blue(`   VM State: ${receipt.neo.vmState}`));
      console.log(chalk.blue(`   Status: ${receipt.status}`));

      return { transactionHash: txResult.hash, receipt };

    } catch (error) {
      console.error(chalk.red("❌ Contract upgrade failed:"));
      console.error(error instanceof Error ? error.message : String(error));
      throw error;
    }
  });

/**
 * Wait for specified number of blocks
 */
async function waitForBlocks(hre: HardhatRuntimeEnvironment, blocks: number): Promise<void> {
  const startBlock = await hre.neoDeploy.rpc.getBlockCount();
  const targetBlock = startBlock + blocks;
  
  console.log(chalk.gray(`   Waiting for block ${targetBlock} (current: ${startBlock})...`));
  
  let currentBlock = await hre.neoDeploy.rpc.getBlockCount();
  while (currentBlock < targetBlock) {
    // Wait 15 seconds (average Neo block time)
    await new Promise(resolve => setTimeout(resolve, 15000));
    process.stdout.write(chalk.gray("."));
    currentBlock = await hre.neoDeploy.rpc.getBlockCount();
  }
  
  console.log(chalk.gray(" ✅"));
}
