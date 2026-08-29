import { existsSync, readFileSync, readdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { defineConfig, type DefaultTheme } from 'vitepress';

const isCI = process.env.GITHUB_ACTIONS === 'true';
const docsRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');

type SidebarItem = DefaultTheme.SidebarItem;
type MarkdownHeading = {
  level: 2 | 3;
  text: string;
};

function markdownPathForLink(link: string): string {
  const cleanLink = link.replace(/^\/+/, '').replace(/#.*$/, '');
  const pagePath = cleanLink.endsWith('/') ? `${cleanLink}index.md` : `${cleanLink}.md`;
  return resolve(docsRoot, pagePath);
}

function stripInlineMarkdown(text: string): string {
  return text
    .replace(/`([^`]+)`/g, '$1')
    .replace(/\[([^\]]+)\]\([^)]+\)/g, '$1')
    .replace(/<[^>]+>/g, '')
    .replace(/\*\*([^*]+)\*\*/g, '$1')
    .trim();
}

function slugifyHeading(text: string): string {
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

function markdownHeadings(content: string): MarkdownHeading[] {
  const headings: MarkdownHeading[] = [];
  let inFence = false;

  for (const line of content.split(/\r?\n/)) {
    if (/^(```|~~~)/.test(line.trim())) {
      inFence = !inFence;
      continue;
    }

    if (inFence) {
      continue;
    }

    const match = line.match(/^(#{2,3})\s+(.+)$/);
    if (!match) {
      continue;
    }

    headings.push({
      level: match[1].length as 2 | 3,
      text: stripInlineMarkdown(match[2])
    });
  }

  return headings;
}

function hasHomeLayout(content: string): boolean {
  const frontmatter = content.match(/^---\n([\s\S]*?)\n---/);
  return Boolean(frontmatter?.[1].match(/^layout:\s*home\s*$/m));
}

function pageTitle(markdownPath: string): string {
  const content = readFileSync(markdownPath, 'utf8');
  const frontmatterTitle = content.match(/^title:\s*["']?(.+?)["']?\s*$/m);
  if (frontmatterTitle) {
    return frontmatterTitle[1];
  }

  return content.match(/^# (.+)$/m)?.[1] ?? markdownPath;
}

function pageSections(link: string): SidebarItem[] {
  const markdownPath = markdownPathForLink(link);
  if (!existsSync(markdownPath)) {
    return [];
  }

  const content = readFileSync(markdownPath, 'utf8');
  const sections: SidebarItem[] = [];

  for (const heading of markdownHeadings(content)) {
    if (heading.text.length === 0) {
      continue;
    }

    const item = {
      text: heading.text,
      link: `${link}#${slugifyHeading(heading.text)}`
    };

    if (heading.level === 2 || sections.length === 0) {
      sections.push(item);
      continue;
    }

    const parent = sections[sections.length - 1];
    parent.items = [...(parent.items ?? []), item];
  }

  if (sections.length === 0 && /^#\s+.+$/m.test(content) && !hasHomeLayout(content)) {
    sections.push({
      text: 'Overview',
      link: `${link}#overview`
    });
  }

  return sections;
}

function page(text: string, link: string): SidebarItem {
  const sections = pageSections(link);
  // Sub-sections (h2/h3) are present so the sidebar is a real content table,
  // but folded by default — clicking the entry expands its TOC.
  return sections.length > 0 ? { text, link, collapsed: true, items: sections } : { text, link };
}

function pageWithChildren(text: string, link: string, children: SidebarItem[]): SidebarItem {
  const items = [...pageSections(link), ...children];
  return items.length > 0 ? { text, link, collapsed: true, items } : { text, link };
}

function orderedChildSlugs(indexContent: string, base: string): string[] {
  const escapedBase = base.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  return [...indexContent.matchAll(new RegExp(`/${escapedBase}/([^\\)/#]+)`, 'g'))]
    .map((match) => match[1])
    .filter((slug, index, all) => all.indexOf(slug) === index);
}

function directorySlugs(directory: string): string[] {
  return readdirSync(directory)
    .filter((file) => file.endsWith('.md'))
    .map((file) => file.replace(/\.md$/, ''))
    .sort();
}

function childPages(base: string): SidebarItem[] {
  const indexPath = resolve(docsRoot, `${base}.md`);
  const sectionDir = resolve(docsRoot, base);
  if (!existsSync(indexPath) || !existsSync(sectionDir)) {
    return [];
  }

  const indexContent = readFileSync(indexPath, 'utf8');
  const ordered = orderedChildSlugs(indexContent, base);
  const slugs = ordered.length > 0 ? ordered : directorySlugs(sectionDir);

  return slugs.map((slug) => page(pageTitle(resolve(sectionDir, `${slug}.md`)), `/${base}/${slug}`));
}

function splitIndex(text: string, base: string): SidebarItem {
  return pageWithChildren(text, `/${base}`, childPages(base));
}

function standardsIndex(text: string, category: string): SidebarItem {
  const base = `standards-mirror/${category}`;
  return pageWithChildren(text, `/${base}`, childPages(base));
}

function originalContractPages(): SidebarItem[] {
  const indexPath = markdownPathForLink('/solidity/original-contracts/');
  if (!existsSync(indexPath)) {
    return [];
  }

  const groups: SidebarItem[] = [];
  let currentGroup: SidebarItem | undefined;

  for (const line of readFileSync(indexPath, 'utf8').split(/\r?\n/)) {
    const heading = line.match(/^## (.+)$/);
    if (heading) {
      currentGroup = heading[1] === 'Project Summary' ? undefined : { text: heading[1], collapsed: true, items: [] };
      if (currentGroup) {
        groups.push(currentGroup);
      }
      continue;
    }

    const row = line.match(/^\| \[([^\]]+)\]\((\/solidity\/original-contracts\/[^/)]+\/[^)]+)\)/);
    if (!row || !currentGroup) {
      continue;
    }

    currentGroup.items = [...(currentGroup.items ?? []), page(stripInlineMarkdown(row[1]), row[2])];
  }

  return groups.filter((group) => (group.items ?? []).length > 0);
}

export default defineConfig({
  title: 'Neo DevPack for Solidity',
  description:
    'Production-grade Solidity to Neo N3 compiler — compile .sol to .nef + .manifest.json with EVM-to-Neo semantic mapping',
  lang: 'en-US',
  base: isCI ? '/neo-devpack-solidity/' : '/',
  cleanUrls: true,
  lastUpdated: true,
  srcExclude: [
    'archive/**',
    'plans/**',
    'ARCHITECTURE.md',
    'RUNTIME_SPEC.md',
    'ERROR_REFERENCE.md',
    'EXCELLENCE_ASSESSMENT.md',
    'FUZZ.md',
    'mapping_lowering_design.md',
    'NEO_VM_PARITY_TODO.md',
    'README.md',
    'SOLIDITY_SUPPORT_MATRIX.md',
    'public/**/*.md'
  ],
  markdown: {
    config(md) {
      md.core.ruler.after('block', 'neo_docs_overview_heading', (state) => {
        const tokens = state.tokens;

        if (
          hasHomeLayout(state.src) ||
          tokens.some((token) => token.type === 'heading_open' && /^h[2-3]$/.test(token.tag))
        ) {
          return;
        }

        const h1Index = tokens.findIndex((token) => token.type === 'heading_open' && token.tag === 'h1');
        if (h1Index === -1) {
          return;
        }

        const h1CloseIndex = tokens.findIndex(
          (token, index) => index > h1Index && token.type === 'heading_close' && token.tag === 'h1'
        );
        if (h1CloseIndex === -1) {
          return;
        }

        const openToken = new state.Token('heading_open', 'h2', 1);
        openToken.markup = '##';
        openToken.block = true;
        openToken.level = 0;
        const inlineToken = new state.Token('inline', '', 0);
        inlineToken.content = 'Overview';
        inlineToken.children = [];
        inlineToken.level = 1;
        const closeToken = new state.Token('heading_close', 'h2', -1);
        closeToken.markup = '##';
        closeToken.block = true;
        closeToken.level = 0;

        tokens.splice(h1CloseIndex + 1, 0, openToken, inlineToken, closeToken);
      });
    }
  },
  transformPageData(pageData) {
    if (
      pageData.headers.length === 0 &&
      pageData.frontmatter.layout !== 'home' &&
      pageData.relativePath.endsWith('.md')
    ) {
      pageData.headers.push({
        level: 2,
        title: 'Overview',
        slug: 'overview',
        link: '#overview',
        children: []
      });
    }
  },
  head: [
    ['meta', { name: 'theme-color', content: '#00E599' }],
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:title', content: 'Neo DevPack for Solidity — Compile Solidity to Neo N3' }],
    [
      'meta',
      {
        property: 'og:description',
        content:
          'Production-grade compiler that transforms Solidity 0.8.x contracts into deployable NeoVM artifacts (.nef + .manifest.json) with full EVM-to-Neo semantic mapping.'
      }
    ],
    ['meta', { property: 'og:image', content: '/assets/neo-devpack-solidity-logo.png' }],
    ['meta', { name: 'twitter:card', content: 'summary' }],
    ['meta', { name: 'twitter:title', content: 'Neo DevPack for Solidity — Compile Solidity to Neo N3' }],
    [
      'meta',
      {
        name: 'twitter:description',
        content:
          'Production-grade Solidity to Neo N3 compiler with broad Solidity 0.8.x support, manifest hardening, and Neo-Express integration.'
      }
    ],
    ['link', { rel: 'icon', type: 'image/png', href: '/assets/neo-devpack-solidity-logo.png' }]
  ],
  vite: {
    build: {
      chunkSizeWarningLimit: 1200
    }
  },
  themeConfig: {
    logo: '/assets/neo-devpack-solidity-logo.png',
    siteTitle: 'Neo DevPack for Solidity',
    nav: [
      { text: 'Getting Started', link: '/basics/introduction-to-smart-contracts', activeMatch: '^/basics/' },
      { text: 'Workflows',       link: '/workflows/',                              activeMatch: '^/workflows/' },
      { text: 'Solidity',        link: '/language-description/layout-of-source-file', activeMatch: '^/(language-description|solidity)/' },
      { text: 'Mapping',         link: '/mapping/',                                activeMatch: '^/mapping/' },
      { text: 'NeoVM',           link: '/internals/architecture',                  activeMatch: '^/internals/' },
      { text: 'Manifest',        link: '/manifest/',                               activeMatch: '^/manifest/' },
      { text: 'Devpack',         link: '/additional-material/neo-devpack/usage',   activeMatch: '^/additional-material/' },
      { text: 'ERC ↔ Neo',       link: '/standards-mirror/',                       activeMatch: '^/standards-mirror/' },
      { text: 'Reference',       link: '/reference/',                              activeMatch: '^/(reference|advisory-content|resources|compiler|use-cases)' },
      { text: 'Forum',           link: 'https://github.com/r3e-network/neo-devpack-solidity/discussions' }
    ],
    sidebar: {
      // ── Getting Started ────────────────────────────────────────────
      '/basics/': [
        {
          text: 'Getting Started',
          collapsed: false,
          items: [
            page('Introduction to Smart Contracts', '/basics/introduction-to-smart-contracts'),
            page('Solidity by Example', '/basics/solidity-by-example'),
            page('Installing the Compiler', '/basics/installing-the-compiler'),
            splitIndex('Quickstart', 'basics/quickstart'),
            splitIndex('Deploying Contracts', 'basics/deploying-contracts'),
            splitIndex('Testing Contracts', 'basics/testing-contracts'),
            page('Use Cases', '/use-cases')
          ]
        }
      ],

      // ── Workflows ──────────────────────────────────────────────────
      '/workflows/': [
        {
          text: 'Workflows',
          collapsed: false,
          items: [
            page('Overview', '/workflows/'),
            { text: 'Build & Compile', collapsed: false, items: [
              page('Installing the compiler', '/basics/installing-the-compiler'),
              splitIndex('Quickstart', 'basics/quickstart'),
              page('Using the compiler', '/compiler/using-the-compiler'),
              page('Analysing compiler output', '/compiler/analysing-the-compiler-output')
            ]},
            { text: 'Deploy', collapsed: false, items: [
              splitIndex('Deploying contracts', 'basics/deploying-contracts'),
              page('Famous contracts on neo-express', '/solidity/famous-contracts-neoxp-deploy'),
              page('Standards mirror — TestNet deployments', '/standards-mirror/deployments/RESULTS')
            ]},
            { text: 'Test & Verify', collapsed: false, items: [
              splitIndex('Testing contracts', 'basics/testing-contracts'),
              page('Fuzz testing', '/compiler/fuzz-testing'),
              page('TestNet runtime verification', '/solidity/famous-contracts-testnet-runtime')
            ]},
            { text: 'Debug & Diagnose', collapsed: false, items: [
              splitIndex('Troubleshooting', 'advisory-content/troubleshooting'),
              splitIndex('Error reference', 'advisory-content/error-reference'),
              page('Known bugs', '/advisory-content/known-bugs')
            ]},
            { text: 'Ship to Production', collapsed: false, items: [
              splitIndex('Production readiness', 'advisory-content/production-readiness'),
              page('Security considerations', '/advisory-content/security-considerations'),
              page('Breaking changes', '/advisory-content/breaking-changes')
            ]}
          ]
        }
      ],

      // ── Solidity (language reference) ──────────────────────────────
      '/language-description/': [
        {
          text: 'Solidity',
          collapsed: false,
          items: [
            splitIndex('EVM Feature Support', 'solidity/feature-support'),
            page('Layout of a Solidity Source File', '/language-description/layout-of-source-file'),
            page('Structure of a Contract', '/language-description/structure-of-a-contract'),
            page('Types', '/language-description/types'),
            page('Units and Globally Available Variables', '/language-description/units-and-global-variables'),
            page('Expressions and Control Structures', '/language-description/expressions-and-control-structures'),
            page('Contracts', '/language-description/contracts'),
            page('Inline Assembly', '/language-description/inline-assembly'),
            page('Cheatsheet', '/language-description/cheatsheet'),
            page('Language Grammar', '/language-description/grammar')
          ]
        },
        {
          text: 'Compiler',
          collapsed: false,
          items: [
            page('Using the Compiler', '/compiler/using-the-compiler'),
            page('Analysing the Compiler Output', '/compiler/analysing-the-compiler-output'),
            page('IR-based Codegen Changes', '/compiler/ir-codegen-changes'),
            page('Fuzz Testing', '/compiler/fuzz-testing')
          ]
        }
      ],
      '/solidity/': [
        {
          text: 'Solidity',
          collapsed: false,
          items: [
            splitIndex('EVM Feature Support', 'solidity/feature-support'),
            page('Layout of a Solidity Source File', '/language-description/layout-of-source-file'),
            page('Structure of a Contract', '/language-description/structure-of-a-contract'),
            page('Types', '/language-description/types'),
            page('Units and Globally Available Variables', '/language-description/units-and-global-variables'),
            page('Expressions and Control Structures', '/language-description/expressions-and-control-structures'),
            page('Contracts', '/language-description/contracts'),
            page('Inline Assembly', '/language-description/inline-assembly'),
            page('Cheatsheet', '/language-description/cheatsheet'),
            page('Language Grammar', '/language-description/grammar')
          ]
        },
        {
          text: 'Famous Contracts',
          collapsed: false,
          items: [
            page('Famous Contracts Audit', '/solidity/famous-contracts-neo-audit'),
            page('Neo-Express Deployment Matrix', '/solidity/famous-contracts-neoxp-deploy'),
            page('Type-3 Runtime Execution', '/solidity/famous-contracts-neoxp-runtime'),
            page('TestNet Runtime Verification', '/solidity/famous-contracts-testnet-runtime'),
            pageWithChildren('Original Source Code', '/solidity/original-contracts/', originalContractPages())
          ]
        }
      ],

      // ── Semantic Mapping ───────────────────────────────────────────
      '/mapping/': [
        {
          text: 'Mapping',
          collapsed: false,
          items: [
            page('Overview', '/mapping/'),
            page('Execution Context', '/mapping/execution-context'),
            page('Types and Values', '/mapping/types-and-values'),
            page('Storage and Mappings', '/mapping/storage-and-mappings'),
            page('Calls and Assets', '/mapping/calls-and-assets'),
            page('Syscalls and Devpack', '/mapping/syscalls-and-devpack'),
            page('Parity and Limitations', '/mapping/parity-and-limitations'),
            page('Standards Mapping', '/mapping/standards'),
            page('Indexed Storage Lowering', '/mapping/indexed-storage-lowering')
          ]
        }
      ],

      // ── NeoVM (internals) ──────────────────────────────────────────
      '/internals/': [
        {
          text: 'NeoVM',
          collapsed: false,
          items: [
            splitIndex('Architecture', 'internals/architecture'),
            splitIndex('Runtime Specification', 'internals/runtime-specification'),
            splitIndex('Syscalls', 'internals/syscalls'),
            splitIndex('Native Contracts', 'internals/native-contracts'),
            page('Source Mappings', '/internals/source-mappings'),
            page('The Optimizer', '/internals/the-optimizer'),
            page('Parity & Limitations', '/internals/parity-and-limitations')
          ]
        },
        {
          text: 'Memory & Storage Layout',
          collapsed: false,
          items: [
            page('Layout of State Variables in Storage', '/internals/layout-in-storage'),
            page('Layout in Memory', '/internals/layout-in-memory'),
            page('Layout of Call Data', '/internals/layout-of-call-data'),
            page('Cleaning Up Variables', '/internals/cleaning-up-variables')
          ]
        },
        {
          text: 'Manifest',
          collapsed: true,
          items: [
            splitIndex('Contract Metadata', 'internals/contract-metadata'),
            page('Contract ABI Specification', '/internals/contract-abi-specification')
          ]
        }
      ],

      // ── Manifest ───────────────────────────────────────────────────
      '/manifest/': [
        {
          text: 'Manifest',
          collapsed: false,
          items: [
            page('Overview', '/manifest/'),
            { text: 'Structure', collapsed: false, items: [
              splitIndex('Contract Metadata', 'internals/contract-metadata'),
              page('Contract ABI Specification', '/internals/contract-abi-specification')
            ]},
            { text: 'Permissions & Trust', collapsed: false, items: [
              page('Permission-conscious development', '/additional-material/neo-devpack/permission-conscious-development'),
              page('Native contract hash reference', '/additional-material/neo-devpack/native-contract-hash-reference'),
              page('Core contracts', '/additional-material/neo-devpack/core-contracts')
            ]},
            { text: 'Standards Declaration', collapsed: false, items: [
              page('Supported standards', '/additional-material/neo-standards/supported-standards'),
              page('Interface detection — EIP-165 vs Manifest', '/additional-material/neo-standards/interface-detection-eip-165-vs-manifest'),
              page('Standards auto-detection', '/additional-material/neo-standards/standards-auto-detection'),
              page('Compiler diagnostics for standards', '/additional-material/neo-standards/compiler-diagnostics-for-standards'),
              page('Testing standards compliance', '/additional-material/neo-standards/testing-standards-compliance')
            ]},
            { text: 'Manifest in the Compiler', collapsed: false, items: [
              page('Compiler intrinsics', '/additional-material/neo-devpack/compiler-intrinsics'),
              page('NatSpec format', '/additional-material/natspec-format')
            ]}
          ]
        }
      ],

      // ── Devpack ────────────────────────────────────────────────────
      '/additional-material/': [
        {
          text: 'Devpack',
          collapsed: false,
          items: [
            splitIndex('Overview', 'additional-material/neo-devpack'),
            page('Usage', '/additional-material/neo-devpack/usage'),
            page('Building custom contracts', '/additional-material/neo-devpack/building-custom-contracts'),
            page('Compiler intrinsics', '/additional-material/neo-devpack/compiler-intrinsics'),
            page('Libraries', '/additional-material/neo-devpack/libraries'),
            page('Token standards', '/additional-material/neo-devpack/token-standards'),
            page('Permission-conscious development', '/additional-material/neo-devpack/permission-conscious-development'),
            page('Core contracts', '/additional-material/neo-devpack/core-contracts'),
            page('Native contract hash reference', '/additional-material/neo-devpack/native-contract-hash-reference'),
            page('Directory layout', '/additional-material/neo-devpack/directory-layout'),
            page('See also', '/additional-material/neo-devpack/see-also')
          ]
        },
        {
          text: 'Neo Standards (NEP)',
          collapsed: false,
          items: [
            splitIndex('Overview', 'additional-material/neo-standards'),
            page('Supported standards', '/additional-material/neo-standards/supported-standards'),
            page('NEP-17 fungible tokens', '/additional-material/neo-standards/nep-17-fungible-tokens'),
            page('NEP-11 non-fungible tokens', '/additional-material/neo-standards/nep-11-non-fungible-tokens'),
            page('NEP-24 royalty standard', '/additional-material/neo-standards/nep-24-royalty-standard'),
            page('Standards auto-detection', '/additional-material/neo-standards/standards-auto-detection'),
            page('Interface detection — EIP-165 vs Manifest', '/additional-material/neo-standards/interface-detection-eip-165-vs-manifest'),
            page('Compiler diagnostics for standards', '/additional-material/neo-standards/compiler-diagnostics-for-standards'),
            page('Testing standards compliance', '/additional-material/neo-standards/testing-standards-compliance'),
            page('Ethereum to Neo standard mapping', '/additional-material/neo-standards/ethereum-to-neo-standard-mapping'),
            page('See also', '/additional-material/neo-standards/see-also')
          ]
        },
        {
          text: 'Other Material',
          collapsed: true,
          items: [
            page('NatSpec Format', '/additional-material/natspec-format'),
            page('SMTChecker and Formal Verification', '/additional-material/smtchecker'),
            page('Yul', '/additional-material/yul'),
            page('Import Path Resolution', '/additional-material/import-path-resolution')
          ]
        }
      ],

      // ── Reference ──────────────────────────────────────────────────
      '/reference/': [
        {
          text: 'Reference',
          collapsed: false,
          items: [
            page('Overview', '/reference/'),
            { text: 'Compiler & Language', collapsed: false, items: [
              splitIndex('EVM feature support matrix', 'solidity/feature-support'),
              page('Language grammar', '/language-description/grammar'),
              page('Cheatsheet', '/language-description/cheatsheet'),
              page('Keyword index', '/resources/keyword-index')
            ]},
            { text: 'Diagnostics & Errors', collapsed: false, items: [
              splitIndex('Error reference', 'advisory-content/error-reference'),
              page('Known bugs', '/advisory-content/known-bugs'),
              page('Breaking changes', '/advisory-content/breaking-changes')
            ]},
            { text: 'Runtime & VM', collapsed: false, items: [
              splitIndex('Runtime specification', 'internals/runtime-specification'),
              splitIndex('Native contracts', 'internals/native-contracts'),
              splitIndex('Syscalls', 'internals/syscalls'),
              splitIndex('Architecture', 'internals/architecture'),
              page('Parity & limitations', '/internals/parity-and-limitations')
            ]},
            { text: 'Standards Mirror', collapsed: false, items: [
              page('Overview', '/standards-mirror/'),
              page('Coverage matrix', '/standards-mirror/coverage-matrix'),
              page('Coverage audit & gaps', '/standards-mirror/coverage-audit'),
              page('TestNet deployments', '/standards-mirror/deployments/RESULTS'),
              page('Deferred deployments', '/standards-mirror/deployments/DEFERRED')
            ]},
            { text: 'Other', collapsed: true, items: [
              splitIndex('Production readiness checklist', 'advisory-content/production-readiness'),
              page('Use cases', '/use-cases'),
              page('Common patterns', '/resources/common-patterns')
            ]}
          ]
        }
      ],

      // ── Standards Mirror (its own detailed sidebar) ────────────────
      '/standards-mirror/': [
        {
          text: 'ERC / EIP ↔ Neo Mirror',
          collapsed: false,
          items: [
            page('Overview', '/standards-mirror/'),
            page('Coverage Matrix', '/standards-mirror/coverage-matrix'),
            page('Coverage Audit & Gaps', '/standards-mirror/coverage-audit'),
            standardsIndex('Token Standards', 'tokens'),
            standardsIndex('Account & Authentication', 'account-and-auth'),
            standardsIndex('Infrastructure & Patterns', 'infrastructure'),
            standardsIndex('DeFi Building Blocks', 'defi'),
            standardsIndex('Protocol-Level EIPs', 'protocol-eips'),
            pageWithChildren(
              'Deployment Reports',
              '/standards-mirror/deployments/README',
              [
                page('Last TestNet Results', '/standards-mirror/deployments/RESULTS'),
                page('Deferred Guardrails', '/standards-mirror/deployments/DEFERRED')
              ]
            )
          ]
        }
      ],

      // ── Advisory & Resources (kept as standalone Reference siblings) ─
      '/advisory-content/': [
        {
          text: 'Advisory Content',
          collapsed: false,
          items: [
            page('Security Considerations', '/advisory-content/security-considerations'),
            page('List of Known Bugs', '/advisory-content/known-bugs'),
            page('Breaking Changes', '/advisory-content/breaking-changes'),
            splitIndex('Troubleshooting', 'advisory-content/troubleshooting'),
            splitIndex('Error Reference', 'advisory-content/error-reference'),
            splitIndex('Production Readiness', 'advisory-content/production-readiness')
          ]
        }
      ],
      '/resources/': [
        {
          text: 'Resources',
          collapsed: false,
          items: [
            page('Style Guide', '/resources/style-guide'),
            page('Common Patterns', '/resources/common-patterns'),
            page('Resources', '/resources/resources'),
            page('Contributing', '/resources/contributing'),
            page('Language Influences', '/resources/language-influences'),
            page('Solidity Brand Guide', '/resources/brand-guide'),
            page('Keyword Index', '/resources/keyword-index')
          ]
        }
      ],
      '/compiler/': [
        {
          text: 'Compiler',
          collapsed: false,
          items: [
            page('Using the Compiler', '/compiler/using-the-compiler'),
            page('Analysing the Compiler Output', '/compiler/analysing-the-compiler-output'),
            page('IR-based Codegen Changes', '/compiler/ir-codegen-changes'),
            page('Fuzz Testing', '/compiler/fuzz-testing')
          ]
        }
      ]
    },
    socialLinks: [{ icon: 'github', link: 'https://github.com/r3e-network/neo-devpack-solidity' }],
    footer: {
      message: 'MIT Licensed',
      copyright: 'Copyright © R3E Network'
    },
    search: {
      provider: 'local'
    },
    outline: false,
    editLink: {
      pattern: 'https://github.com/r3e-network/neo-devpack-solidity/edit/main/docs/:path',
      text: 'Edit this page on GitHub'
    }
  }
});
