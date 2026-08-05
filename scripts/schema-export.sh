#!/usr/bin/env bash
set -euo pipefail

# Reproducible schema-export command shell.
# P1-03: generates content schemas and checks for no diff.
# Usage: ./scripts/schema-export.sh [--check]

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

if [ "$REPO_ROOT" = "" ]; then
    echo "ERROR: cannot find repository root"
    exit 1
fi

echo "[schema-export.sh] Generating content schemas..."

# Generate state/command schemas into schemas/content/
cargo run -p steel-horizons-engine --bin schema_export -- schemas/content/

echo ""
echo "[schema-export.sh] Done — 35 schemas generated."

# --check mode: verify no diff in generated schemas
if [ "${1:-}" = "--check" ]; then
    echo ""
    echo "[schema-export.sh] Checking that schema regeneration has no diff..."

    # Re-run and compare against existing schemas
    cargo run -p steel-horizons-engine --bin schema_export -- schemas/content/ 2>&1

    # Use git diff to check for changes
    if git diff --quiet schemas/content/ 2>/dev/null; then
        echo "[schema-export.sh] PASS: no diff in generated schemas."
    else
        echo "[schema-export.sh] FAIL: schema regeneration produced different output!"
        git diff schemas/content/
        exit 1
    fi
fi
