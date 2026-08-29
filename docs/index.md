---
layout: home

title: Neo DevPack for Solidity
titleTemplate: Compile Solidity to Neo N3

hero:
  name: "{NEO_DEVPACK_SOLIDITY}"
  text: "Compile Solidity to Neo N3"
  tagline: "A statically-typed curly-braces programming language compiler designed for developing smart contracts that run on the Neo N3 blockchain."
  image:
    src: /assets/neo-devpack-solidity-logo.png
    alt: Neo DevPack for Solidity
  actions:
    - theme: brand
      text: Read the docs
      link: /basics/introduction-to-smart-contracts
    - theme: alt
      text: Repository
      link: https://github.com/r3e-network/neo-devpack-solidity
---

<div class="vp-doc home-section">

<div class="solidity-keywords-cloud">
  <code>pragma</code> <code>contract</code> <code>function</code> <code>modifier</code> <code>event</code> <code>struct</code> <code>enum</code> <code>require</code> <code>address</code>
</div>

<br/>

<div class="alert-box">
  <div class="alert-content">
    <h3>Neo DevPack for Solidity v0.27.0</h3>
    <p>Version 0.27.0 (compiler / CLI / workspace and devpack aligned) is the <strong>real-world compatibility &amp; performance</strong> release. The merge/inheritance gaps that blocked flagship protocols are fixed — Aave v3 compilation went <strong>71% → 85%</strong> (every deployable implementation compiles), the Compound v2 core stack is unblocked, and <code>new X{salt: s}()</code> is accepted. Latest-Solidity semantics hardened: <code>bytesN</code> shifts on the big-endian face value, unchecked <code>int256</code> two's-complement wrap, fixed-size array <code>.length</code>, free-function overloads, plus loud <code>W_USER_DEFINED_OPERATOR</code>/<code>W_TRANSIENT_PERSISTED</code> warnings instead of silent miscompiles. The in-tree NeoVM is <strong>~1.9× faster</strong> on compute-heavy tests, a <strong>33-contract hermetic famous-sample corpus</strong> ships in-tree, and a 63-repro validation matrix confirms <strong>zero regressions</strong>. Full suite: <strong>1,964 tests / 55 targets green</strong>. Targets Neo N3 <strong>v3.10.0</strong>.</p>
    <a href="https://github.com/r3e-network/neo-devpack-solidity/releases/tag/v0.27.0" target="_blank">Read the full release notes &rarr;</a>
  </div>
</div>

<br/>

<h2 class="solidity-section-header">NEO DEVPACK FOR SOLIDITY IS EVOLVING RAPIDLY</h2>

Our release cycle prioritizes both stability for production and rapid innovation for developers transitioning from EVM to Neo N3. We regularly ship **non-breaking minor releases** with enhanced mappings, deeper standard support (like NEP-17/NEP-11), and expanded diagnostic intelligence.

<div style="margin-top: 1.5rem;">
  <a href="/basics/introduction-to-smart-contracts" class="cta-button">Get started</a>
</div>

<hr class="solidity-hr" />

<h2 class="solidity-section-header">ERC ↔ NEO STANDARDS MIRROR</h2>

Every meaningful Ethereum standard, mapped to its Neo N3 counterpart. **129 mirror pages** across five categories with side-by-side Solidity and Neo C# implementations; **47 ERC ↔ Neo pairs deployed live on TestNet** with assertion results.

<div class="contribute-grid">
  <div class="contribute-card">
    <h4>Token Standards</h4>
    <p>NEP-17 / NEP-11 / NEP-24 — fungible, non-fungible, multi-token, royalty, RWA, SBT, multi-privilege.</p>
    <a href="/standards-mirror/tokens">Explore tokens &rarr;</a>
  </div>
  <div class="contribute-card">
    <h4>Account & Authentication</h4>
    <p>EIP-712, ERC-1271, ERC-2612 (permit), ERC-4337 (account abstraction), SIWE, paymasters.</p>
    <a href="/standards-mirror/account-and-auth">Explore auth &rarr;</a>
  </div>
  <div class="contribute-card">
    <h4>Infrastructure & Patterns</h4>
    <p>EIP-165, ERC-1167 (clones), ERC-1967 (proxies), ERC-2535 (diamond), ERC-2771 (meta-tx), multicall.</p>
    <a href="/standards-mirror/infrastructure">Explore infra &rarr;</a>
  </div>
  <div class="contribute-card">
    <h4>DeFi Building Blocks</h4>
    <p>ERC-4626 vaults (sync, async, cancellable), ERC-3156 flash loans, ERC-5805 governance, bonds.</p>
    <a href="/standards-mirror/defi">Explore DeFi &rarr;</a>
  </div>
  <div class="contribute-card">
    <h4>Protocol-Level EIPs</h4>
    <p>EIP-1559, EIP-4844, transaction-format EIPs, opcode-only EIPs — what Neo already does natively.</p>
    <a href="/standards-mirror/protocol-eips">Explore protocol &rarr;</a>
  </div>
  <div class="contribute-card">
    <h4>Coverage Matrix & Audit</h4>
    <p>Live deploy status, EIP status, Neo mapping, and gap report for every catalog entry.</p>
    <a href="/standards-mirror/coverage-matrix">View matrix &rarr;</a>
  </div>
</div>

<div style="margin-top: 1.5rem; text-align: center;">
  <a href="/standards-mirror/" class="cta-button">Open the standards mirror</a>
</div>

<hr class="solidity-hr" />

<h2 class="solidity-section-header">CONTRIBUTE TO NEO DEVPACK FOR SOLIDITY</h2>

Neo DevPack for Solidity is an open-source project. We welcome developers, auditors, and technical writers to shape the future of smart contract development on Neo N3.

<div class="contribute-grid">
  <div class="contribute-card">
    <h4>Reporting issues and vulnerabilities</h4>
    <p>Found a bug or an EVM feature that doesn't map correctly? Let us know on GitHub so we can fix it.</p>
    <a href="https://github.com/r3e-network/neo-devpack-solidity/issues">Report an issue &rarr;</a>
  </div>
  <div class="contribute-card">
    <h4>Translating the documentation</h4>
    <p>Help us write tutorials, translate pages, or expand our EVM-to-NeoVM mapping guides.</p>
    <a href="https://github.com/r3e-network/neo-devpack-solidity/tree/main/docs">Edit Docs &rarr;</a>
  </div>
  <div class="contribute-card">
    <h4>Fixing and responding to issues</h4>
    <p>Looking for a place to start? Check our repository for issues tagged with <code>good first issue</code>.</p>
    <a href="https://github.com/r3e-network/neo-devpack-solidity/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22">Browse issues &rarr;</a>
  </div>
  <div class="contribute-card">
    <h4>Contributing to language design</h4>
    <p>Propose new intrinsics or discuss how EVM paradigms should translate to NeoVM.</p>
    <a href="https://github.com/r3e-network/neo-devpack-solidity/discussions">Join Discussions &rarr;</a>
  </div>
</div>

<div style="margin-top: 1.5rem; text-align: center;">
  <a href="/resources/contributing" class="cta-button">Start contributing</a>
</div>

<hr class="solidity-hr" />

<h2 class="solidity-section-header" style="text-align: left;">PLAYGROUND</h2>

<p>Try Neo DevPack for Solidity for yourself. See how standard Ethereum paradigms translate effortlessly to Neo N3 artifacts.</p>

<div class="playground-mockup">
  <div class="playground-sidebar">
    <div class="tab active">SimpleStorage.sol</div>
    <div class="tab">ERC20Token.sol</div>
    <div class="tab">SimpleAuction.sol</div>
  </div>
  <div class="playground-editor">

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.34;

import {Runtime} from "@neo/Runtime.sol";

contract SimpleStorage {
    uint256 private storedData;
    address public owner;

    constructor() {
        owner = Runtime.getCallingScriptHash();
    }

    function set(uint256 x) public {
        require(Runtime.checkWitness(owner), "Unauthorized");
        storedData = x;
    }

    function get() public view returns (uint256) {
        return storedData;
    }
}
```

  </div>
  <div class="playground-output">
    <div class="output-tab active">Compiler result</div>
    <div class="output-tab">Deployment costs</div>
    <div class="output-tab">Bytecode (NEF)</div>
    <div class="output-tab">Manifest</div>
    
    <div class="output-content">
      <strong>Compiler version:</strong> neo-solc 0.27.0<br/><br/>
      ✅ Compilation successful.<br/><br/>
      <strong>Methods:</strong><br/>
      - _deploy(any, bool)<br/>
      - set(uint256)<br/>
      - get() (safe)<br/><br/>
      <strong>Permissions:</strong> Restricted (No wildcards)
    </div>
  </div>
</div>

<hr class="solidity-hr" />

<h2 class="solidity-section-header">NEO DEVPACK FOR SOLIDITY EVENTS</h2>

<h3 style="font-size: 1rem; color: var(--vp-c-text-2); letter-spacing: 0.1em; text-transform: uppercase;">UPCOMING EVENTS</h3>

<p>No upcoming events currently scheduled. Check back later!</p>

</div>
