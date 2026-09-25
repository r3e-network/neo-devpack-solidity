const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

describe("Devpack package metadata", function () {
  it("uses Neo-native task scripts instead of stale EVM helper flows", function () {
    const packageJson = require("../package.json");
    const hardhatConfig = fs.readFileSync(path.join(__dirname, "..", "hardhat.config.js"), "utf8");

    assert.match(packageJson.scripts["deploy:testnet"], /\bneo-deploy\b/);
    assert.doesNotMatch(packageJson.scripts["deploy:testnet"], /scripts\/deploy\.js/);
    assert.match(packageJson.scripts["deploy:mainnet"], /\bneo-deploy\b/);
    assert.match(packageJson.scripts.verify, /\bneo-verify\b/);
    assert.match(packageJson.scripts.test, /\bneo-compile\b.*--force(?:\s|$)/);
    assert.match(packageJson.scripts["test:integration"], /\bneo-compile\b.*--force(?:\s|$)/);
    assert.doesNotMatch(packageJson.scripts.test, /\b--force\s+true\b/);
    assert.doesNotMatch(packageJson.scripts["test:integration"], /\b--force\s+true\b/);
    assert.ok(!packageJson.files.includes("dist/"));
    assert.ok(!("@r3e-network/neo-devpack-solidity" in (packageJson.peerDependencies || {})));
    assert.ok(!("lint" in packageJson.scripts));
    assert.ok(!("size" in packageJson.scripts));
    assert.ok(!("coverage" in packageJson.scripts));
    assert.ok(!("docs" in packageJson.scripts));
    assert.match(hardhatConfig, /solidity:\s*\{\s*version:\s*"0\.8\.34"/);
    assert.match(hardhatConfig, /config\.neoSolc\s*=\s*\{\s*solidity:\s*\{\s*version:\s*"0\.8\.34"/s);

    assert.doesNotMatch(hardhatConfig, /@nomicfoundation\/hardhat-toolbox/);
    assert.doesNotMatch(hardhatConfig, /hardhat-contract-sizer/);
    assert.doesNotMatch(hardhatConfig, /\bdocgen:\s*\{/);
  });
});
