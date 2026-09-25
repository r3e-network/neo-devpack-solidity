#!/usr/bin/env python3
"""Validate and render the canonical Solidity support matrix."""
from __future__ import annotations

import argparse
import json
import re
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REGISTRY = ROOT / "docs" / "data" / "solidity_support_matrix.json"
DOC = ROOT / "docs" / "SOLIDITY_SUPPORT_MATRIX.md"
FEATURE_DOC = ROOT / "FEATURE_MATRIX.md"
ALLOWED = {"supported", "approximate", "manual_migration", "unsupported", "blocked"}
ICON = {"supported": "✅", "approximate": "⚠️", "manual_migration": "🔁", "unsupported": "❌", "blocked": "🚫"}


def load() -> dict:
    with REGISTRY.open(encoding="utf-8") as stream:
        return json.load(stream)


def validate(registry: dict) -> list[str]:
    errors: list[str] = []
    features = registry.get("features")
    if not isinstance(features, list) or not features:
        return ["features must be a non-empty array"]
    ids: set[str] = set()
    if registry.get("schemaVersion") != 1:
        errors.append(f"schemaVersion must be 1, got {registry.get('schemaVersion')!r}")
    for required in ("sourceDate", "compilerVersionSource"):
        if not isinstance(registry.get(required), str) or not registry[required].strip():
            errors.append(f"{required} must be non-empty")
    for index, item in enumerate(features):
        if not isinstance(item, dict):
            errors.append(f"features[{index}] must be an object")
            continue
        ident = item.get("id")
        if not isinstance(ident, str) or not ident:
            errors.append(f"features[{index}] has no non-empty id")
        elif ident in ids:
            errors.append(f"duplicate feature id: {ident}")
        else:
            ids.add(ident)
        status = item.get("status")
        if status not in ALLOWED:
            errors.append(f"{ident or index}: invalid status {status!r}")
        for axis in ("compileStatus", "runtimeStatus", "strictCompileStatus"):
            if not isinstance(item.get(axis), str) or not item[axis].strip():
                errors.append(f"{ident or index}: {axis} must be non-empty")
        for field in ("category", "syntax", "notes"):
            if not isinstance(item.get(field), str) or not item[field].strip():
                errors.append(f"{ident or index}: {field} must be non-empty")
        refs = item.get("sourceRefs")
        if not isinstance(refs, list) or not refs or not all(isinstance(ref, str) and ref for ref in refs):
            errors.append(f"{ident or index}: sourceRefs must contain strings")
        if status in {"blocked", "unsupported"}:
            alternative = item.get("alternative")
            if not isinstance(alternative, str) or not alternative.strip():
                errors.append(f"{ident or index}: {status} requires alternative guidance")
    return errors


def statistics(features: list[dict]) -> Counter[str]:
    return Counter(item["status"] for item in features)


def render_summary(registry: dict) -> str:
    counts = statistics(registry["features"])
    lines = [
        "<!-- Generated from docs/data/solidity_support_matrix.json; do not edit counts manually. -->",
        "## Summary",
        "",
        "| Status | Count |",
        "| --- | ---: |",
    ]
    for status in ("supported", "approximate", "manual_migration", "unsupported", "blocked"):
        lines.append(f"| {ICON[status]} {status} | {counts.get(status, 0)} |")
    lines.extend([f"| **Total** | **{len(registry['features'])}** |", ""])
    return "\n".join(lines)


def update_docs(registry: dict) -> None:
    summary = render_summary(registry)
    text = DOC.read_text(encoding="utf-8")
    text = re.sub(r"<!-- Generated from docs/data/solidity_support_matrix\.json;.*?\n## Summary\n.*?(?=\n\*\*Migration|\Z)", summary, text, flags=re.S)
    if summary not in text:
        marker = "\n---\n\n## Summary"
        start = text.find(marker)
        if start >= 0:
            text = text[:start] + "\n---\n\n" + summary + "\n"
    DOC.write_text(text.rstrip() + "\n", encoding="utf-8")
    FEATURE_DOC.write_text(
        "# Feature Matrix\n\n"
        "The canonical Solidity feature matrix and generated summary live in "
        "[`docs/SOLIDITY_SUPPORT_MATRIX.md`](./docs/SOLIDITY_SUPPORT_MATRIX.md).\n\n"
        "Canonical registry: [`docs/data/solidity_support_matrix.json`](./docs/data/solidity_support_matrix.json).\n\n"
        "Run `python3 scripts/generate_solidity_support_matrix.py --check` to validate the registry and generated summary.\n",
        encoding="utf-8",
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true", help="validate registry and generated documentation")
    parser.add_argument("--explain", metavar="ID", help="show one registry entry")
    args = parser.parse_args()
    registry = load()
    errors = validate(registry)
    if errors:
        for error in errors:
            print(f"error: {error}", file=sys.stderr)
        return 1
    if args.explain:
        match = next((item for item in registry["features"] if item["id"] == args.explain), None)
        if match is None:
            print(f"unknown feature id: {args.explain}", file=sys.stderr)
            return 1
        print(json.dumps(match, ensure_ascii=False, indent=2))
        return 0
    if args.check:
        expected = render_summary(registry)
        doc = DOC.read_text(encoding="utf-8")
        if expected not in doc:
            print("error: support matrix summary is stale; run generator without --check", file=sys.stderr)
            return 1
        source_rows = sum(
            1 for line in doc.splitlines()
            if line.startswith("| ") and line.count("|") >= 4
            and any(icon in line for icon in ("✅", "⚠️", "❌", "🚫"))
        )
        if source_rows != len(registry["features"]):
            print(
                f"error: registry has {len(registry['features'])} features but matrix has {source_rows} feature rows",
                file=sys.stderr,
            )
            return 1
        print(f"valid registry: {len(registry['features'])} features; " + ", ".join(f"{k}={v}" for k, v in sorted(statistics(registry['features']).items())))
        return 0
    update_docs(registry)
    print(f"generated support matrix summary for {len(registry['features'])} features")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
