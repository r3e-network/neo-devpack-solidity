import {
  BuildArtifact,
  ContractDeployment,
  DeploymentConfig,
  DeploymentArtifact,
  NeoContract,
  TransactionResult,
  getNeoAddressVersion,
  isNeoAddress,
  neoAddressToScriptHash,
  neoScriptHashToAddress,
} from "@neo-devpack-solidity/types";
import { HardhatPluginError } from "hardhat/plugins";
import chalk from "chalk";
import Debug from "debug";
import { createHash } from "crypto";

import { NeoRpcClient } from "./rpc-client";
import { AccountManager } from "./account-manager";
import { createAccountFromPrivateKey, NeoSignerAccount } from "./account-primitives";
import { base64ScriptToHex, decimalToIntegerString, normalizeByteArrayInput } from "./neo-primitives";
import { createContractCallScript } from "./script-builder";
import {
  createNeoSigner,
  createNeoTransaction,
  createNeoWitness,
  NeoTransaction,
  signTransaction,
  WitnessScope,
} from "./transaction-primitives";

const debug = Debug("hardhat:neo-deployer:deployer");

const CONTRACT_MANAGEMENT_HASH = "fffdc93764dbaddd97c48f252a53ea4643faa3fd";
const DEFAULT_VALID_UNTIL_BLOCK_INCREMENT = 200;
const DUMMY_SIGNATURE_INVOCATION_SCRIPT = "0c40" + "00".repeat(64);

function strip0x(value: string): string {
  return value.startsWith("0x") ? value.slice(2) : value;
}

function parseU32LEFromHex(checksumHexLe: string): number {
  const normalized = strip0x(checksumHexLe);
  if (normalized.length !== 8) {
    throw new Error(`expected 4-byte checksum hex (8 chars), got ${normalized.length}`);
  }
  return Buffer.from(normalized, "hex").readUInt32LE(0);
}

function hash160(data: Buffer): Buffer {
  const sha = createHash("sha256").update(data).digest();
  return createHash("ripemd160").update(sha).digest();
}

function emitPushBytes(script: number[], data: Buffer): void {
  if (data.length <= 0xff) {
    script.push(0x0c);
    script.push(data.length);
  } else if (data.length <= 0xffff) {
    script.push(0x0d);
    const len = Buffer.allocUnsafe(2);
    len.writeUInt16LE(data.length, 0);
    script.push(...len);
  } else {
    script.push(0x0e);
    const len = Buffer.allocUnsafe(4);
    len.writeUInt32LE(data.length, 0);
    script.push(...len);
  }
  script.push(...data);
}

function emitPushU32(script: number[], value: number): void {
  if (value === 0) {
    script.push(0x10);
    return;
  }
  if (value >= 1 && value <= 16) {
    script.push(0x10 + value);
    return;
  }
  if (value <= 0x7f) {
    script.push(0x00);
    script.push(value);
    return;
  }
  if (value <= 0x7fff) {
    script.push(0x01);
    const buf = Buffer.allocUnsafe(2);
    buf.writeInt16LE(value, 0);
    script.push(...buf);
    return;
  }
  if (value <= 0x7fffffff) {
    script.push(0x02);
    const buf = Buffer.allocUnsafe(4);
    buf.writeInt32LE(value, 0);
    script.push(...buf);
    return;
  }
  script.push(0x03);
  const buf = Buffer.allocUnsafe(8);
  buf.writeBigInt64LE(BigInt(value), 0);
  script.push(...buf);
}

function computeDeployedContractHashHexBE(
  senderScriptHashHexBe: string,
  nefChecksumHexLe: string,
  manifestName: string
): string {
  const senderBe = Buffer.from(strip0x(senderScriptHashHexBe), "hex");
  if (senderBe.length !== 20) {
    throw new Error(`sender script hash must be 20 bytes (got ${senderBe.length})`);
  }
  const senderLe = Buffer.from(senderBe).reverse();
  const checksum = parseU32LEFromHex(nefChecksumHexLe);

  const script: number[] = [];
  script.push(0x38); // ABORT
  emitPushBytes(script, senderLe);
  emitPushU32(script, checksum);
  emitPushBytes(script, Buffer.from(manifestName, "utf8"));

  const outLe = hash160(Buffer.from(script));
  return Buffer.from(outLe).reverse().toString("hex");
}

function parseRpcIntegerLike(value: unknown): string {
  if (typeof value === "number") {
    return Math.trunc(value).toString();
  }
  if (typeof value === "string") {
    // neo-go returns integer strings; some nodes may return fixed-8 decimals.
    if (value.includes(".")) {
      return decimalToIntegerString(value, 8);
    }
    return value;
  }
  return "0";
}

function normalizeVmState(value: unknown): "HALT" | "FAULT" {
  const raw = String(value ?? "").toUpperCase();
  return raw.includes("HALT") ? "HALT" : "FAULT";
}

type ContractParamLike = any;
type SignerAccount = NeoSignerAccount;

function witnessScopeToName(scope: number): string {
  const match = Object.entries(WitnessScope).find(([, value]) => value === scope);
  return match?.[0] ?? "Global";
}

function toNeoTransaction(transaction: NeoTransaction, hash: string): any {
  const exported = transaction.export();
  return {
    hash,
    version: exported.version,
    nonce: exported.nonce,
    systemFee: BigInt(exported.systemFee),
    networkFee: BigInt(exported.networkFee),
    validUntilBlock: exported.validUntilBlock,
    signers: (exported.signers ?? []).map((signer: any) => ({
      account: "0x" + strip0x(String(signer.account ?? "")),
      scopes: witnessScopeToName(Number(signer.scopes ?? 0)),
      allowedContracts: signer.allowedContracts,
      allowedGroups: signer.allowedGroups,
      rules: signer.rules,
    })),
    attributes: (exported.attributes ?? []).map((attribute: any) => ({
      type: String(attribute.type ?? ""),
      data: attribute.data,
    })),
    script: "0x" + strip0x(String(exported.script ?? "")),
    witnesses: (exported.witnesses ?? []).map((witness: any) => ({
      invocationScript: String(witness.invocationScript ?? ""),
      verificationScript: String(witness.verificationScript ?? ""),
    })),
  };
}

/**
 * Neo contract deployer
 */
export class NeoDeployer {
  private rpc: NeoRpcClient;
  private accounts: AccountManager;
  private artifacts: any;
  private networkName: string;
  private addressVersion: number;

  constructor(
    rpc: NeoRpcClient,
    accounts: AccountManager,
    artifacts: any,
    networkName = "default",
    addressVersion = 0x35
  ) {
    this.rpc = rpc;
    this.accounts = accounts;
    this.artifacts = artifacts;
    this.networkName = networkName;
    this.addressVersion = addressVersion;
  }

  /**
   * Deploy a contract to Neo blockchain
   */
  async deploy(
    contractName: string,
    constructorArgs: any[] = [],
    config?: Partial<DeploymentConfig>
  ): Promise<ContractDeployment> {
    debug(`Deploying contract ${contractName}`);

    try {
      const artifact = await this.getBuildArtifact(contractName);
      const deployConfig = this.getDeploymentConfig(config);

      const deployment = await this.executeDeployment(artifact, constructorArgs, deployConfig);
      await this.saveDeploymentArtifact(artifact, deployment, constructorArgs, deployConfig);

      debug(`Contract ${contractName} deployed at ${deployment.address}`);
      return deployment;
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      throw new HardhatPluginError(
        "@neo-devpack-solidity/hardhat-neo-deployer",
        `Deployment failed: ${message}`
      );
    }
  }

  /**
   * Deploy multiple contracts
   */
  async deployBatch(
    deployments: Array<{
      name: string;
      args?: any[];
      config?: Partial<DeploymentConfig>;
    }>
  ): Promise<ContractDeployment[]> {
    const results: ContractDeployment[] = [];

    console.log(chalk.blue(`📦 Deploying ${deployments.length} contracts...`));

    for (const deployment of deployments) {
      console.log(chalk.yellow(`🚀 Deploying ${deployment.name}...`));
      const result = await this.deploy(deployment.name, deployment.args, deployment.config);
      results.push(result);
      console.log(chalk.green(`✅ ${deployment.name} deployed at ${result.address}`));
    }

    return results;
  }

  /**
   * Get deployed contract instance
   */
  async getContract(contractName: string, address: string): Promise<NeoContract> {
    debug(`Getting contract instance for ${contractName} at ${address}`);

    const artifact = await this.getBuildArtifact(contractName);
    return this.createContractInstance(artifact, address);
  }

  /**
   * Estimate deployment fees (system + network)
   */
  async estimateDeploymentGas(
    contractName: string,
    constructorArgs: any[] = []
  ): Promise<{ systemFee: string; networkFee: string }> {
    debug(`Estimating deployment fees for ${contractName}`);

    const artifact = await this.getBuildArtifact(contractName);
    const deployConfig = this.getDeploymentConfig({});
    const signer = this.resolveSigner(deployConfig.from);

    const { systemFee, networkFee } = await this.buildAndEstimateTransaction({
      scriptHex: this.buildDeployScriptHex(artifact, constructorArgs),
      signer,
    });

    return { systemFee, networkFee };
  }

  private async getBuildArtifact(contractName: string): Promise<BuildArtifact> {
    const artifactsAny = this.artifacts as any;

    if (typeof artifactsAny?.getBuildArtifact === "function") {
      const artifact = await artifactsAny.getBuildArtifact(contractName);
      if (!artifact) {
        throw new Error(`Build artifact not found for ${contractName}`);
      }
      return artifact as BuildArtifact;
    }

    if (typeof artifactsAny?.readArtifact === "function") {
      const hardhatArtifact = await artifactsAny.readArtifact(contractName);
      if (hardhatArtifact?.contract?.neo) {
        return hardhatArtifact as BuildArtifact;
      }
    }

    throw new Error(
      `Neo build artifact not found for ${contractName}. ` +
        `Run \`npx hardhat neo-compile\` (from @neo-devpack-solidity/hardhat-solc-neo) before deploying.`
    );
  }

  private createContractInstance(artifact: BuildArtifact, address: string): NeoContract {
    const methods: { [key: string]: any } = {};
    const events: { [key: string]: any } = {};

    const scriptHash = this.addressToScriptHash(address);

    const createMethodHandle = (method: any, operation: string) => ({
      name: method.name,
      parameters: method.parameters,
      returnType: method.returntype,
      safe: method.safe,

      call: async (...args: any[]) => {
        const encodedArgs = this.encodeNeoArgs(method.parameters, args);
        const script = createContractCallScript({
          scriptHash: strip0x(scriptHash),
          operation,
          args: encodedArgs,
        });

        const defaultAccount = this.accounts.getDefaultAccount();
        if (defaultAccount?.scriptHash) {
          const signer = createNeoSigner({
            account: defaultAccount.scriptHash,
            scopes: "CalledByEntry",
          });
          return this.rpc.invokeScript(script, [signer.toJson()]);
        }

        return this.rpc.invokeScript(script);
      },

      invoke: async (...args: any[]) => {
        const encodedArgs = this.encodeNeoArgs(method.parameters, args);
        const script = createContractCallScript({
          scriptHash: strip0x(scriptHash),
          operation,
          args: encodedArgs,
        });

        const signer = this.resolveSigner(this.accounts.getDefaultAccount()?.address || "");
        return this.executeInvocation(script, signer, address);
      },

      estimateGas: async (...args: any[]) => {
        const encodedArgs = this.encodeNeoArgs(method.parameters, args);
        const script = createContractCallScript({
          scriptHash: strip0x(scriptHash),
          operation,
          args: encodedArgs,
        });

        const signer = this.resolveSigner(this.accounts.getDefaultAccount()?.address || "");
        const estimate = await this.buildAndEstimateTransaction({
          scriptHex: script,
          signer,
        });

        return BigInt(estimate.systemFee) + BigInt(estimate.networkFee);
      },
    });

    for (const method of artifact.contract.neo.manifest.abi.methods) {
      methods[method.name] = createMethodHandle(method, method.name);
    }

    const methodMap = artifact.contract.neo.methodMap ?? {};
    if (methodMap && typeof methodMap === "object") {
      const manifestMethodsByNeoName = new Map<string, any>();
      for (const method of artifact.contract.neo.manifest.abi.methods) {
        manifestMethodsByNeoName.set(method.name, method);
      }

      const overloadCounts = new Map<string, number>();
      for (const signature of Object.keys(methodMap)) {
        const baseName = signature.slice(0, signature.indexOf("("));
        overloadCounts.set(baseName, (overloadCounts.get(baseName) ?? 0) + 1);
      }

      for (const [signature, neoName] of Object.entries(methodMap)) {
        const method = manifestMethodsByNeoName.get(neoName);
        if (!method) {
          continue;
        }
        methods[signature] = createMethodHandle(method, neoName);
      }

      for (const [baseName, count] of overloadCounts.entries()) {
        if (count > 1) {
          methods[baseName] = {
            name: baseName,
            parameters: [],
            returnType: "Any",
            safe: false,
            call: async () => {
              throw new Error(
                `Method ${baseName} is overloaded; use a full signature such as ${Object.keys(methodMap).find(sig => sig.startsWith(baseName + "("))}`
              );
            },
            invoke: async () => {
              throw new Error(
                `Method ${baseName} is overloaded; use a full signature such as ${Object.keys(methodMap).find(sig => sig.startsWith(baseName + "("))}`
              );
            },
            estimateGas: async () => {
              throw new Error(
                `Method ${baseName} is overloaded; use a full signature such as ${Object.keys(methodMap).find(sig => sig.startsWith(baseName + "("))}`
              );
            },
          };
        }
      }
    }

    for (const event of artifact.contract.neo.manifest.abi.events) {
      events[event.name] = {
        name: event.name,
        parameters: event.parameters,
        signature: `${event.name}(${event.parameters.map((p: any) => p.type).join(",")})`,
        topicHash: "0x" + createHash("sha256").update(event.name).digest("hex"),
      };
    }

    return {
      address,
      scriptHash,
      abi: artifact.contract.abi,
      manifest: artifact.contract.neo.manifest,
      methods,
      events,
      provider: this.rpc,
      signer: this.accounts.getDefaultSigner(),
    };
  }

  private getDeploymentConfig(config?: Partial<DeploymentConfig>): DeploymentConfig {
    const defaultSigner = this.accounts.getDefaultAccount();

    return {
      network: this.networkName,
      from: defaultSigner?.address || "",
      gasLimit: "0",
      gasPrice: "0",
      constructorArgs: [],
      ...config,
    };
  }

  private resolveSigner(fromAddress?: string): SignerAccount {
    const from = fromAddress?.trim();
    const explicit = from ? this.accounts.getAccount(from) : undefined;
    const defaultSigner = this.accounts.getDefaultSigner();
    const signer = explicit?.privateKey ? explicit : defaultSigner;

    if (!signer?.privateKey) {
      throw new Error(
        `No signing account available. Configure a private key/WIF under neoNetworks.<network>.accounts or pass --from.`
      );
    }

    return createAccountFromPrivateKey(signer.privateKey, this.addressVersion, signer.label);
  }

  private buildDeployScriptHex(artifact: BuildArtifact, constructorArgs: any[]): string {
    const manifestJson = JSON.stringify(artifact.contract.neo.manifest);
    const nefImageHex = strip0x(artifact.contract.neo.nef.image);

    const nefBytes = normalizeByteArrayInput(nefImageHex);
    const encodedCtorArgs = this.encodeConstructorArgs(artifact, constructorArgs);

    return createContractCallScript({
      scriptHash: CONTRACT_MANAGEMENT_HASH,
      operation: "deploy",
      args: [
        { type: "ByteArray", value: nefBytes },
        { type: "String", value: manifestJson },
        { type: "Array", value: encodedCtorArgs },
      ],
    });
  }

  private async buildAndEstimateTransaction(input: {
    scriptHex: string;
    signer: SignerAccount;
  }): Promise<{
    systemFee: string;
    networkFee: string;
    validUntilBlock: number;
    simulation: any;
  }> {
    const height = await this.rpc.getBlockCount();
    const version = await this.rpc.getVersion();
    const maxValidUntilBlockIncrement =
      typeof version?.protocol?.maxvaliduntilblockincrement === "number"
        ? version.protocol.maxvaliduntilblockincrement
        : 5760;

    const validUntilBlock =
      height + Math.min(DEFAULT_VALID_UNTIL_BLOCK_INCREMENT, maxValidUntilBlockIncrement);

    const signer = createNeoSigner({
      account: input.signer.scriptHash,
      scopes: "CalledByEntry",
    });

    const transaction = createNeoTransaction({
      scriptHex: input.scriptHex,
      signers: [signer],
      validUntilBlock,
      systemFee: 0,
      networkFee: 0,
    });

    const verificationScript = base64ScriptToHex(input.signer.contract.script);
    transaction.addWitness(createNeoWitness({
      invocationScript: DUMMY_SIGNATURE_INVOCATION_SCRIPT,
      verificationScript,
    }));

    const networkFee = parseRpcIntegerLike(
      await this.rpc.calculateNetworkFee(transaction.serialize(true))
    );
    const invoke = await this.rpc.invokeScript(input.scriptHex, [signer.toJson()]);
    const invokeStateRaw = invoke?.state ?? invoke?.vmstate ?? invoke?.vmState;
    const vmState = normalizeVmState(invokeStateRaw);
    if (vmState !== "HALT") {
      const exception = invoke?.exception ? `: ${invoke.exception}` : "";
      throw new Error(`Invocation simulation failed (${String(invokeStateRaw ?? "FAULT")})${exception}`);
    }

    const systemFee = parseRpcIntegerLike(invoke?.gasconsumed ?? invoke?.gasConsumed);

    return { systemFee, networkFee, validUntilBlock, simulation: invoke };
  }

  private async executeDeployment(
    artifact: BuildArtifact,
    constructorArgs: any[],
    config: DeploymentConfig
  ): Promise<ContractDeployment> {
    if (!artifact.contract?.neo?.nef?.image || !artifact.contract?.neo?.nef?.checksum) {
      throw new Error(`Artifact ${artifact.contractName} is missing Neo NEF data (image/checksum)`);
    }
    if (!artifact.contract?.neo?.manifest?.name) {
      throw new Error(`Artifact ${artifact.contractName} is missing Neo manifest data (name)`);
    }

    const signerAccount = this.resolveSigner(config.from);
    const scriptHex = this.buildDeployScriptHex(artifact, constructorArgs);

    const estimate = await this.buildAndEstimateTransaction({
      scriptHex,
      signer: signerAccount,
    });

    const transaction = createNeoTransaction({
      scriptHex,
      signers: [
        createNeoSigner({
          account: signerAccount.scriptHash,
          scopes: "CalledByEntry",
        }),
      ],
      validUntilBlock: estimate.validUntilBlock,
      systemFee: decimalToIntegerString(estimate.systemFee, 0),
      networkFee: decimalToIntegerString(estimate.networkFee, 0),
    });

    const verificationScript = base64ScriptToHex(signerAccount.contract.script);
    transaction.addWitness(createNeoWitness({
      invocationScript: DUMMY_SIGNATURE_INVOCATION_SCRIPT,
      verificationScript,
    }));

    signTransaction(transaction, signerAccount, this.rpc.magic);

    const txHex = transaction.serialize(true);
    const send = await this.rpc.sendRawTransaction(txHex);

    const contractHashHexBe = computeDeployedContractHashHexBE(
      signerAccount.scriptHash,
      artifact.contract.neo.nef.checksum,
      artifact.contract.neo.manifest.name
    );
    const contractAddress = neoScriptHashToAddress(contractHashHexBe, this.addressVersion);

    const { blockNumber, receipt } = await this.waitForReceipt(
      send.hash,
      signerAccount.address,
      estimate,
      contractAddress
    );

    if (receipt?.neo?.vmState !== "HALT") {
      const exception = receipt?.neo?.exception ? `: ${receipt.neo.exception}` : "";
      const error: any = new Error(`Deployment transaction FAULT${exception} (tx=${send.hash})`);
      error.receipt = receipt;
      throw error;
    }

    const gasUsed = BigInt(estimate.systemFee) + BigInt(estimate.networkFee);

    return {
      address: contractAddress,
      scriptHash: "0x" + contractHashHexBe,
      transactionHash: send.hash,
      blockNumber,
      gasUsed,
      receipt,
      contract: this.createContractInstance(artifact, contractAddress),
    };
  }

  private async waitForReceipt(
    txHash: string,
    from: string,
    estimate: { systemFee: string; networkFee: string; simulation?: any },
    contractAddress = ""
  ): Promise<{ blockNumber: number; receipt: any }> {
    const deadline = Date.now() + 120_000;
    let blockNumber = 0;
    let hasHeight = false;
    // neo-go returns RPC errors until transaction is persisted.
    while (Date.now() < deadline) {
      try {
        blockNumber = await this.rpc.getTransactionHeight(txHash);
        hasHeight = true;
        break;
      } catch {
        await new Promise((resolve) => setTimeout(resolve, 1500));
      }
    }

    if (!hasHeight) {
      throw new Error(`Timed out waiting for transaction to be persisted (tx=${txHash})`);
    }

    let appLog: any = null;
    try {
      appLog = await this.rpc.getApplicationLog(txHash);
    } catch {
      appLog = null;
    }

    const execution = appLog?.executions?.[0];
    const fallback = estimate.simulation ?? {};
    const vmStateRaw = execution?.vmstate ?? execution?.vmState ?? fallback?.state ?? fallback?.vmstate ?? fallback?.vmState;
    const vmState = normalizeVmState(vmStateRaw);

	    const systemFee = BigInt(
	      parseRpcIntegerLike(
	        execution?.gasconsumed ??
	          execution?.gasConsumed ??
	          fallback?.gasconsumed ??
	          fallback?.gasConsumed ??
	          estimate.systemFee
	      )
	    );
	    const networkFee = BigInt(estimate.networkFee);

    const receipt = {
      transactionHash: txHash,
      blockNumber,
      blockHash: "",
      transactionIndex: 0,
      from,
      contractAddress,
	      gasUsed: systemFee + networkFee,
	      effectiveGasPrice: 0n,
      status: vmState === "HALT" ? "success" : "failed",
      events: [],
      neo: {
        vmState,
        stack: execution?.stack ?? fallback?.stack ?? [],
        notifications: execution?.notifications ?? fallback?.notifications ?? [],
        systemFee,
        networkFee,
        exception: execution?.exception ?? fallback?.exception,
        applicationLogUnavailable: !execution,
      },
    };

    return { blockNumber, receipt };
  }

  private async executeInvocation(
    scriptHex: string,
    signerAccount: SignerAccount,
    contractAddress: string
  ): Promise<TransactionResult> {
    const estimate = await this.buildAndEstimateTransaction({
      scriptHex,
      signer: signerAccount,
    });

    const signer = createNeoSigner({
      account: signerAccount.scriptHash,
      scopes: "CalledByEntry",
    });

    const transaction = createNeoTransaction({
      scriptHex,
      signers: [signer],
      validUntilBlock: estimate.validUntilBlock,
      systemFee: decimalToIntegerString(estimate.systemFee, 0),
      networkFee: decimalToIntegerString(estimate.networkFee, 0),
    });

    const verificationScript = base64ScriptToHex(signerAccount.contract.script);
    transaction.addWitness(createNeoWitness({
      invocationScript: DUMMY_SIGNATURE_INVOCATION_SCRIPT,
      verificationScript,
    }));

    signTransaction(transaction, signerAccount, this.rpc.magic);
    const send = await this.rpc.sendRawTransaction(transaction.serialize(true));
    const txHash = send.hash;

    return {
      hash: txHash,
      transaction: toNeoTransaction(transaction, txHash),
      wait: async (confirmations = 1) => {
        const { blockNumber, receipt } = await this.waitForReceipt(
          txHash,
          signerAccount.address,
          estimate,
          contractAddress
        );

        if (confirmations > 1 && blockNumber > 0) {
          const targetHeight = blockNumber + confirmations;
          const deadline = Date.now() + 180_000;
          while (Date.now() < deadline) {
            const current = await this.rpc.getBlockCount();
            if (current >= targetHeight) break;
            await new Promise((resolve) => setTimeout(resolve, 1500));
          }
        }

        if (receipt?.neo?.vmState !== "HALT") {
          const exception = receipt?.neo?.exception ? `: ${receipt.neo.exception}` : "";
          const error: any = new Error(`Invocation transaction FAULT${exception} (tx=${txHash})`);
          error.receipt = receipt;
          throw error;
        }

        return receipt;
      },
    };
  }

  private async saveDeploymentArtifact(
    buildArtifact: BuildArtifact,
    deployment: ContractDeployment,
    constructorArgs: any[],
    config: DeploymentConfig
  ): Promise<void> {
    const artifactsAny = this.artifacts as any;
    if (typeof artifactsAny?.saveDeploymentArtifact !== "function") {
      return;
    }

    const artifact: DeploymentArtifact = {
      contractName: buildArtifact.contractName,
      networkName: config.network || this.networkName,
      address: deployment.address,
      scriptHash: deployment.scriptHash,
      transactionHash: deployment.transactionHash,
      blockNumber: deployment.blockNumber,
      deployedAt: new Date().toISOString(),
      deployedBy: deployment.receipt?.from || config.from,
      constructorArgs,
      buildArtifact: buildArtifact.buildInfo || `${buildArtifact.sourceName}:${buildArtifact.contractName}`,
      verified: false,
      verification: {
        status: "pending",
      },
    };

    await artifactsAny.saveDeploymentArtifact(artifact);
  }

  private addressToScriptHash(address: string): string {
    if (isNeoAddress(address)) {
      const version = getNeoAddressVersion(address);
      if (typeof version === "number" && version !== this.addressVersion) {
        throw new Error(
          `Address version mismatch for network: expected 0x${this.addressVersion.toString(16)}, got 0x${version.toString(16)}`
        );
      }
      return neoAddressToScriptHash(address);
    }
    return address.startsWith("0x") ? address : "0x" + address;
  }

  private encodeConstructorArgs(artifact: BuildArtifact, constructorArgs: any[]): ContractParamLike[] {
    const abi = Array.isArray(artifact.contract.abi) ? artifact.contract.abi : [];
    const ctor = abi.find((item: any) => item?.type === "constructor");
    const inputs = Array.isArray(ctor?.inputs) ? ctor.inputs : [];

    if (inputs.length !== constructorArgs.length) {
      throw new Error(
        `Constructor argument count mismatch: expected ${inputs.length}, got ${constructorArgs.length}`
      );
    }

    return inputs.map((input: any, index: number) =>
      this.encodeSolidityTypeToNeoParam(String(input.type), constructorArgs[index])
    );
  }

  private encodeNeoArgs(parameters: any[], args: any[]): ContractParamLike[] {
    if (!Array.isArray(parameters) || parameters.length !== args.length) {
      return args;
    }

    return parameters.map((p: any, index: number) =>
      this.encodeNeoTypeToParam(String(p.type), args[index])
    );
  }

  private encodeNeoTypeToParam(neoType: string, value: any): ContractParamLike {
    const upper = neoType.toUpperCase();
    if (upper === "INTEGER") {
      if (typeof value === "bigint") return { type: "Integer", value: value.toString() };
      if (typeof value === "number") return { type: "Integer", value: Math.trunc(value) };
      return { type: "Integer", value: String(value) };
    }
    if (upper === "BOOLEAN") {
      return { type: "Boolean", value: Boolean(value) };
    }
    if (upper === "STRING") {
      return { type: "String", value: String(value) };
    }
    if (upper === "VOID") {
      return { type: "Void" };
    }
    if (upper === "ANY") {
      return this.encodeAnyToParam(value);
    }
    if (upper === "ARRAY") {
      if (!Array.isArray(value)) {
        throw new Error(`Expected array value for Neo type ${neoType}`);
      }
      return { type: "Array", value: value.map((item) => this.encodeAnyToParam(item)) };
    }
    if (upper === "MAP") {
      return this.encodeMapToParam(value);
    }
    if (upper === "HASH160") {
      if (Buffer.isBuffer(value) || value instanceof Uint8Array) {
        return { type: "Hash160", value: Buffer.from(value).toString("hex") };
      }
      return { type: "Hash160", value: String(value) };
    }
    if (upper === "HASH256") {
      if (Buffer.isBuffer(value) || value instanceof Uint8Array) {
        return { type: "Hash256", value: Buffer.from(value).toString("hex") };
      }
      return { type: "Hash256", value: String(value).startsWith("0x") ? String(value) : "0x" + String(value) };
    }
    if (upper === "BYTEARRAY") {
      if (Buffer.isBuffer(value) || value instanceof Uint8Array) {
        return { type: "ByteArray", value: normalizeByteArrayInput(Buffer.from(value)) };
      }
      const asString = String(value);
      if (/^0x[0-9a-fA-F]*$/.test(asString) || /^[0-9a-fA-F]+$/.test(asString)) {
        return { type: "ByteArray", value: normalizeByteArrayInput(asString) };
      }
      return { type: "ByteArray", value: normalizeByteArrayInput(asString) };
    }
    return value;
  }

  private encodeAnyToParam(value: any): ContractParamLike {
    if (value && typeof value === "object" && typeof value.type === "string") {
      // Already a ContractParam-like JSON object.
      return value;
    }

    if (value === null || value === undefined) {
      return { type: "Any", value: null };
    }

    if (typeof value === "boolean") {
      return { type: "Boolean", value };
    }

    if (typeof value === "bigint") {
      return { type: "Integer", value: value.toString() };
    }

    if (typeof value === "number") {
      return { type: "Integer", value: Math.trunc(value) };
    }

    if (typeof value === "string") {
      if (isNeoAddress(value)) {
        return { type: "Hash160", value };
      }
      if (/^0x[0-9a-fA-F]*$/.test(value) && value.length % 2 === 0) {
        return { type: "ByteArray", value: normalizeByteArrayInput(value) };
      }
      return { type: "String", value };
    }

    if (Buffer.isBuffer(value) || value instanceof Uint8Array) {
      return { type: "ByteArray", value: normalizeByteArrayInput(Buffer.from(value)) };
    }

    if (Array.isArray(value)) {
      return { type: "Array", value: value.map((item) => this.encodeAnyToParam(item)) };
    }

    if (value instanceof Map) {
      return {
        type: "Map",
        value: Array.from(value.entries()).map(([key, entryValue]) => ({
          key: this.encodeMapKeyToParam(key),
          value: this.encodeAnyToParam(entryValue),
        })),
      };
    }

    if (typeof value === "object") {
      return this.encodeMapToParam(value);
    }

    return { type: "String", value: String(value) };
  }

  private encodeMapKeyToParam(key: any): ContractParamLike {
    if (key && typeof key === "object" && typeof key.type === "string") {
      return key;
    }

    if (typeof key === "boolean") {
      return { type: "Boolean", value: key };
    }
    if (typeof key === "bigint") {
      return { type: "Integer", value: key.toString() };
    }
    if (typeof key === "number") {
      return { type: "Integer", value: Math.trunc(key) };
    }
    if (typeof key === "string") {
      if (/^0x[0-9a-fA-F]*$/.test(key) && key.length % 2 === 0) {
        return { type: "ByteArray", value: normalizeByteArrayInput(key) };
      }
      return { type: "String", value: key };
    }
    if (Buffer.isBuffer(key) || key instanceof Uint8Array) {
      return { type: "ByteArray", value: normalizeByteArrayInput(Buffer.from(key)) };
    }
    return { type: "String", value: String(key) };
  }

  private encodeMapToParam(value: any): ContractParamLike {
    if (Array.isArray(value)) {
      // Assume already in Neo JSON map-entry form: [{ key, value }]
      return { type: "Map", value };
    }

    if (value instanceof Map) {
      return {
        type: "Map",
        value: Array.from(value.entries()).map(([key, entryValue]) => ({
          key: this.encodeMapKeyToParam(key),
          value: this.encodeAnyToParam(entryValue),
        })),
      };
    }

    if (value && typeof value === "object") {
      return {
        type: "Map",
        value: Object.entries(value).map(([entryKey, entryValue]) => ({
          key: { type: "String", value: entryKey },
          value: this.encodeAnyToParam(entryValue),
        })),
      };
    }

    throw new Error(`Expected map-like value for Neo type Map`);
  }

  private encodeSolidityTypeToNeoParam(solidityType: string, value: any): ContractParamLike {
    const { baseType, arrayDims } = this.parseSolidityArrayType(solidityType);
    if (arrayDims.length > 0) {
      if (!Array.isArray(value)) {
        throw new Error(`Expected array value for type ${solidityType}`);
      }
      const elementType = solidityType.slice(0, solidityType.indexOf("["));
      const items = value.map((item) => this.encodeSolidityTypeToNeoParam(elementType, item));
      return { type: "Array", value: items };
    }

    if (baseType === "bool") {
      return { type: "Boolean", value: Boolean(value) };
    }

    if (baseType === "string") {
      return { type: "String", value: String(value) };
    }

    if (baseType === "address") {
      return { type: "Hash160", value: String(value) };
    }

    if (baseType === "bytes32") {
      if (Buffer.isBuffer(value) || value instanceof Uint8Array) {
        return { type: "Hash256", value: Buffer.from(value).toString("hex") };
      }
      const asString = String(value);
      return { type: "Hash256", value: asString.startsWith("0x") ? asString : "0x" + asString };
    }

    if (baseType === "bytes20") {
      return { type: "Hash160", value: String(value) };
    }

    if (baseType === "bytes" || baseType.startsWith("bytes")) {
      if (Buffer.isBuffer(value) || value instanceof Uint8Array) {
        return { type: "ByteArray", value: normalizeByteArrayInput(Buffer.from(value)) };
      }
      const asString = String(value);
      if (/^0x[0-9a-fA-F]*$/.test(asString) || /^[0-9a-fA-F]+$/.test(asString)) {
        return { type: "ByteArray", value: normalizeByteArrayInput(asString) };
      }
      return { type: "ByteArray", value: normalizeByteArrayInput(asString) };
    }

    if (baseType.startsWith("uint") || baseType.startsWith("int")) {
      if (typeof value === "bigint") return { type: "Integer", value: value.toString() };
      if (typeof value === "number") return { type: "Integer", value: Math.trunc(value).toString() };
      return { type: "Integer", value: String(value) };
    }

    throw new Error(`Unsupported constructor arg type: ${solidityType}`);
  }

  private parseSolidityArrayType(type: string): { baseType: string; arrayDims: Array<number | null> } {
    const dims: Array<number | null> = [];
    let rest = type;
    while (rest.endsWith("]")) {
      const open = rest.lastIndexOf("[");
      const dim = rest.slice(open + 1, -1);
      dims.push(dim.length === 0 ? null : Number(dim));
      rest = rest.slice(0, open);
    }
    return { baseType: rest, arrayDims: dims.reverse() };
  }
}
