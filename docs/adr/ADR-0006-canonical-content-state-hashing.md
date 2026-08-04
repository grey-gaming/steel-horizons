---
status: accepted
owner: Tech Lead
date: 2026-08-04
---

# ADR-0006: Canonical Content/State Hashing

## Context

Phase 1 requires deterministic verification that the same content definitions and
game state produce identical results across builds, platforms, and save/load
cycles. Two distinct hashes are needed:

1. **Content hash** — a stable fingerprint of the authored JSON content files
   (`content/definitions.v1.json`, `content/starting_system.v1.json`) that
   changes only when the authoritative GDD 14 values change. Used by the content
   validator (P1-05) to detect unintended drift and by the runtime to tag the
   `content_version` string in `GameState`.

2. **State hash** — a deterministic fingerprint of the full canonical
   `GameState` snapshot (GDD 13) at a given tick. Used by P1-08 to lock the
   tick-zero golden, by scenario tests to assert exact state transitions, and
   by the replay verifier to prove save/load equivalence.

Neither hash is a checksum of a raw wire-format blob — both require canonical
serialization rules so that JSON field ordering, whitespace, and numeric
representation do not change the hash.

## Decision

### 1. Canonical Serialization (both content and state)

Every hashed value is produced by serializing the Rust struct to a `Vec<u8>`
using Serde JSON with these settings:

- `Serializer::new(&mut writer).canonical(true)` — or the equivalent manual
  configuration: sorted keys (by field name for structs, by enum variant name
  for externally tagged enums), no whitespace/newlines, and no escaping beyond
  JSON's mandatory character escapes.
- Maps use `BTreeMap<K, V>` so iteration order is the stable key ordering of
  the key type. ResourceType and other enums use the `#[serde(enum_rep = "variant")]`
  pattern with variant-name alphabetical ordering enforced at the Serde level
  via `#[serde(rename_all = "camelCase")]` or explicit `#[serde(rename)]` where
  the GDD 13 notation uses camelCase field names.
- Sets use `BTreeSet<T>` with the same key-ordering rules.
- Integers serialize as JSON numbers without leading zeros or trailing decimal
  points. `u32`, `u64`, `i32` values write their base-10 representation.
- Enum variants with data use the canonical Serde externally tagged form:
  `{"VariantName": { ...fields... }}`. The variant name in the JSON tag uses
  the Serde `rename_all` that matches the GDD 13 camelCase convention.
- Option fields that are `None` are omitted entirely from the serialized object
  (via `#[serde(skip_serializing_if = "Option::is_none")]` or equivalent).
- Empty `BTreeMap`/`BTreeSet`/`Vec` values serialize as `{}`/`[]` and are
  never omitted — a zero-length collection is meaningful state.
- All `String` values encode as UTF-8 and JSON-string-escape only the minimum
  required characters (`"`, `\`, control characters U+0000–U+001F).

The canonical serialization is NOT the general Serde JSON output — Serde JSON
by default uses `BTreeMap` key ordering but does NOT guarantee sorted struct
fields or omit `None` fields in a deterministic way across Serde versions.
Therefore the canonical path is an explicit helper:

```rust
/// Serialize `value` into `writer` using the canonical content/state encoding.
/// Every consumer (content hash, state hash, save file, golden snapshot) uses
/// this function so that all fingerprints are comparable.
pub fn serialize_canonical<T: Serialize>(
    value: &T,
    writer: impl io::Write,
) -> Result<(), serde_json::Error> {
    let mut ser = serde_json::Serializer::new(writer);
    ser.sort_keys(true);
    // No pretty-printing: compact output with sorted keys.
    // Serde JSON's default omits `None` fields when the field has
    // `#[serde(skip_serializing_if)]` — that is the correct behavior.
    value.serialize(&mut ser)
}
```

### 2. Hash Function

Both hashes use **SHA-256** (from the `sha2` crate or Rust's standard library
`std::hash::Hash` — specifically `sha2::Sha256` via the `Digest` trait).

Rationale:

- SHA-256 is widely available, fast, and collision-resistant at a
  cryptographic level. The simulation does not need defence against malicious
  manipulation, but using a well-known hash avoids custom hash-bias questions.
- No salting, no length-prefix complications beyond the canonical encoding.
- The output is exactly 32 bytes / 64 hex characters for display/comparison.

### 3. Content Hash

The content hash covers:

- **Included:** Every field in every record of `content/definitions.v1.json`
  and `content/starting_system.v1.json`. This means `ShipDefinition`,
  `StationDefinition`, `RecipeDefinition`, `TechDefinition`, `ShipStats`,
  `StationStats`, `FacilityRequirement`, `ResourceDeposit`, `CelestialBody`,
  the starting-state records (Hub, ship, metadata, deployment kit), and every
  authored value in GDD 14.

- **Excluded:** The outer envelope (file name, file modification timestamp,
  file size, encoding marker). The hash is computed from the canonical JSON
  byte sequence of the combined content records, **not** from the raw file
  bytes, so platform line-ending variation and whitespace differences do not
  affect it.

- **Version tag:** The content_version string is `v1` and is included in
  `GameState.content_version`. If the content file is re-constituted from GDD
  14, the canonical content hash must match a committed golden. Any change to
  GDD 14 that affects the content records updates the golden hash as part of
  the same increment.

Implementation:

```rust
pub fn content_hash(content: &ContentCatalog) -> [u8; 32] {
    let mut hasher = Sha256::new();
    serialize_canonical(content, &mut hasher).expect("canonical serialization");
    hasher.finalize().into()
}
```

### 4. State Hash

The state hash covers:

- **Included:** Every field of `GameState` and every transitively contained
  struct (CelestialBody, Station, Ship, ResearchProject, SurveyOrder,
  BuildOrder, SalvageCache, GateBuild, Reservation, BottleneckTracker,
  RNGState, IdCounters, CommandRecord) as defined in GDD 13. This includes
  `schema_version`, `content_version`, `lifecycle`, `tick`, counters, and
  every mutable gameplay field.

- **Excluded:**
  - The `command_log` field is excluded from the canonical state hash for
    scenario-test assertions but included for save/load and replay equivalence
    verification. Two modes:
    - **Scenario hash (no command_log):** Used by scenario tests (P1-10+) and
      the tick-zero golden (P1-08). Commands are recorded but their sequence
      is part of the test input, not the expected output state.
    - **Replay hash (with command_log):** Used by save/load and replay
      verification (P1-13+). The full command log must reconstruct an
      identical GameState after save/load, so it is included.
  - The event retention ring (runtime-only, not serialized in saves).
  - Supply/demand tables (derived, rebuilt every tick).
  - Presentation/camera data.
  - File format envelope (file name, timestamps).

- **Struct field ordering:** All structs use `#[serde(rename_all = "camelCase")]`
  and Serde's default field ordering (the order they appear in the Rust source)
  for the canonical serialization. This is deterministic within one Rust
  compiler version. If a field is added or reordered, the hash changes by
  design and the golden updates.

Implementation:

```rust
/// Compute the canonical scenario-test state hash (excludes command_log).
pub fn state_hash_scenario(state: &GameState) -> [u8; 32] {
    let mut hasher = Sha256::new();
    let state_for_hash = StateHashView {
        // All fields except command_log
        schema_version: &state.schema_version,
        content_version: &state.content_version,
        lifecycle: &state.lifecycle,
        tick: &state.tick,
        // ... all other fields mirrored from GameState
        command_log: &[], // excluded
    };
    serialize_canonical(&state_for_hash, &mut hasher).expect("canonical serialization");
    hasher.finalize().into()
}

/// Compute the full replay/save-load hash (includes command_log).
pub fn state_hash_replay(state: &GameState) -> [u8; 32] {
    let mut hasher = Sha256::new();
    serialize_canonical(state, &mut hasher).expect("canonical serialization");
    hasher.finalize().into()
}
```

In practice, the two modes are implemented by a `HashMode` enum passed to a
single hashing helper that conditionally serializes `command_log` as an empty
array when excluded.

### 5. Content Version String

`GameState.content_version` is the string `"v1"`. It is **not** the content
hash itself — it is a version label that remains stable as long as the content
record shapes are compatible. The content hash is validated at startup and
logged, but the version label is the migration/compatibility key. A breaking
content schema change bumps the version label and triggers the schema-migration
path.

### 6. Golden-Update Policy

A golden file (content hash golden, tick-zero state hash golden, or scenario
state golden) is the committed expected output of the deterministic hash
function over canonical serialization.

- **When to update:** Only when an increment intentionally changes an
  authoritative GDD/GAME rule that affects the serialized content or state.
  Golden changes must be explained in the increment's evidence log with the
  exact reason and the changed authoritative section.

- **Review process:** The diff of the golden file is reviewed alongside the
  production/content changes in the same increment. The CI gate fails if the
  golden does not match the computed hash unless the increment explicitly
  records the golden update.

- **CI behavior:** Every commit runs content-hash and state-hash checks for the
  tick-zero golden. Scenario test assertions compare against committed goldens.
  A hash mismatch fails the CI gate and must be resolved by either (a) updating
  the golden with an explained reason, or (b) fixing the code to restore the
  prior hash.

- **Content-hash golden path:** `tests/goldens/content_hash.txt` — hex-encoded
  64-character SHA-256 of canonical content.

- **Tick-zero state-hash golden path:** `tests/goldens/state_hash_tick0.txt` —
  hex-encoded SHA-256 of canonical GameState at tick 0 (Paused, authored
  starting state, scenario-hash mode).

- **Scenario goldens:** Named files under `tests/goldens/scenarios/` per
  scenario, e.g. `bootstrap_to_first_cargo.state_hash`.

### 7. Save File Hash

The save file (P1-13) includes a SHA-256 of the canonical `GameState` as an
integrity check within the save envelope. This is the replay-mode hash (with
`command_log`). The save load verifies the hash matches before accepting the
file. This is NOT a content/state golden — it is a runtime integrity check that
has no golden file.

## Consequences

- **Positive:** Every content change and state change produces a deterministic,
  reviewable fingerprint. Scenario tests can assert exact hashes instead of
  diffing entire JSON blobs. Save/load replay gains a cheap integrity check.

- **Positive:** The same canonical serialization is used for content hashing,
  state hashing, save files, and golden snapshots — one code path, no drift
  between them.

- **Negative:** Adding a new field to any serialized struct changes every state
  hash that includes that struct. This is intentional and forces explicit
  golden updates with documented reasons.

- **Negative:** SHA-256 adds a dependency (`sha2` crate) that is not in the
  Rust standard library. The dependency is small, pure-Rust, and widely used.
  An alternative is to use `std::hash::Hash` with a stable hasher, but that
  would require a custom `StableHasher` that matches the canonical
  serialization, adding more code than the `sha2` crate.

- **Negative:** The canonical serialization helper adds a small maintenance
  surface. If Serde JSON changes its `sort_keys` semantics, the golden hashes
  change. Pinning the `serde_json` dependency version and testing golden
  stability in CI mitigates this.

## Related ADRs

- ADR-0002 — Deterministic Tick Simulation (establishes deterministic state
  as a non-negotiable invariant)
- ADR-0005 — Test Architecture (defines golden snapshots and content
  validation; this ADR fills in the exact hashing mechanics)
