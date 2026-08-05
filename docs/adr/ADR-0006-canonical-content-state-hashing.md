---
status: accepted
owner: Tech Lead
date: 2026-08-04
---

# ADR-0006: Canonical Content/State Hashing

## Context

Phase 1 needs deterministic fingerprints for two different purposes:

1. A **content hash** identifies the exact validated authored definitions and
   starting scenario used by a game.
2. A **state hash** identifies a canonical `GameState` at a commit boundary.

Raw JSON file bytes are unsuitable because insignificant whitespace and object
member order may differ. Rust's `Hash` implementations and Serde's ordinary
JSON output are also not a persistent cross-version format. This ADR therefore
defines one project-owned canonical JSON encoding and versioned SHA-256 hash
schemes over that encoding.

## Decision

### 1. Hash scheme identifiers and domain separation

V1 uses SHA-256 with the scheme identifier:

```text
sha256-canonical-json-v1
```

The bytes given to SHA-256 are a domain prefix, one zero byte, then the
canonical JSON bytes:

```text
content:
  "steel-horizons/content/sha256-canonical-json-v1" || 0x00 || canonical_json

scenario state:
  "steel-horizons/state/scenario/sha256-canonical-json-v1" || 0x00 || canonical_json

replay-equivalence state:
  "steel-horizons/state/replay/sha256-canonical-json-v1" || 0x00 || canonical_json

save-integrity state:
  "steel-horizons/state/save/sha256-canonical-json-v1" || 0x00 || canonical_json
```

All prefix characters are the displayed ASCII bytes. Domain separation prevents
a content document and state document with coincidentally equal JSON from
sharing a digest and distinguishes the three state projections.

Every externally stored hash is exactly 32 digest bytes encoded as 64 lowercase
hexadecimal characters. Uppercase, shortened, or prefixed hexadecimal strings
are invalid. Changing the hash function, domain prefix, or canonical byte rules
creates a new scheme identifier; it never silently changes V1 goldens.

### 2. JSON shape precedes canonicalization

Canonicalization operates on the JSON value produced by the approved serialized
schema; it does not choose the schema itself.

- Field names are `snake_case`.
- Tagged variant names are PascalCase.
- Each enum's unit/external/internal/adjacent tagging representation is part of
  its generated schema. Commands retain ADR-0003's explicit `type` tag.
- All fields of canonical content and state structs are present. An `Option`
  whose value is `None` is encoded as JSON `null`; canonical content/state types
  must not use `skip_serializing_if`.
- Empty maps, sets, and vectors are present as `{}` or `[]`.
- Maps that do not have JSON-string keys must be represented by an explicitly
  ordered array shape in their schema rather than relying on serializer-specific
  map-key coercion.

Schema changes are governed by the content/state version rules. Canonicalization
never repairs, defaults, or drops a field.

### 3. Canonical JSON v1 byte encoding

The canonical writer accepts a duplicate-free JSON value and emits bytes using
the following complete rules:

1. **Objects:** Sort member names by ascending UTF-8 byte sequence. Emit members
   in that order as `{` + comma-separated canonical name/value pairs + `}`.
   There is no whitespace. Duplicate member names are rejected while parsing;
   last-key-wins parsing is forbidden on canonical inputs.
2. **Arrays:** Preserve schema/collection order exactly and emit canonical
   elements separated by commas, with no whitespace.
3. **Strings:** Emit valid UTF-8 between double quotes. Escape `"`, `\\`,
   backspace, tab, LF, form feed, and CR as `\"`, `\\`, `\b`, `\t`, `\n`,
   `\f`, and `\r`. Escape every other U+0000 through U+001F code point as a
   lowercase `\u00xx` sequence. Do not escape `/` or non-ASCII scalar values,
   and do not apply Unicode normalization.
4. **Integers:** Emit the shortest base-10 representation, with no leading `+`
   or zeros. Zero is `0`; the checked parser rejects a lexical negative-zero
   token. The writer accepts only JSON integers representable as `i64` or `u64`;
   the typed schema subsequently enforces each field's narrower declared type.
5. **Other scalars:** Emit exactly `true`, `false`, or `null`.
6. **Floating point:** Reject all floating-point JSON numbers, including
   mathematically integral forms such as `1.0`, and reject non-finite values.

Object sorting is a serialization rule only. Simulation iteration order remains
the explicit domain order—for example, GDD 13's declared `ResourceType` order.
These two orders must not be conflated.

The implementation must not depend on a nonexistent "canonical mode" in
`serde_json`. The project helper follows this algorithm explicitly: serialize a
typed value into a checked JSON value, reject floats/duplicates or unsupported
map keys, recursively write the canonical bytes, and stream those bytes to the
selected SHA-256 domain. All stages return typed errors; canonical serialization
and hashing never call `unwrap`, `expect`, or panic in production paths.

### 4. Collection ordering before JSON encoding

Canonical object sorting makes map insertion order irrelevant. Domain
collections still have deterministic in-memory and array serialization rules:

- Serialized keyed collections use `BTreeMap` with an explicitly tested `Ord`.
- Serialized sets use `BTreeSet`; their JSON array elements appear in that
  `Ord` order.
- ID newtypes order by their complete identifier string.
- Domain enums that require authored declaration order implement/test that order
  explicitly; Serde rename text does not define Rust `Ord`.
- Semantically ordered vectors retain their defined FIFO/phase/sequence order.
- Semantically unordered content record lists are normalized to their documented
  stable key order before hashing, or represented as string-keyed maps.

P1-02b serialization fixtures lock field names, tags, nulls, and collection
orders. P1-05/P1-08 exact-byte fixtures then lock the canonical writer itself.

### 5. Canonical content hash input

The content hash covers the two validated V1 content roots as one explicit
object:

```json
{
  "definitions": { "...": "the complete definitions.v1.json value" },
  "starting_system": { "...": "the complete starting_system.v1.json value" }
}
```

This conceptual `CanonicalContentHashInput` is constructed from the fully
validated typed `DefinitionsCatalog` and `StartingScenario`; it is not a byte
concatenation of files. It includes every field in both typed roots, including
their version fields, authored defaults, Gate definition, bodies, deposits,
starting entities, inventory, and starting metadata.

Excluded are file names, directory paths, modification times, raw whitespace,
encoding markers, and generated JSON Schema annotations. Unknown fields,
duplicate keys, invalid record order, or schema-invalid values fail content
validation before hashing; they are not silently discarded.

The content digest is:

```text
SHA-256(content-domain-prefix || 0x00 || canonical(CanonicalContentHashInput))
```

The committed golden is `tests/goldens/content_hash.txt`. The runtime validates
the loaded catalog against that golden and carries the computed digest into each
save envelope under ADR-0007. `GameState.content_version` remains a human-readable
compatibility label, not a substitute for this exact digest.

### 6. Canonical state projections

All projections start from a cloned complete approved `GameState`. Equivalence
projections first apply lifecycle normalization (`Running`/`Advancing` to `Paused`;
`Paused` and `Won` unchanged). This removes execution-control mode, not gameplay.
Unloaded/Loading have no valid projection.

#### Scenario projection

The scenario projection removes exactly the top-level `command_log` and
`next_event_sequence` members and retains every other top-level and transitive
field. Removal fails if either member is absent or malformed; a separately mirrored
Rust view that could forget future fields is not permitted. It is hashed with the
scenario-state domain prefix.

This projection is used by the tick-zero golden and deterministic gameplay
scenario assertions. Commands are test inputs rather than part of the asserted
gameplay state.

#### Replay-equivalence projection

The replay-equivalence projection removes exactly `next_event_sequence` and includes
the complete `command_log`, its payloads, order, effective ticks, server sequences,
and outcomes. It is hashed with the replay-state domain prefix.

This projection is used for uninterrupted/real-time/batch/save-split/command-replay
equivalence. The removed value is a server-session event-stream lower bound: actor
controls and same-process state replacement can advance/rebase it without changing
simulation semantics. Runtime event retention, derived supply/demand tables,
presentation state, and the outer save envelope are absent because they are not
fields of `GameState`.

#### Save-integrity projection

The save-integrity projection is the complete lifecycle-normalized `GameState`,
including both `command_log` and `next_event_sequence`. It is hashed with the save-
state domain prefix. It protects the exact persisted cursor lower bound even though
that session bookkeeping is deliberately excluded from deterministic equivalence.

Exact event equivalence uses a parallel canonical deterministic trace. It retains the
committed tick and complete payloads for ordinary ticks, replayable-command outcomes,
and domain events; removes the runtime `event_sequence`; removes
`next_event_sequence` from StateDelta roots; and excludes actor-control outcome,
control-only lifecycle/StateDelta, and RuntimeError events. Those excluded records are
server-session observations, not command-log replay inputs. Their ordering,
idempotency, retention, and resynchronization remain mandatory protocol tests under
ADR-0012.

All three functions return `Result<[u8; 32], CanonicalHashError>`. Tests also expose
the canonical bytes so a failure can report the first byte/value divergence
rather than only opaque digests.

### 7. Save state hash

ADR-0007's `state_hash` is exactly the save-integrity digest of the normalized
`game_state` stored in the envelope. A loader canonicalizes the nested JSON value
under the envelope's declared hash scheme and verifies the digest before any
schema migration. This is an accidental-corruption/integrity check, not an
authentication mechanism; a party able to edit the save can also recompute an
unkeyed digest.

The envelope separately stores `content_hash`. State integrity and exact content
compatibility are distinct checks and both are mandatory.

### 8. Golden-update policy

A golden changes only with an intentional authoritative content/state rule,
serialized-schema change, or reviewed canonicalization/hash-scheme change.

- The same increment updates the authoritative document, implementation,
  affected fixtures/tests, and evidence-log explanation.
- CI never updates goldens automatically. An unexplained mismatch fails.
- Review includes the old/new canonical bytes or a focused semantic diff, not
  only the two digest strings.
- Reordering an in-memory insertion sequence must not change a golden. A schema
  order change may change it only when that order is semantically serialized.

Golden paths:

- `tests/goldens/content_hash.txt`
- `tests/goldens/state_hash_tick0.txt`
- `tests/goldens/scenarios/<scenario>.state_hash` where a scenario intentionally
  uses a full-state golden

Under the cumulative Phase 1 CI policy, content-hash checks become mandatory at
P1-05, tick-zero state-hash checks at P1-08, and each scenario golden at its
owning increment. Once activated, each remains mandatory on every later commit.

### 9. Required executable proofs

- Canonical byte fixtures cover nested object sorting, non-ASCII strings, every
  escape, signed/unsigned boundaries, nulls, empty collections, and float/
  duplicate-key rejection.
- Property tests permute object/map insertion order and obtain identical bytes
  and hashes.
- Serialization snapshots lock snake_case fields and tagged enum shapes.
- Projection tests prove that changing only `command_log` changes replay/save hashes
  but not scenario, while changing only `next_event_sequence` changes save integrity
  but neither equivalence hash. Running and Advancing normalize to the same Paused
  equivalence bytes.
- Cross-platform CI compares canonical bytes before comparing hashes, making an
  encoding discrepancy diagnosable.
- Save tests recompute both content and save-integrity hashes independently from
  envelope values.

## Consequences

- Content and state fingerprints are stable across supported platforms and do
  not depend on Rust source-field order or ordinary JSON map insertion order.
- The scheme identifier and domain prefixes make future hash evolution explicit.
- Exact content identity travels with saves, while `content_version` continues to
  describe compatibility/release lineage.
- The project owns a small canonical writer and duplicate-key parser/check. That
  maintenance cost is accepted and covered by exact-byte/property fixtures.
- Adding or changing a serialized field intentionally changes affected hashes
  and requires a reviewed golden update.

## Related ADRs

- ADR-0002 — Deterministic Tick Simulation
- ADR-0005 — Test Architecture
- ADR-0007 — Save Envelope Format, Content Hash Placement, and Migration Fixtures
