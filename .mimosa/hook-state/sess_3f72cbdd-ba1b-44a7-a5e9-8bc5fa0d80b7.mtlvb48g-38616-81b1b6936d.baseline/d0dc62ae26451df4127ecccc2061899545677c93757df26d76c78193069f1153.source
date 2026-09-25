import { describe, expect, it } from "vitest";
import { createECDH, createPrivateKey, createPublicKey, verify as cryptoVerify } from "crypto";
import {
  createNeoSigner,
  createNeoTransaction,
  createWitnessFromSignature,
  signTransaction,
} from "../src/transaction-primitives";
import { createAccountFromPrivateKey } from "../src/account-primitives";

function createPublicKeyObject(privateKeyHex: string) {
  const ecdh = createECDH("prime256v1");
  ecdh.setPrivateKey(Buffer.from(privateKeyHex, "hex"));
  const uncompressed = ecdh.getPublicKey(undefined, "uncompressed");
  const privateKey = createPrivateKey({
    key: {
      kty: "EC",
      crv: "P-256",
      x: uncompressed.subarray(1, 33).toString("base64url"),
      y: uncompressed.subarray(33).toString("base64url"),
      d: Buffer.from(privateKeyHex, "hex").toString("base64url"),
    },
    format: "jwk",
  });
  return createPublicKey(privateKey);
}

describe("transaction-primitives", () => {
  const signer = createAccountFromPrivateKey(
    "7d128a6d096f0c14c3a25a2b0c41cf79661bfcb4a8cc95aaaea28bde4d732344",
    0x35,
    "Fixture"
  );

  it("serializes signers like neon-js", () => {
    expect(
      createNeoSigner({
        account: signer.scriptHash,
        scopes: "CalledByEntry",
      }).serialize()
    ).toBe("26eba6592ddfb6b04426048cd5891ff0e3fecba701");
  });

  it("builds signature witnesses like neon-js", () => {
    expect(createWitnessFromSignature("11".repeat(64), signer.publicKey).serialize()).toBe(
      "420c4011111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111280c2102028a99826edc0c97d18e22b6932373d908d323aa7f92656a77ec26e8861699ef4156e7b327"
    );
  });

  it("serializes and signs simple invocation transactions like neon-js", () => {
    const script = "0c017811c01f0c0962616c616e63654f660c14fda3fa4346ea532a258fc497ddaddb6437c9fdff41627d5b52";
    const nonce = 123456789;

    const localTx = createNeoTransaction({
      scriptHex: script,
      nonce,
      signers: [{ account: signer.scriptHash, scopes: "CalledByEntry" }],
      validUntilBlock: 123,
      systemFee: "1000",
      networkFee: "2000",
    });
    signTransaction(localTx, signer, 860833102);

    expect(localTx.serialize(false)).toBe(
      "0015cd5b07e803000000000000d0070000000000007b0000000126eba6592ddfb6b04426048cd5891ff0e3fecba701002c0c017811c01f0c0962616c616e63654f660c14fda3fa4346ea532a258fc497ddaddb6437c9fdff41627d5b52"
    );
    expect(localTx.witnesses[0].verificationScript).toBe(
      "0c2102028a99826edc0c97d18e22b6932373d908d323aa7f92656a77ec26e8861699ef4156e7b327"
    );
    expect(localTx.witnesses[0].invocationScript).toMatch(/^0c40[0-9a-f]{128}$/);
    expect(
      cryptoVerify(
        "sha256",
        Buffer.from(localTx.getMessageForSigning(860833102), "hex"),
        { key: createPublicKeyObject(signer.privateKey), dsaEncoding: "ieee-p1363" },
        Buffer.from(localTx.witnesses[0].invocationScript.slice(4), "hex")
      )
    ).toBe(true);
  });
});
