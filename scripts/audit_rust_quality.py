#!/usr/bin/env python3
"""Audit production Rust for panic-prone calls and documented expectations.

The audit intentionally scans only ``src`` production modules. Test modules and
``#[cfg(test)]`` module bodies are excluded, while comments and string literals
are masked before matching so prose such as ``// call unwrap()`` is harmless.
Forbidden production calls (unwrap/panic!/unimplemented!/todo!) are gate
failures. ``expect()`` is reported separately and becomes a failure only when
``--strict-expect`` is supplied, allowing gradual migration without hiding
findings.
"""
from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path

CALL_PATTERNS: dict[str, re.Pattern[str]] = {
    "unwrap": re.compile(r"\.unwrap\s*\("),
    "panic": re.compile(r"\bpanic!\s*\("),
    "unimplemented": re.compile(r"\bunimplemented!\s*\("),
    "todo": re.compile(r"\btodo!\s*\("),
    "expect": re.compile(r"\.expect\s*\("),
}
INVARIANT_WORDS = re.compile(
    r"\b(invariant|guarantee|guaranteed|infallible|validated|valid|unreachable|"
    r"constant|static|must|safe|cannot fail|should never)\b",
    re.IGNORECASE,
)
TEST_PATH_PARTS = {"test", "tests", "benches", "fuzz", "examples"}


@dataclass(frozen=True)
class Finding:
    """A source finding with a stable path and one-based line number."""

    path: Path
    line: int
    kind: str
    text: str
    documented: bool = False
    allowlisted: bool = False


def mask_non_code(source: str) -> str:
    """Replace comments and literals with spaces, retaining newlines/positions."""
    chars = list(source)
    i = 0
    state = "code"
    while i < len(chars):
        current = chars[i]
        following = chars[i + 1] if i + 1 < len(chars) else ""
        if state == "code":
            # Rust raw strings (r"...", r#"..."#) may contain comment markers,
            # cfg attributes, or call-like text; consume them as one literal.
            if current == "r":
                raw = re.match(r'r(?P<hashes>#{0,16})"', source[i:])
                if raw:
                    hashes = raw.group("hashes")
                    closing = '"' + ("#" * len(hashes))
                    end = source.find(closing, i + len(raw.group(0)))
                    if end >= 0:
                        end += len(closing)
                        for index in range(i, end):
                            if chars[index] != "\n":
                                chars[index] = " "
                        i = end
                        continue
            if current == "/" and following == "/":
                chars[i] = chars[i + 1] = " "
                i += 2
                state = "line_comment"
                continue
            if current == "/" and following == "*":
                chars[i] = chars[i + 1] = " "
                i += 2
                state = "block_comment"
                continue
            if current == '"':
                chars[i] = " "
                state = "string_double"
        elif state == "line_comment":
            if current == "\n":
                state = "code"
            else:
                chars[i] = " "
        elif state == "block_comment":
            if current == "*" and following == "/":
                chars[i] = chars[i + 1] = " "
                i += 2
                state = "code"
                continue
            if current != "\n":
                chars[i] = " "
        elif state in {"string_double", "string_single"}:
            quote = '"' if state == "string_double" else "'"
            if current == "\\":
                chars[i] = " "
                if i + 1 < len(chars) and chars[i + 1] != "\n":
                    chars[i + 1] = " "
                    i += 1
            elif current == quote:
                chars[i] = " "
                state = "code"
            elif current != "\n":
                chars[i] = " "
        i += 1
    return "".join(chars)


def cfg_test_ranges(masked: str) -> list[tuple[int, int]]:
    """Return character ranges occupied by ``#[cfg(test)] mod ... { ... }``."""
    ranges: list[tuple[int, int]] = []
    marker = re.compile(r"#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]")
    for match in marker.finditer(masked):
        module = re.search(r"\bmod\s+[A-Za-z_][A-Za-z0-9_]*\s*\{", masked[match.end() :])
        if not module:
            continue
        open_brace = match.end() + module.end() - 1
        depth = 0
        for index in range(open_brace, len(masked)):
            if masked[index] == "{":
                depth += 1
            elif masked[index] == "}":
                depth -= 1
                if depth == 0:
                    ranges.append((match.start(), index + 1))
                    break
    return ranges


def in_ranges(position: int, ranges: list[tuple[int, int]]) -> bool:
    return any(start <= position < end for start, end in ranges)


def is_test_path(path: Path) -> bool:
    parts = set(path.parts)
    return bool(parts & TEST_PATH_PARTS) or path.name.endswith("_test.rs") or path.name == "tests.rs"


def adjacent_invariant(lines: list[str], line_number: int) -> bool:
    """Check the immediately adjacent source comments for invariant rationale."""
    index = line_number - 1
    neighbors = []
    for candidate in (index - 1, index + 1):
        if 0 <= candidate < len(lines):
            text = lines[candidate].strip()
            if text.startswith("//") or text.startswith("///") or text.startswith("/*") or text.startswith("*"):
                neighbors.append(text)
    return any(INVARIANT_WORDS.search(text) for text in neighbors)


def read_expect_whitelist(root: Path) -> set[tuple[str, str]]:
    """Read simple path|fingerprint entries for reviewed production expects."""
    path = root / "scripts" / "expect-whitelist.txt"
    if not path.is_file():
        return set()
    entries: set[tuple[str, str]] = set()
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#") or "|" not in line:
            continue
        file_name, fingerprint = (part.strip() for part in line.split("|", 1))
        entries.add((file_name.replace("\\", "/"), fingerprint))
    return entries


def scan(root: Path) -> list[Finding]:
    findings: list[Finding] = []
    expect_whitelist = read_expect_whitelist(root)
    source_root = root / "src"
    if not source_root.is_dir():
        return findings
    for path in sorted(source_root.rglob("*.rs")):
        relative = path.relative_to(root)
        if is_test_path(relative):
            continue
        source = path.read_text(encoding="utf-8", errors="replace")
        masked = mask_non_code(source)
        ranges = cfg_test_ranges(masked)
        lines = source.splitlines()
        for kind, pattern in CALL_PATTERNS.items():
            for match in pattern.finditer(masked):
                if in_ranges(match.start(), ranges):
                    continue
                line = source.count("\n", 0, match.start()) + 1
                findings.append(
                    Finding(
                        path=relative,
                        line=line,
                        kind=kind,
                        text=lines[line - 1].strip() if line <= len(lines) else "",
                        documented=kind == "expect" and adjacent_invariant(lines, line),
                        allowlisted=kind == "expect" and (
                            (str(relative).replace("\\", "/"), lines[line - 1].strip()) in expect_whitelist
                        ),
                    )
                )
    return sorted(findings, key=lambda item: (str(item.path), item.line, item.kind))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument(
        "--strict-expect",
        action="store_true",
        help="fail when a production expect() lacks an adjacent invariant comment",
    )
    args = parser.parse_args()
    findings = scan(args.root.resolve())
    forbidden = [item for item in findings if item.kind != "expect"]
    expects = [item for item in findings if item.kind == "expect"]
    missing_invariants = [item for item in expects if not item.documented and not item.allowlisted]

    print("== Production Rust panic/unwrap audit ==")
    print("Scope: src/**/*.rs (test paths and #[cfg(test)] modules excluded)")
    for item in findings:
        suffix = " [invariant comment: ok]" if item.kind == "expect" and item.documented else ""
        if item.kind == "expect" and item.allowlisted:
            suffix = " [reviewed allowlist]"
        print(f"{item.path}:{item.line}: {item.kind}(): {item.text}{suffix}")
    print(f"Forbidden findings (unwrap/panic/unimplemented/todo): {len(forbidden)}")
    print(f"Production expect() findings: {len(expects)}")
    print(f"expect() entries without adjacent invariant comment: {len(missing_invariants)}")
    if forbidden:
        return 1
    if args.strict_expect and missing_invariants:
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
