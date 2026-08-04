---
status: accepted
owner: Tech Lead
date: 2026-08-04
---

# ADR-0008: Accepted-Command Persistence

## Context

Phase 1 requires save/load and process-restart semantics for commands that have
been accepted but not yet executed. ADR-0003 §Command Envelope and Ordering
establishes that the actor assigns `server_sequence` monotonically, queues
commands for future ticks while Running, and applies them immediately while
Paused. ADR-0004 §Save Normalization specifies that a save from Running or
Advancing loads as Paused. ADR-0006 §4 defines the replay-mode state hash that
includes `command_log`. ADR-0007 defines the save envelope and load procedure.
GDD 13 defines `CommandRecord`, `CommandOutcome`, `command_log`, and
`next_server_sequence` in `GameState`.

Four unresolved specification questions remain:

1. **Pending-command serialization.** Commands queued for future ticks live in
   the actor's `pending_commands: BTreeMap<u64, Vec<SequencedCommand>>` (TDD
   01). When `SaveNow` happens, are these commands serialized in the save? Are
   they lost on load? If serialized, where do they live in `GameState`?

2. **Mailbox draining before save.** The actor mailbox may contain
   `SubmitCommand` messages that have not yet been assigned a `server_sequence`
   or placed in `pending_commands`. What happens to them when the save snapshot
   is taken?

3. **CommandRecord outcome for pending commands.** GDD 13 defines
   `CommandOutcome { Accepted, Applied, Rejected }`. A command queued for a
   future tick has been accepted but not yet applied or rejected. Does it get a
   `CommandRecord` with `outcome: Accepted` immediately, or only after
   execution? How does the idempotency map treat pending commands?

4. **Idempotency across save/load and process restart.** The idempotency map
   (command_id → recorded outcome) must survive save/load. After a process
   restart, the loaded save's command_log is the only source of idempotency
   data. What commands are lost? How does the client recover?

## Decision

### 1. Command_log is the single source of truth

Every accepted command is appended to `command_log` immediately when the actor
assigns its `server_sequence`. There is **no separate** `pending_commands`
field in `GameState`. The actor's internal `pending_commands` BTreeMap is a
runtime schedule rebuilt from `command_log` after load and maintained as
commands are accepted during play.

The `CommandRecord` outcome evolves over the command lifecycle:

- **`Accepted`** — The command has been validated at the API level (envelope
  parsed, lifecycle check passed, `expected_tick` checked if provided) and
  assigned a `server_sequence` and `effective_tick`. It is queued for execution
  at `effective_tick` (while Running) or will be applied immediately (while
  Paused). The command payload is stored and idempotency is established.

- **`Applied`** — The command was executed during tick processing (phase 1 of
  the tick transaction) and its effects are committed in `GameState`. The
  outcome is final.

- **`Rejected`** — The command was processed during tick processing but
  rejected by domain validation (e.g., invalid target, insufficient
  prerequisites). The rejection event carries the error details. The outcome is
  final.

A `CommandRecord` with `outcome: Accepted` is a **pending** command. It appears
in `command_log` at the moment of acceptance, before its effective tick. When
the actor drains pending commands for a tick, it changes the outcome to
`Applied` or `Rejected`. No record is ever removed; the `command_log` grows
monotonically.

```rust
enum CommandOutcome { Accepted, Applied, Rejected }

struct CommandRecord {
    id: String,
    effective_tick: u64,
    server_sequence: u64,
    command: Command,
    outcome: CommandOutcome,
}
```

**Rationale.** A single `command_log` eliminates the risk of divergence between
a separate `pending_commands` field and the actor's runtime schedule. Every
accepted command has one canonical representation. The idempotency map is
simply a lookup into `command_log` by command ID. Replay equivalence is
preserved because `command_log` is included in the replay-mode hash
(ADR-0006 §4).

### 2. Mailbox draining before save

Before the actor takes a save snapshot, it processes **every** message in its
mailbox to completion:

- `SubmitCommand` → assign `server_sequence`, validate lifecycle, append to
  `command_log`, queue or apply.
- `SchedulerTick` → ignored during snapshot preparation (the save captures
  state at the current commit boundary; pending scheduler ticks are
  discarded and re-scheduled after load).
- `GetSnapshot` → served normally; the snapshot is taken after mailbox drain.

After draining, the mailbox is empty. The actor then clones the authoritative
`GameState` for the persistence service. This guarantees that the save
snapshot contains every command accepted up to the save moment and that no
mailbox state is lost or serialized.

**Rationale.** Draining the mailbox before save means the snapshot is taken at
a well-defined point with zero unprocessed commands. Commands that arrived
between the last tick commit and the save request are captured. The alternative
— serializing mailbox state — would couple `GameState` to the actor's async
channel, which is implementation-specific and fragile.

### 3. Actor rebuilds pending_commands on load

On load from a save file:

1. `GameState` is deserialized with its full `command_log`.
2. The actor rebuilds its internal `pending_commands` schedule by iterating
   `command_log` and collecting entries where `outcome == Accepted`,
   grouping them by `effective_tick` in `server_sequence` order.
3. The idempotency map is rebuilt by scanning the entire `command_log` and
   indexing by command ID. Duplicate IDs (same ID, same payload) return the
   existing record; same ID with different payload is an invariant violation
   that should not occur in a well-formed save.
4. The `next_server_sequence` counter from `GameState` continues from its
   serialized value.

Commands whose `outcome` is `Applied` or `Rejected` are already finalized;
they contribute to idempotency but are not re-queued. Commands whose `outcome`
is `Accepted` are pending and will be processed at their `effective_tick` in
the correct sequence.

### 4. Save normalization for pending commands

ADR-0004 §Save Normalization specifies that a save from Running or Advancing
loads as Paused. Pending commands survive this normalization:

- Commands with `outcome: Accepted` remain in `command_log` with their original
  `effective_tick` and `server_sequence`.
- After load, the lifecycle is Paused. Pending commands scheduled for future
  ticks remain queued. They will execute when the user resumes or calls
  `AdvanceTicks`.
- Commands that were applied immediately while Paused before the save have
  `outcome: Applied` and do not need re-queuing.

The normalization never changes `command_log` entries. Only the
`GameState.lifecycle` field changes from Running/Advancing to Paused.

### 5. Idempotency across save/load and process restart

**Save/load within the same process.** The full `command_log` is serialized and
restored. After load, the idempotency map is rebuilt. Re-submitting a command
with an existing `id` and identical payload returns the original recorded
outcome. Re-submitting with a different payload returns 409 `IdempotencyConflict`.

**Process restart.** The new process loads the save file and rebuilds the
idempotency map from `command_log`. Commands that were accepted after the last
save but before the crash are **lost** — they existed only in the actor's
mailbox or runtime state and were never serialized. The client detects the
restart by re-reading `connection.json` (new token/port/PID) and must
re-submit those commands. The idempotency map in the new session covers only
commands from the loaded save; lost commands no longer have idempotency
protection and may be re-submitted with the same client-generated ID.

**Scope of idempotency.** Idempotency is per-server-session (ADR-0003). A
process restart begins a new session. However, commands from the loaded save
retain their idempotency records because the save itself carries them. The
session scope rule means: if a command from a prior session was saved, it is
idempotent in the new session. If it was not saved, it is not idempotent.

### 6. Process restart does not reuse conflicting IDs

The `IdCounters` in `GameState` are serialized and survive save/load. After
restart, counters continue from their loaded value. Generated command IDs
(used by clients as `cmd_<counter>`) will not collide with IDs from the
previous session because the counter is higher than any previously issued ID
in the loaded save. Clients that generate IDs independently (e.g.,
client-generated UUIDs) are unaffected — collision is astronomically unlikely
and the idempotency map rejects duplicates regardless.

### 7. Summary of save/load command flow

```
┌────────────────────────────────────────────────────────────────┐
│ 1. Actor drains mailbox                                       │
│    - SubmitCommand → assign server_sequence, record in        │
│      command_log, queue or apply                              │
│    - SchedulerTick → discard                                  │
│    - GetSnapshot → serve normally                             │
│                                                               │
│ 2. Actor clones GameState for persistence                     │
│    - command_log contains ALL accepted commands               │
│    - lifecycle is normalized to Paused (if Running/Advancing) │
│    - No pending_commands field exists in GameState            │
│                                                               │
│ 3. Persistence writes save envelope                           │
│    (format_version, content_version, state_hash, timestamp,   │
│     game_state)                                               │
│                                                               │
│ 4. On load:                                                   │
│    - Deserialize GameState from envelope                      │
│    - Rebuild pending_commands schedule from command_log       │
│      entries with outcome=Accepted, grouped by effective_tick │
│    - Rebuild idempotency map from full command_log            │
│    - Set lifecycle to Paused (unless save was Won)            │
│                                                               │
│ 5. On next tick or resume:                                    │
│    - Drain pending_commands for current tick in               │
│      server_sequence order                                    │
│    - Execute domain validation and state mutation             │
│    - Update outcome to Applied or Rejected                    │
└────────────────────────────────────────────────────────────────┘
```

## Consequences

### Positive

- Single `command_log` eliminates divergence between runtime schedules and
  serialized state.
- Mailbox draining before save guarantees no unaccounted-for commands.
- Pending commands survive save/load and process restart naturally.
- Idempotency is preserved across save/load; clients re-submit only commands
  lost in a crash.
- The actor's `pending_commands` schedule is derived data, not primary
  state — it is always reconstructable from `command_log`.

### Negative

- Mailbox draining adds a small latency to `SaveNow`: the actor must process
  all pending mailbox messages before taking the snapshot. At V1 scale this is
  negligible (a handful of commands).
- `command_log` grows monotonically across the entire game session. Each tick
  may change outcomes for many entries (Accepted → Applied/Rejected), but
  entries are never removed. The replay hash includes the full log, so the
  serialized `GameState` grows over time. Long sessions with many commands
  will produce larger save files and slower hashing. At V1 ceilings this is
  acceptable (thousands of commands over hours of play).
- After a crash, clients must resubmit commands that were accepted but not
  saved. This is inherent to any save-based persistence model and is
  mitigated by the autosave cadence (every 300 ticks + material Paused
  commands).

### Mitigations

- The `command_log` is stored in `GameState` and included in the replay hash.
  If save-file size becomes a concern, the log can be capped to the newest N
  entries for replay verification while retaining idempotency records for all
  commands via a separate compacted idempotency map. This optimization is
  deferred to post-V1.
- Autosave cadence (GDD 12) minimizes the window of lost commands in a crash.
- Clients detect restart via `connection.json` change and re-submit
  gracefully; the idempotency map in the new session prevents duplicate
  effects for saved commands.

## Related ADRs

- ADR-0003 — Command/Query API with WebSocket Streaming (defines command
  envelope, idempotency, server_sequence, effective_tick)
- ADR-0004 — Game Lifecycle State Machine (defines save normalization,
  load-from-lifecycle rules)
- ADR-0006 — Canonical Content/State Hashing (defines replay-mode hash
  includes command_log)
- ADR-0007 — Save Envelope Format, Content Hash Placement, and Migration
  Fixtures (defines save envelope, load procedure)
- ADR-0002 — Deterministic Tick Simulation (establishes deterministic state
  as a non-negotiable invariant)
