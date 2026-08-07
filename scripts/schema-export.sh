#!/usr/bin/env bash
# ─── schema-export.sh ──────────────────────────────────────────────────
# Generate deterministic JSON Schema files from the engine's Rust types.
#
# Run from the repository root.
# Output: engine/schemas/*.json
#
# `--check` mode regenerates into a temp dir and diffs against the committed
# engine/schemas/, failing if the generated schemas drift from the committed
# ones (M3: wired into scripts/ci/check.sh so CI catches schema drift).

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

OUT_DIR="engine/schemas"
mkdir -p "$OUT_DIR"

if [[ "${1:-}" == "--check" ]]; then
    TMP="$(mktemp -d)"
    echo "[schema-export --check] Regenerating schemas into $TMP ..."
    cargo run --quiet --package steel-horizons-engine --bin schema_export -- "$TMP"
    if diff -rq "$TMP" "$OUT_DIR" >/dev/null; then
        echo "[schema-export --check] PASS: generated schemas match committed schemas."
        rm -rf "$TMP"
        exit 0
    else
        echo "[schema-export --check] FAIL: generated schemas differ from committed."
        diff -rq "$TMP" "$OUT_DIR"
        rm -rf "$TMP"
        exit 1
    fi
fi

echo "[schema-export] Generating schemas in $OUT_DIR ..."
cargo run --quiet --package steel-horizons-engine --bin schema_export -- "$OUT_DIR"
echo "[schema-export] Done — $(ls "$OUT_DIR"/*.json 2>/dev/null | wc -l) schemas."
