#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TOOLING_DIR="$REPO_ROOT/tooling"
DOCS_DIR="$REPO_ROOT/docs/public/api/typescript"

if [[ ! -d "$TOOLING_DIR" ]]; then
  echo "Tooling workspace not found; skipping TypeScript documentation generation."
  exit 0
fi

mkdir -p "$REPO_ROOT/docs/public/api"
rm -rf "$DOCS_DIR"
mkdir -p "$DOCS_DIR"

pushd "$TOOLING_DIR" >/dev/null

echo "Installing tooling dependencies..."
npm install || {
  echo "::warning::npm install failed; skipping TypeScript documentation."
  exit 0
}

generate_docs() {
  local pkg_path="$1"
  local pkg_name
  pkg_name="$(basename "$pkg_path")"
  if node -e "const fs = require('fs'); const pkg = JSON.parse(fs.readFileSync(process.argv[1], 'utf8')); process.exit(pkg.private === true ? 0 : 1)" "$pkg_path/package.json"; then
    echo "Skipping $pkg_name (private package)."
    return 0
  fi

  local src_dir="$pkg_path/src"
  local tsconfig="$pkg_path/tsconfig.json"
  local out_dir="$DOCS_DIR/$pkg_name"

  if [[ ! -d "$src_dir" || ! -f "$tsconfig" ]]; then
    echo "Skipping $pkg_name (missing src directory or tsconfig)."
    return 0
  fi

  echo "Generating TypeDoc output for $pkg_name..."
  if npx --yes typedoc --skipErrorChecking --tsconfig "$tsconfig" --out "$out_dir" "$src_dir"; then
    echo "Finished $pkg_name documentation."
    return 0
  fi

  echo "::warning::TypeDoc failed for $pkg_name; falling back to README."
  mkdir -p "$out_dir"
  if [[ -f "$pkg_path/README.md" ]]; then
    cp "$pkg_path/README.md" "$out_dir/README.md"
  else
    {
      echo "# $pkg_name"
      echo ""
      echo "Documentation is not available yet."
    } >"$out_dir/README.md"
  fi
}

for pkg in packages/*; do
  [[ -d "$pkg" ]] || continue
  generate_docs "$pkg"
done

popd >/dev/null
