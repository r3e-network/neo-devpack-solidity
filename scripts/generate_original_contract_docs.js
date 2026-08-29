#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const repoRoot = path.resolve(__dirname, '..');
const inputPath = path.join(repoRoot, 'docs', 'data', 'famous-contracts-audit-results.json');
const outRoot = path.join(repoRoot, 'docs', 'solidity', 'original-contracts');

const blockerPlaybook = {
  abi_overload: {
    title: 'ABI overload collision on Neo',
    actions: [
      'Rename public/external overloads so each exposed method has a unique name.',
      'Keep overloaded variants internal/private if overloading is required for code reuse.',
      'If upstream API compatibility is required, add a thin adapter layer that maps unique Neo entrypoints to canonical behavior.'
    ]
  },
  assembly: {
    title: 'Inline assembly not supported',
    actions: [
      'Replace assembly with high-level Solidity and devpack intrinsics (`Syscalls`, `NativeCalls`, `Runtime`).',
      'For low-level call/value movement, use explicit Neo-native APIs instead of EVM opcodes.',
      'Isolate assembly-heavy modules and rewrite them first as Neo-specific utility contracts.'
    ]
  },
  import_cycle: {
    title: 'Import cycle in source graph',
    actions: [
      'Break cyclic dependencies by extracting interfaces and shared structs to leaf modules.',
      'Split contract logic into acyclic layers (`interfaces` -> `base` -> `impl`).',
      'Avoid barrel imports that re-export modules participating in cycles.'
    ]
  },
  named_mapping: {
    title: 'Named mapping syntax/shape unsupported in current pipeline',
    actions: [
      'Rewrite to plain mapping declarations (for example `mapping(address => uint256)`).',
      'Flatten nested mapping wrappers where possible to reduce type complexity.',
      'Track compiler updates for full named mapping lowering and migrate back if desired.'
    ]
  },
  name_resolution: {
    title: 'Name resolution / symbol flattening gap',
    actions: [
      'Fully qualify symbol access and reduce implicit inheritance lookups.',
      'Refactor ambiguous symbols into explicit library/internal calls.',
      'Minimize cross-file wildcard imports to simplify resolution.'
    ]
  },
  value_call_options: {
    title: 'EVM call options with value not supported',
    actions: [
      'Replace `call{value: ...}` / `send` / `transfer` with `NativeCalls.gasTransfer` or `NativeCalls.neoTransfer`.',
      'Receive funds via NEP callback methods (`onNEP17Payment` / `onNEP11Payment`) instead of `receive()`.',
      'Separate transfer side effects from contract call logic to keep manifests least-privilege.'
    ]
  },
  inheritance_linearization: {
    title: 'Inheritance linearization conflict',
    actions: [
      'Reorder base contracts to satisfy C3 linearization constraints.',
      'Split conflicting base behaviors into composition-style helper contracts.',
      'Reduce deep diamond inheritance trees before compiling to NeoVM.'
    ]
  },
  ctor_modifier_mismatch: {
    title: 'Constructor/modifier argument mismatch',
    actions: [
      'Align constructor arguments through the full inheritance chain.',
      'Avoid hidden parameter propagation through modifiers in constructors.',
      'Move complex initialization into explicit `initialize` routines when practical.'
    ]
  },
  unsupported_param_type: {
    title: 'Unsupported parameter type at contract boundary',
    actions: [
      'Replace complex interface/struct boundary types with primitive ABI-safe values.',
      'Pass opaque bytes payloads and decode internally when interoperability is needed.',
      'Keep cross-contract entrypoints narrow and use internal adapters for complex types.'
    ]
  },
  duplicate_state_var: {
    title: 'Duplicate state variable symbol',
    actions: [
      'Rename colliding state variables across inheritance hierarchy.',
      'Consolidate duplicated storage slots into one authoritative declaration.',
      'Review overshadowing patterns introduced by upgrades/merges.'
    ]
  },
  solidity_version: {
    title: 'Solidity version outside compiler support range',
    actions: [
      'Upgrade source pragmas and syntax to Solidity 0.8.x.',
      'Replace legacy patterns (for example legacy SafeMath flows and old constructor style) with 0.8-native code.',
      'Re-run audit after each migration step to isolate non-version blockers.'
    ]
  },
  other: {
    title: 'General compiler compatibility gap',
    actions: [
      'Use diagnostics to isolate the minimal failing construct.',
      'Refactor toward Neo-native patterns (`Runtime`, `Syscalls`, `NativeCalls`).',
      'Open a focused compiler issue with a minimized reproducer when behavior should be supported.'
    ]
  }
};

function slugify(value) {
  return String(value)
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');
}

function mdEscape(value) {
  return String(value || '')
    .replace(/\|/g, '\\|')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');
}

function asCode(value) {
  return `\`${String(value || '').replace(/`/g, '\\`')}\``;
}

function cleanPathText(value) {
  if (!value) {
    return '';
  }

  let text = String(value).split(path.sep).join('/');
  text = text.replace(/\/(?:Users|home|private\/tmp|tmp)\/[^\s"'<>]+\/neo-devpack-solidity\//g, '');
  text = text.replaceAll('/private/tmp/neo-famous-contracts-audit/node_modules/', 'node_modules/');
  text = text.replaceAll('/tmp/neo-famous-contracts-audit/node_modules/', 'node_modules/');

  const repoPrefix = repoRoot.split(path.sep).join('/') + '/';
  if (text.startsWith(repoPrefix)) {
    text = text.slice(repoPrefix.length);
  }

  return text;
}

function normalizeSourcePath(result) {
  if (result.source === 'npm' && result.path) {
    if (result.vendoredSourcePath) {
      const vendored = cleanPathText(result.vendoredSourcePath);
      if (vendored) {
        return vendored;
      }
    }

    if (result.sourcePath) {
      const cleaned = cleanPathText(result.sourcePath);
      if (cleaned && cleaned !== result.sourcePath) {
        return cleaned;
      }
    }
    return `third_party/famous-contracts/sources/${String(result.path).replace(/^\/+/, '')}`;
  }

  if (result.path) {
    return cleanPathText(result.path);
  }

  if (result.sourcePath) {
    return cleanPathText(result.sourcePath);
  }

  return 'Unknown';
}

function statusLabel(result) {
  return result.status === 'pass' ? '✅ pass' : '❌ fail';
}

function countBy(diagnostics, key) {
  const counts = new Map();
  for (const diag of diagnostics) {
    const value = String(diag[key] || 'unknown');
    counts.set(value, (counts.get(value) || 0) + 1);
  }

  return [...counts.entries()].sort((a, b) => {
    if (b[1] !== a[1]) {
      return b[1] - a[1];
    }
    return a[0].localeCompare(b[0]);
  });
}

function diagnosticsSummary(diagnostics) {
  if (!Array.isArray(diagnostics) || diagnostics.length === 0) {
    return 'No diagnostics were captured for this contract in the audit run.';
  }

  const lines = [
    `Total diagnostics captured: ${asCode(String(diagnostics.length))}`,
    '',
    '### By Severity',
    '',
    '| Severity | Count |',
    '| --- | ---: |'
  ];

  for (const [severity, count] of countBy(diagnostics, 'severity')) {
    lines.push(`| ${mdEscape(severity)} | ${count} |`);
  }

  lines.push('');
  lines.push('### Most Common Codes');
  lines.push('');
  lines.push('| Code | Count | Example |');
  lines.push('| --- | ---: | --- |');

  for (const [code, count] of countBy(diagnostics, 'code').slice(0, 12)) {
    const example = diagnostics.find((diag) => String(diag.code || 'unknown') === code);
    lines.push(`| ${mdEscape(code)} | ${count} | ${mdEscape(cleanPathText(example?.message || ''))} |`);
  }

  lines.push('');
  lines.push(
    'Source diagnostic payload: `docs/data/famous-contracts-audit-results.json`.'
  );

  return lines.join('\n');
}

function failGuidance(result) {
  const tag = result.blockerTag || 'other';
  const entry = blockerPlaybook[tag] || blockerPlaybook.other;
  const actions = entry.actions.map((item) => `1. ${item}`).join('\n');

  return [
    '## What Must Change To Compile On NeoVM',
    '',
    `- Primary blocker tag: ${asCode(tag)}`,
    `- Need on Neo (from audit): ${result.neoRequirement ? mdEscape(result.neoRequirement) : 'Not provided in audit output.'}`,
    '',
    `### Migration Playbook: ${entry.title}`,
    '',
    actions
  ].join('\n');
}

function renderContractPage(result, projectSlug, contractSlug, totals) {
  const title = `${result.contract} (${result.project})`;
  const sourcePath = normalizeSourcePath(result);
  const sourceType = result.source || 'Unknown';
  const mainIssue = cleanPathText(result.mainIssue || 'No primary issue recorded.');
  const relAudit = '/solidity/famous-contracts-neo-audit';
  const relIndex = '/solidity/original-contracts/';

  const sections = [
    `# ${title}`,
    '',
    '## Audit Snapshot',
    '',
    `- Status: ${statusLabel(result)}`,
    `- Source type: ${asCode(sourceType)}`,
    `- Source path: ${asCode(sourcePath)}`,
    `- Primary issue: ${mdEscape(mainIssue)}`,
    `- Audit corpus size: ${totals.total} contracts`,
    '',
    ...(result.status === 'pass' ? [] : [failGuidance(result), '']),
    '## Diagnostics',
    '',
    diagnosticsSummary(result.diagnostics),
    '',
    '## References',
    '',
    `- Global audit report: [Famous Contracts on NeoVM](${relAudit})`,
    `- Per-contract index: [Original Famous Contracts](${relIndex})`,
    `- Upstream contract path: ${asCode(sourcePath)}`
  ];

  return sections.join('\n');
}

function buildProjectTableRows(results, projectSlug) {
  return results
    .map((result) => {
      const contractSlug = slugify(result.contract);
      const link = `/solidity/original-contracts/${projectSlug}/${contractSlug}`;
      const blocker = result.blockerTag || '-';
      const need = result.neoRequirement ? mdEscape(result.neoRequirement) : '-';
      return `| [${mdEscape(result.contract)}](${link}) | ${statusLabel(result)} | ${asCode(blocker)} | ${need} |`;
    })
    .join('\n');
}

function renderIndex(data, npmResults, grouped) {
  const generatedAt = data.generatedAt || 'unknown';
  const lines = [
    '# Original Famous Solidity Contracts (Per Contract)',
    '',
    'This section documents **upstream famous Solidity contracts** (vendored in-repo sources), not simplified demo ports.',
    '',
    `- Generated at (UTC): ${asCode(generatedAt)}`,
    '- Snapshot scope: historical compatibility output; rerun `npm run audit:famous-contracts` and `npm run docs:generate:original-contracts` before treating these per-contract results as current release evidence.',
    `- Contracts in this section: ${asCode(String(npmResults.length))}`,
    '',
    'Each contract has a dedicated page with:',
    '',
    '1. Compilation status on NeoVM',
    '1. Primary blocker and required Neo-side capability/refactor',
    '1. Diagnostics summarized from the audit run',
    '',
    '## Common Production Hardening',
    '',
    'For contracts that compiled in this historical snapshot:',
    '',
    '1. Review generated manifest permissions and remove wildcard entries when possible.',
    '1. Run Neo-Express state-changing tests for target workflows, not only read-only calls.',
    '1. Validate semantic differences such as `tx.origin`, payable semantics, callback models, and manifest permissions for the integration context.',
    '',
    '## Project Summary',
    '',
    '| Project | Contracts | Pass | Fail |',
    '| --- | ---: | ---: | ---: |'
  ];

  const projectNames = Object.keys(grouped).sort((a, b) => a.localeCompare(b));
  for (const projectName of projectNames) {
    const items = grouped[projectName];
    const pass = items.filter((item) => item.status === 'pass').length;
    const fail = items.length - pass;
    lines.push(`| ${mdEscape(projectName)} | ${items.length} | ${pass} | ${fail} |`);
  }

  for (const projectName of projectNames) {
    const items = grouped[projectName].slice().sort((a, b) => a.contract.localeCompare(b.contract));
    const projectSlug = slugify(projectName);
    lines.push('');
    lines.push(`## ${projectName}`);
    lines.push('');
    lines.push('| Contract | Status | Blocker | Need On Neo |');
    lines.push('| --- | --- | --- | --- |');
    lines.push(buildProjectTableRows(items, projectSlug));
  }

  return lines.join('\n');
}

function main() {
  if (!fs.existsSync(inputPath)) {
    throw new Error(`Audit JSON not found: ${inputPath}`);
  }

  const raw = fs.readFileSync(inputPath, 'utf8');
  const data = JSON.parse(raw);
  const allResults = Array.isArray(data.results) ? data.results : [];
  const npmResults = allResults.filter((result) => result.source === 'npm');

  const grouped = {};
  for (const result of npmResults) {
    if (!grouped[result.project]) {
      grouped[result.project] = [];
    }
    grouped[result.project].push(result);
  }

  fs.rmSync(outRoot, { recursive: true, force: true });
  fs.mkdirSync(outRoot, { recursive: true });

  for (const [projectName, projectResults] of Object.entries(grouped)) {
    const projectSlug = slugify(projectName);
    const projectDir = path.join(outRoot, projectSlug);
    fs.mkdirSync(projectDir, { recursive: true });

    const sortedContracts = projectResults.slice().sort((a, b) => a.contract.localeCompare(b.contract));
    for (const result of sortedContracts) {
      const contractSlug = slugify(result.contract);
      const pagePath = path.join(projectDir, `${contractSlug}.md`);
      const page = renderContractPage(
        result,
        projectSlug,
        contractSlug,
        data.totals || { total: npmResults.length }
      );
      fs.writeFileSync(pagePath, page, 'utf8');
    }
  }

  const indexPath = path.join(outRoot, 'index.md');
  const indexContent = renderIndex(data, npmResults, grouped);
  fs.writeFileSync(indexPath, indexContent, 'utf8');

  const totalPages = npmResults.length + 1;
  process.stdout.write(
    `Generated ${totalPages} docs pages under ${path.relative(repoRoot, outRoot)}\n`
  );
}

main();
