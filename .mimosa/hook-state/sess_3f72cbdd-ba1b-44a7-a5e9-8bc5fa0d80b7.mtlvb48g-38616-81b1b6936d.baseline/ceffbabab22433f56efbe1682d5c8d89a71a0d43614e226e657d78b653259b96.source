import { createECDH, createHash, createPrivateKey, sign as cryptoSign } from "crypto";
import { getScriptHashFromVerificationScript, NeoSignerAccount, getVerificationScriptFromPublicKey } from "./account-primitives";

export const WitnessScope = {
  None: 0,
  CalledByEntry: 1,
  CustomContracts: 16,
  CustomGroups: 32,
  WitnessRules: 64,
  Global: 128,
} as const;

type WitnessScopeName = keyof typeof WitnessScope;

export interface NeoSignerLike {
  account: string;
  scopes: WitnessScopeName | number;
  allowedContracts?: string[];
  allowedGroups?: string[];
  rules?: any[];
}

export interface NeoWitnessLike {
  invocationScript: string;
  verificationScript: string;
}

export interface NeoTransactionLike {
  version?: number;
  nonce?: number;
  systemFee?: string | number | bigint;
  networkFee?: string | number | bigint;
  validUntilBlock: number;
  signers: NeoSigner[];
  scriptHex: string;
  witnesses?: NeoWitness[];
}

function strip0x(value: string): string {
  return value.startsWith("0x") ? value.slice(2) : value;
}

function reverseHex(value: string): string {
  return Buffer.from(strip0x(value), "hex").reverse().toString("hex");
}

function sha256Hex(value: string): string {
  return createHash("sha256").update(Buffer.from(strip0x(value), "hex")).digest("hex");
}

function toU32LE(value: number): string {
  const buf = Buffer.allocUnsafe(4);
  buf.writeUInt32LE(value, 0);
  return buf.toString("hex");
}

function bigintToFixedLE(value: string | number | bigint, bytes: number): string {
  const bigint = BigInt(value);
  const buf = Buffer.alloc(bytes);
  let current = BigInt.asUintN(bytes * 8, bigint);
  for (let i = 0; i < bytes; i++) {
    buf[i] = Number(current & 0xffn);
    current >>= 8n;
  }
  return buf.toString("hex");
}

function numToVarInt(value: number): string {
  if (value < 0xfd) {
    return value.toString(16).padStart(2, "0");
  }
  if (value <= 0xffff) {
    const buf = Buffer.allocUnsafe(2);
    buf.writeUInt16LE(value, 0);
    return "fd" + buf.toString("hex");
  }
  if (value <= 0xffffffff) {
    return "fe" + toU32LE(value);
  }
  return "ff" + bigintToFixedLE(BigInt(value), 8);
}

function serializeArray(items: Array<{ serialize(): string } | string>): string {
  return numToVarInt(items.length) + items.map((item) => (typeof item === "string" ? item : item.serialize())).join("");
}

function witnessScopeToString(scope: number): string {
  if (scope === WitnessScope.None) return "None";
  return (Object.entries(WitnessScope) as Array<[WitnessScopeName, number]>)
    .filter(([, value]) => scope & value)
    .map(([name]) => name)
    .join(",");
}

function witnessScopeFromValue(scope: WitnessScopeName | number): number {
  if (typeof scope === "number") return scope & 0xff;
  return WitnessScope[scope] ?? WitnessScope.None;
}

function createPrivateKeyObject(account: NeoSignerAccount) {
  const ecdh = createECDH("prime256v1");
  ecdh.setPrivateKey(Buffer.from(account.privateKey, "hex"));
  const uncompressed = ecdh.getPublicKey(undefined, "uncompressed");
  const x = uncompressed.subarray(1, 33).toString("base64url");
  const y = uncompressed.subarray(33).toString("base64url");
  const d = Buffer.from(account.privateKey, "hex").toString("base64url");
  return createPrivateKey({
    key: { kty: "EC", crv: "P-256", x, y, d },
    format: "jwk",
  });
}

function signHex(hex: string, account: NeoSignerAccount): string {
  const key = createPrivateKeyObject(account);
  return cryptoSign("sha256", Buffer.from(strip0x(hex), "hex"), {
    key,
    dsaEncoding: "ieee-p1363",
  }).toString("hex");
}

export class NeoSigner {
  public readonly account: string;
  public readonly scopes: number;

  constructor(input: NeoSignerLike) {
    this.account = strip0x(input.account).toLowerCase();
    this.scopes = witnessScopeFromValue(input.scopes);
  }

  serialize(): string {
    return reverseHex(this.account) + this.scopes.toString(16).padStart(2, "0");
  }

  toJson() {
    return {
      account: "0x" + this.account,
      scopes: witnessScopeToString(this.scopes),
    };
  }
}

export class NeoWitness {
  public readonly invocationScript: string;
  public readonly verificationScript: string;

  constructor(input: NeoWitnessLike) {
    this.invocationScript = strip0x(input.invocationScript).toLowerCase();
    this.verificationScript = strip0x(input.verificationScript).toLowerCase();
  }

  get scriptHash(): string {
    return getScriptHashFromVerificationScript(this.verificationScript);
  }

  serialize(): string {
    return (
      numToVarInt(this.invocationScript.length / 2) +
      this.invocationScript +
      numToVarInt(this.verificationScript.length / 2) +
      this.verificationScript
    );
  }

  export() {
    return {
      invocationScript: this.invocationScript,
      verificationScript: this.verificationScript,
    };
  }
}

export class NeoTransaction {
  public readonly version: number;
  public readonly nonce: number;
  public readonly systemFee: string;
  public readonly networkFee: string;
  public readonly validUntilBlock: number;
  public readonly signers: NeoSigner[];
  public readonly scriptHex: string;
  public readonly witnesses: NeoWitness[];

  constructor(input: NeoTransactionLike) {
    this.version = input.version ?? 0;
    this.nonce = input.nonce ?? createHash("sha256").update(Buffer.from(Math.random().toString())).digest().readUInt32LE(0);
    this.systemFee = String(input.systemFee ?? 0);
    this.networkFee = String(input.networkFee ?? 0);
    this.validUntilBlock = input.validUntilBlock;
    this.signers = input.signers;
    this.scriptHex = strip0x(input.scriptHex).toLowerCase();
    this.witnesses = input.witnesses ?? [];
  }

  addWitness(witness: NeoWitness): this {
    const existing = this.witnesses.findIndex((item) => item.verificationScript === witness.verificationScript);
    if (existing >= 0) {
      this.witnesses.splice(existing, 1, witness);
    } else {
      this.witnesses.push(witness);
    }
    this.orderWitnesses();
    return this;
  }

  orderWitnesses(): this {
    this.signers.forEach((signer, index) => {
      const witnessIndex = this.witnesses.findIndex((witness) => witness.scriptHash === signer.account);
      if (witnessIndex >= 0) {
        const [witness] = this.witnesses.splice(witnessIndex, 1);
        this.witnesses.splice(index, 0, witness);
      }
    });
    return this;
  }

  serialize(signed = true): string {
    let out = "";
    out += this.version.toString(16).padStart(2, "0");
    out += toU32LE(this.nonce);
    out += bigintToFixedLE(this.systemFee, 8);
    out += bigintToFixedLE(this.networkFee, 8);
    out += toU32LE(this.validUntilBlock);
    out += serializeArray(this.signers);
    out += "00"; // attributes
    out += numToVarInt(this.scriptHex.length / 2);
    out += this.scriptHex;
    if (signed) {
      out += serializeArray(this.witnesses);
    }
    return out;
  }

  hash(): string {
    return reverseHex(sha256Hex(this.serialize(false)));
  }

  getMessageForSigning(networkMagic: number): string {
    return toU32LE(networkMagic) + reverseHex(this.hash());
  }

  export() {
    return {
      version: this.version,
      nonce: this.nonce,
      systemFee: this.systemFee,
      networkFee: this.networkFee,
      validUntilBlock: this.validUntilBlock,
      attributes: [],
      signers: this.signers.map((signer) => ({
        account: signer.account,
        scopes: signer.scopes,
      })),
      witnesses: this.witnesses.map((witness) => witness.export()),
      script: this.scriptHex,
    };
  }
}

export function createNeoSigner(input: NeoSignerLike): NeoSigner {
  return new NeoSigner(input);
}

export function createNeoWitness(input: NeoWitnessLike): NeoWitness {
  return new NeoWitness(input);
}

export function createWitnessFromSignature(signature: string, publicKey: string): NeoWitness {
  return new NeoWitness({
    invocationScript: "0c40" + strip0x(signature),
    verificationScript: getVerificationScriptFromPublicKey(publicKey),
  });
}

export function createNeoTransaction(input: {
  scriptHex: string;
  signers: NeoSignerLike[];
  validUntilBlock: number;
  systemFee?: string | number | bigint;
  networkFee?: string | number | bigint;
  nonce?: number;
}): NeoTransaction {
  return new NeoTransaction({
    scriptHex: input.scriptHex,
    signers: input.signers.map(createNeoSigner),
    validUntilBlock: input.validUntilBlock,
    systemFee: input.systemFee,
    networkFee: input.networkFee,
    nonce: input.nonce,
  });
}

export function signTransaction(
  transaction: NeoTransaction,
  signerAccount: NeoSignerAccount,
  networkMagic: number,
  _k?: string
): NeoTransaction {
  const signature = signHex(transaction.getMessageForSigning(networkMagic), signerAccount);
  transaction.addWitness(createWitnessFromSignature(signature, signerAccount.publicKey));
  return transaction;
}
