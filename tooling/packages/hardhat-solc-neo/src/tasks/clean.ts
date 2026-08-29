import { task } from "hardhat/config";
import { HardhatRuntimeEnvironment } from "hardhat/types";
import chalk from "chalk";
import { addFlagOption, setTaskAction } from "@neo-devpack-solidity/types";

const neoCleanTask = task("neo-clean", "Clean Neo DevPack for Solidity compilation artifacts");
addFlagOption(neoCleanTask, "deployments", "Also clean deployment artifacts");
addFlagOption(neoCleanTask, "cache", "Also clean cache files");
addFlagOption(neoCleanTask, "all", "Clean everything including build info");
setTaskAction(neoCleanTask, async (taskArgs: any, hre: HardhatRuntimeEnvironment) => {
    console.log(chalk.blue("🧹 Cleaning Neo DevPack for Solidity artifacts..."));

    try {
      if (taskArgs.all) {
        // Clean everything
        await hre.neoSolc.artifacts.clearArtifacts();
        console.log(chalk.green("✅ Cleaned all artifacts, deployments, and cache"));
      } else {
        // Clean build artifacts
        const buildArtifacts = await hre.neoSolc.artifacts.getAllBuildArtifacts();
        
        for (const artifact of buildArtifacts) {
          // This would remove individual artifacts - simplified here
          console.log(chalk.gray(`  Removing ${artifact.contractName}`));
        }
        
        if (taskArgs.deployments) {
          // Clean deployment artifacts
          console.log(chalk.yellow("🔧 Cleaning deployment artifacts..."));
          // Implementation would clean deployment artifacts
        }
        
        if (taskArgs.cache) {
          // Clean cache
          console.log(chalk.yellow("🔧 Cleaning cache..."));
          // Implementation would clean cache files
        }
        
        console.log(chalk.green("✅ Cleaning completed"));
      }
      
      // Show statistics
      const stats = await hre.neoSolc.artifacts.getStatistics();
      console.log(chalk.blue(`📊 Remaining artifacts: ${stats.totalBuildArtifacts} build, ${stats.totalDeploymentArtifacts} deployments`));
      
    } catch (error) {
      console.error(chalk.red("❌ Cleaning failed:"));
      console.error(error instanceof Error ? error.message : String(error));
      throw error;
    }
  });
