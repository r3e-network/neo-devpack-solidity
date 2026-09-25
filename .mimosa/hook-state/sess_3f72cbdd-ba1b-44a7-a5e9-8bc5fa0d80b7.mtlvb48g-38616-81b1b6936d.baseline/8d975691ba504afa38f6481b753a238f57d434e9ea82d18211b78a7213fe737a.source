#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUNTIME_DIR="$REPO_ROOT/src/Neo.Sol.Runtime"
DOCS_DIR="$REPO_ROOT/docs/public/api/csharp"

if [[ ! -d "$RUNTIME_DIR" ]]; then
  echo "Neo.Sol.Runtime directory not found; skipping C# documentation."
  exit 0
fi

rm -rf "$DOCS_DIR"
mkdir -p "$DOCS_DIR"

pushd "$RUNTIME_DIR" >/dev/null

echo "Building Neo.Sol.Runtime (Release) with XML docs..."
if dotnet build --configuration Release /p:GenerateDocumentationFile=true; then
  echo "Copying generated XML files into docs tree..."
  find . -maxdepth 5 -name "*.xml" -print0 | while IFS= read -r -d '' file; do
    cp "$file" "$DOCS_DIR/"
  done
else
  echo "::warning::Failed to build Neo.Sol.Runtime; using README as placeholder documentation."
  if [[ -f "README.md" ]]; then
    cp "README.md" "$DOCS_DIR/README.md"
  fi
fi

popd >/dev/null
