#!/usr/bin/env node

const { existsSync, readdirSync, readFileSync, statSync } = require('node:fs');
const { join, relative } = require('node:path');

const docsRoot = 'docs';
const extraContentCheckFiles = [
  'README.md',
  'FEATURE_MATRIX.md',
  'docs/SOLIDITY_SUPPORT_MATRIX.md',
  'scripts/GITHUB_CONTRACTS_PIPELINE.md'
];
const duplicateProseMinLength = 220;
const excludedPages = [
  /^\.vitepress\//,
  /^archive\//,
  /^plans\//,
  /^public\//,
  /^(ARCHITECTURE|RUNTIME_SPEC|ERROR_REFERENCE|EXCELLENCE_ASSESSMENT|FUZZ|mapping_lowering_design|NEO_VM_PARITY_TODO|README|SOLIDITY_SUPPORT_MATRIX)\.md$/
];
const staleContentChecks = [
  {
    pattern: /\bNode\.js\*\*: 20\.0\b|\bNode\.js\s*\|\s*20\.0\+|\bnode`\s*\(v18\+\)|\bv18\+\), `npm`|\b>=20\.0\.0\b/,
    message: 'stale Node.js prerequisite; use 20.19+ or 22.12+'
  },
  {
    pattern: /\bofficial Docker image \(once published\)|\bonce published\b/,
    message: 'stale Docker publishing wording; document the checked-in Dockerfile or a published image explicitly'
  },
  {
    pattern: /\b0\.14\.x release lineage\b/,
    message: 'stale release lineage; update known-bugs wording to the current release line'
  },
  {
    pattern: /Checked-arithmetic `0x11` still uses the legacy ByteString/,
    message: 'stale panic lowering wording; checked arithmetic now uses the canonical panic helper'
  },
  {
    pattern: /unchecked(?:\s*\{[^}]*\})?[^.\n]*(?:no-ops?|no behavioral effect|compile as normal blocks|ignored)|all arithmetic is inherently unchecked|do not rely on `unchecked` wrapping behavior|No Overflow\/Underflow at runtime/,
    message: 'stale unchecked arithmetic wording; unchecked now suppresses supported overflow guards and preserves wrap semantics'
  },
  {
    pattern: /Oracle integration \(stub only|Oracle Integration\*\*\s*\|\s*Stub\b/,
    message: 'stale Oracle wording; document deterministic embedded request/price behavior and the lack of live oracle callbacks'
  },
  {
    pattern: /assembly(?:\s*\{[^}]*\})?[^.\n]*(?:safely compiles it to a no-op|compiled as a no-op|will be skipped at runtime|silently skipped at runtime)/i,
    message: 'stale inline assembly wording; document limited Yul subset lowering plus fallback warnings for unsupported EVM-only operations'
  },
  {
    pattern: /neo_devpack_solidity::compiler|Compiler::new\(config\)|compile_file\("contract\.yul"\)/,
    message: 'stale Rust API example; use the current public neo_devpack_solidity::cli entry points'
  },
  {
    pattern: /neo-devpack-solidity-0\.1\.0\b/,
    message: 'stale compiler ID example; use the current 0.18.0 compiler line in manifest and NEF examples'
  },
  {
    pattern: /new EvmRuntime\(\)|runtime\.(?:MStore|MLoad|SStore|SLoad|Add|Mul)\(|new AbiEncoder\(\)|\.EncodeFunction\(|\.EncodeEvent\(/,
    message: 'stale C# runtime API example; use Evm.CreateRuntime(), runtime.Memory/runtime.Storage, static AbiEncoder methods, and runtime.Events'
  },
  {
    pattern: /\b44(?:-vector|\s+(?:test\s+)?vectors?)\b/,
    message: 'stale conformance test count; cargo test --test conformance_tests -- --list currently reports 40 tests'
  },
  {
    pattern: /\bover 1,100 tests\b/,
    message: 'stale aggregate test count; describe the layered suites instead of a cross-language total'
  }
];

function walkMarkdown(dir, files = []) {
  for (const entry of readdirSync(dir)) {
    const path = join(dir, entry);
    const stat = statSync(path);
    if (stat.isDirectory()) {
      walkMarkdown(path, files);
    } else if (path.endsWith('.md')) {
      files.push(path);
    }
  }

  return files;
}

function stripFrontmatter(source) {
  return source.replace(/^---\n[\s\S]*?\n---\n?/, '');
}

function isHomePage(source) {
  return /^---\n[\s\S]*?^layout:\s*home\s*$/m.test(source);
}

function markdownHeadings(source) {
  const headings = [];
  let inFence = false;

  source.split(/\r?\n/).forEach((line, index) => {
    if (/^(```|~~~)/.test(line.trim())) {
      inFence = !inFence;
      return;
    }

    if (inFence) {
      return;
    }

    const match = line.match(/^(#{1,6})\s+(.+)$/);
    if (match) {
      headings.push({
        level: match[1].length,
        text: match[2].trim(),
        line: index + 1
      });
    }
  });

  return headings;
}

function stripInlineMarkdown(text) {
  return text
    .replace(/`([^`]+)`/g, '$1')
    .replace(/\[([^\]]+)\]\([^)]+\)/g, '$1')
    .replace(/<[^>]+>/g, '')
    .replace(/\*\*([^*]+)\*\*/g, '$1')
    .trim();
}

function slugifyHeading(text) {
  return stripInlineMarkdown(text)
    .normalize('NFKD')
    .replace(/[\u0300-\u036F]/g, '')
    .replace(/[\u0000-\u001f]/g, '')
    .replace(/[\s~`!@#$%^&*()\-_+=[\]{}|\\;:"'“”‘’<>,.?/]+/g, '-')
    .replace(/-{2,}/g, '-')
    .replace(/^-+|-+$/g, '')
    .replace(/^(\d)/, '_$1')
    .toLowerCase();
}

function checkStaleContent(path, source) {
  for (const check of staleContentChecks) {
    const match = source.match(check.pattern);
    if (match) {
      failures.push(`${path}: ${check.message} (${match[0]})`);
    }
  }
}

function proseBlocks(source) {
  const blocks = [];
  let inFence = false;
  let current = [];

  function flush() {
    const text = current.join(' ').replace(/\s+/g, ' ').trim();
    if (text.length > 0) {
      blocks.push(text);
    }
    current = [];
  }

  for (const rawLine of stripFrontmatter(source).split(/\r?\n/)) {
    const line = rawLine.trim();
    if (/^(```|~~~)/.test(line)) {
      flush();
      inFence = !inFence;
      continue;
    }

    if (inFence) {
      continue;
    }

    if (line.length === 0) {
      flush();
      continue;
    }

    if (/^(#{1,6}\s|\||[-*+]\s|\d+\.\s|>|:::|<\/?[A-Za-z])/.test(line)) {
      flush();
      continue;
    }

    current.push(line);
  }

  flush();
  return blocks;
}

function normalizeProse(text) {
  return text
    .replace(/`([^`]+)`/g, '$1')
    .replace(/\[([^\]]+)\]\([^)]+\)/g, '$1')
    .replace(/[*_]/g, '')
    .replace(/\s+/g, ' ')
    .trim()
    .toLowerCase();
}

const pages = walkMarkdown(docsRoot)
  .map((path) => relative(docsRoot, path))
  .filter((path) => !excludedPages.some((pattern) => pattern.test(path)))
  .sort();

const failures = [];
const pageTitles = new Map();
const proseByText = new Map();

function supportStatusKeys() {
  return {
    full: String.fromCodePoint(0x2705),
    partial: `${String.fromCodePoint(0x26a0)}\ufe0f`,
    unsupported: String.fromCodePoint(0x274c),
    blocked: String.fromCodePoint(0x1f6ab)
  };
}

function splitMarkdownTableRow(line) {
  const cells = [];
  let current = '';
  let escaped = false;

  for (const char of line) {
    if (char === '|' && !escaped) {
      cells.push(current.trim().replace(/\\\|/g, '|'));
      current = '';
      escaped = false;
      continue;
    }

    current += char;
    escaped = char === '\\' && !escaped;
    if (char !== '\\') {
      escaped = false;
    }
  }

  cells.push(current.trim().replace(/\\\|/g, '|'));

  if (cells[0] === '') {
    cells.shift();
  }

  if (cells[cells.length - 1] === '') {
    cells.pop();
  }

  return cells;
}

function emptySupportCounts() {
  return {
    total: 0,
    full: 0,
    partial: 0,
    unsupported: 0,
    blocked: 0
  };
}

function countSupportMatrixRows(source) {
  const statusKeys = supportStatusKeys();
  const counts = emptySupportCounts();
  const statusToKey = new Map(Object.entries(statusKeys).map(([key, status]) => [status, key]));

  for (const line of source.split(/\r?\n/)) {
    if (/^##\s+Summary\s*$/.test(line)) {
      break;
    }

    if (!line.startsWith('|')) {
      continue;
    }

    const key = splitMarkdownTableRow(line).map((cell) => cell.trim()).find((cell) => statusToKey.has(cell));
    if (!key) {
      continue;
    }

    const countKey = statusToKey.get(key);
    counts.total += 1;
    counts[countKey] += 1;
  }

  return counts;
}

function countSupportMatrixSections(source) {
  const statusKeys = {
    ...supportStatusKeys()
  };
  const statusToKey = new Map(Object.entries(statusKeys).map(([key, status]) => [status, key]));
  const sections = new Map();
  let sectionKey = null;

  for (const line of source.split(/\r?\n/)) {
    if (/^##\s+Summary\s*$/.test(line)) {
      break;
    }

    const heading = line.match(/^##\s+([A-I]\.)\s+/);
    if (heading) {
      sectionKey = heading[1];
      sections.set(sectionKey, emptySupportCounts());
      continue;
    }

    if (!line.startsWith('|')) {
      continue;
    }

    const key = splitMarkdownTableRow(line).map((cell) => cell.trim()).find((cell) => statusToKey.has(cell));
    if (!key) {
      continue;
    }

    const countKey = statusToKey.get(key);
    const counts = sections.get(sectionKey);
    if (!counts) {
      continue;
    }

    counts.total += 1;
    counts[countKey] += 1;
  }

  return sections;
}

function featureSupportExpectations(counts) {
  const percent = (count) => Math.round((count / counts.total) * 100);
  return [
    { label: 'Total audited features', key: 'total', count: counts.total, percent: 100 },
    { label: 'Fully supported', key: 'full', count: counts.full, percent: percent(counts.full) },
    { label: 'Partial support', key: 'partial', count: counts.partial, percent: percent(counts.partial) },
    { label: 'Not supported', key: 'unsupported', count: counts.unsupported, percent: percent(counts.unsupported) },
    { label: 'Intentionally blocked', key: 'blocked', count: counts.blocked, percent: percent(counts.blocked) }
  ];
}

function parseSummaryRows(source) {
  const rows = new Map();
  const rowPattern = /^\|\s*(Total audited features|Fully supported|Partial support|Not supported|Intentionally blocked)\s*\|\s*(\d+)\s*\|\s*(\d+)%\s*\|/gm;

  for (const match of source.matchAll(rowPattern)) {
    rows.set(match[1], {
      count: Number(match[2]),
      percent: Number(match[3])
    });
  }

  return rows;
}

function parseCountCell(value) {
  return Number(value.replace(/\*/g, '').trim());
}

function checkMatrixSummaryTable(matrixPath, source, counts) {
  const sections = countSupportMatrixSections(source);
  let inSummary = false;
  let foundTotalRow = false;

  for (const line of source.split(/\r?\n/)) {
    if (/^##\s+Summary\s*$/.test(line)) {
      inSummary = true;
      continue;
    }

    if (!inSummary || !line.startsWith('|')) {
      continue;
    }

    const cells = splitMarkdownTableRow(line);
    const label = cells[0]?.replace(/\*/g, '').trim();
    const sectionMatch = label?.match(/^([A-I]\.)/);

    if (label === 'Total') {
      foundTotalRow = true;
      const actual = {
        full: parseCountCell(cells[1]),
        partial: parseCountCell(cells[2]),
        unsupported: parseCountCell(cells[3]),
        blocked: parseCountCell(cells[4])
      };

      for (const key of ['full', 'partial', 'unsupported', 'blocked']) {
        if (actual[key] !== counts[key]) {
          failures.push(`${matrixPath}: summary total ${key} count is ${actual[key]}, expected ${counts[key]} from feature rows`);
        }
      }

      continue;
    }

    if (!sectionMatch) {
      continue;
    }

    const expected = sections.get(sectionMatch[1]);
    if (!expected) {
      failures.push(`${matrixPath}: summary row "${label}" does not match a feature section`);
      continue;
    }

    const actual = {
      full: parseCountCell(cells[1]),
      partial: parseCountCell(cells[2]),
      unsupported: parseCountCell(cells[3]),
      blocked: parseCountCell(cells[4])
    };

    for (const key of ['full', 'partial', 'unsupported', 'blocked']) {
      if (actual[key] !== expected[key]) {
        failures.push(`${matrixPath}: summary row "${label}" ${key} count is ${actual[key]}, expected ${expected[key]} from feature rows`);
      }
    }
  }

  if (!foundTotalRow) {
    failures.push(`${matrixPath}: missing feature-support summary total row`);
  }
}

function checkMatrixSummaryBullets(matrixPath, source, counts) {
  const totalMatch = source.match(/\*\*Total features audited:\s*(\d+)\*\*/);
  if (!totalMatch) {
    failures.push(`${matrixPath}: missing total features audited line`);
  } else if (Number(totalMatch[1]) !== counts.total) {
    failures.push(`${matrixPath}: total features audited is ${Number(totalMatch[1])}, expected ${counts.total} from feature rows`);
  }

  const expectations = featureSupportExpectations(counts).filter((expected) => expected.key !== 'total');

  for (const expected of expectations) {
    const pattern = new RegExp(`${expected.label}:\\s*(\\d+)\\s*\\((\\d+)%\\)`);
    const match = source.match(pattern);

    if (!match) {
      failures.push(`${matrixPath}: missing "${expected.label}" summary bullet`);
      continue;
    }

    const actualCount = Number(match[1]);
    const actualPercent = Number(match[2]);
    if (actualCount !== expected.count) {
      failures.push(`${matrixPath}: ${expected.label} summary count is ${actualCount}, expected ${expected.count} from feature rows`);
    }

    if (actualPercent !== expected.percent) {
      failures.push(`${matrixPath}: ${expected.label} summary percentage is ${actualPercent}%, expected ${expected.percent}% from feature rows`);
    }
  }
}

function checkCategorySummaryPage(categorySummaryPath, source, sections, counts) {
  const foundSections = new Set();
  let foundTotalRow = false;

  for (const line of source.split(/\r?\n/)) {
    if (!line.startsWith('|')) {
      continue;
    }

    const cells = splitMarkdownTableRow(line);
    const label = cells[0]?.replace(/\*/g, '').trim();
    const sectionMatch = label?.match(/^([A-I]\.)/);

    if (label === 'Total') {
      foundTotalRow = true;
      const actual = {
        full: parseCountCell(cells[1]),
        partial: parseCountCell(cells[2]),
        unsupported: parseCountCell(cells[3]),
        blocked: parseCountCell(cells[4])
      };

      for (const key of ['full', 'partial', 'unsupported', 'blocked']) {
        if (actual[key] !== counts[key]) {
          failures.push(`${categorySummaryPath}: total ${key} count is ${actual[key]}, expected ${counts[key]} from docs/SOLIDITY_SUPPORT_MATRIX.md`);
        }
      }

      continue;
    }

    if (!sectionMatch) {
      continue;
    }

    const expected = sections.get(sectionMatch[1]);
    if (!expected) {
      failures.push(`${categorySummaryPath}: row "${label}" does not match a feature section`);
      continue;
    }

    foundSections.add(sectionMatch[1]);
    const actual = {
      full: parseCountCell(cells[1]),
      partial: parseCountCell(cells[2]),
      unsupported: parseCountCell(cells[3]),
      blocked: parseCountCell(cells[4])
    };

    for (const key of ['full', 'partial', 'unsupported', 'blocked']) {
      if (actual[key] !== expected[key]) {
        failures.push(`${categorySummaryPath}: row "${label}" ${key} count is ${actual[key]}, expected ${expected[key]} from docs/SOLIDITY_SUPPORT_MATRIX.md`);
      }
    }
  }

  for (const sectionKey of sections.keys()) {
    if (!foundSections.has(sectionKey)) {
      failures.push(`${categorySummaryPath}: missing category summary row for "${sectionKey}"`);
    }
  }

  if (!foundTotalRow) {
    failures.push(`${categorySummaryPath}: missing category summary total row`);
  }
}

function checkRootFeatureMatrix(rootMatrixPath, source, expectations) {
  const rows = new Map();

  for (const line of source.split(/\r?\n/)) {
    if (!line.startsWith('|')) {
      continue;
    }

    const cells = splitMarkdownTableRow(line);
    const label = cells[0]?.replace(/\*/g, '').trim();
    const count = Number(cells[1]?.replace(/\*/g, '').trim());

    if (Number.isFinite(count)) {
      rows.set(label, count);
    }
  }

  for (const expected of expectations) {
    const actual = rows.get(expected.label);
    if (actual === undefined) {
      failures.push(`${rootMatrixPath}: missing feature-support count row "${expected.label}"`);
      continue;
    }

    if (actual !== expected.count) {
      failures.push(`${rootMatrixPath}: ${expected.label} count is ${actual}, expected ${expected.count} from docs/SOLIDITY_SUPPORT_MATRIX.md`);
    }
  }
}

function syscallNamesFromRegistry(source) {
  return [...new Set([...source.matchAll(/"(System\.[A-Za-z0-9.]+)"/g)].map((match) => match[1]))];
}

function syscallCategoryCounts(names) {
  const counts = new Map();
  for (const name of names) {
    const category = name.split('.')[1];
    counts.set(category, (counts.get(category) || 0) + 1);
  }

  return counts;
}

function checkSyscallDocs() {
  const registryPath = 'src/runtime/spec/syscalls.rs';
  const categoryPath = 'docs/internals/syscalls/syscall-categories.md';
  const gasPath = 'docs/internals/syscalls/gas-cost-reference.md';

  if (!existsSync(registryPath) || !existsSync(categoryPath) || !existsSync(gasPath)) {
    failures.push('syscall docs check requires src/runtime/spec/syscalls.rs, docs/internals/syscalls/syscall-categories.md, and docs/internals/syscalls/gas-cost-reference.md');
    return;
  }

  const names = syscallNamesFromRegistry(readFileSync(registryPath, 'utf8'));
  const categories = syscallCategoryCounts(names);
  const categorySource = readFileSync(categoryPath, 'utf8');
  const totalMatch = categorySource.match(/contains\s+(\d+)\s+syscall names/);

  if (!totalMatch) {
    failures.push(`${categoryPath}: missing syscall total count sentence`);
  } else if (Number(totalMatch[1]) !== names.length) {
    failures.push(`${categoryPath}: syscall total is ${Number(totalMatch[1])}, expected ${names.length} from ${registryPath}`);
  }

  const seenCategories = new Set();
  for (const line of categorySource.split(/\r?\n/)) {
    if (!line.startsWith('|')) {
      continue;
    }

    const cells = splitMarkdownTableRow(line);
    const label = cells[0]?.replace(/\*/g, '').trim();
    const count = Number(cells[1]?.replace(/\*/g, '').trim());

    if (!categories.has(label)) {
      continue;
    }

    seenCategories.add(label);
    const expected = categories.get(label);
    if (count !== expected) {
      failures.push(`${categoryPath}: ${label} syscall count is ${count}, expected ${expected} from ${registryPath}`);
    }
  }

  for (const category of categories.keys()) {
    if (!seenCategories.has(category)) {
      failures.push(`${categoryPath}: missing syscall category row "${category}"`);
    }
  }

  const gasRows = [];
  for (const line of readFileSync(gasPath, 'utf8').split(/\r?\n/)) {
    if (!line.startsWith('|')) {
      continue;
    }

    const cells = splitMarkdownTableRow(line);
    const match = cells[0]?.match(/^`(System\.[^`]+)`$/);
    if (match) {
      gasRows.push(match[1]);
    }
  }

  const gasSet = new Set(gasRows);
  if (gasSet.size !== gasRows.length) {
    const duplicates = gasRows.filter((name, index) => gasRows.indexOf(name) !== index);
    failures.push(`${gasPath}: duplicate syscall gas rows: ${[...new Set(duplicates)].join(', ')}`);
  }

  for (const name of names) {
    if (!gasSet.has(name)) {
      failures.push(`${gasPath}: missing gas row for ${name}`);
    }
  }

  for (const name of gasSet) {
    if (!names.includes(name)) {
      failures.push(`${gasPath}: gas row for unregistered syscall ${name}`);
    }
  }
}

function checkFeatureSupportCounts() {
  const matrixPath = 'docs/SOLIDITY_SUPPORT_MATRIX.md';
  const summaryPath = 'docs/solidity/feature-support/summary.md';
  const categorySummaryPath = 'docs/solidity/feature-support/category-summary.md';
  const rootMatrixPath = 'FEATURE_MATRIX.md';
  const readmePath = 'README.md';

  if (
    !existsSync(matrixPath) ||
    !existsSync(summaryPath) ||
    !existsSync(categorySummaryPath) ||
    !existsSync(rootMatrixPath) ||
    !existsSync(readmePath)
  ) {
    failures.push('feature support count check requires README.md, FEATURE_MATRIX.md, docs/SOLIDITY_SUPPORT_MATRIX.md, docs/solidity/feature-support/summary.md, and docs/solidity/feature-support/category-summary.md');
    return;
  }

  const matrix = readFileSync(matrixPath, 'utf8');
  const counts = countSupportMatrixRows(matrix);
  const sections = countSupportMatrixSections(matrix);
  const expectations = featureSupportExpectations(counts);
  const summaryRows = parseSummaryRows(readFileSync(summaryPath, 'utf8'));

  checkMatrixSummaryTable(matrixPath, matrix, counts);
  checkMatrixSummaryBullets(matrixPath, matrix, counts);
  checkCategorySummaryPage(categorySummaryPath, readFileSync(categorySummaryPath, 'utf8'), sections, counts);
  checkRootFeatureMatrix(rootMatrixPath, readFileSync(rootMatrixPath, 'utf8'), expectations);

  for (const expected of expectations) {
    const actual = summaryRows.get(expected.label);
    if (!actual) {
      failures.push(`${summaryPath}: missing feature-support summary row "${expected.label}"`);
      continue;
    }

    if (actual.count !== expected.count) {
      failures.push(`${summaryPath}: ${expected.label} count is ${actual.count}, expected ${expected.count} from ${matrixPath}`);
    }

    if (actual.percent !== expected.percent) {
      failures.push(`${summaryPath}: ${expected.label} percentage is ${actual.percent}%, expected ${expected.percent}% from ${matrixPath}`);
    }
  }

  const readme = readFileSync(readmePath, 'utf8');
  const readmeMatch = readme.match(
    /\*\*(\d+) Solidity features audited\*\*[\s\S]*?(\d+) fully supported \((\d+)%\)[\s\S]*?(\d+) partial \((\d+)%\)[\s\S]*?(\d+) unsupported \((\d+)%\)[\s\S]*?(\d+) intentionally blocked \((\d+)%\)/
  );

  if (!readmeMatch) {
    failures.push(`${readmePath}: missing feature-support count summary`);
    return;
  }

  const readmeActual = {
    total: { count: Number(readmeMatch[1]), percent: 100 },
    full: { count: Number(readmeMatch[2]), percent: Number(readmeMatch[3]) },
    partial: { count: Number(readmeMatch[4]), percent: Number(readmeMatch[5]) },
    unsupported: { count: Number(readmeMatch[6]), percent: Number(readmeMatch[7]) },
    blocked: { count: Number(readmeMatch[8]), percent: Number(readmeMatch[9]) }
  };

  for (const expected of expectations) {
    const actual = readmeActual[expected.key];
    if (actual.count !== expected.count) {
      failures.push(`${readmePath}: ${expected.label} count is ${actual.count}, expected ${expected.count} from ${matrixPath}`);
    }

    if (actual.percent !== expected.percent) {
      failures.push(`${readmePath}: ${expected.label} percentage is ${actual.percent}%, expected ${expected.percent}% from ${matrixPath}`);
    }
  }
}

for (const page of pages) {
  const path = join(docsRoot, page);
  const source = readFileSync(path, 'utf8');
  const body = stripFrontmatter(source);

  if (isHomePage(source)) {
    continue;
  }

  if (!/^#\s+\S.+$/m.test(body)) {
    failures.push(`${page}: missing top-level # title`);
  }

  const h1 = body.match(/^#\s+(.+)$/m)?.[1]?.trim();
  if (h1) {
    const titleKey = h1.toLowerCase();
    if (pageTitles.has(titleKey)) {
      failures.push(`${page}: duplicate top-level title "${h1}" (first seen in ${pageTitles.get(titleKey)})`);
    } else {
      pageTitles.set(titleKey, page);
    }
  }

  const headings = markdownHeadings(body);
  const sectionSlugs = new Map();

  if (!headings.some((heading) => heading.level === 2 || heading.level === 3)) {
    failures.push(`${page}: missing H2/H3 section for the left contents table`);
  }

  for (let index = 1; index < headings.length; index += 1) {
    const previous = headings[index - 1];
    const current = headings[index];
    if (current.level > previous.level + 1) {
      failures.push(
        `${page}:${current.line}: heading jumps from H${previous.level} to H${current.level} (${current.text})`
      );
    }
  }

  for (const heading of headings.filter((entry) => entry.level === 2 || entry.level === 3)) {
    const slug = slugifyHeading(heading.text);
    if (slug.length === 0) {
      continue;
    }

    if (sectionSlugs.has(slug)) {
      failures.push(`${page}:${heading.line}: duplicate left contents anchor "${slug}" (first seen at line ${sectionSlugs.get(slug)})`);
      continue;
    }

    sectionSlugs.set(slug, heading.line);
  }

  checkStaleContent(page, source);

  for (const block of proseBlocks(source)) {
    const key = normalizeProse(block);
    if (key.length < duplicateProseMinLength) {
      continue;
    }

    if (proseByText.has(key)) {
      failures.push(`${page}: duplicate prose block also appears in ${proseByText.get(key)}`);
      continue;
    }

    proseByText.set(key, page);
  }
}

for (const file of extraContentCheckFiles) {
  if (existsSync(file)) {
    checkStaleContent(file, readFileSync(file, 'utf8'));
  }
}

checkFeatureSupportCounts();
checkSyscallDocs();

if (!existsSync(docsRoot)) {
  failures.push(`missing docs root: ${docsRoot}`);
}

if (failures.length > 0) {
  console.error(`Docs structure check failed (${failures.length} issue${failures.length === 1 ? '' : 's'}):`);
  for (const failure of failures) {
    console.error(`- ${failure}`);
  }
  process.exit(1);
}

console.log(`Docs structure check passed for ${pages.length} routable pages.`);
