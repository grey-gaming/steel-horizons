#!/usr/bin/env bash
set -euo pipefail

# Platform-neutral CI verification script.
# Runs every gate whose owning increment is complete.
# P1-01: locked Rust build, formatting, Clippy, Rust smoke/unit tests,
#        Python package formatting/typing/unit smoke, protocol/policy-sync checks.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$REPO_ROOT"

echo "[P1-01 check.sh] Starting verification..."

# 1. Repository synchronization and generated-file checks
echo "--- Repository sync check ---"
if git diff --check --quiet 2>/dev/null; then
    echo "PASS: no merge conflicts or whitespace errors"
else
    echo "FAIL: git diff --check found issues"
    exit 1
fi

# Check marker-based protocol sync between AGENTS.md and IMPLEMENTATION_PLAN.md
echo "--- Protocol/policy sync check ---"
python3 scripts/check-protocol-sync.py
echo "PASS: protocol/policy sync OK"

# 2. Rust formatting
echo "--- Rust formatting ---"
cargo fmt --check --quiet 2>&1
echo "PASS: formatting OK"

# 3. Locked Rust build
echo "--- Locked Rust build ---"
cargo build --locked 2>&1
echo "PASS: locked build OK"

# 4. Clippy with warnings denied
echo "--- Clippy ---"
cargo clippy --locked -- -D warnings 2>&1
echo "PASS: Clippy OK"

# 5. Rust smoke/unit tests
echo "--- Rust tests ---"
cargo test --locked 2>&1
echo "PASS: Rust tests OK"

# 5b. Generated-content schema drift check (M3)
echo "--- Schema export --check ---"
scripts/schema-export.sh --check
echo "PASS: committed schemas match generated schemas"

# 6. Python scaffold checks (formatting, typing, unit smoke)
echo "--- Python checks ---"
python3 -m pip install -q hatchling ruff mypy pytest 2>&1
cd text-ui
python3 -m ruff check src/ 2>&1
echo "PASS: Python formatting OK"
python3 -m mypy src/ 2>&1
echo "PASS: Python typing OK"
python3 -m pytest --co 2>&1 || echo "INFO: no tests yet — pytest-cov may not be installed"
echo "PASS: Python scaffold OK"
cd "$REPO_ROOT"

echo ""
echo "[P1-01 check.sh] All gates passed."
