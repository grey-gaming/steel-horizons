---
status: accepted
owner: Tech Lead
date: 2026-08-04
---

# ADR-0007: Save Envelope Format, Content Hash Placement, and Migration Fixtures

## Context

Phase 1 requires a durable save file format that is deterministic, verifiable, and
compatible across schema versions. GDD 12 §Save, Load, and Replay establishes
serialization rules, autosave cadence, and atomic write semantics. ADR-0006 §7
specifies that the save file includes a SHA-256 of the canonical `GameState` as
an integrity check within the save envelope. GDD 13 defines the serialized
`GameState` shape, and ADR-0004 defines lifecycle transitions around save/load.

Three unresolved specification questions remain:

1. **Save envelope format** — What wrapping structure (if any) surrounds the
   canonical `GameState` JSON? The raw JSON blob has no integrity marker,
   version marker for the envelope itself, or content-version anchor that a
   loader can check before parsing `GameState`.

2. **Content hash placement** — ADR-0006 defines a normalized content hash (SHA-256
   of canonical content definitions) validated at startup. Should that hash also
   appear inside `GameState`, in the save envelope, or neither? And should the
   content-version label alone suffice for save-load compatibility?

3. **Schema/compatibility and migration fixtures** — How are version numbers
   assigned to schema, content, and envelope formats? What constitutes a breaking
   change? What fixture strategy proves that save/load works across versions?

## Decision

### 1. Save Envelope Format

Every save file is a JSON wrapper, **not** raw canonical `GameState`. The wrapper
is the minimal metadata needed to verify integrity and reject incompatible files
without parsing the full state:

```json
{
  "format_version": 1,
  "content_version": "v1",
  "state_hash": "<64-hex-char SHA-256 of canonical game_state JSON>",
  "timestamp": "<ISO-8601 UTC string, e.g. \"2026-08-04T12:00:00Z\">",
  "game_state": { /* canonical GameState JSON */ }
}
```

Fields:

| Field | Type | Purpose |
|-------|------|---------|
| `format_version` | `u32` | Envelope format version. Starts at 1. Bumped only when the envelope structure itself changes (new required metadata fields, hash algorithm change, etc.). |
| `content_version` | `string` | Mirrors `GameState.content_version`. Duplicated at the envelope level so the loader can reject incompatible content without parsing `GameState`. Must match exactly for load to succeed. |
| `state_hash` | `string` | Lowercase hex-encoded SHA-256 (64 characters) of the canonical JSON byte sequence of `game_state` (replay-mode hash, which includes `command_log`). The loader computes the same hash and rejects the file on mismatch. |
| `timestamp` | `string` | ISO-8601 UTC timestamp for human file identification. **Not deterministic** — never used in simulation hashing, ordering, or replay verification. Written at save time; ignored on load. |
| `game_state` | `object` | The canonical JSON of `GameState`, produced by `serialize_canonical` (ADR-0006 §1). |

**Load procedure:**

1. Read and parse the envelope JSON.
2. Check `format_version == 1` (or the current supported version). Reject with
   a clear error if the envelope format is too new.
3. Check `content_version` matches the currently loaded content version.
   Reject with a clear error if they differ.
4. Compute SHA-256 of the canonical JSON byte sequence of `game_state` and
   compare with `state_hash`. Reject on mismatch (corruption or tampering).
5. Deserialize `game_state` into `GameState`. Validate invariants.
6. Apply any one-way schema migration (see §3 below).

**Rationale for the wrapper:**

- A raw JSON file has no self-integrity marker. Partial writes, truncation, or
  disk corruption produce a valid-looking partial JSON that deserializes without
  error but has silently lost data. The `state_hash` inside the envelope catches
  truncation and corruption after deserialization completes.
- The atomic-temp-write-sync-rename pattern (GDD 12) prevents torn writes at
  the filesystem level, but the hash provides a defence against media-level
  corruption, user-copy errors, and accidental truncation during manual file
  manipulation.
- Duplicating `content_version` at the envelope level avoids parsing
  `GameState` to reject incompatible content — important when `GameState` shape
  changes across schema versions.
- The `timestamp` field is purely informational. It is never part of any
  deterministic hash or replay comparison.

**Save procedure (mirrors GDD 12):**

1. Clone or snapshot the authoritative `GameState` (replay-mode, with
   `command_log`).
2. Serialize `GameState` to canonical JSON bytes.
3. Compute SHA-256 of those bytes → `state_hash`.
4. Build the envelope with `format_version`, `content_version`,
   `state_hash`, current UTC timestamp, and the canonical JSON bytes.
5. Serialize the envelope to JSON bytes.
6. Write to a temporary file in the same directory.
7. Perform `fsync` (or OS-level file-sync equivalent).
8. Rename the temporary file to the final save path (atomic on macOS/Windows).
9. On failure, delete the temporary file. The prior save file is unchanged.

### 2. Content Hash Placement

**Decision:** The normalized content hash (32-byte SHA-256 of canonical content
definitions, defined in ADR-0006 §3) is **not** placed in the save envelope or
inside `GameState`. It remains a startup-validation concern.

Rationale:

- Content files are loaded independently at process startup, not embedded in
  save files. The content hash is validated against a committed golden file
  (`tests/goldens/content_hash.txt`) at content load time, before any save is
  loaded.
- The `content_version` string (e.g. `"v1"`) is the migration/compatibility key.
  A save made with content version `"v1"` can only be loaded by a binary whose
  content files also report `"v1"`. The content hash is an integrity check
  against the content files themselves, not against save files.
- Storing the content hash in the save envelope would add an unnecessary coupling:
  a save file created on one build would embed the content hash of that build's
  content files. On a different build with identical `content_version` but
  different content files (which would fail the startup content-hash golden check
  anyway), the save would be rejected unnecessarily. The `content_version` check
  at the envelope level is the correct compatibility boundary.
- If a future version wants to verify that a save was created with byte-identical
  content definitions, it can add an optional `content_hash` field to the
  envelope. That addition does not change the envelope format version because
  unknown fields in the envelope are ignored (see §3).

**Clarification on `GameState.content_version`:** This field is the same string
as the envelope-level `content_version`. It exists in `GameState` for runtime
reference (e.g., logging which content version produced the current state). The
envelope-level copy is authoritative for load-time filtering.

### 3. Schema/Content Compatibility and Migration Fixtures

#### Version numbering

Three independent version numbers:

| Version | Location | Start value | Bumped when |
|---------|----------|-------------|-------------|
| `format_version` | Envelope root | 1 | Envelope structure changes (new required metadata, hash algorithm change). |
| `schema_version` | `GameState.schema_version` | 1 | `GameState` struct shape changes: field additions/removals/renames, type changes, enum variant changes. |
| `content_version` | Envelope + `GameState` | `"v1"` | Content definition shape changes (new resource types, changed struct shapes in `content/` files). |

#### Compatibility rules

1. **Envelope format:** `format_version` must equal the binary's supported
   version exactly. Forward-compatible envelope parsing is not required in V1.
   Unknown fields at the envelope level are silently ignored (so adding
   optional metadata does not bump `format_version`).

2. **Schema version:** A binary at schema version M can load saves with
   `schema_version ≤ M`. If `schema_version > M`, the loader rejects with a
   clear error ("save file requires a newer engine version"). Migration from
   N → N+1 is one-way: a save loaded at version M > N is upgraded to the
   current schema version at load time and re-saved at the current version.
   There is no downgrade path.

3. **Content version:** The envelope `content_version` must match the binary's
   content version exactly. If they differ, the loader rejects with a clear
   error ("save file was created with a different content version"). Content
   version changes are rare and require a new content file release; they are
   not expected during V1 development.

#### One-way schema migration

When a save file's `schema_version` is older than the binary's current version:

1. Deserialize `game_state` into the older `GameState` type.
2. Apply a sequence of migration transforms, one per version gap, each
   producing the next version's `GameState`.
3. Set `schema_version` to the current version.
4. Proceed to invariant validation and normal load.

Each migration transform is a pure function:

```rust
trait Migration: Named {
    fn from_version() -> u32;
    fn to_version() -> u32;
    fn migrate(state: &mut GameState) -> Result<(), MigrationError>;
}
```

Migrations are additive-only where possible. A migration may:
- Add a new field with its default value.
- Remove a field that was unused or has a computed replacement.
- Change an enum by adding a variant with a defined default mapping.
- Recompute a derived field that changed shape.

A migration must NOT:
- Lose gameplay-significant data (conservation-violating drops).
- Change the deterministic replay outcome of an existing command log.
- Skip a version gap (all intermediate migrations must run in order).

#### Migration fixtures

Two fixture classes prove save/load compatibility:

1. **Schema-version-N fixtures** — A save file created by binary version N is
   loadable by binary version M (M ≥ N). The loaded state hash after migration
   must match the hash produced by a fresh game at the same tick under binary
   version M with the same command sequence. These fixtures live under
   `tests/fixtures/saves/` with version-numbered extensions, e.g.
   `canonical_tick0.schema_v1.save`.

2. **Schema-migration golden** — A fixture that exercises every registered
   migration transform and asserts that conservation and replay equivalence
   hold across the migration boundary. This is a Rust test that loads each
   fixture save, applies migrations, and compares against a golden state hash.

The fixture strategy is:

- `P1-01` generates the canonical tick-zero save file as `schema_version=1`.
- When `schema_version` bumps, the previous version's save file is preserved
  as a fixture, and a new golden is generated for the current version.
- Every migration transform has a unit test that runs it over a known state
  and checks the output hash against a committed golden.
- The `save_load_equivalence` scenario (P1-13) uses the current-version save
  file; the migration fixture tests prove cross-version compatibility.

## Consequences

### Positive

- The save envelope is self-verifying: truncation and corruption are detected
  before deserialized state is used.
- Content-version filtering happens at the envelope level without parsing
  `GameState`, which is important when `GameState` shape changes.
- The content hash stays where it belongs (content-load validation), avoiding
  unnecessary coupling between content definitions and save files.
- One-way schema migration with additive transforms preserves replay
  equivalence and conservation invariants.
- Migration fixtures provide a reproducible proof that save/load works across
  schema versions.

### Negative

- The envelope adds a small overhead to every save/load: parsing the wrapper,
  computing the hash, and verifying. At V1 scale this is negligible.
- One-way migrations require maintaining the previous `GameState` type
  definition until the migration transform is registered. If many schema
  versions accumulate, the binary carries multiple `GameState` representations.
- Adding optional fields to the envelope is safe but requires discipline:
  unknown-field-ignore policy means a typo in a field name is silently ignored.

### Mitigations

- The `state_hash` is computed from canonical JSON bytes, not from an
  in-memory struct, so serialization and hashing are always in sync.
- Migration transforms are tested with golden-hash assertions.
- Schema version bumps are gated by design review and documented in the
  evidence log, just like golden hash updates.

## Related ADRs

- ADR-0006 — Canonical Content/State Hashing (defines canonical serialization,
  SHA-256, content hash, state hash, and save-file hash integrity)
- ADR-0004 — Game Lifecycle State Machine (defines save normalization,
  load-from-lifecycle rules)
- ADR-0002 — Deterministic Tick Simulation (establishes deterministic state
  as a non-negotiable invariant)
- ADR-0005 — Test Architecture (defines save/load equivalence scenario,
  content validation gate)
