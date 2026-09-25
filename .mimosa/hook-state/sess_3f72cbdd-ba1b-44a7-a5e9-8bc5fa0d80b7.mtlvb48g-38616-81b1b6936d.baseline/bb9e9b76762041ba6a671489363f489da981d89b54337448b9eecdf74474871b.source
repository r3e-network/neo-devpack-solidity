#!/usr/bin/env node
'use strict';

/**
 * Standards Mirror — TestNet deploy + invoke pipeline.
 *
 * Reads docs/standards-mirror/deployments/manifest.json. For each pair:
 *   1. Compiles Solidity via target/release/neo-solc
 *   2. Compiles C# via nccs
 *   3. Deploys both NEFs to Neo N3 TestNet
 *   4. Runs each pair's read + write test cases
 *   5. Records contract hashes + tx ids + assertion results
 *
 * Output:
 *   docs/standards-mirror/deployments/results.json   — machine-readable
 *   docs/standards-mirror/deployments/RESULTS.md     — human-readable summary
 *
 * Env:
 *   NEO_TESTNET_WIF   — required, deployer WIF
 *   NEO_TESTNET_RPC   — optional, default http://seed1t5.neo.org:20332
 */

const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');
const { wallet, rpc, sc, tx, u, CONST, experimental } = require('@cityofzion/neon-js');

const ROOT = path.resolve(__dirname, '..');
const DEPLOYMENTS_DIR = path.join(ROOT, 'docs/standards-mirror/deployments');
const MANIFEST_PATH = path.join(DEPLOYMENTS_DIR, 'manifest.json');
const RESULTS_JSON = path.join(DEPLOYMENTS_DIR, 'results.json');
const RESULTS_MD = path.join(DEPLOYMENTS_DIR, 'RESULTS.md');

const RPC_URL = process.env.NEO_TESTNET_RPC || 'http://seed1t5.neo.org:20332';
const WIF = process.env.NEO_TESTNET_WIF;
const NEO_SOLC = path.join(ROOT, 'target/release/neo-solc');
const NCCS = path.join(process.env.HOME || '~', '.dotnet/tools/nccs');

const SUPPRESSED_WARNINGS = ['W101', 'W103', 'W113', 'W200', 'W121', 'W111', 'W116', 'W106', 'W105'];

function stripAnsi(s) {
  return String(s || '').replace(/\[[0-9;]*m/g, '');
}

function run(cmd, args, opts = {}) {
  const res = spawnSync(cmd, args, { encoding: 'utf8', ...opts });
  return res;
}

function compileSolidity(pair) {
  const sourcePath = path.join(DEPLOYMENTS_DIR, pair.solidity.source);
  const outPrefix = path.join('/tmp', pair.solidity.outName);

  const args = [
    sourcePath,
    '-I', path.join(ROOT, 'devpack'),
    '--contract', pair.solidity.contract,
    '-o', outPrefix
  ];
  for (const w of SUPPRESSED_WARNINGS) args.push('--Wno', w);

  const res = run(NEO_SOLC, args, { cwd: ROOT });
  if (res.status !== 0) {
    return { ok: false, reason: 'compile failed', stderr: stripAnsi(res.stderr) };
  }
  const nefPath = `${outPrefix}.nef`;
  const manifestPath = `${outPrefix}.manifest.json`;
  if (!fs.existsSync(nefPath) || !fs.existsSync(manifestPath)) {
    return { ok: false, reason: 'compile output missing' };
  }

  if (pair.solidity.clearSupportedStandards) {
    const m = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
    m.supportedstandards = [];
    fs.writeFileSync(manifestPath, JSON.stringify(m, null, 2));
  }

  return { ok: true, nefPath, manifestPath };
}

function compileCsharp(pair) {
  const csprojPath = path.join(DEPLOYMENTS_DIR, pair.csharp.csproj);
  const outDir = path.join('/tmp', `${pair.id}-cs`);
  fs.mkdirSync(outDir, { recursive: true });

  const res = run(NCCS, [csprojPath, '-o', outDir], { cwd: path.dirname(csprojPath) });
  if (res.status !== 0) {
    return { ok: false, reason: 'compile failed', stderr: stripAnsi(res.stderr) };
  }
  const nefName = path.basename(csprojPath).replace('.csproj', '.nef');
  const manifestName = path.basename(csprojPath).replace('.csproj', '.manifest.json');
  const nefPath = path.join(outDir, nefName);
  const manifestPath = path.join(outDir, manifestName);
  if (!fs.existsSync(nefPath) || !fs.existsSync(manifestPath)) {
    return { ok: false, reason: 'compile output missing' };
  }
  return { ok: true, nefPath, manifestPath };
}

function to0x(hex) {
  if (!hex) return hex;
  const s = String(hex);
  return s.startsWith('0x') ? s : `0x${s}`;
}

function strip0x(hex) {
  const s = String(hex || '');
  return s.startsWith('0x') ? s.slice(2) : s;
}

function reverseScriptHash(value) {
  // Application-log stack values come as base64 strings for ByteString/Hash160.
  // We decode base64, then reverse bytes (LE -> BE) to get the display script hash.
  let s = String(value || '');
  if (s.startsWith('0x')) s = s.slice(2);
  let buf;
  // If it's exactly 40 hex chars (20 bytes), treat as hex.
  if (s.length === 40 && /^[0-9a-fA-F]+$/.test(s)) {
    buf = Buffer.from(s, 'hex');
  } else {
    // base64 (typical for application log stack output)
    buf = Buffer.from(s, 'base64');
  }
  return buf.reverse().toString('hex');
}

// Stack-top "ByteString" decoder: returns hex string of the underlying bytes.
function bytestringToHex(stackVal) {
  let s = String(stackVal || '');
  if (s.length === 40 && /^[0-9a-fA-F]+$/.test(s)) return s;
  return Buffer.from(s, 'base64').toString('hex');
}

async function deploy(client, networkConfig, account, nef, manifest, deployData) {
  let dataParam;
  if (deployData === null || deployData === undefined) {
    dataParam = sc.ContractParam.any(null);
  } else if (Array.isArray(deployData)) {
    // Solidity convention: JSON-encoded array of constructor args, even if empty.
    dataParam = sc.ContractParam.string(JSON.stringify(deployData));
  } else {
    dataParam = sc.ContractParam.any(deployData);
  }
  const args = [
    sc.ContractParam.byteArray(u.HexString.fromHex(nef.serialize(), true)),
    sc.ContractParam.string(JSON.stringify(manifest.toJson())),
    dataParam
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
  await experimental.txHelpers.setBlockExpiry(transaction, networkConfig, networkConfig.blocksTillExpiry);
  transaction.addSigner({ account: account.scriptHash, scopes: 'CalledByEntry' });
  try {
    await experimental.txHelpers.addFees(transaction, networkConfig);
  } catch (e) {
    // Detect "Contract Already Exists: 0x..." and surface as a structured error
    // so the caller can fetch the existing contract hash and skip redeploy.
    const m = String(e.message || e).match(/Contract Already Exists:\s*(0x[a-fA-F0-9]+)/);
    if (m) {
      const err = new Error('contract already exists');
      err.contractHash = m[1].slice(2);
      throw err;
    }
    throw e;
  }
  transaction.sign(account, networkConfig.networkMagic);
  return await client.sendRawTransaction(transaction);
}

async function waitForTx(client, txid, timeoutMs = 180000) {
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {
    try {
      const log = await client.getApplicationLog(txid);
      if (log && Array.isArray(log.executions) && log.executions.length > 0) return log;
    } catch {}
    await new Promise((r) => setTimeout(r, 3000));
  }
  throw new Error(`timeout waiting for tx ${txid}`);
}

async function waitForContract(client, contractHash, timeoutMs = 180000) {
  const started = Date.now();
  const want = to0x(contractHash);
  while (Date.now() - started < timeoutMs) {
    try {
      const state = await client.getContractState(want);
      if (state && state.hash) return state;
    } catch {}
    await new Promise((r) => setTimeout(r, 3000));
  }
  throw new Error(`timeout waiting for contract ${want}`);
}

function materialize(value, ctx) {
  if (Array.isArray(value)) return value.map((v) => materialize(v, ctx));
  if (value && typeof value === 'object') {
    const o = {};
    for (const [k, v] of Object.entries(value)) o[k] = materialize(v, ctx);
    return o;
  }
  if (typeof value === 'string' && /^\$[A-Z0-9_]+$/.test(value)) {
    const k = value.slice(1);
    if (!Object.prototype.hasOwnProperty.call(ctx, k)) {
      throw new Error(`unknown template: ${value}`);
    }
    return ctx[k];
  }
  return value;
}

function toContractParam(value) {
  if (value && typeof value === 'object' && !Array.isArray(value)) {
    const t = String(value.type || '').toLowerCase();
    if (t === 'hash160') return sc.ContractParam.hash160(String(value.value));
    if (t === 'publickey') return sc.ContractParam.publicKey(String(value.value));
    if (t === 'integer') return sc.ContractParam.integer(String(value.value));
    if (t === 'boolean') return sc.ContractParam.boolean(Boolean(value.value));
    if (t === 'string') return sc.ContractParam.string(String(value.value));
    if (t === 'bytearray') {
      const raw = String(value.value);
      return raw.startsWith('0x')
        ? sc.ContractParam.byteArray(u.HexString.fromHex(strip0x(raw)))
        : sc.ContractParam.byteArray(raw);
    }
  }
  if (Array.isArray(value)) return sc.ContractParam.array(...value.map(toContractParam));
  if (typeof value === 'number') return sc.ContractParam.integer(value);
  if (typeof value === 'boolean') return sc.ContractParam.boolean(value);
  if (typeof value === 'string') return sc.ContractParam.string(value);
  if (value === null || value === undefined) return sc.ContractParam.any(null);
  throw new Error(`unsupported param: ${typeof value}`);
}

function decodeBase64ToHex(b64) {
  return Buffer.from(b64, 'base64').toString('hex');
}

function decodeBase64ToBigIntLE(b64) {
  const buf = Buffer.from(b64, 'base64');
  let n = 0n;
  for (let i = buf.length - 1; i >= 0; i--) {
    n = (n << 8n) | BigInt(buf[i]);
  }
  return n;
}

function normalizeStackValue(stackTop, expectType) {
  let val = stackTop.value;
  const actualType = stackTop.type;

  if (expectType === 'Integer') {
    if (actualType === 'Integer') return BigInt(String(val));
    if (actualType === 'ByteString') {
      // base64-encoded LE bytes
      return decodeBase64ToBigIntLE(String(val));
    }
    if (actualType === 'Boolean') return BigInt(val ? 1 : 0);
  }

  if (expectType === 'ByteString') {
    if (actualType === 'ByteString') {
      // Compare as hex (manifest's expect.value is hex)
      return decodeBase64ToHex(String(val));
    }
    if (actualType === 'Integer') {
      return String(val);
    }
  }

  if (expectType === 'Hash160') {
    if (actualType === 'Hash160' || actualType === 'ByteString') {
      // base64 in raw stack → reverse for big-endian display form
      const buf = Buffer.from(String(val), 'base64');
      return buf.reverse().toString('hex');
    }
  }

  if (expectType === 'Boolean') {
    if (actualType === 'Boolean') return Boolean(val);
    if (actualType === 'Integer') return BigInt(val) !== 0n;
  }

  return val;
}

function compareExpected(stackTop, expect) {
  if (!expect) return { ok: true };
  if (!stackTop) return { ok: false, reason: 'empty stack' };
  if (!Object.prototype.hasOwnProperty.call(expect, 'value')) return { ok: true };

  const normalizedActual = normalizeStackValue(stackTop, expect.type);

  let expectedNormalized;
  if (expect.type === 'Integer') expectedNormalized = BigInt(String(expect.value));
  else if (expect.type === 'Boolean') expectedNormalized = Boolean(expect.value);
  else expectedNormalized = String(expect.value);

  const actualStr = String(normalizedActual);
  const expectStr = String(expectedNormalized);

  if (actualStr !== expectStr) {
    return { ok: false, reason: `actual=${actualStr} (raw type=${stackTop.type}, value=${stackTop.value}), expected ${expectStr}` };
  }
  return { ok: true };
}

async function invokeRead(client, contractHash, op, args) {
  const params = args.map(toContractParam).map((p) => p.toJson());
  return await client.invokeFunction(to0x(contractHash), op, params);
}

async function invokeWrite(client, networkConfig, account, contractHash, op, args) {
  const params = args.map(toContractParam);
  const builder = new sc.ScriptBuilder();
  builder.emitContractCall({
    scriptHash: contractHash,
    operation: op,
    callFlags: sc.CallFlags.All,
    args: params
  });
  const transaction = new tx.Transaction();
  transaction.script = u.HexString.fromHex(builder.build());
  await experimental.txHelpers.setBlockExpiry(transaction, networkConfig, networkConfig.blocksTillExpiry);
  transaction.addSigner({ account: account.scriptHash, scopes: 'CalledByEntry' });
  await experimental.txHelpers.addFees(transaction, networkConfig);
  transaction.sign(account, networkConfig.networkMagic);
  return await client.sendRawTransaction(transaction);
}

async function deployAndTest({ tag, compile, deployData, tests, pair, ctx, client, account, networkConfig }) {
  if (!compile.ok) {
    process.stdout.write(`  [${tag}] ❌ compile: ${compile.reason}\n`);
    if (compile.stderr) process.stdout.write(`        ${compile.stderr.split('\n').slice(0, 3).join('\n        ')}\n`);
    return { status: 'compile-fail', reason: compile.reason };
  }
  const nef = sc.NEF.fromBuffer(fs.readFileSync(compile.nefPath));
  const cm = sc.ContractManifest.fromJson(JSON.parse(fs.readFileSync(compile.manifestPath, 'utf8')));

  // Precompute contract hash deterministically from (deployer, nef.checksum, manifest.name)
  // to avoid parsing it out of application logs (which return a Contract struct).
  // Sender must be passed as HexString WITHOUT the littleEndian=true flag here —
  // experimental.getContractHash already handles the byte order internally.
  const expectedHash = experimental.getContractHash(
    u.HexString.fromHex(account.scriptHash),
    nef.checksum,
    cm.name
  ).toString();
  let contractHashHex = expectedHash.startsWith('0x') ? expectedHash.slice(2) : expectedHash;
  let deployTx = null;
  let reused = false;

  try {
    const txid = await deploy(client, networkConfig, account, nef, cm, deployData);
    const log = await waitForTx(client, txid);
    const exec = log.executions[0];
    if (!exec || exec.vmstate !== 'HALT') {
      return { status: 'deploy-fail', reason: exec ? exec.exception : 'no execution', deployTx: txid };
    }
    deployTx = txid;
    process.stdout.write(`  [${tag}] ✅ deployed: ${contractHashHex} tx=${txid}\n`);
  } catch (e) {
    if (e.contractHash) {
      reused = true;
      process.stdout.write(`  [${tag}] ♻️  reused existing: ${contractHashHex}\n`);
    } else {
      process.stdout.write(`  [${tag}] ❌ deploy: ${e.message || e}\n`);
      return { status: 'deploy-fail', reason: String(e.message || e) };
    }
  }

  const contractAddress = wallet.getAddressFromScriptHash(contractHashHex);
  try {
    await waitForContract(client, contractHashHex);
  } catch (e) {
    return {
      status: 'liveness-fail',
      reason: String(e.message || e),
      contractHash: contractHashHex,
      contractAddress,
      deployTx,
      reused,
      tests: []
    };
  }
  const testResults = await runTests(pair.id, tag, ctx, client, account, networkConfig, contractHashHex, tests || []);
  const failedTests = testResults.filter((t) => t.status !== 'pass').length;
  return {
    status: failedTests === 0 ? 'deployed' : 'test-fail',
    contractHash: contractHashHex,
    contractAddress,
    deployTx,
    reused,
    tests: testResults
  };
}

async function runTests(pairId, kind, ctx, client, account, networkConfig, contractHash, tests) {
  const results = [];
  for (const t of tests) {
    const args = (t.args || []).map((a) => materialize(a, ctx));
    if (t.kind === 'read') {
      try {
        const expect = materialize(t.expect, ctx);
        const res = await invokeRead(client, contractHash, t.operation, args);
        const stackTop = res && res.stack && res.stack[0];
        const cmp = compareExpected(stackTop, expect);
        results.push({
          name: t.name,
          kind: 'read',
          operation: t.operation,
          status: cmp.ok ? 'pass' : 'fail',
          actual: stackTop || null,
          expect: expect || null,
          reason: cmp.ok ? undefined : cmp.reason
        });
        process.stdout.write(`  [${kind}] ${cmp.ok ? '✅' : '❌'} ${t.name}\n`);
      } catch (e) {
        results.push({ name: t.name, kind: 'read', operation: t.operation, status: 'fail', reason: String(e.message || e) });
        process.stdout.write(`  [${kind}] ❌ ${t.name} — ${e.message || e}\n`);
      }
    } else if (t.kind === 'write') {
      try {
        const txid = await invokeWrite(client, networkConfig, account, contractHash, t.operation, args);
        const log = await waitForTx(client, txid);
        const exec = log.executions[0];
        const ok = exec && exec.vmstate === 'HALT';
        results.push({
          name: t.name,
          kind: 'write',
          operation: t.operation,
          status: ok ? 'pass' : 'fail',
          txHash: txid,
          vmstate: exec ? exec.vmstate : null,
          gasconsumed: exec ? exec.gasconsumed : null
        });
        process.stdout.write(`  [${kind}] ${ok ? '✅' : '❌'} ${t.name} — tx=${txid}\n`);
      } catch (e) {
        results.push({ name: t.name, kind: 'write', operation: t.operation, status: 'fail', reason: String(e.message || e) });
        process.stdout.write(`  [${kind}] ❌ ${t.name} — ${e.message || e}\n`);
      }
    }
  }
  return results;
}

function collectFailures(out) {
  const failures = [];
  for (const pair of out.pairs) {
    for (const which of ['solidity', 'csharp']) {
      const result = pair[which] || {};
      if (result.status && result.status !== 'deployed') {
        failures.push(`${pair.id}/${which}: ${result.status}${result.reason ? ` (${result.reason})` : ''}`);
      }
      for (const test of result.tests || []) {
        if (test.status !== 'pass') {
          failures.push(`${pair.id}/${which}/${test.name || test.operation}: ${test.reason || test.vmstate || 'failed'}`);
        }
      }
    }
  }
  return failures;
}

async function main() {
  if (process.argv.includes('--render-only')) {
    if (!fs.existsSync(RESULTS_JSON)) throw new Error(`missing ${RESULTS_JSON}`);
    const out = JSON.parse(fs.readFileSync(RESULTS_JSON, 'utf8'));
    fs.writeFileSync(RESULTS_MD, renderMarkdown(out));
    process.stdout.write(`Wrote ${RESULTS_MD}\n`);
    return;
  }

  if (!WIF) throw new Error('NEO_TESTNET_WIF required');
  if (!fs.existsSync(NEO_SOLC)) {
    process.stdout.write('Building neo-solc...\n');
    const b = run('cargo', ['build', '--release', '--bin', 'neo-solc'], { cwd: ROOT });
    if (b.status !== 0) throw new Error(`failed to build neo-solc:\n${b.stderr}`);
  }

  const account = new wallet.Account(WIF);
  const client = new rpc.RPCClient(RPC_URL);
  const version = await client.getVersion();
  const networkMagic = version?.protocol?.network;
  if (!networkMagic) throw new Error('failed to read network magic');
  const NEO_TESTNET_MAGIC = 894710606;
  if (networkMagic !== NEO_TESTNET_MAGIC) {
    throw new Error(
      `refusing to deploy: RPC ${RPC_URL} reports network magic ${networkMagic} ` +
      `but this script is hard-pinned to TestNet (${NEO_TESTNET_MAGIC}). ` +
      `Override only after auditing scripts/standards_mirror_testnet.js for any mainnet-unsafe assumptions.`
    );
  }

  const networkConfig = {
    networkMagic,
    rpcAddress: RPC_URL,
    account,
    blocksTillExpiry: 240
  };

  const ctx = {
    DEPLOYER_ADDRESS: account.address,
    DEPLOYER_ADDRESS_HEX: account.scriptHash, // little-endian hex
    DEPLOYER_PUBLIC_KEY: account.publicKey
  };

  const manifest = JSON.parse(fs.readFileSync(MANIFEST_PATH, 'utf8'));
  const out = {
    generatedAt: new Date().toISOString(),
    rpc: RPC_URL,
    networkMagic,
    deployer: account.address,
    pairs: []
  };

  for (const pair of manifest.pairs) {
    process.stdout.write(`\n=== ${pair.title} (${pair.id}) ===\n`);
    const pairResult = { id: pair.id, title: pair.title, solidity: {}, csharp: {} };

    pairResult.solidity = await deployAndTest({
      tag: 'sol',
      compile: compileSolidity(pair),
      deployData: pair.solidity.deployData,
      tests: pair.solidity.tests,
      pair, ctx, client, account, networkConfig
    });

    pairResult.csharp = await deployAndTest({
      tag: 'cs ',
      compile: compileCsharp(pair),
      deployData: pair.csharp.deployData,
      tests: pair.csharp.tests,
      pair, ctx, client, account, networkConfig
    });

    out.pairs.push(pairResult);

    // Persist after each pair so partial progress is preserved
    fs.writeFileSync(RESULTS_JSON, JSON.stringify(out, null, 2));
  }

  // Render markdown summary
  const md = renderMarkdown(out);
  fs.writeFileSync(RESULTS_MD, md);
  process.stdout.write(`\nWrote ${RESULTS_JSON}\nWrote ${RESULTS_MD}\n`);

  const failures = collectFailures(out);
  if (failures.length > 0) {
    process.stderr.write(`\n${failures.length} deployment/assertion failure(s):\n`);
    for (const f of failures) process.stderr.write(`- ${f}\n`);
    process.exitCode = 1;
  }
}

function renderMarkdown(out) {
  const lines = [];
  lines.push('# Standards Mirror — TestNet Deployments');
  lines.push('');
  lines.push(`- Generated: \`${out.generatedAt}\``);
  lines.push(`- RPC: \`${out.rpc}\``);
  lines.push(`- Network magic: \`${out.networkMagic}\``);
  lines.push(`- Deployer: \`${out.deployer}\``);
  lines.push('');
  lines.push('## Summary');
  lines.push('');
  lines.push('| Pair | Implementation | Address | Deploy Tx | Tests |');
  lines.push('|---|---|---|---|---|');
  for (const p of out.pairs) {
    for (const which of ['solidity', 'csharp']) {
      const r = p[which];
      const passed = (r.tests || []).filter((t) => t.status === 'pass').length;
      const total = (r.tests || []).length;
      const status = r.status || '-';
      lines.push(
        `| ${p.title} | ${which} | \`${r.contractAddress || '-'}\` | \`${r.deployTx || '-'}\` | ${passed}/${total} (${status}) |`
      );
    }
  }
  lines.push('');
  lines.push('## Failed Assertions');
  lines.push('');
  const failures = [];
  for (const p of out.pairs) {
    for (const which of ['solidity', 'csharp']) {
      const r = p[which] || {};
      if (r.status && r.status !== 'deployed' && (!r.tests || r.tests.length === 0)) {
        failures.push([p.title, which, r.status, '-', r.reason || 'deployment failed']);
      }
      for (const t of r.tests || []) {
        if (t.status !== 'pass') {
          failures.push([p.title, which, t.name || '-', t.operation || '-', t.reason || t.vmstate || 'failed']);
        }
      }
    }
  }

  if (failures.length === 0) {
    lines.push('No deployment or assertion failures were recorded.');
  } else {
    lines.push('| Pair | Implementation | Test | Operation | Reason |');
    lines.push('|---|---|---|---|---|');
    for (const row of failures) {
      lines.push(`| ${row.map(mdCell).join(' | ')} |`);
    }
  }
  lines.push('');
  lines.push('## Full Result Data');
  lines.push('');
  lines.push(
    'The full per-test payload is stored in [`results.json`](https://github.com/r3e-network/neo-devpack-solidity/blob/main/docs/standards-mirror/deployments/results.json).'
  );
  return lines.join('\n');
}

function mdCell(value) {
  const text = String(value || '-')
    .replace(/\r?\n/g, ' ')
    .replace(/[\u0000-\u001f\u007f]/g, ' ')
    .replace(/\uFFFD/g, '')
    .replace(/\s+/g, ' ')
    .trim();
  const alreadyClaimed = text.match(/([A-Za-z][A-Za-z0-9]*: already claimed)/);
  if (alreadyClaimed) {
    return alreadyClaimed[1].replace(/\|/g, '\\|');
  }
  const clipped = text.length > 240 ? `${text.slice(0, 237)}...` : text;
  return clipped.replace(/\|/g, '\\|');
}

main().catch((e) => {
  process.stderr.write(`error: ${e.message || e}\n`);
  process.exit(1);
});
