#!/usr/bin/env bash
# ─── schema-export.sh ──────────────────────────────────────────────────
# Generate deterministic JSON Schema files from the engine's Rust types.
#
# Run from the repository root.
# Output: engine/schemas/*.json

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

OUT_DIR="engine/schemas"
mkdir -p "$OUT_DIR"

echo "[schema-export] Generating schemas in $OUT_DIR ..."
cargo run --quiet --package steel-horizons-engine --bin schema_export -- "$OUT_DIR"
echo "[schema-export] Done — $(ls "$OUT_DIR"/*.json 2>/dev/null | wc -l) schemas."
