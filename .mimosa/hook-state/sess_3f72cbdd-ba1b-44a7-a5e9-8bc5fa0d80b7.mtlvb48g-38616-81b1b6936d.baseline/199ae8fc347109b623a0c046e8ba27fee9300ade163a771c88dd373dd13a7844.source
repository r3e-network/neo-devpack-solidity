import { isNeoAddress, NeoAccount } from "@neo-devpack-solidity/types";
import { HardhatPluginError } from "hardhat/plugins";
import Debug from "debug";
import { createAccountFromPrivateKey, decodePrivateKey, NeoSignerAccount } from "./account-primitives";

const debug = Debug("hardhat:neo-deployer:accounts");

function strip0x(value: string): string {
  return value.startsWith("0x") ? value.slice(2) : value;
}

export type ManagedAccount = NeoAccount & Partial<NeoSignerAccount>;

/**
 * Account manager for Neo deployments
 */
export class AccountManager {
  private accounts: ManagedAccount[];
  private defaultAccountIndex = 0;
  private readonly addressVersion: number;

  constructor(accountsConfig: (string | NeoAccount)[], addressVersion = 0x35) {
    this.accounts = [];
    this.addressVersion = addressVersion;
    for (const config of accountsConfig) {
      this.accounts.push(this.processAccountConfig(config));
    }
  }

  /**
   * Get account by address
   */
  getAccount(address: string): ManagedAccount | undefined {
    return this.accounts.find(account => account.address === address);
  }

  /**
   * Get account by index
   */
  getAccountByIndex(index: number): ManagedAccount | undefined {
    return this.accounts[index];
  }

  /**
   * Get default account
   */
  getDefaultAccount(): ManagedAccount | undefined {
    return this.accounts[this.defaultAccountIndex];
  }

  /**
   * Get all accounts
   */
  getAllAccounts(): ManagedAccount[] {
    return [...this.accounts];
  }

  /**
   * Set default account
   */
  setDefaultAccount(addressOrIndex: string | number): void {
    if (typeof addressOrIndex === "number") {
      if (addressOrIndex < 0 || addressOrIndex >= this.accounts.length) {
        throw new Error(`Account index ${addressOrIndex} out of range`);
      }
      this.defaultAccountIndex = addressOrIndex;
    } else {
      const index = this.accounts.findIndex(account => account.address === addressOrIndex);
      if (index === -1) {
        throw new Error(`Account with address ${addressOrIndex} not found`);
      }
      this.defaultAccountIndex = index;
    }
  }

  /**
   * Add account
   */
  addAccount(account: NeoAccount | string): void {
    const processedAccount = this.processAccountConfig(account);
    this.accounts.push(processedAccount);
  }

  /**
   * Remove account
   */
  removeAccount(address: string): boolean {
    const index = this.accounts.findIndex(account => account.address === address);
    if (index === -1) {
      return false;
    }
    
    this.accounts.splice(index, 1);
    
    // Adjust default account index if necessary
    if (this.defaultAccountIndex >= this.accounts.length) {
      this.defaultAccountIndex = Math.max(0, this.accounts.length - 1);
    }
    
    return true;
  }

  /**
   * Get account count
   */
  getAccountCount(): number {
    return this.accounts.length;
  }

  /**
   * Check if account exists
   */
  hasAccount(address: string): boolean {
    return this.accounts.some(account => account.address === address);
  }

  /**
   * Get accounts with private keys (for signing)
   */
  getSigningAccounts(): ManagedAccount[] {
    return this.accounts.filter(account => account.privateKey);
  }

  /**
   * Get default signer
   */
  getDefaultSigner(): ManagedAccount | undefined {
    const defaultAccount = this.getDefaultAccount();
    return defaultAccount?.privateKey ? defaultAccount : undefined;
  }

  /**
   * Validate account configuration
   */
  validateAccount(account: NeoAccount): boolean {
    if (!account.address || !account.scriptHash) {
      return false;
    }

    if (!isNeoAddress(account.address, this.addressVersion)) {
      return false;
    }

    const scriptHash = strip0x(account.scriptHash);
    if (!/^[0-9a-fA-F]{40}$/.test(scriptHash)) {
      return false;
    }

    if (account.privateKey) {
      try {
        const decoded = decodePrivateKey(account.privateKey);
        if (!/^[0-9a-f]{64}$/u.test(decoded)) {
          return false;
        }
      } catch {
        return false;
      }
    }

    return true;
  }

  /**
   * Import accounts from file
   */
  async importAccountsFromFile(filePath: string): Promise<void> {
    try {
      const fs = await import('fs/promises');
      const content = await fs.readFile(filePath, 'utf-8');
      const accountsData = JSON.parse(content);
      
      if (!Array.isArray(accountsData)) {
        throw new Error("Accounts file must contain an array of accounts");
      }
      
      for (const accountData of accountsData) {
        const account = this.processAccountConfig(accountData);
        this.accounts.push(account);
      }
      
      debug(`Imported ${accountsData.length} accounts from ${filePath}`);
    } catch (error) {
      throw new HardhatPluginError(
        "@neo-devpack-solidity/hardhat-neo-deployer",
        `Failed to import accounts: ${error}`
      );
    }
  }

  /**
   * Export accounts to file
   */
  async exportAccountsToFile(filePath: string, includePrivateKeys = false): Promise<void> {
    try {
      const fs = await import('fs/promises');
      
      const exportData = this.accounts.map(account => {
        const exported: any = {
          address: account.address,
          scriptHash: account.scriptHash,
          publicKey: account.publicKey,
          label: account.label,
          isMultiSig: account.isMultiSig
        };
        
        if (includePrivateKeys && account.privateKey) {
          exported.privateKey = account.privateKey;
        }
        
        if (account.multiSig) {
          exported.multiSig = account.multiSig;
        }
        
        return exported;
      });
      
      await fs.writeFile(filePath, JSON.stringify(exportData, null, 2));
      debug(`Exported ${this.accounts.length} accounts to ${filePath}`);
    } catch (error) {
      throw new HardhatPluginError(
        "@neo-devpack-solidity/hardhat-neo-deployer",
        `Failed to export accounts: ${error}`
      );
    }
  }

  // Private methods

  private processAccountConfig(config: string | NeoAccount): ManagedAccount {
    if (typeof config === "string") {
      // Assume it's a WIF or a private key.
      return this.createAccountFromSigningKey(config);
    } else {
      if (config.privateKey) {
        const derived = this.createAccountFromSigningKey(config.privateKey);

        if (config.address && config.address !== derived.address) {
          throw new Error(
            `Invalid account configuration: address does not match privateKey (expected ${derived.address}, got ${config.address})`
          );
        }

        const providedScriptHash = strip0x(config.scriptHash || "");
        if (providedScriptHash && providedScriptHash !== strip0x(derived.scriptHash)) {
          throw new Error(
            `Invalid account configuration: scriptHash does not match privateKey (expected ${derived.scriptHash}, got ${config.scriptHash})`
          );
        }

        const merged: NeoAccount = {
          ...derived,
          ...config,
          address: derived.address,
          scriptHash: strip0x(derived.scriptHash),
          publicKey: derived.publicKey,
        };

        if (!this.validateAccount(merged)) {
          throw new Error(`Invalid account configuration: ${JSON.stringify(config)}`);
        }
        return merged;
      }

      if (!this.validateAccount(config)) {
        throw new Error(`Invalid account configuration: ${JSON.stringify(config)}`);
      }
      return {
        ...config,
        scriptHash: strip0x(config.scriptHash),
      };
    }
  }

  private createAccountFromSigningKey(signingKey: string): ManagedAccount {
    const neoAccount = createAccountFromPrivateKey(signingKey, this.addressVersion);
    const nextIndex = this.accounts.length;
    return {
      address: neoAccount.address,
      scriptHash: neoAccount.scriptHash,
      privateKey: neoAccount.privateKey,
      publicKey: neoAccount.publicKey,
      contract: neoAccount.contract,
      label: `Account ${nextIndex + 1}`,
      isMultiSig: false,
    };
  }
}
