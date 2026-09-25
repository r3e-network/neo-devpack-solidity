#!/usr/bin/env node
'use strict';

/**
 * GitHub Solidity Contracts → Neo N3 Pipeline
 *
 * Collects Solidity contracts from GitHub repos (or local paths),
 * compiles them with neo-solc, deploys to Neo-Express, and runs
 * runtime assertions.
 *
 * Modes:
 *   collect   — Clone/download contracts from GitHub or npm
 *   compile   — Compile .sol files to .nef + .manifest.json
 *   deploy    — Create neoxp chain and deploy compiled contracts
 *   test      — Run read/write assertions against deployed contracts
 *   pipeline  — Run collect → compile → deploy → test end-to-end
 *
 * Usage:
 *   node scripts/github_contracts_pipeline.js collect --repo OpenZeppelin/openzeppelin-contracts --branch master --out ./contracts
 *   node scripts/github_contracts_pipeline.js compile --in ./contracts --out ./build
 *   node scripts/github_contracts_pipeline.js deploy --in ./build --chain ./chain.neo-express
 *   node scripts/github_contracts_pipeline.js test --chain ./chain.neo-express --manifest ./test-manifest.json
 *   node scripts/github_contracts_pipeline.js pipeline --repo OpenZeppelin/openzeppelin-contracts --config ./pipeline.json
 */

const fs = require('fs');
const os = require('os');
const path = require('path');
const { spawnSync } = require('child_process');

const ROOT = path.resolve(__dirname, '..');

// ---------------------------------------------------------------------------
// CLI helpers
// ---------------------------------------------------------------------------

function parseArgs(argv) {
  const mode = argv[2];
  const args = {};
  for (let i = 3; i < argv.length; i++) {
    const arg = argv[i];
    if (arg.startsWith('--')) {
      const key = arg.slice(2);
      const next = argv[i + 1];
      if (next && !next.startsWith('--')) {
        setArgValue(args, key, next);
        i++;
      } else {
        setArgValue(args, key, true);
      }
    }
  }
  return { mode, args };
}

function setArgValue(args, key, value) {
  if (args[key] === undefined) {
    args[key] = value;
    return;
  }
  for (let i = 2; ; i++) {
    const numbered = `${key}${i}`;
    if (args[numbered] === undefined) {
      args[numbered] = value;
      return;
    }
  }
}

function normalizeRepeatableArgs(args, keys) {
  const normalized = { ...args };
  for (const key of keys) {
    const values = [];
    for (let i = 1; ; i++) {
      const numbered = i === 1 ? key : `${key}${i}`;
      if (normalized[numbered] === undefined) break;
      const raw = normalized[numbered];
      if (Array.isArray(raw)) {
        values.push(...raw);
      } else {
        values.push(raw);
      }
    }

    for (const existing of Object.keys(normalized)) {
      const suffix = existing.slice(key.length);
      if (existing === key || (existing.startsWith(key) && /^\d+$/.test(suffix))) {
        delete normalized[existing];
      }
    }

    values
      .filter(value => value !== undefined && value !== null && value !== false)
      .forEach((value, index) => {
        normalized[index === 0 ? key : `${key}${index + 1}`] = value;
      });
  }
  return normalized;
}

function usage() {
  console.log(`
GitHub Solidity Contracts → Neo N3 Pipeline

Modes:
  collect   Download/clone contracts from GitHub or npm
  compile   Compile .sol files with neo-solc
  deploy    Create neoxp chain and deploy compiled contracts
  test      Run runtime assertions against deployed contracts
  pipeline  Full end-to-end: collect → compile → deploy → test

Collect:
  --repo <owner/repo>      GitHub repository to clone
  --branch <name>          Git branch (default: main)
  --path <dir>             Sub-directory within repo to extract (default: .)
  --npm <pkg[@ver]>        NPM package to install (alternative to --repo)
  --out <dir>              Output directory for collected contracts

Compile:
  --in <dir>               Directory containing .sol files
  --out <dir>              Directory for .nef + .manifest.json output
  --import <dir>           Additional import path (repeatable)
  --contract <name>        Specific contract name to compile (repeatable)
  --Wno <code>             Suppress warning code (repeatable)
  --O0|--O1|--O2|--O3      Optimization level
  --fail-on-partial        Exit non-zero if any target fails

Deploy:
  --in <dir>               Directory with compiled .nef/.manifest.json files
  --chain <file>           Neo-Express chain file path
  --account <name>         Deployer account (default: node1)
  --data <json>            Constructor args as JSON array
  --clear-standards        Clear supportedstandards in manifest before deploy
  --fail-on-partial        Exit non-zero if any target fails

Test:
  --chain <file>           Neo-Express chain file path
  --manifest <file>        JSON test manifest (see docs below)
  --account <name>         Account for invoke transactions (default: node1)
  --fail-on-partial        Exit non-zero if any assertion suite fails

Pipeline:
  --config <file>          Pipeline config JSON (see docs below)
  --repo <owner/repo>      GitHub repo (if not using --config)
  --out <dir>              Working directory (default: ./pipeline-out)
  --fail-on-partial        Exit non-zero if compile/deploy/test has any failure

Examples:
  # Collect OpenZeppelin ERC20
  node scripts/github_contracts_pipeline.js collect \\
    --repo OpenZeppelin/openzeppelin-contracts --branch master \\
    --path contracts/token/ERC20 --out ./my-contracts

  # Compile everything in ./my-contracts
  node scripts/github_contracts_pipeline.js compile \\
    --in ./my-contracts --out ./my-build \\
    --import ./devpack

  # Deploy to a fresh Neo-Express chain
  node scripts/github_contracts_pipeline.js deploy \\
    --in ./my-build --chain ./my-chain.neo-express

  # Full pipeline with config
  node scripts/github_contracts_pipeline.js pipeline --config ./my-pipeline.json
`);
}

// ---------------------------------------------------------------------------
// Shell / process helpers
// ---------------------------------------------------------------------------

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
  return String(input || '').replace(/\u001b\[[0-9;]*m/g, '');
}

function parseJsonFromMixedOutput(raw) {
  const text = stripAnsi(raw).trim();
  if (!text) return null;
  try {
    return JSON.parse(text);
  } catch {
    const first = text.indexOf('{');
    const last = text.lastIndexOf('}');
    if (first === -1 || last === -1 || last < first) return null;
    try {
      return JSON.parse(text.slice(first, last + 1));
    } catch {
      return null;
    }
  }
}

function sanitizeName(input) {
  return String(input).replace(/[^a-zA-Z0-9_.-]+/g, '_');
}

function optionEnabled(value) {
  if (value === undefined || value === null) return false;
  if (typeof value === 'boolean') return value;
  return ['1', 'true', 'yes', 'on'].includes(String(value).toLowerCase());
}

function failOnPartialEnabled(args, config = null) {
  return optionEnabled(args['fail-on-partial'])
    || optionEnabled(args.failOnPartial)
    || optionEnabled(config?.['fail-on-partial'])
    || optionEnabled(config?.failOnPartial)
    || optionEnabled(config?.pipeline?.['fail-on-partial'])
    || optionEnabled(config?.pipeline?.failOnPartial)
    || optionEnabled(process.env.NEO_PIPELINE_FAIL_ON_PARTIAL);
}

function enforceCompletePass(label, results, isPass, failOnPartial) {
  if (!failOnPartial) return;
  const failures = results.filter(result => !isPass(result));
  if (results.length === 0 || failures.length > 0) {
    throw new Error(`${label} failed strict gate: ${results.length - failures.length}/${results.length} passed`);
  }
}

// ---------------------------------------------------------------------------
// Resolvers
// ---------------------------------------------------------------------------

function resolveNeoSolc() {
  if (process.env.NEO_SOLC) return process.env.NEO_SOLC;
  const releaseBin = path.join(ROOT, 'target/release/neo-solc');
  if (fs.existsSync(releaseBin)) return releaseBin;
  const debugBin = path.join(ROOT, 'target/debug/neo-solc');
  if (fs.existsSync(debugBin)) return debugBin;

  console.log('[pipeline] Building neo-solc (release)...');
  const build = run('cargo', ['build', '--release', '--bin', 'neo-solc'], { cwd: ROOT });
  if (build.status !== 0) {
    throw new Error(`Failed to build neo-solc:\n${build.stdout}\n${build.stderr}`);
  }
  if (!fs.existsSync(releaseBin)) {
    throw new Error('neo-solc binary missing after build');
  }
  return releaseBin;
}

function resolveNeoxp() {
  if (process.env.NEOXP) return process.env.NEOXP;
  const local = path.join(ROOT, 'build/dotnet-tools/neoxp/neoxp');
  if (fs.existsSync(local)) return local;

  const which = run('which', ['neoxp']);
  if (which.status === 0 && which.stdout.trim()) return which.stdout.trim();

  console.log('[pipeline] Installing neo.express dotnet tool...');
  fs.mkdirSync(path.join(ROOT, 'build/dotnet-tools'), { recursive: true });
  const install = run(
    'dotnet',
    ['tool', 'install', 'neo.express', '--tool-path', path.join(ROOT, 'build/dotnet-tools/neoxp'), '--version', '3.9.1'],
    { cwd: ROOT }
  );
  if (install.status !== 0) {
    throw new Error(`Failed to install neo.express:\n${install.stdout}\n${install.stderr}`);
  }
  return local;
}

// ---------------------------------------------------------------------------
// Mode: collect
// ---------------------------------------------------------------------------

function modeCollect(args) {
  const outDir = args.out || './collected-contracts';
  fs.mkdirSync(outDir, { recursive: true });

  if (args.npm) {
    // Install npm package and extract .sol files
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'neo-pipeline-npm-'));
    console.log(`[collect] Installing npm package: ${args.npm}`);
    const install = run('npm', ['install', args.npm, '--prefix', tmpDir], { cwd: ROOT });
    if (install.status !== 0) {
      throw new Error(`npm install failed:\n${install.stderr}`);
    }

    const nodeModules = path.join(tmpDir, 'node_modules');
    // Parse npm package name: handle scoped (@scope/pkg@ver) and unscoped (pkg@ver)
    const npmSpec = args.npm;
    let pkgDir;
    if (npmSpec.startsWith('@')) {
      // Scoped: @scope/name@version → @scope/name
      pkgDir = npmSpec.replace(/^(@[^/]+\/[^@]+)@.*/, '$1');
    } else {
      // Unscoped: name@version → name
      pkgDir = npmSpec.split('@')[0];
    }
    const pkgPath = path.join(nodeModules, pkgDir);
    const srcPath = args.path || '.';
    const fullSrcPath = path.join(pkgPath, srcPath);

    if (!fs.existsSync(fullSrcPath)) {
      throw new Error(`Path not found in package: ${srcPath}`);
    }

    const solFiles = findSolFiles(fullSrcPath);
    console.log(`[collect] Found ${solFiles.length} .sol files in ${args.npm}`);
    for (const f of solFiles) {
      const rel = path.relative(fullSrcPath, f);
      const dest = path.join(outDir, sanitizeName(pkgDir), rel);
      fs.mkdirSync(path.dirname(dest), { recursive: true });
      fs.copyFileSync(f, dest);
    }
    console.log(`[collect] Copied to: ${outDir}`);
    return outDir;
  }

  if (args.repo) {
    // Clone GitHub repo
    const repoUrl = `https://github.com/${args.repo}.git`;
    const branch = args.branch || 'main';
    const srcPath = args.path || '.';
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'neo-pipeline-git-'));
    const cloneDir = path.join(tmpDir, 'repo');

    console.log(`[collect] Cloning ${repoUrl} (branch: ${branch})...`);
    const clone = run('git', ['clone', '--depth', '1', '--branch', branch, repoUrl, cloneDir], { cwd: ROOT });
    if (clone.status !== 0) {
      throw new Error(`git clone failed:\n${clone.stderr}`);
    }

    const fullSrcPath = path.join(cloneDir, srcPath);
    if (!fs.existsSync(fullSrcPath)) {
      throw new Error(`Path not found in repo: ${srcPath}`);
    }

    const solFiles = findSolFiles(fullSrcPath);
    console.log(`[collect] Found ${solFiles.length} .sol files in ${args.repo}`);
    for (const f of solFiles) {
      const rel = path.relative(fullSrcPath, f);
      const dest = path.join(outDir, sanitizeName(args.repo.replace('/', '_')), rel);
      fs.mkdirSync(path.dirname(dest), { recursive: true });
      fs.copyFileSync(f, dest);
    }
    console.log(`[collect] Copied to: ${outDir}`);
    return outDir;
  }

  console.error('[collect] Error: specify --repo <owner/repo> or --npm <package>');
  process.exit(1);
}

function findSolFiles(dir) {
  const results = [];
  function walk(d) {
    for (const entry of fs.readdirSync(d, { withFileTypes: true })) {
      const full = path.join(d, entry.name);
      if (entry.isDirectory()) {
        // Skip node_modules and common non-source dirs
        if (entry.name === 'node_modules' || entry.name === '.git' || entry.name === 'test' || entry.name === 'tests') continue;
        walk(full);
      } else if (entry.name.endsWith('.sol')) {
        results.push(full);
      }
    }
  }
  walk(dir);
  return results;
}

// ---------------------------------------------------------------------------
// Mode: compile
// ---------------------------------------------------------------------------

function modeCompile(args) {
  args = normalizeRepeatableArgs(args, ['import', 'contract', 'Wno']);
  const inDir = args.in || args['in'];
  const outDir = args.out || './build';
  if (!inDir) {
    console.error('[compile] Error: specify --in <dir>');
    process.exit(1);
  }
  if (!fs.existsSync(inDir)) {
    console.error(`[compile] Error: input directory not found: ${inDir}`);
    process.exit(1);
  }

  const neoSolc = resolveNeoSolc();
  fs.mkdirSync(outDir, { recursive: true });

  // Gather import paths
  const imports = [];
  for (let i = 1; args[`import${i}`] || (i === 1 && args.import); i++) {
    const val = i === 1 ? args.import : args[`import${i}`];
    if (!val) break;
    imports.push(val);
  }
  // Also add devpack by default
  const devpackDir = path.join(ROOT, 'devpack');
  if (fs.existsSync(devpackDir) && !imports.includes(devpackDir)) {
    imports.push(devpackDir);
  }

  // Gather sol files
  const solFiles = findSolFiles(inDir);
  console.log(`[compile] Found ${solFiles.length} .sol files in ${inDir}`);

  // If specific contracts requested, filter
  let targets = solFiles;
  if (args.contract) {
    const wanted = new Set();
    for (let i = 1; args[`contract${i}`] || (i === 1 && args.contract); i++) {
      const val = i === 1 ? args.contract : args[`contract${i}`];
      if (!val) break;
      wanted.add(val);
    }
    targets = solFiles.filter(f => {
      const name = path.basename(f, '.sol');
      return wanted.has(name);
    });
  }

  // Build suppressed warnings list
  const suppressed = [];
  for (let i = 1; args[`Wno${i}`] || (i === 1 && args.Wno); i++) {
    const val = i === 1 ? args.Wno : args[`Wno${i}`];
    if (!val) break;
    suppressed.push(val);
  }

  // Optimization level
  const optLevel = args.O3 ? '-O3' : args.O2 ? '-O2' : args.O1 ? '-O1' : args.O0 ? '-O0' : '-O1';

  const results = [];
  for (const solFile of targets) {
    const contractName = path.basename(solFile, '.sol');
    const outPrefix = path.join(outDir, sanitizeName(contractName));
    const compileArgs = [solFile, optLevel, '-o', outPrefix];

    for (const imp of imports) {
      compileArgs.push('-I', imp);
    }
    for (const w of suppressed) {
      compileArgs.push('--Wno', w);
    }
    compileArgs.push('--contract', contractName);

    console.log(`[compile] ${contractName} ...`);
    const compile = run(neoSolc, compileArgs, { cwd: ROOT });
    const nefPath = `${outPrefix}.nef`;
    const manifestPath = `${outPrefix}.manifest.json`;
    const ok = compile.status === 0 && fs.existsSync(nefPath) && fs.existsSync(manifestPath);

    results.push({
      contract: contractName,
      source: solFile,
      ok,
      nef: ok ? nefPath : null,
      manifest: ok ? manifestPath : null,
      stderr: stripAnsi(compile.stderr)
    });

    if (ok) {
      console.log(`[compile]   ✅ ${contractName}`);
    } else {
      console.log(`[compile]   ❌ ${contractName}`);
      if (compile.stderr) console.log(compile.stderr);
    }
  }

  // Write compile report
  const reportPath = path.join(outDir, 'compile-report.json');
  fs.writeFileSync(reportPath, JSON.stringify({ generatedAt: new Date().toISOString(), results }, null, 2));
  console.log(`[compile] Report written to: ${reportPath}`);

  const passCount = results.filter(r => r.ok).length;
  console.log(`[compile] ${passCount}/${results.length} compiled successfully`);
  enforceCompletePass('compile', results, r => r.ok, failOnPartialEnabled(args));
  return results;
}

// ---------------------------------------------------------------------------
// Mode: deploy
// ---------------------------------------------------------------------------

function modeDeploy(args) {
  const inDir = args.in || args['in'];
  const chainPath = args.chain;
  const account = args.account || 'node1';
  const deployData = args.data ? JSON.parse(args.data) : null;
  const clearStandards = args['clear-standards'] || false;

  if (!inDir) {
    console.error('[deploy] Error: specify --in <dir>');
    process.exit(1);
  }
  if (!chainPath) {
    console.error('[deploy] Error: specify --chain <file.neo-express>');
    process.exit(1);
  }

  const neoxp = resolveNeoxp();
  const workDir = path.dirname(chainPath);
  fs.mkdirSync(workDir, { recursive: true });
  const homeDir = path.join(workDir, 'neoxp-home');
  fs.mkdirSync(homeDir, { recursive: true });

  // Create chain if it doesn't exist
  if (!fs.existsSync(chainPath)) {
    console.log(`[deploy] Creating Neo-Express chain: ${chainPath}`);
    const create = run(neoxp, ['create', '-f', '-o', chainPath], {
      env: { ...process.env, HOME: homeDir },
      cwd: ROOT
    });
    if (create.status !== 0) {
      throw new Error(`neoxp create failed:\n${create.stdout}\n${create.stderr}`);
    }
  }

  // Ensure deployer account has enough GAS
  const balanceCheck = run(neoxp, ['show', 'balance', '-i', chainPath, 'GAS', account], {
    env: { ...process.env, HOME: homeDir },
    cwd: ROOT
  });
  const balanceMatch = stripAnsi(balanceCheck.stdout).match(/balance:\s*([0-9.]+)/);
  const balance = balanceMatch ? parseFloat(balanceMatch[1]) : 0;
  if (balance < 100) {
    console.log(`[deploy] Account ${account} has ${balance} GAS — transferring 50000 GAS from genesis...`);
    const transfer = run(neoxp, ['transfer', '-i', chainPath, '50000', 'GAS', 'genesis', account], {
      env: { ...process.env, HOME: homeDir },
      cwd: ROOT
    });
    if (transfer.status !== 0) {
      console.log(`[deploy] Warning: GAS transfer failed: ${stripAnsi(transfer.stderr).slice(0, 200)}`);
    } else {
      console.log(`[deploy] GAS transfer complete`);
    }
  }

  // Find compiled outputs
  const nefFiles = fs.readdirSync(inDir)
    .filter(f => f.endsWith('.nef'))
    .map(f => path.join(inDir, f));

  console.log(`[deploy] Found ${nefFiles.length} .nef files`);

  const results = [];
  for (const nefPath of nefFiles) {
    const contractName = path.basename(nefPath, '.nef');
    const manifestPath = nefPath.replace('.nef', '.manifest.json');

    if (!fs.existsSync(manifestPath)) {
      console.log(`[deploy]   ⚠️  skipping ${contractName} — manifest missing`);
      results.push({ contract: contractName, ok: false, reason: 'manifest missing' });
      continue;
    }

    // Optionally clear supportedstandards
    if (clearStandards) {
      const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
      manifest.supportedstandards = [];
      fs.writeFileSync(manifestPath, JSON.stringify(manifest, null, 2));
    }

    const deployArgs = ['contract', 'deploy', '-i', chainPath];
    if (deployData) {
      deployArgs.push('-d', JSON.stringify(deployData));
    }
    deployArgs.push(nefPath, account, '-j');

    console.log(`[deploy] ${contractName} ...`);
    const deploy = run(neoxp, deployArgs, {
      env: { ...process.env, HOME: homeDir },
      cwd: ROOT
    });

    if (deploy.status !== 0) {
      console.log(`[deploy]   ❌ ${contractName} — deploy command failed`);
      results.push({ contract: contractName, ok: false, reason: 'deploy command failed', stderr: stripAnsi(deploy.stderr) });
      continue;
    }

    const deployJson = parseJsonFromMixedOutput(deploy.stdout);
    if (!deployJson || !deployJson['contract-hash']) {
      console.log(`[deploy]   ❌ ${contractName} — no contract hash in output`);
      results.push({ contract: contractName, ok: false, reason: 'no contract hash', stdout: stripAnsi(deploy.stdout) });
      continue;
    }

    const contractHash = deployJson['contract-hash'];
    const txHash = deployJson['tx-hash'];

    // Verify transaction succeeded
    const txInfo = run(neoxp, ['show', 'transaction', '-i', chainPath, txHash], {
      env: { ...process.env, HOME: homeDir },
      cwd: ROOT
    });
    const txJson = parseJsonFromMixedOutput(txInfo.stdout);
    const vmstate = txJson?.['application-log']?.executions?.[0]?.vmstate || 'UNKNOWN';

    if (vmstate !== 'HALT') {
      console.log(`[deploy]   ❌ ${contractName} — vmstate=${vmstate}`);
      results.push({ contract: contractName, ok: false, reason: `vmstate=${vmstate}`, contractHash, txHash });
      continue;
    }

    console.log(`[deploy]   ✅ ${contractName} — ${contractHash}`);
    results.push({ contract: contractName, ok: true, contractHash, txHash });
  }

  // Write deploy report
  const reportPath = path.join(workDir, 'deploy-report.json');
  fs.writeFileSync(reportPath, JSON.stringify({ generatedAt: new Date().toISOString(), chain: chainPath, results }, null, 2));
  console.log(`[deploy] Report written to: ${reportPath}`);

  const passCount = results.filter(r => r.ok).length;
  console.log(`[deploy] ${passCount}/${results.length} deployed successfully`);
  enforceCompletePass('deploy', results, r => r.ok, failOnPartialEnabled(args));
  return results;
}

// ---------------------------------------------------------------------------
// Mode: test
// ---------------------------------------------------------------------------

function modeTest(args) {
  const chainPath = args.chain;
  const manifestPath = args.manifest;
  const account = args.account || 'node1';

  if (!chainPath) {
    console.error('[test] Error: specify --chain <file.neo-express>');
    process.exit(1);
  }
  if (!manifestPath) {
    console.error('[test] Error: specify --manifest <test-manifest.json>');
    process.exit(1);
  }

  const neoxp = resolveNeoxp();
  const workDir = path.dirname(chainPath);
  const homeDir = path.join(workDir, 'neoxp-home');
  fs.mkdirSync(homeDir, { recursive: true });

  const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
  const cases = manifest.cases || [];

  console.log(`[test] Running ${cases.length} test cases`);

  const results = [];
  for (const testCase of cases) {
    console.log(`[test] ${testCase.name || testCase.contract} ...`);
    const caseResults = [];

    for (const assertion of (testCase.assertions || [])) {
      const invokeFile = path.join(workDir, `invoke-${sanitizeName(testCase.contract)}-${sanitizeName(assertion.name)}.json`);

      // Resolve variable placeholders
      const resolvedArgs = (assertion.args || []).map(arg => resolveArg(arg, manifest.variables || {}));

      fs.writeFileSync(invokeFile, JSON.stringify({
        contract: testCase.contractHash,
        operation: assertion.operation,
        args: resolvedArgs
      }, null, 2));

      const invokeArgs = ['contract', 'invoke', '-j', '-i', chainPath, invokeFile, account];
      if (assertion.kind === 'read') {
        invokeArgs.push('-r'); // read-only / test invocation
      } else {
        // For write operations, send as actual transaction with gas
        invokeArgs.push('--gas', assertion.gas || '10000000');
      }

      const invoke = run(neoxp, invokeArgs, {
        env: { ...process.env, HOME: homeDir },
        cwd: ROOT
      });

      const invokeJson = parseJsonFromMixedOutput(invoke.stdout);
      const state = invokeJson?.state || 'UNKNOWN';
      const stack = invokeJson?.stack || [];

      let pass = state === 'HALT';
      if (pass && assertion.expect) {
        pass = checkExpectation(stack, assertion.expect);
      }

      caseResults.push({
        name: assertion.name,
        pass,
        state,
        stack,
        expect: assertion.expect || null
      });

      const mark = pass ? '✅' : '❌';
      console.log(`[test]   ${mark} ${assertion.name}`);
      if (!pass && invoke.stderr) {
        console.log(`[test]      ${stripAnsi(invoke.stderr).slice(0, 200)}`);
      }
    }

    const allPass = caseResults.every(r => r.pass);
    results.push({
      contract: testCase.contract,
      contractHash: testCase.contractHash,
      pass: allPass,
      assertions: caseResults
    });
  }

  const reportPath = path.join(workDir, 'test-report.json');
  fs.writeFileSync(reportPath, JSON.stringify({ generatedAt: new Date().toISOString(), results }, null, 2));
  console.log(`[test] Report written to: ${reportPath}`);

  const passCount = results.filter(r => r.pass).length;
  console.log(`[test] ${passCount}/${results.length} test suites passed`);
  enforceCompletePass('test', results, r => r.pass, failOnPartialEnabled(args));
  return results;
}

function resolveArg(arg, variables) {
  if (typeof arg === 'object' && arg !== null) {
    if (typeof arg.value === 'string' && arg.value.startsWith('$')) {
      const varName = arg.value.slice(1);
      if (variables[varName] !== undefined) {
        return { ...arg, value: variables[varName] };
      }
    }
    return arg;
  }
  if (typeof arg === 'string' && arg.startsWith('$')) {
    const varName = arg.slice(1);
    return variables[varName] !== undefined ? variables[varName] : arg;
  }
  return arg;
}

function checkExpectation(stack, expect) {
  if (!stack || stack.length === 0) return false;
  const top = stack[0];
  if (expect.type && top.type !== expect.type) return false;
  if (expect.value !== undefined) {
    // Normalize for comparison
    const got = top.value;
    const want = expect.value;
    if (typeof got === 'number' && typeof want === 'number') return got === want;
    if (typeof got === 'boolean' && typeof want === 'boolean') return got === want;
    return String(got) === String(want);
  }
  return true;
}

// ---------------------------------------------------------------------------
// Mode: pipeline
// ---------------------------------------------------------------------------

function modePipeline(args) {
  const configPath = args.config;
  const outDir = args.out || './pipeline-out';
  fs.mkdirSync(outDir, { recursive: true });

  let config = null;
  if (configPath && fs.existsSync(configPath)) {
    config = JSON.parse(fs.readFileSync(configPath, 'utf8'));
  }
  const failOnPartial = failOnPartialEnabled(args, config);

  // Step 1: Collect
  const contractsDir = path.join(outDir, 'contracts');
  let collectedDir;
  if (config && config.collect) {
    // Use config-based collection
    if (config.collect.repo) {
      collectedDir = modeCollect({
        repo: config.collect.repo,
        branch: config.collect.branch,
        path: config.collect.path,
        out: contractsDir
      });
    } else if (config.collect.npm) {
      collectedDir = modeCollect({
        npm: config.collect.npm,
        path: config.collect.path,
        out: contractsDir
      });
    }
  } else if (args.repo) {
    collectedDir = modeCollect({
      repo: args.repo,
      branch: args.branch,
      path: args.path,
      out: contractsDir
    });
  } else {
    console.error('[pipeline] Error: specify --config <file> or --repo <owner/repo>');
    process.exit(1);
  }

  // Step 2: Compile
  const buildDir = path.join(outDir, 'build');
  const compileResults = modeCompile({
    in: collectedDir,
    out: buildDir,
    ...(config?.compile || {}),
    'fail-on-partial': failOnPartial
  });

  // Step 3: Deploy
  const chainPath = path.join(outDir, 'chain.neo-express');
  const deployResults = modeDeploy({
    in: buildDir,
    chain: chainPath,
    ...(config?.deploy || {}),
    'fail-on-partial': failOnPartial
  });

  // Step 4: Test (if manifest provided)
  if (config && config.test && config.test.manifest) {
    // Build test manifest with resolved contract hashes
    const testManifest = JSON.parse(fs.readFileSync(config.test.manifest, 'utf8'));
    for (const c of testManifest.cases || []) {
      const deployed = deployResults.find(d => d.contract === c.contract);
      if (deployed && deployed.contractHash) {
        c.contractHash = deployed.contractHash;
      }
    }
    const resolvedManifestPath = path.join(outDir, 'test-manifest-resolved.json');
    fs.writeFileSync(resolvedManifestPath, JSON.stringify(testManifest, null, 2));

    modeTest({
      chain: chainPath,
      manifest: resolvedManifestPath,
      ...(config?.test || {}),
      'fail-on-partial': failOnPartial
    });
  }

  console.log(`\n[pipeline] Complete! Output directory: ${outDir}`);
  console.log(`[pipeline]   Contracts: ${collectedDir}`);
  console.log(`[pipeline]   Build:     ${buildDir}`);
  console.log(`[pipeline]   Chain:     ${chainPath}`);
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

const { mode, args } = parseArgs(process.argv);

if (!mode || args.help) {
  usage();
  process.exit(0);
}

try {
  switch (mode) {
    case 'collect':
      modeCollect(args);
      break;
    case 'compile':
      modeCompile(args);
      break;
    case 'deploy':
      modeDeploy(args);
      break;
    case 'test':
      modeTest(args);
      break;
    case 'pipeline':
      modePipeline(args);
      break;
    default:
      console.error(`Unknown mode: ${mode}`);
      usage();
      process.exit(1);
  }
} catch (err) {
  console.error(`[pipeline] Error: ${err.message}`);
  if (err.stack) console.error(err.stack);
  process.exit(1);
}
