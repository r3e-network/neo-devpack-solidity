#!/usr/bin/env node
'use strict';

const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');

const ROOT = path.resolve(__dirname, '..');
const TARGETS_PATH = path.join(ROOT, 'docs/data/famous-contracts-targets.json');
const AUDIT_DIR = process.env.NEO_FAMOUS_AUDIT_DIR || '/tmp/neo-famous-contracts-audit';
const VENDORED_ROOT = path.join(ROOT, 'third_party/famous-contracts/sources');

const NPM_PACKAGES = [
  '@openzeppelin/contracts@5.4.0',
  '@openzeppelin/contracts-upgradeable@5.4.0',
  '@aave/core-v3@1.19.3',
  '@safe-global/safe-contracts@1.4.1-2',
  '@chainlink/contracts@1.5.0',
  'solmate',
  '@uniswap/v4-core@1.0.2',
  '@uniswap/v4-periphery@1.0.3',
  '@uniswap/v3-core@1.0.1',
  '@uniswap/v3-periphery@1.4.4'
];

const REQUIRED_PACKAGE_PROBES = [
  '@openzeppelin/contracts/package.json',
  '@openzeppelin/contracts-upgradeable/package.json',
  '@aave/core-v3/package.json',
  '@safe-global/safe-contracts/package.json',
  '@chainlink/contracts/package.json',
  'solmate/package.json',
  '@uniswap/v2-core/package.json',
  '@uniswap/v4-core/package.json',
  '@uniswap/v4-periphery/package.json'
];

function runCommand(cmd, args, options = {}) {
  const result = spawnSync(cmd, args, {
    encoding: 'utf8',
    maxBuffer: 50 * 1024 * 1024,
    ...options
  });
  if (result.error) {
    throw result.error;
  }
  return result;
}

function ensureAuditWorkspace() {
  fs.mkdirSync(AUDIT_DIR, { recursive: true });
  const nodeModulesDir = path.join(AUDIT_DIR, 'node_modules');
  const hasAllPackages = REQUIRED_PACKAGE_PROBES.every((probe) =>
    fs.existsSync(path.join(nodeModulesDir, probe))
  );

  if (hasAllPackages) {
    return;
  }

  if (!fs.existsSync(path.join(AUDIT_DIR, 'package.json'))) {
    const init = runCommand('npm', ['init', '-y'], { cwd: AUDIT_DIR });
    if (init.status !== 0) {
      throw new Error(`npm init failed:\n${init.stdout}\n${init.stderr}`);
    }
  }

  const install = runCommand('npm', ['install', '--silent', ...NPM_PACKAGES], { cwd: AUDIT_DIR });
  if (install.status !== 0) {
    throw new Error(`npm install failed:\n${install.stdout}\n${install.stderr}`);
  }
}

function copyIfChanged(src, dst) {
  const srcBytes = fs.readFileSync(src);
  if (fs.existsSync(dst)) {
    const dstBytes = fs.readFileSync(dst);
    if (srcBytes.equals(dstBytes)) {
      return false;
    }
  }

  fs.mkdirSync(path.dirname(dst), { recursive: true });
  fs.writeFileSync(dst, srcBytes);
  return true;
}

function main() {
  ensureAuditWorkspace();

  const targets = JSON.parse(fs.readFileSync(TARGETS_PATH, 'utf8'));
  const npmTargets = targets.filter((target) => target.source === 'npm');

  fs.mkdirSync(VENDORED_ROOT, { recursive: true });

  let copied = 0;
  let unchanged = 0;
  const missing = [];

  for (const target of npmTargets) {
    const src = path.join(AUDIT_DIR, 'node_modules', target.path);
    const dst = path.join(VENDORED_ROOT, target.path);

    if (!fs.existsSync(src)) {
      missing.push(target.path);
      continue;
    }

    if (copyIfChanged(src, dst)) {
      copied += 1;
    } else {
      unchanged += 1;
    }
  }

  if (missing.length > 0) {
    process.stderr.write('[vendor] missing upstream source files:\n');
    for (const item of missing) {
      process.stderr.write(`- ${item}\n`);
    }
    process.exit(1);
  }

  process.stdout.write(
    `[vendor] completed: copied=${copied}, unchanged=${unchanged}, total=${npmTargets.length}\n`
  );
  process.stdout.write(`[vendor] root: ${path.relative(ROOT, VENDORED_ROOT)}\n`);
}

main();
