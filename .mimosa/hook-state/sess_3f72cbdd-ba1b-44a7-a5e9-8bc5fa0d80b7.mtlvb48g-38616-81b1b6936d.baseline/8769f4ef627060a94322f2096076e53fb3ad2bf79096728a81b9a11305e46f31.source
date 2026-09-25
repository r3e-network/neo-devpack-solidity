import { createECDH, createHash, randomBytes } from "crypto";
import { neoScriptHashToAddress } from "@neo-devpack-solidity/types";

const BASE58_ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
const DEFAULT_SIGNATURE_CONTRACT = {
  parameters: [{ name: "signature", type: "Signature" }],
  deployed: false,
};

export interface NeoSignerAccount {
  address: string;
  scriptHash: string;
  privateKey: string;
  publicKey: string;
  label?: string;
  contract: {
    script: string;
    parameters: Array<{ name: string; type: string }>;
    deployed: boolean;
  };
}

function strip0x(value: string): string {
  return value.startsWith("0x") ? value.slice(2) : value;
}

function sha256(data: Buffer): Buffer {
  return createHash("sha256").update(data).digest();
}

function base58Encode(bytes: Buffer): string {
  let value = BigInt("0x" + (bytes.toString("hex") || "00"));
  let encoded = "";
  while (value > 0n) {
    const remainder = Number(value % 58n);
    encoded = BASE58_ALPHABET[remainder] + encoded;
    value /= 58n;
  }

  const leadingZeroCount = bytes.findIndex((byte) => byte !== 0);
  const zeroCount = leadingZeroCount === -1 ? bytes.length : leadingZeroCount;
  return "1".repeat(zeroCount) + (encoded || "");
}

function base58Decode(value: string): Buffer {
  let num = 0n;
  for (const char of value) {
    const index = BASE58_ALPHABET.indexOf(char);
    if (index === -1) {
      throw new Error(`Invalid base58 character: ${char}`);
    }
    num = num * 58n + BigInt(index);
  }

  const hex = num.toString(16);
  const decoded = Buffer.from(hex.length % 2 === 0 ? hex : "0" + hex, "hex");
  const leadingOnes = value.match(/^1*/u)?.[0]?.length ?? 0;
  return Buffer.concat([Buffer.alloc(leadingOnes), decoded]);
}

function base58CheckEncode(payload: Buffer): string {
  const checksum = sha256(sha256(payload)).subarray(0, 4);
  return base58Encode(Buffer.concat([payload, checksum]));
}

function base58CheckDecode(value: string): Buffer {
  const bytes = base58Decode(value);
  if (bytes.length < 5) {
    throw new Error("Invalid base58check payload");
  }
  const payload = bytes.subarray(0, bytes.length - 4);
  const checksum = bytes.subarray(bytes.length - 4);
  const expected = sha256(sha256(payload)).subarray(0, 4);
  if (!expected.equals(checksum)) {
    throw new Error("Invalid base58check checksum");
  }
  return payload;
}

export function decodePrivateKey(value: string): string {
  const trimmed = value.trim();
  const hex = strip0x(trimmed).toLowerCase();
  if (/^[0-9a-f]{64}$/u.test(hex)) {
    return hex;
  }

  const payload = base58CheckDecode(trimmed);
  if (payload.length !== 34 || payload[0] !== 0x80 || payload[payload.length - 1] !== 0x01) {
    throw new Error("Invalid Neo WIF");
  }
  return payload.subarray(1, 33).toString("hex");
}

export function encodeWif(privateKey: string): string {
  const normalized = decodePrivateKey(privateKey);
  const payload = Buffer.concat([
    Buffer.from([0x80]),
    Buffer.from(normalized, "hex"),
    Buffer.from([0x01]),
  ]);
  return base58CheckEncode(payload);
}

export function deriveCompressedPublicKey(privateKey: string): string {
  const normalized = decodePrivateKey(privateKey);
  const ecdh = createECDH("prime256v1");
  ecdh.setPrivateKey(Buffer.from(normalized, "hex"));
  return ecdh.getPublicKey("hex", "compressed");
}

export function getVerificationScriptFromPublicKey(publicKey: string): string {
  const normalized = strip0x(publicKey).toLowerCase();
  if (!/^(02|03)[0-9a-f]{64}$/u.test(normalized)) {
    throw new Error(`Invalid compressed public key: ${publicKey}`);
  }
  return `0c21${normalized}4156e7b327`;
}

export function getScriptHashFromVerificationScript(verificationScript: string): string {
  const normalized = strip0x(verificationScript);
  const hash = createHash("ripemd160")
    .update(createHash("sha256").update(Buffer.from(normalized, "hex")).digest())
    .digest();
  return Buffer.from(hash).reverse().toString("hex");
}

export function getScriptHashFromPublicKey(publicKey: string): string {
  return getScriptHashFromVerificationScript(getVerificationScriptFromPublicKey(publicKey));
}

export function generatePrivateKeyHex(): string {
  let generated = "";
  let valid = false;
  while (!valid) {
    const candidate = randomBytes(32).toString("hex");
    try {
      deriveCompressedPublicKey(candidate);
      generated = candidate;
      valid = true;
    } catch {
      // retry until the key is accepted by the P-256 curve implementation
    }
  }
  return generated;
}

export function createAccountFromPrivateKey(
  signingKey: string,
  addressVersion = 0x35,
  label?: string
): NeoSignerAccount {
  const privateKey = decodePrivateKey(signingKey);
  const publicKey = deriveCompressedPublicKey(privateKey);
  const scriptHash = getScriptHashFromPublicKey(publicKey);
  const verificationScript = getVerificationScriptFromPublicKey(publicKey);

  return {
    address: neoScriptHashToAddress("0x" + scriptHash, addressVersion),
    scriptHash,
    privateKey,
    publicKey,
    label,
    contract: {
      ...DEFAULT_SIGNATURE_CONTRACT,
      script: Buffer.from(verificationScript, "hex").toString("base64"),
    },
  };
}
