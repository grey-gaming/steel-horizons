---
status: accepted
owner: Tech Lead
date: 2026-08-04
---

# ADR-0008: Accepted-Command Persistence

## Context

The command endpoint carries both deterministic game-timeline commands and actor
controls. Those categories have different persistence requirements:

- Gameplay and configuration commands can be accepted while Running for a
  future tick. They must survive save/load and participate in deterministic
  replay.
- Actor controls (`NewGame`, `LoadAutosave`, `SaveNow`, `Pause`, `Resume`, and
  `AdvanceTicks`) operate the actor, scheduler, or persistence boundary. Replaying
  them as gameplay would recursively load/save, manufacture tick advancement, or
  replace the state being reconstructed.

The API still requires structural idempotency and one total command-receipt order
within a server session. A successful `NewGame` or `LoadAutosave` replaces
`GameState`, so that session-wide receipt state cannot live solely inside the
replaceable game state.

This ADR defines the persistent replay log, the runtime session receipt ledger,
the `SaveNow` linearization point, and exact load invariants.

## Decision

### 1. Classify commands before recording them

Commands are divided into two closed sets.

**Replayable game commands** are every construction, recovery, configuration,
research, survey, and Gate command in ADR-0003. They mutate the deterministic
game timeline. An accepted replayable command is represented in
`GameState.command_log`.

**Actor-control commands** are:

- `NewGame`
- `LoadAutosave`
- `SaveNow`
- `Pause`
- `Resume`
- `AdvanceTicks`

Actor controls never appear in `GameState.command_log`, are never placed in the
pending gameplay schedule, and are never executed by command-log replay. A replay
driver creates/loads its explicit initial fixture and consumes records in
`server_sequence` order. `ScheduledTick` implies acceptance at
`effective_tick - 1`; the driver advances to that committed tick, queues the command,
and later executes all commands for one effective tick as phase 1 of that single
ordinary tick. `PausedImmediate` implies a separate no-tick transaction at
`effective_tick`; the driver advances to that tick first, then executes each such
record separately in sequence order. This is an internal validated replay path, not
an API field clients may choose. It preserves sequence gaps consumed by controls or
rejected envelopes without re-executing controls and reproduces the number/order of
StateDelta boundaries. Lifecycle normalization is asserted separately.

This separation prevents an `Accepted` `SaveNow` from recursively saving after
load and prevents `NewGame`/`LoadAutosave` from replacing replay state.

### 2. Runtime receipt ledger owns server-session idempotency

The actor owns a runtime `SessionReceiptLedger`, outside `GameState`. It survives
successful `NewGame` and `LoadAutosave` operations but is reset when the process
starts a new authenticated server session.

For every syntactically valid command envelope dequeued by the actor, the ledger
stores:

```text
struct SessionReceipt {
  id: string
  expected_tick: u64 | null
  command: Command
  server_sequence: u64
  accepted: bool
  effective_tick: u64 | null
  application_boundary: PausedImmediate | ScheduledTick | null
  status: Processing | Accepted | Applied | Rejected | Failed
  result: CommandResult | null
  rejection: CommandRejection | null
}
```

`CommandRejection` contains the stable machine-readable reason and structured
integer/string/ID details needed to reproduce the original result. `Failed` is
reserved for actor-control infrastructure failures such as an unsuccessful save;
it is never written to the deterministic replay log.
`CommandResult` is the canonical tagged union in GDD 13. The wire response's
`resulting_tick` is a convenience derived only from
`CommandResult::AdvanceTicksCompleted { resulting_tick }`; it is null for every
other result and is not a second source of truth.

`accepted` records whether the envelope crossed the actor-acceptance boundary and is
the source of the two wire shapes: false plus `Rejected` reproduces the ordinary
pre-acceptance API error, while true produces ADR-0003's acknowledgement. `Processing`
starts false. `Accepted` has accepted=true and null `result`/`rejection`; `Applied` has
accepted=true, a non-null result (including `CommandResult::None`), and null rejection; `Rejected`
and `Failed` have null result and a non-null typed rejection. A Rejected receipt may
have either accepted value; Failed always has accepted=true. HTTP/WS handlers do
not expose `Processing`: an original request or identical concurrent duplicate
waits until the actor records `Accepted` or a terminal status.

Structural equality compares the complete normalized envelope: the required nullable
value of `expected_tick` and the complete tagged `command` payload. Omitting the field
is malformed rather than a third idempotency state. The `id` is the lookup key.
Reusing an ID with the same normalized envelope returns or waits
for the existing receipt; it never starts a second operation. Reusing it with any
different envelope field returns `409 IdempotencyConflict`.

The ID lookup and structural comparison happen before current lifecycle or
`expected_tick` validation. Consequently, an exact retry returns its original
wire shape and receipt/error even when the game tick has since changed.
`effective_tick` is null for an envelope rejected before acceptance and for an accepted state-replacement control
started while Unloaded; otherwise it records the accepted game tick or
actor-control boundary. Replayable command records always have an integer
effective tick because they require an existing `GameState`.
`application_boundary` is set only for accepted replayable commands: PausedImmediate
for a separate no-tick actor transaction, or ScheduledTick for phase-1 execution in a
normal future tick. It is null for actor controls and pre-acceptance errors.

The actor assigns `server_sequence` from a runtime monotonically increasing
session counter before actor-level lifecycle, expected-tick, or domain validation.
Thus concurrent valid envelopes have one stable order, including envelopes that
are rejected. A successful state replacement does not reset this runtime counter.

`GameState.next_server_sequence` is the persisted lower bound for replayable
command sequencing, not the owner of actor-control receipt sequencing. When the
actor accepts a replayable command, it uses the runtime sequence and advances the
persisted value to `server_sequence + 1`. Gaps caused by actor controls are valid.
On process startup, the runtime counter starts at least as high as the loaded
`next_server_sequence`. On a same-process load it remains the maximum of its
current value and the loaded lower bound.

Event receipt order follows the parallel ADR-0012 rule: a runtime session event
allocator/ring survives state replacement, and `GameState.next_event_sequence` is a
persisted lower bound rebased before a candidate is published. This runtime event
state is not part of the idempotency ledger, but neither may be replaced by an older
save.

Command IDs are nonempty opaque client-supplied UTF-8 strings. Empty IDs are rejected
before acceptance and can never enter a receipt/log or save. The engine does not generate them,
and they are not part of `IdCounters`. Deterministic clients may use their own
monotonic labels such as `cmd_000123`; those labels do not create simulation
entities.

### 3. `command_log` is the source of replayable pending state

Every accepted replayable command is appended once to `command_log` when the
actor assigns its sequence. There is no separate serialized `pending_commands`
field. The runtime schedule is derived from log entries whose outcome is
`Accepted`.

The authoritative record is:

```text
enum CommandOutcome { Accepted, Applied, Rejected }
enum CommandApplicationBoundary { PausedImmediate, ScheduledTick }

struct CommandRecord {
  id: string
  expected_tick: u64 | null
  effective_tick: u64
  server_sequence: u64
  application_boundary: CommandApplicationBoundary
  command: ReplayableGameCommand
  outcome: CommandOutcome
  result: CommandResult | null
  rejection: CommandRejection | null
}
```

The lifecycle is:

- `Accepted`: API/lifecycle/expected-tick validation passed and the replayable
  `ScheduledTick` command is queued for `effective_tick`. Its complete envelope is
  durable in the next save. Both `result` and `rejection` are null.
- `Applied`: effective-tick domain validation passed and the tick or paused actor
  transaction committed. `result` is non-null (including the explicit `None`
  variant) and `rejection` is null.
- `Rejected`: effective-tick domain validation failed and that rejection record
  and event committed. `result` is null and `rejection` is present.

An ordinary domain outcome transition is part of the same atomic transaction as the
command's domain mutation or rejection event. Internal transaction/invariant failure
uses a separate failure-isolation commit so no accepted command is left overdue:

- A PausedImmediate candidate rolls back completely, including any staged ID
  allocation. The failure-isolation commit appends that command once as Rejected with
  typed `TransactionInvariantFailed`, commits its receipt/outcome and RuntimeError,
  and leaves gameplay unchanged and Paused.
- If an ordinary scheduled tick rolls back, every command due in that tick remains
  gameplay-unapplied and the failure-isolation commit changes all of those records to
  Rejected with typed `TickTransactionFailed`, in server-sequence order. It emits the
  matching outcomes plus RuntimeError, leaves the tick at the last committed value,
  and transitions runtime/state lifecycle to Paused. Future commands remain Accepted.

These internal Rejected outcomes are persisted because replayable commands must never
become dangling session-only failures. They carry null result, consume no generated ID,
and an exact retry returns the same failure. A simulation failure with no due command
has only the lifecycle/RuntimeError failure-isolation transaction.

While Paused, a replayable command is appended and finalized in one serialized
actor transaction without advancing the tick. No reader can observe the transient
`Accepted` state. While Running, it remains `Accepted` until the recorded future
tick commits.

### 4. `SaveNow` is a FIFO snapshot barrier, not a mailbox drain

The actor processes its mailbox serially. Dequeuing the `SaveNow` message is its
snapshot linearization point:

1. All earlier mailbox messages have already completed by FIFO ordering.
2. The actor assigns the control receipt sequence and validates the envelope,
   lifecycle, and optional `expected_tick`.
3. At the next safe committed-state boundary, it clones a normalized save
   snapshot. The clone includes every replayable command accepted before the
   barrier, including future `Accepted` records, and excludes every message after
   the barrier.
4. It enqueues that immutable clone on ADR-0007's single FIFO worker for the
   autosave target. The actor may process later ordinary mailbox messages and
   enqueue later snapshots while the clone is written, but replacements and
   LoadAutosave reads execute strictly in enqueue order.
5. The receipt becomes `Applied` only after atomic write/sync/replace succeeds.
   It becomes `Failed` with a typed persistence error if the write fails. An
   identical duplicate waits for or returns this same result; retrying an actual
   failed save requires a new command ID.

An accepted control whose infrastructure work fails receives the external
`CommandFailed` outcome event defined by ADR-0012, carrying the same persistence
error; its internal receipt status is also `Failed`. Deterministic domain
rejections use `CommandRejected`/`Rejected` instead. A successful control emits
`CommandApplied`. None of these control outcomes makes the control replayable.

The actor does not drain messages that arrived after `SaveNow`, does not discard
`SchedulerTick`, and does not serialize mailbox/channel state. A scheduler message
before the barrier has already been handled; one after it remains ordinary future
work. `GetSnapshot` requests retain their own mailbox positions and never move
across the save barrier.

If the process crashes after the atomic rename but before the acknowledgement,
the file is valid. The restarted process is a new server session, so the client
may safely issue a new `SaveNow`; replayable game commands in the file remain
idempotent through their saved log.

Autosave and shutdown snapshots use the same committed-state barrier and
normalization, but they are internal triggers rather than command envelopes. Shutdown
waits for its final queued write. `LoadAutosave` enters Loading and queues its read
behind all earlier target operations, so SaveNow immediately followed by LoadAutosave
cannot read the pre-SaveNow file. No parallel persistence task may access the canonical
target outside this lane.

### 5. Load rebuild and validation

The loader validates `GameState.command_log` before publishing the loaded state.
All of these are mandatory invariants:

- Every record contains a replayable game command; actor controls are invalid.
- Command IDs are non-empty and unique within the log.
- `server_sequence` values are unique and strictly increasing in array order.
- `next_server_sequence` is greater than every persisted record sequence. Gaps
  are valid.
- A record's implied acceptance tick is `effective_tick - 1` for ScheduledTick
  (which therefore requires `effective_tick > 0`) and `effective_tick` for
  PausedImmediate. Implied acceptance ticks are nondecreasing in server-sequence
  order.
- `Accepted` records are ScheduledTick with `effective_tick > GameState.tick`, null
  result, and null rejection. PausedImmediate is always terminal.
- `Applied` records have `effective_tick <= GameState.tick`, a non-null typed result
  (including `CommandResult::None`), and null rejection.
- Ordinary `Rejected` records have `effective_tick <= GameState.tick`, null result,
  and a non-null typed rejection. A ScheduledTick `TickTransactionFailed` record may
  instead have `effective_tick == GameState.tick + 1` immediately after the failed
  tick; the checked addition must not overflow. No other future terminal record is
  valid.
- Every Applied result variant matches its exact command under ADR-0012's
  exhaustive command-to-result table; generated IDs have the correct kind/prefix.
- The complete record is structurally valid for its schema version. Unknown
  command variants or fields are not repaired silently.

Any violation rejects the load atomically with a typed malformed-save error. In
particular, an overdue `Accepted` command is not guessed forward to a later tick.

After validation, the actor:

1. Groups `Accepted` ScheduledTick records by `effective_tick`.
2. Sorts each group by `server_sequence` (already unique by invariant).
3. Rebuilds the runtime pending schedule from those groups.
4. Seeds the new process session receipt ledger with every saved log record so
   identical resubmissions return the exact recorded result or rejection and
   conflicting payloads return `IdempotencyConflict`.
5. Sets the runtime sequence counter to at least `next_server_sequence`.

Every receipt reconstructed from a replayable `CommandRecord` has `accepted = true`;
pre-acceptance errors and actor-control receipts are session-only and are never
invented from a save.

For same-process `LoadAutosave`, the actor first checks imported saved IDs against
the existing session receipt ledger. A same-ID/different-envelope collision, or
an identical envelope carrying a different recorded `server_sequence`, rejects
the load before state publication as an ambiguous receipt collision. For an
identical envelope and sequence present in the loaded log, the loaded record
replaces the receipt's gameplay outcome and effective tick because the explicitly
restored timeline is authoritative; an
`Accepted` loaded record is therefore reported as pending, not as an `Applied`
result from the abandoned later timeline. Existing session receipts absent from
the loaded log remain idempotency tombstones: an exact retry returns its original
receipt without recreating the effect that `LoadAutosave`/`NewGame` deliberately
rewound, and the client must use a new ID to request that action in the new game
state. This preserves server-session idempotency across state replacement rather
than silently forgetting or double-applying an earlier request.

### 6. Save normalization preserves future game commands

A save from Running or Advancing loads Paused, while Won remains Won. Normalizing
the lifecycle does not edit command-log records:

- Future `Accepted` records retain their original effective ticks and sequences.
- They execute only when `AdvanceTicks` or resumed scheduling reaches that exact
  tick.
- Commands already finalized while Paused remain `Applied` or `Rejected` and are
  never requeued.

Submitting a new Paused command after load may have an earlier effective tick than
an older future command; this is explicit in the recorded `effective_tick` and is
replayable. Commands sharing an effective tick always execute by
`server_sequence`.

### 7. Process restart boundary

A new process creates a new authenticated server session and therefore a new
runtime control-receipt ledger. Loading a save imports its replayable command
records, preserving idempotency for every command whose effect is present or
pending in that save.

Commands submitted after the snapshot barrier fall into two groups:

- A command still waiting in the old mailbox was never accepted and had no
  acknowledgement.
- A command acknowledged after the barrier existed in later runtime state but not
  in the saved snapshot.

Both are absent after a crash. Clients may resubmit them with their original IDs.
Saved commands deduplicate through the imported log; unsaved commands are accepted
once in the new session. Actor-control receipts from the old process are not
imported because those controls are not deterministic game state.

## Required executable proofs

P1-12/P1-13 must cover at least:

- Same ID/same complete envelope, including `expected_tick`, returns one receipt.
- Same ID with a changed command or changed `expected_tick` conflicts.
- Running future commands survive save/load and execute at the same tick/order.
- Paused immediate commands never leave an observable `Accepted` record.
- Save barriers include all earlier and no later commands under concurrent
  submission, without dropping scheduler ticks.
- `SaveNow` is absent from the replay log and cannot recursively execute on load.
- Failed save I/O returns one stable failed control receipt and preserves the
  prior save.
- Delayed overlapping SaveNow/autosave/shutdown writes replace in actor enqueue
  order across success/failure combinations; a queued LoadAutosave observes every
  successful earlier barrier and no later one.
- NewGame/LoadAutosave retain same-session control idempotency and sequence order.
- Loading an earlier same-session save restores matching saved outcomes, retains
  absent IDs as tombstones, and rejects envelope/sequence receipt collisions.
- Same-session state replacement never reuses an event sequence, preserves the
  runtime ring, and emits one complete replacement delta; failure from Unloaded has
  a typed outcome with `tick = null`.
- Startup imports saved replayable IDs; saved resubmissions deduplicate while
  unsaved resubmissions execute once.
- An applied command that created an entity returns the same recorded generated
  ID after that entity is later removed, after save/load, and on an identical
  resubmission; the ID counter is not consulted by the retry.
- Every malformed-log invariant above is rejected without altering prior state.
- Uninterrupted, save-split, and replayed runs have identical ADR-0006
  replay-equivalence bytes/hashes and canonical deterministic event traces.

## Consequences

- Pending gameplay commands are durable without serializing an async mailbox.
- Actor controls cannot recursively enter deterministic replay.
- `SaveNow` has a precise snapshot and durability boundary under concurrency.
- State replacement does not erase current-session idempotency.
- The runtime receipt ledger is additional actor state, but it is deliberately
  non-gameplay and session-scoped.
- Command logs grow monotonically for the active game timeline. Compaction is
  deferred beyond V1 because it would need to preserve replay and saved
  idempotency evidence.

## Related ADRs

- ADR-0002 — Deterministic Tick Simulation
- ADR-0003 — Command/Query API with WebSocket Streaming
- ADR-0004 — Game Lifecycle State Machine
- ADR-0006 — Canonical Content/State Hashing
- ADR-0007 — Save Envelope
