/**
 * Neo network configuration
 */
export interface NeoNetworkConfig {
  /** Network name */
  name: string;
  /** Neo RPC endpoint URLs */
  rpcUrls: string[];
  /** Network magic number */
  magic: number;
  /** Address version (for address generation) */
  addressVersion: number;
  /** Native token configurations */
  nativeTokens: {
    gas: NeoToken;
    neo: NeoToken;
  };
  /** Block explorer URLs */
  explorers?: {
    name: string;
    url: string;
    apiUrl?: string;
  }[];
  /** Faucet URLs for testnets */
  faucets?: string[];
  /** Whether this is a testnet */
  testnet?: boolean;
}

/**
 * Native token configuration
 */
export interface NeoToken {
  /** Token name */
  name: string;
  /** Token symbol */
  symbol: string;
  /** Token script hash */
  hash: string;
  /** Token decimals */
  decimals: number;
}

/**
 * Predefined Neo networks
 */
export const NeoNetworks = {
  mainnet: {
    name: 'Neo MainNet',
    rpcUrls: [
      'https://mainnet1.neo.coz.io:443',
      'https://mainnet2.neo.coz.io:443',
      'https://n3seed1.ngd.network:10332',
      'https://n3seed2.ngd.network:10332'
    ],
    magic: 860833102,
    addressVersion: 0x35,
    nativeTokens: {
      gas: {
        name: 'GasToken',
        symbol: 'GAS',
        hash: '0xd2a4cff31913016155e38e474a2c06d08be276cf',
        decimals: 8
      },
      neo: {
        name: 'NeoToken', 
        symbol: 'NEO',
        hash: '0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5',
        decimals: 0
      }
    },
    explorers: [
      {
        name: 'Neo Explorer',
        url: 'https://explorer.neo.org',
        apiUrl: 'https://explorer.neo.org/api'
      },
      {
        name: 'NeoTracker',
        url: 'https://neotracker.io',
        apiUrl: 'https://api.neotracker.io'
      }
    ]
  } as NeoNetworkConfig,

  testnet: {
    name: 'Neo TestNet',
    rpcUrls: [
      'https://testnet1.neo.coz.io:443',
      'https://testnet2.neo.coz.io:443',
      'https://n3seed3.ngd.network:20332',
      'https://n3seed4.ngd.network:20332'
    ],
    magic: 894710606,
    addressVersion: 0x35,
    nativeTokens: {
      gas: {
        name: 'GasToken',
        symbol: 'GAS',
        hash: '0xd2a4cff31913016155e38e474a2c06d08be276cf',
        decimals: 8
      },
      neo: {
        name: 'NeoToken',
        symbol: 'NEO', 
        hash: '0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5',
        decimals: 0
      }
    },
    explorers: [
      {
        name: 'Neo TestNet Explorer',
        url: 'https://testnet.explorer.neo.org',
        apiUrl: 'https://testnet.explorer.neo.org/api'
      }
    ],
    faucets: [
      'https://testnet.neo.org/faucet'
    ],
    testnet: true
  } as NeoNetworkConfig,

  private: {
    name: 'Neo Private',
    rpcUrls: ['http://localhost:40332'],
    magic: 12345,
    addressVersion: 0x35,
    nativeTokens: {
      gas: {
        name: 'GasToken',
        symbol: 'GAS', 
        hash: '0xd2a4cff31913016155e38e474a2c06d08be276cf',
        decimals: 8
      },
      neo: {
        name: 'NeoToken',
        symbol: 'NEO',
        hash: '0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5', 
        decimals: 0
      }
    },
    testnet: true
  } as NeoNetworkConfig
};

/**
 * Account configuration for deployment
 */
export interface NeoAccount {
  /** Account address */
  address: string;
  /** Private key (for signing) */
  privateKey?: string;
  /** Public key */
  publicKey?: string;
  /** Script hash */
  scriptHash: string;
  /** Label for the account */
  label?: string;
  /** Whether this account is a multisig account */
  isMultiSig?: boolean;
  /** Multisig configuration */
  multiSig?: {
    threshold: number;
    publicKeys: string[];
  };
}

/**
 * Deployment configuration
 */
export interface DeploymentConfig {
  /** Network to deploy to */
  network: string;
  /** Account to deploy from */
  from: string;
  /** Gas limit for deployment */
  gasLimit?: string;
  /** Gas price */
  gasPrice?: string;
  /** Constructor arguments */
  constructorArgs?: any[];
  /** Libraries to link before deployment */
  libraries?: {
    [libraryName: string]: string;
  };
  /** Verification settings */
  verify?: {
    enabled: boolean;
    apiKey?: string;
    constructorArgsParams?: any[];
  };
  /** Deployment metadata */
  metadata?: {
    tags?: string[];
    description?: string;
    version?: string;
  };
}

/**
 * Hardhat network configuration extension for Neo
 */
export interface HardhatNeoNetworkConfig extends NeoNetworkConfig {
  /** Hardhat-specific settings */
  hardhat?: {
    /** Enable forking from this network */
    forking?: {
      enabled: boolean;
      blockNumber?: number;
    };
    /** Gas settings */
    gas?: {
      limit: string;
      price: string;
    };
    /** Mining settings */
    mining?: {
      auto: boolean;
      interval?: number;
    };
  };
}

/**
 * Network user configuration (can be partial)
 */
export type NeoNetworkUserConfig = Partial<NeoNetworkConfig> & {
  name: string;
};