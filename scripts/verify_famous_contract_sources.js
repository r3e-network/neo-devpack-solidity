#!/usr/bin/env node
'use strict';

const fs = require('fs');
const os = require('os');
const path = require('path');
const { spawnSync } = require('child_process');

const ROOT = path.resolve(__dirname, '..');
const TARGETS_PATH = path.join(ROOT, 'docs/data/famous-contracts-targets.json');
const AUDIT_RESULTS_PATH = path.join(ROOT, 'docs/data/famous-contracts-audit-results.json');
const AUDIT_DIR = process.env.NEO_FAMOUS_AUDIT_DIR || '/tmp/neo-famous-contracts-audit';
const VENDORED_SOURCES_ROOT = path.join(ROOT, 'third_party/famous-contracts/sources');

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

function run(cmd, args, options = {}) {
  const res = spawnSync(cmd, args, {
    encoding: 'utf8',
    maxBuffer: 50 * 1024 * 1024,
    ...options
  });
  if (res.error) {
    throw res.error;
  }
  return res;
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
    const init = run('npm', ['init', '-y'], { cwd: AUDIT_DIR });
    if (init.status !== 0) {
      throw new Error(`npm init failed:\n${init.stdout}\n${init.stderr}`);
    }
  }

  const install = run('npm', ['install', '--silent', ...NPM_PACKAGES], { cwd: AUDIT_DIR });
  if (install.status !== 0) {
    throw new Error(`npm install failed:\n${install.stdout}\n${install.stderr}`);
  }
}

function packageFromTargetPath(targetPath) {
  const parts = targetPath.split('/');
  if (targetPath.startsWith('@')) {
    return parts.slice(0, 2).join('/');
  }
  return parts[0];
}

function relativePathInPackage(targetPath) {
  const pkg = packageFromTargetPath(targetPath);
  return targetPath.slice(pkg.length + 1);
}

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
}

function packAndExtractPackage(pkgName, version) {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'neo-famous-verify-'));
  const spec = `${pkgName}@${version}`;
  const pack = run('npm', ['pack', '--silent', spec], { cwd: tempDir });
  if (pack.status !== 0) {
    throw new Error(`npm pack failed for ${spec}:\n${pack.stdout}\n${pack.stderr}`);
  }
  const tarball = pack.stdout
    .trim()
    .split(/\r?\n/)
    .filter(Boolean)
    .pop();
  if (!tarball) {
    throw new Error(`npm pack did not produce tarball name for ${spec}`);
  }

  const tarPath = path.join(tempDir, tarball);
  const extract = run('tar', ['-xzf', tarPath], { cwd: tempDir });
  if (extract.status !== 0) {
    throw new Error(`tar extraction failed for ${spec}:\n${extract.stdout}\n${extract.stderr}`);
  }

  const extractedRoot = path.join(tempDir, 'package');
  return { tempDir, extractedRoot, spec };
}

function verifyRepoSoliditySource(filePath) {
  const content = fs.readFileSync(filePath, 'utf8');
  return /\b(contract|interface|library)\b/.test(content);
}

function main() {
  ensureAuditWorkspace();

  const targets = readJson(TARGETS_PATH);
  const audit = readJson(AUDIT_RESULTS_PATH);

  const npmTargets = targets.filter((target) => target.source === 'npm');
  const repoTargets = targets.filter((target) => target.source === 'repo');
  const packageTargets = new Map();

  for (const target of npmTargets) {
    const pkg = packageFromTargetPath(target.path);
    if (!packageTargets.has(pkg)) {
      packageTargets.set(pkg, []);
    }
    packageTargets.get(pkg).push(target);
  }

  const errors = [];
  const mismatchDetails = [];
  let comparedNpmFiles = 0;
  const tempDirs = [];

  try {
    for (const [pkgName, pkgFiles] of packageTargets.entries()) {
      const installedRoot = path.join(AUDIT_DIR, 'node_modules', pkgName);
      const installedPackageJson = path.join(installedRoot, 'package.json');
      if (!fs.existsSync(installedPackageJson)) {
        errors.push(`missing installed package: ${pkgName} (${installedPackageJson})`);
        continue;
      }

      const { version } = readJson(installedPackageJson);
      const { tempDir, extractedRoot, spec } = packAndExtractPackage(pkgName, version);
      tempDirs.push(tempDir);

      for (const target of pkgFiles) {
        const rel = relativePathInPackage(target.path);
        const vendoredFile = path.join(VENDORED_SOURCES_ROOT, target.path);
        const officialFile = path.join(extractedRoot, rel);

        if (!fs.existsSync(vendoredFile)) {
          mismatchDetails.push({
            type: 'missing_vendored_file',
            package: spec,
            target: target.contract,
            file: target.path
          });
          continue;
        }

        if (!fs.existsSync(officialFile)) {
          mismatchDetails.push({
            type: 'missing_official_file',
            package: spec,
            target: target.contract,
            file: rel
          });
          continue;
        }

        const vendoredBytes = fs.readFileSync(vendoredFile);
        const officialBytes = fs.readFileSync(officialFile);
        comparedNpmFiles += 1;
        if (!vendoredBytes.equals(officialBytes)) {
          mismatchDetails.push({
            type: 'content_mismatch',
            package: spec,
            target: target.contract,
            file: target.path,
            vendoredBytes: vendoredBytes.length,
            officialBytes: officialBytes.length
          });
        }
      }
    }
  } finally {
    for (const tempDir of tempDirs) {
      try {
        fs.rmSync(tempDir, { recursive: true, force: true });
      } catch {
        // ignore cleanup failures
      }
    }
  }

  for (const target of repoTargets) {
    const sourcePath = path.join(ROOT, target.path);
    if (!fs.existsSync(sourcePath)) {
      errors.push(`missing repo source target: ${target.path}`);
      continue;
    }
    if (!verifyRepoSoliditySource(sourcePath)) {
      errors.push(`repo target does not look like complete Solidity source: ${target.path}`);
    }
  }

  if (audit?.totals?.pass !== audit?.totals?.total) {
    errors.push(
      `audit not fully passing: pass=${audit?.totals?.pass}, total=${audit?.totals?.total}`
    );
  }

  if (mismatchDetails.length > 0) {
    errors.push(`found ${mismatchDetails.length} vendored source mismatches against official tarballs`);
  }

  if (errors.length > 0) {
    console.error('[verify] FAILED');
    for (const err of errors) {
      console.error(`- ${err}`);
    }
    if (mismatchDetails.length > 0) {
      console.error('[verify] mismatch details:');
      for (const item of mismatchDetails) {
        console.error(`- ${JSON.stringify(item)}`);
      }
    }
    process.exit(1);
  }

  console.log('[verify] OK');
  console.log(`- npm targets compared (vendored vs official): ${comparedNpmFiles}`);
  console.log(`- repo targets checked: ${repoTargets.length}`);
  console.log(`- audit totals: ${audit.totals.pass}/${audit.totals.total} pass`);
}

main();
