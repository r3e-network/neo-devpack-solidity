function loadOptionalPlugin(name) {
  try {
    require(name);
    return true;
  } catch (error) {
    const message = error && error.message ? error.message : String(error);
    console.warn(`[devpack] optional plugin '${name}' not loaded: ${message}`);
    return false;
  }
}

const hasNeoSolcPlugin = loadOptionalPlugin("@neo-devpack-solidity/hardhat-solc-neo");
const hasNeoDeployerPlugin = loadOptionalPlugin("@neo-devpack-solidity/hardhat-neo-deployer");

/**
 * Hardhat configuration for Neo N3 Devpack
 * Author: Jimmy <jimmy@r3e.network>
 */
const config = {
  solidity: {
    version: "0.8.19",
    settings: {
      optimizer: {
        enabled: true,
        runs: 200,
      },
    },
  },

  networks: {
    neo_local: {
      url: "http://localhost:20332",
      accounts: {
        mnemonic: "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
      },
      gas: "auto",
      gasPrice: "auto",
    },

    neo_testnet: {
      url: "http://seed1t5.neo.org:20332",
      accounts: process.env.PRIVATE_KEY ? [process.env.PRIVATE_KEY] : [],
      gas: "auto",
      gasPrice: "auto",
      timeout: 60000,
    },

    neo_mainnet: {
      url: "http://seed1.neo.org:10332",
      accounts: process.env.MAINNET_PRIVATE_KEY ? [process.env.MAINNET_PRIVATE_KEY] : [],
      gas: "auto",
      gasPrice: "auto",
      timeout: 120000,
    },
  },

  paths: {
    sources: "./contracts",
    tests: "./test",
    cache: "./cache",
    artifacts: "./artifacts",
  },

  mocha: {
    timeout: 40000,
  },
};

if (hasNeoSolcPlugin) {
  config.neoSolc = {
    solidity: {
      version: "0.8.19",
      settings: {
        optimizer: {
          enabled: true,
          runs: 200,
        },
        neo: {
          target: "3.0",
          optimization: 3,
          gasModel: "neo",
          devpack: {
            enabled: true,
            syscalls: "all",
            nativeContracts: "all",
            nepStandards: ["NEP-17", "NEP-11", "NEP-24"],
            libraries: ["Neo", "Storage", "Runtime"],
          },
        },
      },
    },
    neo: {
      rpcUrl: "http://seed1t5.neo.org:20332",
      privateKey: process.env.PRIVATE_KEY || "",
      addressVersion: 53,
      magic: 894710606,
      gasLimit: "100000000",
      gasPrice: "0",
    },
  };
}

if (hasNeoDeployerPlugin) {
  config.neoNetworks = {
    neo_testnet: {
      name: "Neo TestNet",
      rpcUrls: ["http://seed1t5.neo.org:20332"],
      magic: 894710606,
      addressVersion: 53,
      testnet: true,
      accounts: process.env.PRIVATE_KEY ? [process.env.PRIVATE_KEY] : [],
    },
    neo_mainnet: {
      name: "Neo MainNet",
      rpcUrls: ["http://seed1.neo.org:10332"],
      magic: 860833102,
      addressVersion: 53,
      accounts: process.env.MAINNET_PRIVATE_KEY ? [process.env.MAINNET_PRIVATE_KEY] : [],
    },
  };
}

module.exports = config;
