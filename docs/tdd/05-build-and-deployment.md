---
status: Approved
owner: Tech Lead
date: 2026-08-04
---

# Build & Deployment

## Phase 1 Deliverables

- `steel-horizons-engine` native Rust binary
- Python package/entry point `steel-horizons-text-ui`
- Versioned canonical content JSON
- Python agent play-tests

Phase 2 adds a Tauri desktop bundle containing the PixiJS v8 client and managed engine child process.

## Repository Layout

```text
Cargo.toml                 # workspace containing engine
engine/Cargo.toml
engine/src/
content/
text-ui/pyproject.toml
tests/playtest/
```

The text UI is Python and is not shown as a Cargo crate. API integration tests live under `engine/tests/`.

## Rust Dependencies

```toml
[dependencies]
axum = { version = "0.8", features = ["ws"] }
tokio = { version = "1", features = ["macros", "rt-multi-thread", "signal", "sync", "time"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tower-http = { version = "0.6", features = ["trace"] }
clap = { version = "4", features = ["derive"] }
dirs = "6"
getrandom = "0.3"             # nondeterministic API session token only
thiserror = "2"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

[dev-dependencies]
proptest = "1"
reqwest = { version = "0.12", features = ["json"] }
tokio-tungstenite = "0.26"
```

The engine implements its simulation PRNG directly and golden-tests it. `getrandom` is used only for the nondeterministic API session token and never enters `GameState`. Dependency versions are resolved by committed `Cargo.lock`; automated updates must pass cross-platform replay tests.

Permissive CORS is intentionally absent. Browser Origin validation is implemented against an explicit allowlist.

## Targets

| OS | Rust target | Output |
|----|-------------|--------|
| macOS Apple Silicon | `aarch64-apple-darwin` | engine binary / Tauri app |
| macOS Intel | `x86_64-apple-darwin` | engine binary / Tauri app |
| Windows x64 | `x86_64-pc-windows-msvc` | engine `.exe` / Tauri app |

Linux may run in developer CI but is not a V1 distribution target.

## Application Data

Use `dirs::data_local_dir()` plus `Steel Horizons`:

- `connection.json` — ephemeral port/token/PID, owner-only
- `save.json` — autosave
- `logs/` — structured rotating logs

On Unix, create discovery data with mode `0600`. On Windows, inherit a user-only application-data ACL. Never place the token in command-line arguments in packaged builds.

## Engine CLI

```text
steel-horizons-engine [flags]

  --preferred-port <port>      default 4880; scans through 4890
  --data-dir <path>            development/test override
  --scenario <id>              default starting_system
  --no-autoload                start lifecycle Unloaded
  --no-save                    disable persistence in tests
  --insecure-no-auth           development only; loud warning
  --log-format <pretty|json>
  --version
```

There is no player-facing tick-rate flag. Tests use `AdvanceTicks` while Paused.

## CI Staging and Matrix

P1-01 establishes one platform-neutral repository entry point for each active
gate and a macOS workflow that invokes those entry points. The initial workflow
does not claim Windows coverage:

```yaml
jobs:
  test-macos:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - run: ./scripts/ci/check.sh
```

`scripts/ci/check.sh` (or the exact platform-neutral command established by
P1-01) runs only gates whose owning increment is complete. It begins with the
locked Rust/Python scaffold checks and grows cumulatively under ADR-0005 and
TDD 04. Shell portability is not treated as proof that the commands passed on
Windows.

P1-36 adds the supported-platform jobs. OS/target pairs use an `include` matrix
rather than an invalid Cartesian product:

```yaml
jobs:
  test-supported:
    strategy:
      fail-fast: false
      matrix:
        include:
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: windows-latest
            target: x86_64-pc-windows-msvc
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
          targets: ${{ matrix.target }}
      - uses: Swatinem/rust-cache@v2
      - run: ./scripts/ci/check.sh
        if: runner.os != 'Windows'
      - run: pwsh -File scripts/ci/check.ps1
        if: runner.os == 'Windows'
      - run: cargo build --workspace --release --locked --target ${{ matrix.target }}
```

The Windows PowerShell entry point invokes the same ordered logical gates and
arguments as the Unix entry point. P1-36's cross-platform job runs the same
command-log fixture on the supported targets, compares ADR-0006 replay-equivalence
canonical bytes/hashes, and uploads the two states plus first divergent canonical
deterministic event on mismatch. Each hash
result must come from executing the native binary on a runner with the stated
OS and architecture; a successful cross-compiled build is not hash evidence.
P1-36 records the actual reference-runner labels and versions used for
qualification.

The release job downloads both macOS thin binaries and combines them with
`lipo -create`; a multi-target Cargo build alone does not create a universal
binary. Windows is built only on a Windows/MSVC runner.

## Python CI

The text UI uses a locked Python dependency file. P1-01 runs package-shell
formatting, typing, and one unit smoke test. P1-14 adds focused generated-client
tests against the REST walking skeleton. P1-34a activates the cumulative Python
client/TUI gate, which expands through P1-34d to cover formatting, typing, unit
tests, every renderer, real-engine integration, reconnect, and
resynchronization. Every test starts the engine with an isolated temporary data
directory and reads its discovery file.

P1-35 makes its fast deterministic unit/schema tests and reusable fixtures
cumulative, while its four long-running agent clients are qualification checks:
they are required to complete P1-35 and thereafter run nightly, pre-release,
and at Phase 1 completion, not on every ordinary commit. P1-36 adds Windows
locked build/test CI to the ordinary cumulative gate. Its supported-platform
hash comparison, stress benchmark, private packaging, and SBOM are qualification
checks required to complete P1-36 and in later qualification runs.

## Release Process

1. Every merge passes every cumulative gate activated by its completed owning
   increment; Windows locked build/test CI joins that set at P1-36.
2. A version tag builds locked native artifacts.
3. Cross-platform command-log hashes are compared.
4. Binaries/content/SBOM are code-signed where applicable.
5. Phase 1 publishes CLI artifacts privately.
6. Phase 2 packages notarized macOS and signed Windows Tauri apps for Steam/itch.io.

License selection and AI-assisted asset provenance must be complete before public distribution, but do not block private Phase 1 development.

## Development Workflow

```bash
cargo run -p steel-horizons-engine --bin content_validate -- content/
cargo test --workspace
cargo run -p steel-horizons-engine -- --data-dir <temporary-project-dir>
python -m steel_horizons_text_ui
python tests/playtest/minimal_gate.py
```

Tests create isolated temporary data directories; they never use or overwrite a developer's real save/discovery files.

## Related ADRs

- ADR-0001 — Rust Simulation Engine with HTTP/WS API
- ADR-0005 — Test Architecture
