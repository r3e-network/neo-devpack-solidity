#!/usr/bin/env node
'use strict';

const fs = require('fs');
const os = require('os');
const path = require('path');
const { spawnSync } = require('child_process');

const ROOT = path.resolve(__dirname, '..');
const AUDIT_DIR = process.env.NEO_FAMOUS_AUDIT_DIR || '/tmp/neo-famous-contracts-audit';
const AUDIT_NODE_MODULES = path.join(AUDIT_DIR, 'node_modules');
const REPORT_JSON_PATH = path.join(ROOT, 'docs/data/famous-contracts-neoxp-deploy-results.json');
const REPORT_MD_PATH = path.join(ROOT, 'docs/solidity/famous-contracts-neoxp-deploy.md');

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
    probe: { operation: 'name', args: [] },
    clearSupportedStandards: true,
    note: 'Clears supportedstandards for EVM ERC-20 vs NEP-17 strict schema mismatch.'
  },
  {
    id: 'openzeppelin-erc721',
    project: 'OpenZeppelin',
    contract: 'ERC721',
    sourcePath: '@openzeppelin/contracts/token/ERC721/ERC721.sol',
    deployData: ['NeoNFT', 'nNFT'],
    probe: { operation: 'name', args: [] },
    clearSupportedStandards: true,
    note: 'Clears supportedstandards for EVM ERC-721 vs NEP-11 strict schema mismatch.'
  },
  {
    id: 'openzeppelin-accesscontrol',
    project: 'OpenZeppelin',
    contract: 'AccessControl',
    sourcePath: '@openzeppelin/contracts/access/AccessControl.sol',
    probe: { operation: 'DEFAULT_ADMIN_ROLE', args: [] },
    clearSupportedStandards: false,
    note: 'Original upstream contract deployed as-is.'
  },
  {
    id: 'aave-weth9',
    project: 'Aave V3',
    contract: 'WETH9',
    sourcePath: '@aave/core-v3/contracts/dependencies/weth/WETH9.sol',
    probe: { operation: 'name', args: [] },
    clearSupportedStandards: true,
    note: 'Clears supportedstandards to bypass strict NEP method-shape checks.'
  },
  {
    id: 'aave-poolconfigurator',
    project: 'Aave V3',
    contract: 'PoolConfigurator',
    sourcePath: '@aave/core-v3/contracts/protocol/pool/PoolConfigurator.sol',
    clearSupportedStandards: false,
    note: 'Original upstream contract deployed; runtime integration calls require further protocol wiring.'
  },
  {
    id: 'safe-safe',
    project: 'Safe',
    contract: 'Safe',
    sourcePath: '@safe-global/safe-contracts/contracts/Safe.sol',
    probe: { operation: 'getThreshold', args: [] },
    clearSupportedStandards: false,
    note: 'Original upstream multisig core contract deployed.'
  },
  {
    id: 'safe-safel2',
    project: 'Safe',
    contract: 'SafeL2',
    sourcePath: '@safe-global/safe-contracts/contracts/SafeL2.sol',
    probe: { operation: 'getThreshold', args: [] },
    clearSupportedStandards: false,
    note: 'Original upstream L2-flavored Safe contract deployed.'
  },
  {
    id: 'safe-proxyfactory',
    project: 'Safe',
    contract: 'SafeProxyFactory',
    sourcePath: '@safe-global/safe-contracts/contracts/proxies/SafeProxyFactory.sol',
    probe: { operation: 'proxyCreationCode', args: [] },
    clearSupportedStandards: false,
    note: 'Original upstream proxy factory deployed.'
  },
  {
    id: 'safe-multisend',
    project: 'Safe',
    contract: 'MultiSend',
    sourcePath: '@safe-global/safe-contracts/contracts/libraries/MultiSend.sol',
    clearSupportedStandards: false,
    note: 'Original upstream library contract deployed (no zero-arg probe method).'
  },
  {
    id: 'uniswap-v2-pair',
    project: 'Uniswap V2 Core',
    contract: 'UniswapV2Pair',
    sourcePath: '@uniswap/v2-core/contracts/UniswapV2Pair.sol',
    probe: { operation: 'factory', args: [] },
    clearSupportedStandards: true,
    note: 'Clears supportedstandards due ERC-style strict schema mismatch.'
  },
  {
    id: 'uniswap-v2-erc20',
    project: 'Uniswap V2 Core',
    contract: 'UniswapV2ERC20',
    sourcePath: '@uniswap/v2-core/contracts/UniswapV2ERC20.sol',
    probe: { operation: 'name', args: [] },
    clearSupportedStandards: true,
    note: 'Clears supportedstandards due ERC-style strict schema mismatch.'
  },
  {
    id: 'chainlink-mockv3aggregator',
    project: 'Chainlink',
    contract: 'MockV3Aggregator',
    sourcePath: '@chainlink/contracts/src/v0.8/shared/mocks/MockV3Aggregator.sol',
    deployData: [8, 123456789],
    probe: { operation: 'decimals', args: [] },
    clearSupportedStandards: false,
    note: 'Original upstream oracle mock deployed.'
  },
  {
    id: 'chainlink-owneriscreator',
    project: 'Chainlink',
    contract: 'OwnerIsCreator',
    sourcePath: '@chainlink/contracts/src/v0.8/shared/access/OwnerIsCreator.sol',
    probe: { operation: 'owner', args: [] },
    clearSupportedStandards: false,
    note: 'Original upstream access-control primitive deployed.'
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

function parseJsonFromMixedOutput(raw) {
  const text = stripAnsi(raw).trim();
  if (!text) {
    return null;
  }

  try {
    return JSON.parse(text);
  } catch {
    // continue
  }

  const first = text.indexOf('{');
  const last = text.lastIndexOf('}');
  if (first === -1 || last === -1 || last < first) {
    return null;
  }

  const candidate = text.slice(first, last + 1);
  try {
    return JSON.parse(candidate);
  } catch {
    return null;
  }
}

function sanitizeName(input) {
  return String(input).replace(/[^a-zA-Z0-9_.-]+/g, '_');
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

function resolveNeoxp() {
  if (process.env.NEOXP) {
    return process.env.NEOXP;
  }

  const local = path.join(ROOT, 'build/dotnet-tools/neoxp');
  if (fs.existsSync(local)) {
    return local;
  }

  const probe = run('which', ['neoxp']);
  if (probe.status === 0) {
    const found = probe.stdout.trim();
    if (found) {
      return found;
    }
  }

  fs.mkdirSync(path.join(ROOT, 'build/dotnet-tools'), { recursive: true });
  const install = run(
    'dotnet',
    ['tool', 'install', 'Neo.Express', '--tool-path', path.join(ROOT, 'build/dotnet-tools'), '--version', '3.9.1'],
    { cwd: ROOT }
  );
  if (install.status !== 0) {
    throw new Error(`failed to install Neo.Express 3.9.1:\n${install.stdout}\n${install.stderr}`);
  }
  if (!fs.existsSync(local)) {
    throw new Error('neoxp install reported success but binary not found');
  }
  return local;
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
  const vendoredPath = path.join(ROOT, 'third_party/famous-contracts/sources', testCase.sourcePath);
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
      reason: `compile failed`,
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
    vendoredPath,
    nefPath,
    manifestPath,
    warningCount: (stripAnsi(compile.stderr).match(/warning/gi) || []).length
  };
}

function deployCase(neoxp, homeDir, chainPath, compiled, testCase) {
  const deployArgs = ['contract', 'deploy', '-i', chainPath];
  if (Array.isArray(testCase.deployData)) {
    deployArgs.push('-d', JSON.stringify(testCase.deployData));
  }
  deployArgs.push(compiled.nefPath, 'node1', '-j');

  const deploy = run(neoxp, deployArgs, {
    env: { ...process.env, HOME: homeDir },
    cwd: ROOT
  });
  if (deploy.status !== 0) {
    return {
      ok: false,
      reason: 'deploy command failed',
      stderr: stripAnsi(deploy.stderr),
      stdout: stripAnsi(deploy.stdout)
    };
  }

  const deployJson = parseJsonFromMixedOutput(deploy.stdout);
  if (!deployJson) {
    return {
      ok: false,
      reason: 'deploy output is not valid JSON',
      stdout: stripAnsi(deploy.stdout)
    };
  }

  const contractHash = deployJson['contract-hash'];
  const txHash = deployJson['tx-hash'];
  if (!contractHash || !txHash) {
    return {
      ok: false,
      reason: 'deploy JSON missing contract-hash or tx-hash',
      deployJson
    };
  }

  const txInfo = run(neoxp, ['show', 'transaction', '-i', chainPath, txHash], {
    env: { ...process.env, HOME: homeDir },
    cwd: ROOT
  });
  if (txInfo.status !== 0) {
    return {
      ok: false,
      reason: 'show transaction failed',
      stderr: stripAnsi(txInfo.stderr)
    };
  }

  const txJson = parseJsonFromMixedOutput(txInfo.stdout);
  if (!txJson) {
    return {
      ok: false,
      reason: 'show transaction output is not valid JSON'
    };
  }

  const vmstate =
    txJson?.['application-log']?.executions?.[0]?.vmstate ||
    txJson?.['application-log']?.executions?.[0]?.state ||
    'UNKNOWN';
  if (vmstate !== 'HALT') {
    return {
      ok: false,
      reason: `deploy vmstate=${vmstate}`,
      contractHash,
      txHash
    };
  }

  if (!testCase.probe) {
    return {
      ok: true,
      contractHash,
      txHash,
      probeState: 'SKIPPED'
    };
  }

  const invokeFile = path.join(path.dirname(compiled.nefPath), `${sanitizeName(testCase.id)}.invoke.json`);
  fs.writeFileSync(
    invokeFile,
    JSON.stringify(
      {
        contract: contractHash,
        operation: testCase.probe.operation,
        args: Array.isArray(testCase.probe.args) ? testCase.probe.args : []
      },
      null,
      2
    )
  );

  const invoke = run(neoxp, ['contract', 'invoke', '-r', '-j', '-i', chainPath, invokeFile, 'node1'], {
    env: { ...process.env, HOME: homeDir },
    cwd: ROOT
  });
  if (invoke.status !== 0) {
    return {
      ok: false,
      reason: 'probe invoke command failed',
      contractHash,
      txHash,
      stderr: stripAnsi(invoke.stderr),
      stdout: stripAnsi(invoke.stdout)
    };
  }

  const invokeJson = parseJsonFromMixedOutput(invoke.stdout);
  if (!invokeJson) {
    return {
      ok: false,
      reason: 'probe output is not valid JSON',
      contractHash,
      txHash
    };
  }

  const probeState = invokeJson.state || 'UNKNOWN';
  if (probeState !== 'HALT') {
    return {
      ok: false,
      reason: `probe state=${probeState}`,
      contractHash,
      txHash
    };
  }

  return {
    ok: true,
    contractHash,
    txHash,
    probeState
  };
}

function renderMarkdown(report) {
  const lines = [];
  lines.push('# Famous EVM Contracts Deployed on Neo N3 (Neo Express)');
  lines.push('');
  lines.push(`- Generated at (UTC): \`${report.generatedAt}\``);
  lines.push('- Snapshot scope: historical Neo Express deployment output; rerun `npm run deploy:famous-contracts:neoxp` before treating these results as current release evidence.');
  lines.push(`- neoxp: \`${report.neoxpVersion}\``);
  lines.push(`- Total cases: \`${report.totals.total}\``);
  lines.push(`- Pass: \`${report.totals.pass}\``);
  lines.push(`- Fail: \`${report.totals.fail}\``);
  lines.push('');
  lines.push('This matrix deploys upstream contracts (EVM ecosystem) onto Neo N3 via `neo-solc` + `neoxp`.');
  lines.push('For selected ERC-style contracts, `supportedstandards` is cleared in manifest to bypass strict NEP schema checks while keeping source code unchanged.');
  lines.push('');
  lines.push('| # | Project | Contract | Result | Probe | Contract Hash | Source | Note |');
  lines.push('|---:|---|---|---|---|---|---|---|');

  report.results.forEach((row, idx) => {
    const mark = row.status === 'pass' ? '✅ pass' : '❌ fail';
    const probe = row.probeState || '-';
    const hash = row.contractHash || '-';
    const source = `\`${row.vendoredPath || row.sourcePath || '-'}\``;
    const note = (row.note || row.reason || '-').replace(/\|/g, '\\|');
    lines.push(
      `| ${idx + 1} | ${row.project} | ${row.contract} | ${mark} | ${probe} | ${hash} | ${source} | ${note} |`
    );
  });

  lines.push('');
  lines.push('## Notes');
  lines.push('');
  lines.push('- Deployment target in this report is local Neo N3 (`neoxp`) for deterministic CI-style validation.');
  lines.push('- Runtime/business integration tests for protocol-level flows (AMM swaps, Safe execution pipelines, Aave pool wiring) still require additional fixture contracts and state setup.');

  return lines.join('\n');
}

function main() {
  ensureAuditWorkspace();

  const neoSolc = resolveNeoSolc();
  const neoxp = resolveNeoxp();

  const neoSolcVersion = run(neoSolc, ['--version'], { cwd: ROOT });
  const neoxpVersion = run(neoxp, ['--version'], { cwd: ROOT });
  const neoSolcVersionText = neoSolcVersion.status === 0 ? neoSolcVersion.stdout.trim() : 'unknown';
  const neoxpVersionText = neoxpVersion.status === 0 ? neoxpVersion.stdout.trim() : 'unknown';

  const workDir = fs.mkdtempSync(path.join(os.tmpdir(), 'neo-famous-neoxp-'));
  const homeDir = path.join(workDir, 'home');
  const chainPath = path.join(workDir, 'chain.neo-express');
  fs.mkdirSync(homeDir, { recursive: true });

  try {
    const create = run(neoxp, ['create', '-f', '-o', chainPath], {
      env: { ...process.env, HOME: homeDir },
      cwd: ROOT
    });
    if (create.status !== 0) {
      throw new Error(`neoxp create failed:\n${create.stdout}\n${create.stderr}`);
    }

    const transfer = run(neoxp, ['transfer', '-i', chainPath, '200000', 'GAS', 'genesis', 'node1'], {
      env: { ...process.env, HOME: homeDir },
      cwd: ROOT
    });
    if (transfer.status !== 0) {
      throw new Error(`neoxp transfer failed:\n${transfer.stdout}\n${transfer.stderr}`);
    }

    const results = [];
    for (const testCase of CASES) {
      const compiled = compileCase(neoSolc, workDir, testCase);
      if (!compiled.ok) {
        results.push({
          ...testCase,
          status: 'fail',
          reason: compiled.reason,
          sourcePath: testCase.sourcePath,
          vendoredPath: `third_party/famous-contracts/sources/${testCase.sourcePath}`,
          warningCount: compiled.warningCount || 0
        });
        continue;
      }

      const deployed = deployCase(neoxp, homeDir, chainPath, compiled, testCase);
      if (!deployed.ok) {
        results.push({
          ...testCase,
          status: 'fail',
          reason: deployed.reason,
          sourcePath: testCase.sourcePath,
          vendoredPath: `third_party/famous-contracts/sources/${testCase.sourcePath}`,
          compileSourcePath: testCase.sourcePath,
          warningCount: compiled.warningCount || 0,
          contractHash: deployed.contractHash || null,
          txHash: deployed.txHash || null
        });
        continue;
      }

      results.push({
        ...testCase,
        status: 'pass',
        sourcePath: testCase.sourcePath,
        vendoredPath: `third_party/famous-contracts/sources/${testCase.sourcePath}`,
        compileSourcePath: testCase.sourcePath,
        warningCount: compiled.warningCount || 0,
        contractHash: deployed.contractHash,
        txHash: deployed.txHash,
        probeState: deployed.probeState || 'SKIPPED'
      });
    }

    const report = {
      generatedAt: new Date().toISOString(),
      neoSolcVersion: neoSolcVersionText,
      neoxpVersion: neoxpVersionText,
      totals: {
        total: results.length,
        pass: results.filter((row) => row.status === 'pass').length,
        fail: results.filter((row) => row.status === 'fail').length
      },
      results
    };

    fs.mkdirSync(path.dirname(REPORT_JSON_PATH), { recursive: true });
    fs.mkdirSync(path.dirname(REPORT_MD_PATH), { recursive: true });
    fs.writeFileSync(REPORT_JSON_PATH, JSON.stringify(report, null, 2));
    fs.writeFileSync(REPORT_MD_PATH, renderMarkdown(report));

    process.stdout.write(`[neoxp] report json: ${path.relative(ROOT, REPORT_JSON_PATH)}\n`);
    process.stdout.write(`[neoxp] report md:   ${path.relative(ROOT, REPORT_MD_PATH)}\n`);
    process.stdout.write(
      `[neoxp] totals: pass=${report.totals.pass}, fail=${report.totals.fail}, total=${report.totals.total}\n`
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

main();
