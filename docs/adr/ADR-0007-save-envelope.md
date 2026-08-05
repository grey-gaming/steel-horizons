---
status: accepted
owner: Tech Lead
date: 2026-08-04
---

# ADR-0007: Save Envelope Format, Content Hash Placement, and Migration Fixtures

## Context

GDD 12 requires atomic JSON persistence, ADR-0004 requires lifecycle
normalization, ADR-0006 defines canonical content/equivalence/save-integrity hashes, and GDD 13
owns the serialized `GameState`. A save must therefore answer four questions
before state becomes authoritative:

1. Is the envelope format and hash scheme supported?
2. Was the nested state stored without accidental corruption?
3. Is the currently loaded authored content exactly the content that created the
   save?
4. Can the saved state schema be migrated to the current schema without losing
   gameplay-significant state or replay determinism?

## Decision

### 1. Save envelope v1

Every V1 save is a JSON object with exactly these fields:

```json
{
  "format_version": 1,
  "hash_scheme": "sha256-canonical-json-v1",
  "schema_version": 1,
  "content_version": "v1",
  "content_hash": "<64 lowercase hex characters>",
  "state_hash": "<64 lowercase hex characters>",
  "timestamp": "2026-08-04T12:00:00Z",
  "game_state": { "...": "complete normalized GameState" }
}
```

| Field | Type | Contract |
|-------|------|----------|
| `format_version` | `u32` | Envelope structure/interpretation version; exactly `1` for this ADR. |
| `hash_scheme` | string | Exactly `sha256-canonical-json-v1`; identifies ADR-0006's byte encoding, domains, and SHA-256 use. |
| `schema_version` | `u32` | Saved `GameState` schema. It must equal the nested `game_state.schema_version` before migration. |
| `content_version` | string | Content compatibility/release label. It must equal both the nested `game_state.content_version` and the running catalog label. |
| `content_hash` | string | ADR-0006 content digest for the exact validated catalog used to create the save. It must equal the running catalog digest. |
| `state_hash` | string | ADR-0006 save-integrity digest of the complete normalized nested `game_state`, including `command_log` and the event lower bound. |
| `timestamp` | string | Informational UTC RFC 3339 timestamp. It is excluded from all deterministic hashes and ignored by simulation/replay. |
| `game_state` | object | Complete normalized serialized state under the declared schema version. |

V1 rejects missing, duplicate, and unknown envelope members. Optional metadata is
not added ad hoc; changing the envelope shape requires a reviewed
`format_version` change. Hash strings must use exactly 64 lowercase hexadecimal
characters.

The envelope itself need not have deterministic bytes because `timestamp` is
intentionally nondeterministic and pretty-printing is permitted. Hashes are
computed from ADR-0006's canonical semantic values, never from the raw envelope
substring or whole-file bytes.

### 2. Exact content compatibility

The normalized content hash is stored in the envelope, not in `GameState`.
Keeping it outside mutable simulation state avoids duplicating a runtime
constant in every tick hash while still anchoring persistence to exact content.

A load succeeds only when all of these are true:

```text
envelope.content_version == game_state.content_version
envelope.content_version == loaded_catalog.content_version
envelope.content_hash    == computed_hash(loaded_catalog)
```

The process has already schema/semantically validated the loaded catalog and
checked its committed content golden before a game load begins. The envelope
comparison then proves that this particular save originated from those exact
definitions. A content value change that retains the label `v1` still produces a
different digest and correctly rejects the old save.

V1 has no content-migration path. Loading a save under different content requires
a future explicit content migration/rebase decision with conservation and replay
proofs; changing a golden or ignoring `content_hash` is not a migration.

### 3. Save procedure

The actor and persistence service perform the following ordered operation:

1. The actor reaches a committed transaction boundary and completes ADR-0008's
   FIFO snapshot-barrier protocol. No mailbox or partially committed tick
   state is serialized.
2. Clone the authoritative `GameState`, including the complete `command_log`.
3. Normalize the clone for loading: `Running` or `Advancing` becomes `Paused`,
   `Paused` remains `Paused`, and `Won` remains `Won`. `Unloaded` and `Loading`
   cannot be saved. Do not mutate the live lifecycle merely to save it.
4. Set/check the clone's current `schema_version` and `content_version`; obtain
   the already validated catalog's current content digest.
5. Produce the normalized state's ADR-0006 canonical save-integrity projection and
   digest. Any serialization/hash failure returns a typed persistence error.
6. Construct the envelope with matching versions, `hash_scheme`,
   `content_hash`, `state_hash`, current UTC `timestamp`, and normalized state.
7. Serialize the envelope. Re-canonicalizing its `game_state` member must produce
   exactly the bytes hashed in step 5.
8. Enqueue the immutable envelope on the single FIFO persistence worker for the
   canonical autosave target. Manual SaveNow, internal autosave, LoadAutosave reads,
   and shutdown saves share this ordered lane; no second worker may replace or read
   that target concurrently.
9. When this operation reaches the worker head, create a uniquely named
   same-directory temporary file with exclusive-create
   semantics; write all bytes and sync the file.
10. Atomically replace the target using the platform persistence abstraction:
   POSIX rename semantics on macOS and an atomic replace operation on Windows.
   A plain cross-platform `rename` call is not assumed to replace an existing
   Windows file safely.
11. Sync the containing directory where the platform supports it, then report
    success. On any earlier failure, close/remove only the known temporary file;
    the prior save remains intact.

The actor may continue ordinary gameplay and may enqueue later immutable save
snapshots while a write runs, but the worker performs replacements in enqueue order.
Therefore an older slow write can never overwrite a newer snapshot. `LoadAutosave`
enters Loading and enqueues its read behind every previously accepted operation for
the target; it observes the last successful preceding replacement. Later commands are
handled under the Loading lifecycle rules. Shutdown enqueues its final snapshot after
prior operations and waits for the lane to finish. Each command receipt completes only
from its own operation; one failed operation does not reorder or cancel later entries.

Tests inject failures after create, write, file sync, replacement preparation,
and replacement. Every failure must leave either the complete prior save or the
complete new save—never a truncated/partially mixed target. Delayed-worker tests
prove A-then-B ends at B, A-success/B-failure ends at A, A-failure/B-success ends at
B, SaveNow-then-LoadAutosave reads the SaveNow state, and autosave/manual/shutdown
operations retain actor enqueue order.

### 4. Load procedure

Load constructs a candidate off to the side. It does not replace the actor's
current immutable state/lifecycle until every step succeeds:

1. Read the bounded save file into a duplicate-key-rejecting raw JSON object and
   extract `format_version`. Do not deserialize `game_state` into the current
   `GameState` yet.
2. Dispatch by `format_version`; reject unsupported versions with a typed error.
   For format 1, require exactly the V1 member set (no missing or unknown fields),
   retain `game_state` as a raw checked value, and require the exact
   `hash_scheme`.
3. Validate the V1 envelope's version/hash field shapes. Reject
   `schema_version > current_schema_version`. Require a registered contiguous
   migration chain for every older version.
4. Require envelope `content_version` and `content_hash` to equal the already
   validated current catalog.
5. Inspect the raw state's `schema_version` and `content_version`. Require exact
   equality with the envelope copies before migration.
6. Canonicalize the raw saved state under the declared hash scheme, compute the
   save-integrity digest, and compare it in constant time with `state_hash`. Reject
   a mismatch before running migrations or domain constructors.
7. If necessary, apply each schema migration in order to the raw value. Each
   transform must produce the next version exactly and retain
   `content_version`.
8. Deserialize the migrated value into the current strict `GameState`, reject
   unknown/missing fields, and validate all cheap and whole-load invariants,
   including material/conservation relationships and legal normalized lifecycle.
9. Rebuild derived runtime state such as pending-command schedules, idempotency
   indexes, and supply/demand tables. Rebuilding must not mutate primary state.
10. At the actor boundary, checked-rebase `candidate.next_event_sequence` to at
    least ADR-0012's runtime session allocator. Fresh-process autoload normally
    changes nothing; explicit same-session LoadAutosave may advance only this
    cursor bookkeeping before its outcome and complete replacement delta.
11. Publish the candidate atomically. A failure at any prior step restores the
   exact prior actor state/lifecycle, or leaves `Unloaded` when no prior game
   existed, per ADR-0004, except that committed receipt/event allocator bookkeeping
   remains monotonic.

V1 files must contain `Paused` or `Won` after save normalization. A migration may
normalize a legacy lifecycle as an explicit transform; the ordinary V1 loader
does not silently rewrite a `Running`/`Advancing` state after verifying its hash.

State-hash verification occurs before migration and event-cursor rebasing because it
verifies what was actually stored. Migration tests compute new ADR-0006
replay-equivalence bytes/hash and a canonical deterministic event trace over the
migrated current state for equivalence/golden assertions; a new save-integrity digest
is written only when the game is next saved. Same-session rebasing is separately
asserted as the only permitted post-load cursor difference before subsequent commands.

### 5. Independent versions

| Version | Location | Changes when |
|---------|----------|--------------|
| `format_version` | Envelope | Required envelope members or their interpretation changes. |
| `hash_scheme` | Envelope | Canonical byte rules, hash function, or domain prefixes change. A corresponding format change is normally required. |
| `schema_version` | Envelope + `GameState` | Serialized `GameState` shape changes. |
| `content_version` | Envelope + `GameState` + content roots | Content schema/release lineage changes. Exact values are additionally guarded by `content_hash`. |

Envelope format support is exact in V1. State schema compatibility is backward
through registered migrations only; forward state schemas are rejected. Content
compatibility is exact label plus digest. Downgrade migrations are not supported.

### 6. Schema migrations

Migrations operate on the duplicate-free raw JSON value or on explicit adjacent
version types, not on the current `GameState` type pretending to be every prior
shape. A raw-value implementation has this logical interface:

```rust
trait Migration: Send + Sync {
    fn from_version(&self) -> u32;
    fn to_version(&self) -> u32;
    fn migrate(&self, state: serde_json::Value)
        -> Result<serde_json::Value, MigrationError>;
}
```

Registration rules:

- `to_version == from_version + 1`.
- Exactly one migration exists for every supported adjacent pair.
- The chain has no gaps, duplicates, cycles, or skipped transforms.
- Each transform verifies its input version and writes its output version.
- A transform may add, rename, reshape, or deliberately remove a field only
  when the authoritative schema change defines the mapping.
- It may not lose gameplay-significant inventory, reservations, research credit,
  command ordering/outcomes, RNG state, remainders, or recovery state.
- It may not consult wall-clock time, random state outside the saved RNG, map
  insertion order, network state, or current mutable gameplay state.
- It returns a typed/path-specific error; it never guesses a fallback.

After the complete chain, only the current strict type is deserialized and its
invariants are checked. Migration registration itself is unit tested for
contiguity at engine startup/test time.

### 7. Fixtures and executable proofs

P1-13, when persistence and canonical state exist, creates the baseline V1
fixtures—not P1-01. Baseline coverage includes:

- canonical tick-zero V1 save;
- a V1 save with applied and accepted/pending command records;
- uninterrupted versus save-split replay-equivalence bytes and canonical
  deterministic event traces, plus exact save-integrity validation;
- save normalization from Running and Advancing plus Won preservation;
- corrupted `state_hash`, wrong `content_hash`, inner/outer version mismatch,
  future schema/format, unsupported scheme, missing/unknown/duplicate members,
  invalid lifecycle, and invariant failure;
- injected atomic-write/replace failures preserving the prior save.

Fixtures live under `tests/fixtures/saves/`, for example
`canonical_tick0.schema_v1.save`. Hash assertions live in reviewed golden files.

When schema version N+1 is introduced:

1. Preserve at least one nontrivial save created by the released/version-N
   writer; never synthesize it by serializing the new type with omitted fields.
2. Add the N -> N+1 transform and focused unit fixtures for every changed field.
3. Load the historical save, run the full chain, validate invariants/conservation,
   and compare its replay-equivalence bytes/hash and canonical deterministic event
   trace with a current-version run driven by the same canonical command sequence
   where semantic equivalence is promised.
4. Add a current-version fixture and record every intentional hash change in the
   implementation-plan evidence log.

No claim of cross-version migration coverage is made while only schema version 1
exists. P1-13 proves the framework, current-version path, and rejection behavior;
historical migration proof activates with the first real schema bump.

## Consequences

- Every save is bound to exact validated content as well as intact state.
- The loader verifies stored bytes before transforming them and publishes only a
  fully migrated/validated candidate.
- Version-specific raw/typed migrations can actually represent shape changes;
  the current Rust type does not masquerade as older schemas.
- Lifecycle normalization and pending-command persistence are explicit inputs to
  the state hash.
- Strict envelopes catch misspelled metadata instead of silently ignoring it.
- Platform-specific atomic replacement requires a small abstraction and injected
  failure tests, accepted in exchange for preserving the prior save.
- Content changes intentionally reject old saves until an explicit migration
  policy exists; this is safer than replaying under silently different rules.

## Related ADRs

- ADR-0002 — Deterministic Tick Simulation
- ADR-0004 — Game Lifecycle State Machine
- ADR-0005 — Test Architecture
- ADR-0006 — Canonical Content/State Hashing
- ADR-0008 — Accepted-Command Persistence
