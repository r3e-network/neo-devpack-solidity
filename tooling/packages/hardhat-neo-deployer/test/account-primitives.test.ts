import { describe, expect, it } from "vitest";
import {
  createAccountFromPrivateKey,
  decodePrivateKey,
  encodeWif,
  generatePrivateKeyHex,
} from "../src/account-primitives";

describe("account-primitives", () => {
  const privateKey = "7d128a6d096f0c14c3a25a2b0c41cf79661bfcb4a8cc95aaaea28bde4d732344";
  const wif = "L1QqQJnpBwbsPGAuutuzPTac8piqvbR1HRjrY5qHup48TBCBFe4g";
  const publicKey = "02028a99826edc0c97d18e22b6932373d908d323aa7f92656a77ec26e8861699ef";
  const scriptHash = "a7cbfee3f01f89d58c042644b0b6df2d59a6eb26";
  const address = "NPTmAHDxo6Pkyic8Nvu3kwyXoYJCvcCB6i";
  const contractScript = "DCECAoqZgm7cDJfRjiK2kyNz2QjTI6p/kmVqd+wm6IYWme9BVuezJw==";

  it("derives the same account fields from a private key", () => {
    const account = createAccountFromPrivateKey(privateKey, 0x35, "Fixture");
    expect(account.privateKey).toBe(privateKey);
    expect(account.publicKey).toBe(publicKey);
    expect(account.scriptHash).toBe(scriptHash);
    expect(account.address).toBe(address);
    expect(account.contract.script).toBe(contractScript);
    expect(account.contract.parameters).toEqual([{ name: "signature", type: "Signature" }]);
  });

  it("decodes WIF to the same private key", () => {
    expect(decodePrivateKey(wif)).toBe(privateKey);
    expect(encodeWif(privateKey)).toBe(wif);
  });

  it("generates valid private keys", () => {
    const generated = generatePrivateKeyHex();
    expect(generated).toMatch(/^[0-9a-f]{64}$/);
  });
});
