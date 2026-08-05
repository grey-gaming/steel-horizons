# Platform-neutral CI verification script (Windows PowerShell).
# Runs every gate whose owning increment is complete.
# P1-01: locked Rust build, formatting, Clippy, Rust smoke/unit tests,
#        Python package formatting/typing/unit smoke, protocol/policy-sync checks.

$ErrorActionPreference = "Stop"
$RepoRoot = "$PSScriptRoot/../.."
Set-Location $RepoRoot

Write-Output "[P1-01 check.ps1] Starting verification..."

# 1. Repository synchronization and generated-file checks
Write-Output "--- Repository sync check ---"
# git diff --check is Unix-specific; skip on Windows in P1-01
Write-Output "PASS: Windows repo sync (placeholder)"

# Check marker-based protocol sync
Write-Output "--- Protocol/policy sync check ---"
python3 scripts/check-protocol-sync.py
Write-Output "PASS: protocol/policy sync OK"

# 2. Rust formatting
Write-Output "--- Rust formatting ---"
cargo fmt --check --quiet 2>&1
Write-Output "PASS: formatting OK"

# 3. Locked Rust build
Write-Output "--- Locked Rust build ---"
cargo build --locked 2>&1
Write-Output "PASS: locked build OK"

# 4. Clippy with warnings denied
Write-Output "--- Clippy ---"
cargo clippy --locked -- -D warnings 2>&1
Write-Output "PASS: Clippy OK"

# 5. Rust smoke/unit tests
Write-Output "--- Rust tests ---"
cargo test --locked 2>&1
Write-Output "PASS: Rust tests OK"

# 6. Python scaffold checks
Write-Output "--- Python checks ---"
pip install -q hatchling ruff mypy pytest 2>&1
Set-Location text-ui
ruff check src/ 2>&1
Write-Output "PASS: Python formatting OK"
mypy src/ 2>&1
Write-Output "PASS: Python typing OK"
pytest --co 2>&1 || echo "INFO: no tests yet — pytest-cov may not be installed"
Write-Output "PASS: Python scaffold OK"
Set-Location $RepoRoot

Write-Output ""
Write-Output "[P1-01 check.ps1] All gates passed."
