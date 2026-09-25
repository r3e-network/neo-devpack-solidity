import { HardhatRuntimeEnvironment } from "hardhat/types";
import { getHardhatSelectedNetworkName } from "@neo-devpack-solidity/types";

import { NeoDeployer } from "./deployer";
import { NeoRpcClient } from "./rpc-client";
import { AccountManager } from "./account-manager";

type NeoDeployRuntime = {
  deployer: NeoDeployer;
  rpc: NeoRpcClient;
  accounts: AccountManager;
};

function createMissingNeoDeployRuntime(message: string): NeoDeployRuntime {
  const throwMissing = (): never => {
    throw new Error(message);
  };

  const runtime = {} as NeoDeployRuntime;
  Object.defineProperties(runtime, {
    deployer: {
      enumerable: true,
      get: throwMissing,
    },
    rpc: {
      enumerable: true,
      get: throwMissing,
    },
    accounts: {
      enumerable: true,
      get: throwMissing,
    },
  });

  return runtime;
}

export function createNeoDeployRuntime(hre: HardhatRuntimeEnvironment): NeoDeployRuntime {
  const networkName = getHardhatSelectedNetworkName(hre);
  const networkConfig = hre.config.neoNetworks?.[networkName];

  if (!networkConfig) {
    return createMissingNeoDeployRuntime(
      `Neo network configuration not found for "${networkName}"`,
    );
  }

  const rpc = new NeoRpcClient(networkConfig);
  const accounts = new AccountManager(
    networkConfig.accounts,
    Number(networkConfig.addressVersion ?? 0x35),
  );
  // Prefer neo-solc build artifacts when available (from @neo-devpack-solidity/hardhat-solc-neo).
  const artifactProvider = (hre as any).neoSolc?.artifacts ?? hre.artifacts;
  const deployer = new NeoDeployer(
    rpc,
    accounts,
    artifactProvider,
    networkName,
    Number(networkConfig.addressVersion ?? 0x35),
  );

  return {
    deployer,
    rpc,
    accounts,
  };
}
