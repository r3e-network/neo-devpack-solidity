#!/usr/bin/env node
'use strict';

const fs = require('fs');
const os = require('os');
const path = require('path');
const { spawnSync } = require('child_process');

const ROOT = path.resolve(__dirname, '..');
const AUDIT_DIR = process.env.NEO_FAMOUS_AUDIT_DIR || '/tmp/neo-famous-contracts-audit';
const AUDIT_NODE_MODULES = path.join(AUDIT_DIR, 'node_modules');
const REPORT_JSON_PATH = path.join(ROOT, 'docs/data/famous-contracts-neoxp-runtime-results.json');
const REPORT_MD_PATH = path.join(ROOT, 'docs/solidity/famous-contracts-neoxp-runtime.md');

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
        name: 'approve(genesis, 123)',
        kind: 'write',
        operation: 'approve',
        args: [{ type: 'Hash160', value: '$GENESIS_HASH' }, 123]
      },
      {
        name: 'allowance(node1, genesis) == 123',
        kind: 'read',
        operation: 'allowance',
        args: [
          { type: 'Hash160', value: '$NODE1_HASH' },
          { type: 'Hash160', value: '$GENESIS_HASH' }
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
        name: 'setApprovalForAll(genesis, true)',
        kind: 'write',
        operation: 'setApprovalForAll',
        args: [{ type: 'Hash160', value: '$GENESIS_HASH' }, true]
      },
      {
        name: 'isApprovedForAll(node1, genesis) == true',
        kind: 'read',
        operation: 'isApprovedForAll',
        args: [
          { type: 'Hash160', value: '$NODE1_HASH' },
          { type: 'Hash160', value: '$GENESIS_HASH' }
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
        name: 'approve(genesis, 77)',
        kind: 'write',
        operation: 'approve',
        args: [{ type: 'Hash160', value: '$GENESIS_HASH' }, 77]
      },
      {
        name: 'allowance(node1, genesis) == 77',
        kind: 'read',
        operation: 'allowance',
        args: [
          { type: 'Hash160', value: '$NODE1_HASH' },
          { type: 'Hash160', value: '$GENESIS_HASH' }
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
        name: 'approve(genesis, 321)',
        kind: 'write',
        operation: 'approve',
        args: [{ type: 'Hash160', value: '$GENESIS_HASH' }, 321]
      },
      {
        name: 'allowance(node1, genesis) == 321',
        kind: 'read',
        operation: 'allowance',
        args: [
          { type: 'Hash160', value: '$NODE1_HASH' },
          { type: 'Hash160', value: '$GENESIS_HASH' }
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
        name: 'approve(genesis, 456)',
        kind: 'write',
        operation: 'approve',
        args: [{ type: 'Hash160', value: '$GENESIS_HASH' }, 456]
      },
      {
        name: 'allowance(node1, genesis) == 456',
        kind: 'read',
        operation: 'allowance',
        args: [
          { type: 'Hash160', value: '$NODE1_HASH' },
          { type: 'Hash160', value: '$GENESIS_HASH' }
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
    note: 'Original upstream Chainlink mock oracle with end-to-end update/read verification.',
    assertions: [
      {
        name: 'latestAnswer() initial == 123456789',
        kind: 'read',
        operation: 'latestAnswer',
        args: [],
        expect: { type: 'Integer', value: '123456789' }
      },
      {
        name: 'updateAnswer(987654321)',
        kind: 'write',
        operation: 'updateAnswer',
        args: [987654321]
      },
      {
        name: 'latestAnswer() final == 987654321',
        kind: 'read',
        operation: 'latestAnswer',
        args: [],
        expect: { type: 'Integer', value: '987654321' }
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

function extractTxHash(text) {
  const clean = stripAnsi(text);
  const m = clean.match(/0x[0-9a-fA-F]{64}/);
  return m ? m[0] : null;
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
    vendoredPath,
    nefPath,
    manifestPath,
    warningCount: (stripAnsi(compile.stderr).match(/warning/gi) || []).length
  };
}

function deployContract(neoxp, homeDir, chainPath, compiled, testCase) {
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
      reason: 'show deploy transaction failed',
      stderr: stripAnsi(txInfo.stderr)
    };
  }

  const txJson = parseJsonFromMixedOutput(txInfo.stdout);
  if (!txJson) {
    return {
      ok: false,
      reason: 'deploy transaction output is not valid JSON'
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

  return {
    ok: true,
    contractHash,
    txHash
  };
}

function readWalletContext(neoxp, homeDir, chainPath) {
  const out = run(neoxp, ['wallet', 'list', '-i', chainPath, '-j'], {
    env: { ...process.env, HOME: homeDir },
    cwd: ROOT
  });
  if (out.status !== 0) {
    throw new Error(`failed to list wallets:\n${out.stdout}\n${out.stderr}`);
  }
  const json = parseJsonFromMixedOutput(out.stdout);
  if (!json) {
    throw new Error('wallet list output is not valid JSON');
  }

  const node1Default = Array.isArray(json.node1) ? json.node1[0] : null;
  const genesis = json.genesis || null;
  if (!node1Default?.['script-hash'] || !genesis?.['script-hash']) {
    throw new Error('wallet list JSON missing node1/genesis script-hash');
  }

  return {
    NODE1_HASH: node1Default['script-hash'],
    GENESIS_HASH: genesis['script-hash']
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

function buildInvocationPayload(contractHash, operation, args, templateContext) {
  return {
    contract: contractHash,
    operation,
    args: materialize(args || [], templateContext)
  };
}

function runWriteAssertion(neoxp, homeDir, chainPath, invokeFile) {
  const invoke = run(neoxp, ['contract', 'invoke', '-j', '-i', chainPath, invokeFile, 'node1'], {
    env: { ...process.env, HOME: homeDir },
    cwd: ROOT
  });
  if (invoke.status !== 0) {
    return {
      ok: false,
      reason: 'write invoke command failed',
      stdout: stripAnsi(invoke.stdout),
      stderr: stripAnsi(invoke.stderr)
    };
  }

  const txHash = extractTxHash(invoke.stdout);
  if (!txHash) {
    return {
      ok: false,
      reason: 'write invoke did not return tx hash',
      stdout: stripAnsi(invoke.stdout)
    };
  }

  const txInfo = run(neoxp, ['show', 'transaction', '-i', chainPath, txHash], {
    env: { ...process.env, HOME: homeDir },
    cwd: ROOT
  });
  if (txInfo.status !== 0) {
    return {
      ok: false,
      reason: 'show write transaction failed',
      txHash,
      stderr: stripAnsi(txInfo.stderr)
    };
  }

  const txJson = parseJsonFromMixedOutput(txInfo.stdout);
  if (!txJson) {
    return {
      ok: false,
      reason: 'write transaction output is not valid JSON',
      txHash
    };
  }

  const vmstate =
    txJson?.['application-log']?.executions?.[0]?.vmstate ||
    txJson?.['application-log']?.executions?.[0]?.state ||
    'UNKNOWN';
  if (vmstate !== 'HALT') {
    return {
      ok: false,
      reason: `write vmstate=${vmstate}`,
      txHash
    };
  }

  return {
    ok: true,
    txHash,
    vmstate
  };
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

function runReadAssertion(neoxp, homeDir, chainPath, invokeFile, expect) {
  const invoke = run(neoxp, ['contract', 'invoke', '-r', '-j', '-i', chainPath, invokeFile, 'node1'], {
    env: { ...process.env, HOME: homeDir },
    cwd: ROOT
  });
  if (invoke.status !== 0) {
    return {
      ok: false,
      reason: 'read invoke command failed',
      stdout: stripAnsi(invoke.stdout),
      stderr: stripAnsi(invoke.stderr)
    };
  }

  const json = parseJsonFromMixedOutput(invoke.stdout);
  if (!json) {
    return {
      ok: false,
      reason: 'read invoke output is not valid JSON',
      stdout: stripAnsi(invoke.stdout)
    };
  }

  if (json.state !== 'HALT') {
    return {
      ok: false,
      reason: `read state=${json.state || 'UNKNOWN'}`,
      output: json
    };
  }

  const stackTop = Array.isArray(json.stack) && json.stack.length > 0 ? json.stack[0] : null;
  const cmp = compareExpectedStackTop(stackTop, expect);
  if (!cmp.ok) {
    return {
      ok: false,
      reason: cmp.reason,
      output: json
    };
  }

  return {
    ok: true,
    stackTop
  };
}

function renderMarkdown(report) {
  const lines = [];
  lines.push('# Famous EVM Contracts Runtime-Verified on Neo N3 (Type-3)');
  lines.push('');
  lines.push(`- Generated at (UTC): \`${report.generatedAt}\``);
  lines.push('- Snapshot scope: historical Neo Express runtime output; rerun `npm run verify:famous-contracts:neoxp-runtime` before treating these results as current release evidence.');
  lines.push(`- neoxp: \`${report.neoxpVersion}\``);
  lines.push(`- Total contracts: \`${report.totals.total}\``);
  lines.push(`- Pass: \`${report.totals.pass}\``);
  lines.push(`- Fail: \`${report.totals.fail}\``);
  lines.push(`- Assertions passed: \`${report.totals.assertionsPassed}/${report.totals.assertionsTotal}\``);
  lines.push('');
  lines.push('Type-3 criteria in this report:');
  lines.push('- Deploy transaction reaches `HALT`.');
  lines.push('- At least one state-changing invocation reaches `HALT`.');
  lines.push('- Post-state readback equals expected value.');
  lines.push('');
  lines.push('| # | Project | Contract | Result | Assertions | Contract Hash | Source | Note |');
  lines.push('|---:|---|---|---|---:|---|---|---|');

  report.results.forEach((row, idx) => {
    const mark = row.status === 'pass' ? '✅ pass' : '❌ fail';
    const assertions = `${row.assertionsPassed}/${row.assertionsTotal}`;
    const hash = row.contractHash || '-';
    const source = `\`${row.vendoredPath || row.sourcePath || '-'}\``;
    const note = (row.note || row.reason || '-').replace(/\|/g, '\\|');
    lines.push(
      `| ${idx + 1} | ${row.project} | ${row.contract} | ${mark} | ${assertions} | ${hash} | ${source} | ${note} |`
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
      const tx = a.txHash ? ` tx=${a.txHash}` : '';
      const reason = a.reason ? ` reason=${String(a.reason).replace(/`/g, "'")}` : '';
      lines.push(`  - ${mark} ${a.kind} \`${a.operation}\`${tx}${expected}${actual}${reason}`);
    }
    lines.push('');
  });

  lines.push('## Notes');
  lines.push('');
  lines.push('- Source contracts are upstream originals from the referenced package paths.');
  lines.push('- For ERC-style contracts, `supportedstandards` is cleared in manifest to avoid strict NEP schema shape rejection while preserving Solidity source logic.');
  lines.push('- This report intentionally excludes deploy-only/probe-only cases and keeps only runtime-verified positive flows.');

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

  const workDir = fs.mkdtempSync(path.join(os.tmpdir(), 'neo-famous-neoxp-runtime-'));
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

    const templateContext = readWalletContext(neoxp, homeDir, chainPath);
    const results = [];

    for (const testCase of CASES) {
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
        caseResult.reason = compiled.reason;
        results.push(caseResult);
        continue;
      }
      caseResult.warningCount = compiled.warningCount || 0;

      const deployed = deployContract(neoxp, homeDir, chainPath, compiled, testCase);
      if (!deployed.ok) {
        caseResult.reason = deployed.reason;
        caseResult.contractHash = deployed.contractHash || null;
        caseResult.deployTxHash = deployed.txHash || null;
        results.push(caseResult);
        continue;
      }

      caseResult.contractHash = deployed.contractHash;
      caseResult.deployTxHash = deployed.txHash;

      let failed = false;
      for (const assertion of testCase.assertions || []) {
        const invokePayload = buildInvocationPayload(
          deployed.contractHash,
          assertion.operation,
          assertion.args || [],
          templateContext
        );
        const invokeFile = path.join(
          workDir,
          `${sanitizeName(testCase.id)}.${sanitizeName(assertion.operation)}.${assertion.kind}.neo-invoke.json`
        );
        fs.writeFileSync(invokeFile, JSON.stringify(invokePayload, null, 2));

        if (assertion.kind === 'write') {
          const wr = runWriteAssertion(neoxp, homeDir, chainPath, invokeFile);
          if (!wr.ok) {
            caseResult.assertions.push({
              name: assertion.name,
              kind: 'write',
              operation: assertion.operation,
              status: 'fail',
              reason: wr.reason || 'write assertion failed',
              actual: wr
            });
            caseResult.reason = wr.reason || 'write assertion failed';
            failed = true;
            break;
          }

          caseResult.assertions.push({
            name: assertion.name,
            kind: 'write',
            operation: assertion.operation,
            status: 'pass',
            txHash: wr.txHash,
            actual: { vmstate: wr.vmstate }
          });
          caseResult.assertionsPassed += 1;
          continue;
        }

        if (assertion.kind === 'read') {
          const rd = runReadAssertion(neoxp, homeDir, chainPath, invokeFile, assertion.expect || null);
          if (!rd.ok) {
            caseResult.assertions.push({
              name: assertion.name,
              kind: 'read',
              operation: assertion.operation,
              status: 'fail',
              reason: rd.reason || 'read assertion failed',
              expect: assertion.expect || null,
              actual: rd.output || null
            });
            caseResult.reason = rd.reason || 'read assertion failed';
            failed = true;
            break;
          }

          caseResult.assertions.push({
            name: assertion.name,
            kind: 'read',
            operation: assertion.operation,
            status: 'pass',
            expect: assertion.expect || null,
            actual: rd.stackTop || null
          });
          caseResult.assertionsPassed += 1;
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
      }

      results.push(caseResult);
    }

    const report = {
      generatedAt: new Date().toISOString(),
      neoSolcVersion: neoSolcVersionText,
      neoxpVersion: neoxpVersionText,
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

    process.stdout.write(`[neoxp-runtime] report json: ${path.relative(ROOT, REPORT_JSON_PATH)}\n`);
    process.stdout.write(`[neoxp-runtime] report md:   ${path.relative(ROOT, REPORT_MD_PATH)}\n`);
    process.stdout.write(
      `[neoxp-runtime] totals: pass=${report.totals.pass}, fail=${report.totals.fail}, total=${report.totals.total}, assertions=${report.totals.assertionsPassed}/${report.totals.assertionsTotal}\n`
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
