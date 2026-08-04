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

## CI Matrix

OS/target pairs use an `include` matrix rather than an invalid Cartesian product:

```yaml
jobs:
  test:
    strategy:
      matrix:
        os: [macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --all -- --check
      - run: cargo clippy --workspace --all-targets -- -D warnings
      - run: cargo run -p steel-horizons-engine --bin content_validate -- content/
      - run: cargo test --workspace

  build:
    needs: test
    strategy:
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
          targets: ${{ matrix.target }}
      - run: cargo build --workspace --release --locked --target ${{ matrix.target }}
```

The release job downloads both macOS thin binaries and combines them with `lipo -create`; a multi-target Cargo build alone does not create a universal binary. Windows is built only on Windows/MSVC runners.

## Python CI

The text UI uses a locked Python dependency file. CI installs it, runs formatting/type/unit tests, starts a release engine with an isolated temporary data directory, reads its discovery file, and runs protocol smoke tests. Full agent victory/stress runs are nightly and pre-release.

## Release Process

1. Every merge passes content validation, gameplay scenarios, API conformance, and supported-OS tests.
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
