import { task } from "hardhat/config";
import { HardhatRuntimeEnvironment } from "hardhat/types";
import chalk from "chalk";
import {
  VerificationResult,
  addOptionalStringOption,
  addRequiredStringOption,
  base64ToHex,
  getHardhatSelectedNetworkName,
  isNeoAddress,
  neoAddressToScriptHash,
  setTaskAction,
} from "@neo-devpack-solidity/types";

const neoVerifyTask = task("neo-verify", "Verify on-chain NEF/manifest matches local build artifact");
addRequiredStringOption(neoVerifyTask, "contract", "Contract name");
addRequiredStringOption(neoVerifyTask, "address", "Contract address");
addOptionalStringOption(neoVerifyTask, "neoNetwork", "Network name");
addOptionalStringOption(neoVerifyTask, "constructorArgs", "Constructor arguments (JSON)");
setTaskAction(neoVerifyTask, async (taskArgs: any, hre: HardhatRuntimeEnvironment) => {
    const { contract, address, neoNetwork, constructorArgs } = taskArgs;
    const networkName = getHardhatSelectedNetworkName(hre, neoNetwork);
    
    console.log(chalk.blue(`🔍 Verifying contract ${contract} at ${address} on ${networkName}...`));

    try {
      // Get deployment artifact
      const deployment = await hre.neoSolc.artifacts.getDeploymentArtifact(contract, networkName);
      if (!deployment) {
        throw new Error(`No deployment found for ${contract} on ${networkName}`);
      }

      // Get build artifact
      const buildArtifact = await hre.neoSolc.artifacts.getBuildArtifact(contract);
      if (!buildArtifact) {
        throw new Error(`No build artifact found for ${contract}`);
      }

      // Parse constructor arguments
      let args: any[] = [];
      if (constructorArgs) {
        try {
          args = JSON.parse(constructorArgs);
        } catch (error) {
          throw new Error(`Invalid constructor arguments JSON: ${error}`);
        }
      }

      // Perform verification
      if (!(hre as any).neoDeploy?.rpc) {
        throw new Error(
          "Neo RPC provider not available. Install and enable @neo-devpack-solidity/hardhat-neo-deployer (it provides hre.neoDeploy.rpc)."
        );
      }

      const result = await verifyContract((hre as any).neoDeploy.rpc, {
        contractName: contract,
        address,
        networkName,
        buildArtifact,
        constructorArgs: args
      });

      if (result.success) {
        console.log(chalk.green("✅ Contract verified successfully"));

        // Update deployment artifact with verification status
        deployment.verified = true;
        deployment.verification = {
          status: "verified"
        };
        await hre.neoSolc.artifacts.saveDeploymentArtifact(deployment);
      } else {
        console.log(chalk.red("❌ Contract verification failed"));
        console.log(chalk.red(result.message));
        
        // Update deployment artifact with failed verification
        deployment.verification = {
          status: "failed"
        };
        await hre.neoSolc.artifacts.saveDeploymentArtifact(deployment);
      }

    } catch (error) {
      console.error(chalk.red("❌ Verification failed:"));
      console.error(error instanceof Error ? error.message : String(error));
      throw error;
    }
  });

/**
 * Verify contract on Neo blockchain explorer
 */
async function verifyContract(
  rpc: { getContractState: (scriptHash: string) => Promise<any> },
  params: {
  contractName: string;
  address: string;
  networkName: string;
  buildArtifact: any;
  constructorArgs: any[];
}): Promise<VerificationResult> {
  const { address, buildArtifact } = params;

  try {
    // On-chain bytecode/manifest verification:
    // - Fetch `getcontractstate` from the configured Neo RPC
    // - Compare NEF script + checksum + manifest contents with local build artifact
    if (!rpc || typeof rpc.getContractState !== "function") {
      throw new Error("Invalid Neo RPC provider");
    }

    if (!buildArtifact?.contract?.neo?.nef?.script || !buildArtifact?.contract?.neo?.nef?.checksum) {
      throw new Error("Build artifact is missing Neo NEF (script/checksum)");
    }
    if (!buildArtifact?.contract?.neo?.manifest) {
      throw new Error("Build artifact is missing Neo manifest");
    }

    const scriptHash = toScriptHash(address);
    const state = await rpc.getContractState(scriptHash);

    const localScript = normalizeHex(buildArtifact.contract.neo.nef.script);
    const chainScript = decodeNefScript(state?.nef?.script);

    const localChecksum = parseChecksumU32(buildArtifact.contract.neo.nef.checksum);
    const chainChecksum = Number(state?.nef?.checksum ?? 0);

    const manifestOk = compareManifests(buildArtifact.contract.neo.manifest, state?.manifest);

    const matches =
      localScript === chainScript &&
      localChecksum === chainChecksum &&
      manifestOk;

    if (!matches) {
      return {
        success: false,
        message:
          "On-chain contract does not match local build artifact. " +
          "Re-check network, address, and ensure you compiled the same sources/settings.",
      };
    }

    return {
      success: true,
      message: "On-chain NEF + manifest match local build artifact",
    };

  } catch (error) {
    return {
      success: false,
      message: error instanceof Error ? error.message : String(error)
    };
  }
}

function normalizeHex(value: string): string {
  const trimmed = value.startsWith("0x") ? value.slice(2) : value;
  return trimmed.toLowerCase();
}

function decodeNefScript(value: unknown): string {
  const raw = String(value ?? "");
  try {
    return normalizeHex(base64ToHex(raw));
  } catch {
    return normalizeHex(raw);
  }
}

function parseChecksumU32(checksumHexLe: string): number {
  const normalized = normalizeHex(checksumHexLe);
  if (normalized.length !== 8) {
    throw new Error(`Expected NEF checksum as 8 hex chars, got ${normalized.length}`);
  }
  return Buffer.from(normalized, "hex").readUInt32LE(0);
}

function toScriptHash(addressOrHash: string): string {
  const value = addressOrHash.trim();
  if (isNeoAddress(value)) {
    return neoAddressToScriptHash(value);
  }
  if (/^0x[0-9a-fA-F]{40}$/.test(value) || /^[0-9a-fA-F]{40}$/.test(value)) {
    return value.startsWith("0x") ? value : "0x" + value;
  }
  throw new Error(`Invalid contract address or script hash: ${addressOrHash}`);
}

function normalizeManifestJson(value: any): any {
  if (Array.isArray(value)) {
    return value.map(normalizeManifestJson);
  }
  if (value && typeof value === "object") {
    const out: any = {};
    for (const [k, v] of Object.entries(value)) {
      out[String(k).toLowerCase()] = normalizeManifestJson(v);
    }

    if (Array.isArray(out.supportedstandards)) {
      out.supportedstandards = [...out.supportedstandards].map(String).sort();
    }
    if (Array.isArray(out.trusts)) {
      out.trusts = [...out.trusts].map(String).sort();
    }
    if (out.abi && typeof out.abi === "object") {
      if (Array.isArray(out.abi.methods)) {
        out.abi.methods = [...out.abi.methods].sort((a: any, b: any) =>
          String(a?.name ?? "").localeCompare(String(b?.name ?? ""))
        );
      }
      if (Array.isArray(out.abi.events)) {
        out.abi.events = [...out.abi.events].sort((a: any, b: any) =>
          String(a?.name ?? "").localeCompare(String(b?.name ?? ""))
        );
      }
    }
    if (Array.isArray(out.permissions)) {
      out.permissions = [...out.permissions].map((perm: any) => {
        const normalized = normalizeManifestJson(perm);
        if (Array.isArray(normalized.methods)) {
          normalized.methods = [...normalized.methods].map(String).sort();
        }
        return normalized;
      }).sort((a: any, b: any) => String(a?.contract ?? "").localeCompare(String(b?.contract ?? "")));
    }

    return out;
  }
  return value;
}

function compareManifests(localManifest: any, chainManifest: any): boolean {
  const normalizedLocal = normalizeManifestJson(localManifest);
  const normalizedChain = normalizeManifestJson(chainManifest);
  return JSON.stringify(normalizedLocal) === JSON.stringify(normalizedChain);
}

const neoVerifyAllTask = task("neo-verify-all", "Verify all deployed contracts match local artifacts");
addOptionalStringOption(neoVerifyAllTask, "neoNetwork", "Network name");
setTaskAction(neoVerifyAllTask, async (taskArgs: any, hre: HardhatRuntimeEnvironment) => {
    const networkName = getHardhatSelectedNetworkName(hre, taskArgs.neoNetwork);
    
    console.log(chalk.blue(`🔍 Verifying all contracts on ${networkName}...`));

    try {
      if (!(hre as any).neoDeploy?.rpc) {
        throw new Error(
          "Neo RPC provider not available. Install and enable @neo-devpack-solidity/hardhat-neo-deployer (it provides hre.neoDeploy.rpc)."
        );
      }

      const deployments = await hre.neoSolc.artifacts.getNetworkDeployments(networkName);
      
      if (deployments.length === 0) {
        console.log(chalk.yellow(`No deployments found on ${networkName}`));
        return;
      }

      console.log(chalk.blue(`Found ${deployments.length} deployments`));

      let verified = 0;
      let failed = 0;
      let skipped = 0;

      for (const deployment of deployments) {
        if (deployment.verified) {
          console.log(chalk.gray(`⏩ Skipping ${deployment.contractName} (already verified)`));
          skipped++;
          continue;
        }

        console.log(chalk.blue(`🔍 Verifying ${deployment.contractName}...`));

        try {
          const buildArtifact = await hre.neoSolc.artifacts.getBuildArtifact(deployment.contractName);
          if (!buildArtifact) {
            console.log(chalk.yellow(`⚠️  No build artifact for ${deployment.contractName}`));
            failed++;
            continue;
          }

          const result = await verifyContract((hre as any).neoDeploy.rpc, {
            contractName: deployment.contractName,
            address: deployment.address,
            networkName,
            buildArtifact,
            constructorArgs: deployment.constructorArgs
          });

          if (result.success) {
            console.log(chalk.green(`✅ ${deployment.contractName} verified`));
            deployment.verified = true;
            deployment.verification = {
              status: "verified"
            };
            await hre.neoSolc.artifacts.saveDeploymentArtifact(deployment);
            verified++;
          } else {
            console.log(chalk.red(`❌ ${deployment.contractName} failed: ${result.message}`));
            failed++;
          }
        } catch (error) {
          console.log(chalk.red(`❌ ${deployment.contractName} error: ${error}`));
          failed++;
        }
      }

      console.log(chalk.blue("\n📊 Verification Summary:"));
      console.log(chalk.green(`✅ Verified: ${verified}`));
      console.log(chalk.red(`❌ Failed: ${failed}`));
      console.log(chalk.gray(`⏩ Skipped: ${skipped}`));

    } catch (error) {
      console.error(chalk.red("❌ Batch verification failed:"));
      console.error(error instanceof Error ? error.message : String(error));
      throw error;
    }
  });
