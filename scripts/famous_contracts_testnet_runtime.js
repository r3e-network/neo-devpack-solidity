#!/usr/bin/env node
'use strict';

const fs = require('fs');
const os = require('os');
const path = require('path');
const { spawnSync } = require('child_process');

const { wallet, rpc, sc, tx, u, CONST, experimental } = require('@cityofzion/neon-js');

const ROOT = path.resolve(__dirname, '..');
const AUDIT_DIR = process.env.NEO_FAMOUS_AUDIT_DIR || '/tmp/neo-famous-contracts-audit';
const AUDIT_NODE_MODULES = path.join(AUDIT_DIR, 'node_modules');
const REPORT_JSON_PATH = path.join(ROOT, 'docs/data/famous-contracts-testnet-runtime-results.json');
const REPORT_MD_PATH = path.join(ROOT, 'docs/solidity/famous-contracts-testnet-runtime.md');

const DEFAULT_RPC = process.env.NEO_TESTNET_RPC || 'http://seed1t5.neo.org:20332';
const DEPLOYER_WIF = process.env.NEO_TESTNET_WIF;

const SUPPRESSED_WARNINGS = [
  'W200',
  'W121',
  'W111',
  'W116',
  'W106',
  'W105',
  'MANIFEST_WILDCARD_CONTRACT',
  'INVALID_STORAGE_RETURN'
];

const CASES = [
  {
    id: 'openzeppelin-erc20',
    project: 'OpenZeppelin',
    contract: 'ERC20',
    sourcePath: '@openzeppelin/contracts/token/ERC20/ERC20.sol',
    deployData: ['NeoUSD', 'nUSD'],
    clearSupportedStandards: true,
    note: 'Original upstream ERC20; clears supportedstandards due ERC-20 vs NEP-17 shape mismatch.',
    assertions: [
      {
        name: 'approve(deployer, 123)',
        kind: 'write',
        operation: 'approve',
        args: [{ type: 'Hash160', value: '$DEPLOYER_ADDRESS' }, 123]
      },
      {
        name: 'allowance(deployer, deployer) == 123',
        kind: 'read',
        operation: 'allowance',
        args: [
          { type: 'Hash160', value: '$DEPLOYER_ADDRESS' },
          { type: 'Hash160', value: '$DEPLOYER_ADDRESS' }
        ],
        expect: { type: 'Integer', value: '123' }
      }
    ]
  },
  {
    id: 'openzeppelin-erc721',
    project: 'OpenZeppelin',
    contract: 'ERC721',
    sourcePath: '@openzeppelin/contracts/token/ERC721/ERC721.sol',
    deployData: ['NeoNFT', 'nNFT'],
    clearSupportedStandards: true,
    note: 'Original upstream ERC721; clears supportedstandards due ERC-721 vs NEP-11 shape mismatch.',
    assertions: [
      {
        name: 'setApprovalForAll(self, true)',
        kind: 'write',
        operation: 'setApprovalForAll',
        args: [{ type: 'Hash160', value: '$DEPLOYER_ADDRESS' }, true]
      },
      {
        name: 'isApprovedForAll(self, self) == true',
        kind: 'read',
        operation: 'isApprovedForAll',
        args: [
          { type: 'Hash160', value: '$DEPLOYER_ADDRESS' },
          { type: 'Hash160', value: '$DEPLOYER_ADDRESS' }
        ],
        expect: { type: 'Boolean', value: true }
      }
    ]
  },
  {
    id: 'aave-weth9',
    project: 'Aave V3',
    contract: 'WETH9',
    sourcePath: '@aave/core-v3/contracts/dependencies/weth/WETH9.sol',
    clearSupportedStandards: true,
    note: 'Original upstream WETH9; clears supportedstandards due ERC-style schema mismatch.',
    assertions: [
      {
        name: 'approve(self, 77)',
        kind: 'write',
        operation: 'approve',
        args: [{ type: 'Hash160', value: '$DEPLOYER_ADDRESS' }, 77]
      },
      {
        name: 'allowance(self, self) == 77',
        kind: 'read',
        operation: 'allowance',
        args: [
          { type: 'Hash160', value: '$DEPLOYER_ADDRESS' },
          { type: 'Hash160', value: '$DEPLOYER_ADDRESS' }
        ],
        expect: { type: 'Integer', value: '77' }
      }
    ]
  },
  {
    id: 'uniswap-v2-pair',
    project: 'Uniswap V2 Core',
    contract: 'UniswapV2Pair',
    sourcePath: '@uniswap/v2-core/contracts/UniswapV2Pair.sol',
    clearSupportedStandards: true,
    note: 'Original upstream pair contract; approve/allowance path runtime-verified.',
    assertions: [
      {
        name: 'approve(self, 321)',
        kind: 'write',
        operation: 'approve',
        args: [{ type: 'Hash160', value: '$DEPLOYER_ADDRESS' }, 321]
      },
      {
        name: 'allowance(self, self) == 321',
        kind: 'read',
        operation: 'allowance',
        args: [
          { type: 'Hash160', value: '$DEPLOYER_ADDRESS' },
          { type: 'Hash160', value: '$DEPLOYER_ADDRESS' }
        ],
        expect: { type: 'Integer', value: '321' }
      }
    ]
  },
  {
    id: 'uniswap-v2-erc20',
    project: 'Uniswap V2 Core',
    contract: 'UniswapV2ERC20',
    sourcePath: '@uniswap/v2-core/contracts/UniswapV2ERC20.sol',
    clearSupportedStandards: true,
    note: 'Original upstream UniswapV2ERC20; approve/allowance runtime-verified.',
    assertions: [
      {
        name: 'approve(self, 456)',
        kind: 'write',
        operation: 'approve',
        args: [{ type: 'Hash160', value: '$DEPLOYER_ADDRESS' }, 456]
      },
      {
        name: 'allowance(self, self) == 456',
        kind: 'read',
        operation: 'allowance',
        args: [
          { type: 'Hash160', value: '$DEPLOYER_ADDRESS' },
          { type: 'Hash160', value: '$DEPLOYER_ADDRESS' }
        ],
        expect: { type: 'Integer', value: '456' }
      }
    ]
  },
  {
    id: 'chainlink-mockv3aggregator',
    project: 'Chainlink',
    contract: 'MockV3Aggregator',
    sourcePath: '@chainlink/contracts/src/v0.8/shared/mocks/MockV3Aggregator.sol',
    deployData: [8, 123456789],
    clearSupportedStandards: false,
    note: 'Original upstream Chainlink mock oracle with update/read verification against a run-scoped value.',
    assertions: [
      {
        name: 'updateAnswer(dynamic)',
        kind: 'write',
        operation: 'updateAnswer',
        args: [{ type: 'Integer', value: '$CHAINLINK_TEST_VALUE' }]
      },
      {
        name: 'latestAnswer() final == dynamic',
        kind: 'read',
        operation: 'latestAnswer',
        args: [],
        expect: { type: 'Integer', value: '$CHAINLINK_TEST_VALUE' }
      }
    ]
  }
];

function run(cmd, args, options = {}) {
  const res = spawnSync(cmd, args, {
    encoding: 'utf8',
    maxBuffer: 128 * 1024 * 1024,
    ...options
  });
  if (res.error) {
    throw res.error;
  }
  return res;
}

function stripAnsi(input) {
  return String(input || '').replace(
    // eslint-disable-next-line no-control-regex
    /\u001b\[[0-9;]*m/g,
    ''
  );
}

function sanitizeName(input) {
  return String(input).replace(/[^a-zA-Z0-9_.-]+/g, '_');
}

function to0x(hash) {
  const raw = String(hash).toLowerCase().replace(/^0x/, '');
  return `0x${raw}`;
}

function resolveNeoSolc() {
  if (process.env.NEO_SOLC) {
    return process.env.NEO_SOLC;
  }

  const releaseBin = path.join(ROOT, 'target/release/neo-solc');
  if (fs.existsSync(releaseBin)) {
    return releaseBin;
  }

  const debugBin = path.join(ROOT, 'target/debug/neo-solc');
  if (fs.existsSync(debugBin)) {
    return debugBin;
  }

  const build = run('cargo', ['build', '--release', '--bin', 'neo-solc'], { cwd: ROOT });
  if (build.status !== 0) {
    throw new Error(`failed to build neo-solc:\n${build.stdout}\n${build.stderr}`);
  }
  if (!fs.existsSync(releaseBin)) {
    throw new Error('neo-solc build succeeded but binary not found');
  }
  return releaseBin;
}

function ensureAuditWorkspace() {
  const vendorScript = path.join(ROOT, 'scripts/vendor_famous_contracts.js');
  if (!fs.existsSync(vendorScript)) {
    throw new Error(`missing vendor script: ${vendorScript}`);
  }

  const mustHave = [
    '@openzeppelin/contracts/package.json',
    '@aave/core-v3/package.json',
    '@safe-global/safe-contracts/package.json',
    '@chainlink/contracts/package.json',
    '@uniswap/v2-core/package.json'
  ];

  const ready = mustHave.every((probe) => fs.existsSync(path.join(AUDIT_NODE_MODULES, probe)));
  if (ready) {
    return;
  }

  const sync = run('node', [vendorScript], { cwd: ROOT });
  if (sync.status !== 0) {
    throw new Error(`failed to prepare audit workspace:\n${sync.stdout}\n${sync.stderr}`);
  }
}

function compileCase(neoSolc, workDir, testCase) {
  const sourcePath = path.join(AUDIT_NODE_MODULES, testCase.sourcePath);
  if (!fs.existsSync(sourcePath)) {
    return {
      ok: false,
      reason: `source missing: ${testCase.sourcePath}`
    };
  }

  const outPrefix = path.join(workDir, sanitizeName(testCase.id));
  const args = [
    sourcePath,
    '-I',
    path.join(ROOT, 'devpack'),
    '-I',
    AUDIT_NODE_MODULES,
    '--contract',
    testCase.contract,
    '-o',
    outPrefix
  ];

  for (const warn of SUPPRESSED_WARNINGS) {
    args.push('--Wno', warn);
  }

  const compile = run(neoSolc, args, { cwd: ROOT });
  if (compile.status !== 0) {
    return {
      ok: false,
      reason: 'compile failed',
      stderr: stripAnsi(compile.stderr)
    };
  }

  const nefPath = `${outPrefix}.nef`;
  const manifestPath = `${outPrefix}.manifest.json`;
  if (!fs.existsSync(nefPath) || !fs.existsSync(manifestPath)) {
    return {
      ok: false,
      reason: 'compiler output missing .nef or .manifest.json'
    };
  }

  if (testCase.clearSupportedStandards) {
    const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
    manifest.supportedstandards = [];
    fs.writeFileSync(manifestPath, JSON.stringify(manifest, null, 2));
  }

  return {
    ok: true,
    sourcePath,
    nefPath,
    manifestPath,
    warningCount: (stripAnsi(compile.stderr).match(/warning/gi) || []).length
  };
}

function materialize(value, context) {
  if (Array.isArray(value)) {
    return value.map((item) => materialize(item, context));
  }

  if (value && typeof value === 'object') {
    const out = {};
    for (const [key, inner] of Object.entries(value)) {
      out[key] = materialize(inner, context);
    }
    return out;
  }

  if (typeof value === 'string' && /^\$[A-Z0-9_]+$/.test(value)) {
    const key = value.slice(1);
    if (!Object.prototype.hasOwnProperty.call(context, key)) {
      throw new Error(`unknown template variable: ${value}`);
    }
    return context[key];
  }

  return value;
}

function materializeExpect(expect, context) {
  if (!expect || typeof expect !== 'object') {
    return expect;
  }
  return materialize(expect, context);
}

function toContractParam(value) {
  if (value && typeof value === 'object' && !Array.isArray(value)) {
    const maybeType = String(value.type || '').toLowerCase();
    if (maybeType) {
      if (maybeType === 'hash160') {
        return sc.ContractParam.hash160(String(value.value));
      }
      if (maybeType === 'integer') {
        return sc.ContractParam.integer(String(value.value));
      }
      if (maybeType === 'boolean') {
        return sc.ContractParam.boolean(Boolean(value.value));
      }
      if (maybeType === 'string') {
        return sc.ContractParam.string(String(value.value));
      }
      if (maybeType === 'bytearray') {
        return sc.ContractParam.byteArray(String(value.value));
      }
    }
  }

  if (Array.isArray(value)) {
    return sc.ContractParam.array(...value.map((x) => toContractParam(x)));
  }

  if (typeof value === 'number') {
    return sc.ContractParam.integer(value);
  }
  if (typeof value === 'boolean') {
    return sc.ContractParam.boolean(value);
  }
  if (typeof value === 'string') {
    // When ambiguous, keep string as string. Hash160 should use typed object form.
    return sc.ContractParam.string(value);
  }
  if (value === null || value === undefined) {
    return sc.ContractParam.any(null);
  }

  throw new Error(`unsupported param value type: ${typeof value}`);
}

function compareExpectedStackTop(stackTop, expect) {
  if (!expect) {
    return { ok: true };
  }

  if (!stackTop) {
    return { ok: false, reason: 'read result stack is empty' };
  }

  if (expect.type && stackTop.type !== expect.type) {
    return {
      ok: false,
      reason: `stack[0].type=${stackTop.type}, expected ${expect.type}`
    };
  }

  if (Object.prototype.hasOwnProperty.call(expect, 'value')) {
    if (expect.type === 'Boolean') {
      const actualBool =
        typeof stackTop.value === 'boolean' ? stackTop.value : String(stackTop.value).toLowerCase() === 'true';
      const expectedBool = typeof expect.value === 'boolean' ? expect.value : String(expect.value).toLowerCase() === 'true';
      if (actualBool !== expectedBool) {
        return {
          ok: false,
          reason: `stack[0].value=${String(stackTop.value)}, expected ${String(expect.value)}`
        };
      }
    } else if (String(stackTop.value) !== String(expect.value)) {
      return {
        ok: false,
        reason: `stack[0].value=${String(stackTop.value)}, expected ${String(expect.value)}`
      };
    }
  }

  return { ok: true };
}

async function waitForApplicationLog(client, txid, timeoutMs = 180000, intervalMs = 3000) {
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {
    try {
      const appLog = await client.getApplicationLog(txid);
      if (appLog && Array.isArray(appLog.executions) && appLog.executions.length > 0) {
        return appLog;
      }
    } catch {
      // tx may not be persisted yet
    }
    // eslint-disable-next-line no-await-in-loop
    await new Promise((resolve) => setTimeout(resolve, intervalMs));
  }
  throw new Error(`timed out waiting application log for tx ${txid}`);
}

async function waitForContractState(client, contractHash, timeoutMs = 180000, intervalMs = 3000) {
  const started = Date.now();
  const withPrefix = to0x(contractHash);
  while (Date.now() - started < timeoutMs) {
    try {
      const state = await client.getContractState(withPrefix);
      if (state && state.hash) {
        return state;
      }
    } catch {
      // not ready yet
    }
    // eslint-disable-next-line no-await-in-loop
    await new Promise((resolve) => setTimeout(resolve, intervalMs));
  }
  throw new Error(`timed out waiting contract state for ${withPrefix}`);
}

async function deployToTestnet(client, config, account, nef, manifest, deployData) {
  const args = [
    sc.ContractParam.byteArray(u.HexString.fromHex(nef.serialize(), true)),
    sc.ContractParam.string(JSON.stringify(manifest.toJson())),
    sc.ContractParam.string(JSON.stringify(Array.isArray(deployData) ? deployData : []))
  ];

  const builder = new sc.ScriptBuilder();
  builder.emitContractCall({
    scriptHash: CONST.NATIVE_CONTRACT_HASH.ManagementContract,
    operation: 'deploy',
    callFlags: sc.CallFlags.All,
    args
  });

  const transaction = new tx.Transaction();
  transaction.script = u.HexString.fromHex(builder.build());
  await experimental.txHelpers.setBlockExpiry(transaction, config, config.blocksTillExpiry);
  transaction.addSigner({
    account: account.scriptHash,
    scopes: 'CalledByEntry'
  });
  await experimental.txHelpers.addFees(transaction, config);
  transaction.sign(account, config.networkMagic);

  const txid = await client.sendRawTransaction(transaction);
  return txid;
}

function renderMarkdown(report) {
  const lines = [];
  lines.push('# Famous EVM Contracts Runtime-Verified on Neo N3 TestNet');
  lines.push('');
  lines.push(`- Generated at (UTC): \`${report.generatedAt}\``);
  lines.push('- Snapshot scope: historical TestNet runtime output; rerun the TestNet runtime script before treating these results as current release evidence.');
  lines.push(`- RPC endpoint: \`${report.rpcAddress}\``);
  lines.push(`- Network magic: \`${report.networkMagic}\``);
  lines.push(`- Deployer address: \`${report.deployerAddress}\``);
  lines.push(`- Total contracts: \`${report.totals.total}\``);
  lines.push(`- Pass: \`${report.totals.pass}\``);
  lines.push(`- Fail: \`${report.totals.fail}\``);
  lines.push(`- Assertions passed: \`${report.totals.assertionsPassed}/${report.totals.assertionsTotal}\``);
  lines.push(`- GAS before: \`${report.gasBefore}\``);
  lines.push(`- GAS after: \`${report.gasAfter}\``);
  lines.push('');
  lines.push('| # | Project | Contract | Result | Assertions | Contract Hash | Deploy Tx | Source |');
  lines.push('|---:|---|---|---|---:|---|---|---|');
  report.results.forEach((row, idx) => {
    const mark = row.status === 'pass' ? '✅ pass' : '❌ fail';
    const assertions = `${row.assertionsPassed}/${row.assertionsTotal}`;
    const source = `\`${row.vendoredPath || row.sourcePath || '-'}\``;
    lines.push(
      `| ${idx + 1} | ${row.project} | ${row.contract} | ${mark} | ${assertions} | ${row.contractHash || '-'} | ${row.deployTxHash || '-'} | ${source} |`
    );
  });

  lines.push('');
  lines.push('## Assertion Details');
  lines.push('');
  report.results.forEach((row) => {
    lines.push(`### ${row.project} / ${row.contract}`);
    lines.push('');
    lines.push(`- Status: \`${row.status}\``);
    lines.push(`- Contract hash: \`${row.contractHash || '-'}\``);
    lines.push(`- Deploy tx: \`${row.deployTxHash || '-'}\``);
    if (row.reason) {
      lines.push(`- Failure reason: \`${String(row.reason).replace(/`/g, "'")}\``);
    }
    lines.push('- Assertions:');
    for (const a of row.assertions || []) {
      const mark = a.status === 'pass' ? '✅' : '❌';
      const expected = a.expect ? ` expected=${JSON.stringify(a.expect)}` : '';
      const actual = a.actual ? ` actual=${JSON.stringify(a.actual)}` : '';
      const txHash = a.txHash ? ` tx=${a.txHash}` : '';
      const reason = a.reason ? ` reason=${String(a.reason).replace(/`/g, "'")}` : '';
      lines.push(`  - ${mark} ${a.kind} \`${a.operation}\`${txHash}${expected}${actual}${reason}`);
    }
    lines.push('');
  });

  return lines.join('\n');
}

async function getGasBalance(client, address) {
  const balances = await client.getNep17Balances(address);
  const gas = (balances.balance || []).find(
    (item) => String(item.symbol || '').toUpperCase() === 'GAS' || String(item.assethash || '').toLowerCase() === '0xd2a4cff31913016155e38e474a2c06d08be276cf'
  );
  return gas ? String(gas.amount) : '0';
}

async function main() {
  if (!DEPLOYER_WIF) {
    throw new Error('NEO_TESTNET_WIF is required');
  }

  ensureAuditWorkspace();
  const neoSolc = resolveNeoSolc();

  const account = new wallet.Account(DEPLOYER_WIF);
  const client = new rpc.RPCClient(DEFAULT_RPC);
  const version = await client.getVersion();
  const networkMagic = version?.protocol?.network;
  if (!networkMagic) {
    throw new Error(`failed to read network magic from ${DEFAULT_RPC}`);
  }

  const config = {
    rpcAddress: DEFAULT_RPC,
    networkMagic,
    account,
    blocksTillExpiry: 300
  };

  const gasBefore = await getGasBalance(client, account.address);
  const neoSolcVersion = run(neoSolc, ['--version'], { cwd: ROOT });
  const neoSolcVersionText = neoSolcVersion.status === 0 ? neoSolcVersion.stdout.trim() : 'unknown';

  const workDir = fs.mkdtempSync(path.join(os.tmpdir(), 'neo-famous-testnet-'));
  try {
    const results = [];
    const templateContext = {
      DEPLOYER_ADDRESS: account.address,
      CHAINLINK_TEST_VALUE: String(Date.now())
    };

    for (const testCase of CASES) {
      process.stdout.write(`[testnet-runtime] case start: ${testCase.project}/${testCase.contract}\n`);
      const caseResult = {
        ...testCase,
        status: 'fail',
        sourcePath: testCase.sourcePath,
        vendoredPath: `third_party/famous-contracts/sources/${testCase.sourcePath}`,
        contractHash: null,
        deployTxHash: null,
        assertions: [],
        assertionsTotal: Array.isArray(testCase.assertions) ? testCase.assertions.length : 0,
        assertionsPassed: 0,
        warningCount: 0
      };

      const compiled = compileCase(neoSolc, workDir, testCase);
      if (!compiled.ok) {
        process.stdout.write(`[testnet-runtime] case fail (compile): ${testCase.contract} -> ${compiled.reason}\n`);
        caseResult.reason = compiled.reason;
        results.push(caseResult);
        continue;
      }
      caseResult.warningCount = compiled.warningCount || 0;

      const nef = sc.NEF.fromBuffer(fs.readFileSync(compiled.nefPath));
      const manifestJson = JSON.parse(fs.readFileSync(compiled.manifestPath, 'utf8'));
      const manifest = sc.ContractManifest.fromJson(manifestJson);

      const predictedHash = to0x(
        experimental.getContractHash(u.HexString.fromHex(account.scriptHash), nef.checksum, manifest.name)
      );
      caseResult.contractHash = predictedHash;

      let deployTxHash = null;
      try {
        // eslint-disable-next-line no-await-in-loop
        deployTxHash = await deployToTestnet(client, config, account, nef, manifest, testCase.deployData || null);
        caseResult.deployTxHash = deployTxHash;
        process.stdout.write(`[testnet-runtime] deploy tx: ${testCase.contract} -> ${deployTxHash}\n`);

        // eslint-disable-next-line no-await-in-loop
        const deployLog = await waitForApplicationLog(client, deployTxHash);
        const vmstate = deployLog?.executions?.[0]?.vmstate || 'UNKNOWN';
        if (vmstate !== 'HALT') {
          process.stdout.write(`[testnet-runtime] case fail (deploy vmstate): ${testCase.contract} -> ${vmstate}\n`);
          caseResult.reason = `deploy vmstate=${vmstate}`;
          results.push(caseResult);
          continue;
        }
      } catch (err) {
        const msg = String(err && err.message ? err.message : err);
        if (!/already exists|already deployed|hash already exists|already have/i.test(msg)) {
          process.stdout.write(`[testnet-runtime] case fail (deploy): ${testCase.contract} -> ${msg}\n`);
          caseResult.reason = `deploy failed: ${msg}`;
          results.push(caseResult);
          continue;
        }
        process.stdout.write(`[testnet-runtime] deploy skipped (already exists): ${testCase.contract}\n`);
        caseResult.deployTxHash = 'ALREADY_DEPLOYED';
      }

      try {
        // eslint-disable-next-line no-await-in-loop
        await waitForContractState(client, predictedHash);
      } catch (err) {
        process.stdout.write(`[testnet-runtime] case fail (contract state): ${testCase.contract}\n`);
        caseResult.reason = String(err && err.message ? err.message : err);
        results.push(caseResult);
        continue;
      }

      const contract = new experimental.SmartContract(predictedHash, config);
      let failed = false;
      for (const assertion of testCase.assertions || []) {
        const materialized = materialize(assertion.args || [], templateContext);
        const params = materialized.map((v) => toContractParam(v));

        if (assertion.kind === 'write') {
          try {
            // eslint-disable-next-line no-await-in-loop
            const txid = await contract.invoke(assertion.operation, params);
            process.stdout.write(`[testnet-runtime] write tx: ${testCase.contract}.${assertion.operation} -> ${txid}\n`);
            // eslint-disable-next-line no-await-in-loop
            const appLog = await waitForApplicationLog(client, txid);
            const vmstate = appLog?.executions?.[0]?.vmstate || 'UNKNOWN';
            if (vmstate !== 'HALT') {
              caseResult.assertions.push({
                name: assertion.name,
                kind: 'write',
                operation: assertion.operation,
                status: 'fail',
                txHash: txid,
                reason: `write vmstate=${vmstate}`
              });
              caseResult.reason = `write vmstate=${vmstate}`;
              failed = true;
              break;
            }
            caseResult.assertions.push({
              name: assertion.name,
              kind: 'write',
              operation: assertion.operation,
              status: 'pass',
              txHash: txid,
              actual: { vmstate }
            });
            caseResult.assertionsPassed += 1;
          } catch (err) {
            caseResult.assertions.push({
              name: assertion.name,
              kind: 'write',
              operation: assertion.operation,
              status: 'fail',
              reason: String(err && err.message ? err.message : err)
            });
            caseResult.reason = String(err && err.message ? err.message : err);
            failed = true;
            break;
          }
          continue;
        }

        if (assertion.kind === 'read') {
          try {
            // eslint-disable-next-line no-await-in-loop
            const out = await contract.testInvoke(assertion.operation, params);
            if (out.state !== 'HALT') {
              caseResult.assertions.push({
                name: assertion.name,
                kind: 'read',
                operation: assertion.operation,
                status: 'fail',
                reason: `read state=${out.state}`,
                actual: out
              });
              caseResult.reason = `read state=${out.state}`;
              failed = true;
              break;
            }
            const top = Array.isArray(out.stack) && out.stack.length > 0 ? out.stack[0] : null;
            const expected = materializeExpect(assertion.expect || null, templateContext);
            const cmp = compareExpectedStackTop(top, expected);
            if (!cmp.ok) {
              caseResult.assertions.push({
                name: assertion.name,
                kind: 'read',
                operation: assertion.operation,
                status: 'fail',
                reason: cmp.reason,
                expect: expected,
                actual: top
              });
              caseResult.reason = cmp.reason;
              failed = true;
              break;
            }
            caseResult.assertions.push({
              name: assertion.name,
              kind: 'read',
              operation: assertion.operation,
              status: 'pass',
              expect: expected,
              actual: top
            });
            caseResult.assertionsPassed += 1;
          } catch (err) {
            caseResult.assertions.push({
              name: assertion.name,
              kind: 'read',
              operation: assertion.operation,
              status: 'fail',
              reason: String(err && err.message ? err.message : err)
            });
            caseResult.reason = String(err && err.message ? err.message : err);
            failed = true;
            break;
          }
          continue;
        }

        caseResult.assertions.push({
          name: assertion.name,
          kind: assertion.kind || 'unknown',
          operation: assertion.operation || 'unknown',
          status: 'fail',
          reason: `unsupported assertion kind: ${String(assertion.kind)}`
        });
        caseResult.reason = `unsupported assertion kind: ${String(assertion.kind)}`;
        failed = true;
        break;
      }

      if (!failed && caseResult.assertionsPassed === caseResult.assertionsTotal) {
        caseResult.status = 'pass';
        process.stdout.write(`[testnet-runtime] case pass: ${testCase.contract}\n`);
      } else {
        process.stdout.write(`[testnet-runtime] case fail: ${testCase.contract} -> ${caseResult.reason || 'assertion failed'}\n`);
      }

      results.push(caseResult);
    }

    const gasAfter = await getGasBalance(client, account.address);
    const report = {
      generatedAt: new Date().toISOString(),
      rpcAddress: DEFAULT_RPC,
      networkMagic,
      neoSolcVersion: neoSolcVersionText,
      deployerAddress: account.address,
      deployerScriptHash: to0x(account.scriptHash),
      gasBefore,
      gasAfter,
      totals: {
        total: results.length,
        pass: results.filter((row) => row.status === 'pass').length,
        fail: results.filter((row) => row.status === 'fail').length,
        assertionsTotal: results.reduce((n, row) => n + (row.assertionsTotal || 0), 0),
        assertionsPassed: results.reduce((n, row) => n + (row.assertionsPassed || 0), 0)
      },
      results
    };

    fs.mkdirSync(path.dirname(REPORT_JSON_PATH), { recursive: true });
    fs.mkdirSync(path.dirname(REPORT_MD_PATH), { recursive: true });
    fs.writeFileSync(REPORT_JSON_PATH, JSON.stringify(report, null, 2));
    fs.writeFileSync(REPORT_MD_PATH, renderMarkdown(report));

    process.stdout.write(`[testnet-runtime] report json: ${path.relative(ROOT, REPORT_JSON_PATH)}\n`);
    process.stdout.write(`[testnet-runtime] report md:   ${path.relative(ROOT, REPORT_MD_PATH)}\n`);
    process.stdout.write(
      `[testnet-runtime] totals: pass=${report.totals.pass}, fail=${report.totals.fail}, total=${report.totals.total}, assertions=${report.totals.assertionsPassed}/${report.totals.assertionsTotal}\n`
    );

    if (report.totals.fail > 0) {
      process.exitCode = 1;
    }
  } finally {
    try {
      fs.rmSync(workDir, { recursive: true, force: true });
    } catch {
      // ignore cleanup failures
    }
  }
}

main().catch((err) => {
  process.stderr.write(`${err && err.stack ? err.stack : err}\n`);
  process.exit(1);
});
