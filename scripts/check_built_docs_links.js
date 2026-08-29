#!/usr/bin/env node

const { existsSync, readdirSync, readFileSync, statSync } = require('node:fs');
const { join, posix, relative } = require('node:path');

const distRoot = 'docs/.vitepress/dist';

function walkHtml(dir, files = []) {
  for (const entry of readdirSync(dir)) {
    const path = join(dir, entry);
    const stat = statSync(path);
    if (stat.isDirectory()) {
      walkHtml(path, files);
    } else if (path.endsWith('.html')) {
      files.push(path);
    }
  }

  return files;
}

function decodeHtml(value) {
  return value.replace(/&amp;/g, '&').replace(/&quot;/g, '"').replace(/&#39;/g, "'");
}

function decodeHash(value) {
  try {
    return decodeURIComponent(value);
  } catch {
    return value;
  }
}

function routeForHtml(file) {
  const rel = relative(distRoot, file).split('/').join(posix.sep);
  if (rel === 'index.html') {
    return '/';
  }

  if (rel.endsWith('/index.html')) {
    return `/${rel.slice(0, -'index.html'.length)}`;
  }

  return `/${rel.slice(0, -'.html'.length)}`;
}

function htmlForPath(pathname) {
  const cleanPath = pathname.replace(/\/+$/, '');
  if (cleanPath === '') {
    return join(distRoot, 'index.html');
  }

  const direct = join(distRoot, `${cleanPath}.html`);
  const index = join(distRoot, cleanPath, 'index.html');

  if (existsSync(direct) && statSync(direct).isFile()) {
    return direct;
  }

  if (existsSync(index) && statSync(index).isFile()) {
    return index;
  }

  return undefined;
}

function htmlIds(file) {
  const html = readFileSync(file, 'utf8');
  return new Set([...html.matchAll(/\sid="([^"]+)"/g)].map((match) => decodeHtml(match[1])));
}

if (!existsSync(distRoot)) {
  console.error(`Built docs directory is missing: ${distRoot}`);
  console.error('Run `npm run docs:build` before `npm run docs:check:links`.');
  process.exit(1);
}

const htmlFiles = walkHtml(distRoot).filter((file) => {
  const rel = relative(distRoot, file);
  if (rel.startsWith('api/')) {
    return false;
  }

  return readFileSync(file, 'utf8').includes('VitePress v1.6.4');
});

const idsByFile = new Map(htmlFiles.map((file) => [file, htmlIds(file)]));
const failures = [];

for (const file of htmlFiles) {
  const route = routeForHtml(file);
  const html = readFileSync(file, 'utf8');
  const hrefs = [...html.matchAll(/\shref="([^"]*#[^"]+)"/g)].map((match) => decodeHtml(match[1]));

  for (const href of hrefs) {
    if (/^(https?:|mailto:|tel:|javascript:)/.test(href)) {
      continue;
    }

    const [rawPath, rawHash] = href.split('#');
    const hash = decodeHash((rawHash || '').split('?')[0]);
    if (hash.length === 0 || hash === 'VPContent') {
      continue;
    }

    let pathname;
    try {
      const current = new URL(`http://example.test${route.endsWith('/') ? route : `${route}.html`}`);
      pathname = new URL(href, current).pathname;
    } catch {
      continue;
    }

    if (rawPath === '') {
      pathname = route;
    }

    const target = htmlForPath(pathname);
    if (!target) {
      failures.push(`${relative(distRoot, file)}: ${href} target page missing (${pathname})`);
      continue;
    }

    const ids = idsByFile.get(target);
    if (ids && !ids.has(hash)) {
      failures.push(`${relative(distRoot, file)}: ${href} target id missing`);
    }
  }
}

if (failures.length > 0) {
  console.error(`Built docs link check failed (${failures.length} issue${failures.length === 1 ? '' : 's'}):`);
  for (const failure of failures) {
    console.error(`- ${failure}`);
  }
  process.exit(1);
}

console.log(`Built docs link check passed for ${htmlFiles.length} VitePress pages.`);
