---
status: accepted
owner: Tech Lead
date: 2026-08-04
---

# ADR-0012 — Complete Command and Event Wire Contracts

## Context

ADR-0003 owns the exhaustive V1 command union and the shared command envelope.
The protocol TDD owns HTTP/WebSocket framing and snake_case JSON field names.
Before P1-22 and P1-31, the external event union, lossless cache-update rules,
backpressure behavior, and research resume/reassignment semantics also need one
complete deterministic contract.

The contract must ensure that:

- generated OpenAPI/JSON Schema represents every command, event, tagged union,
  and error payload;
- a client can maintain the externally exposed `GameSnapshot` without guessing
  recursive-patch or deletion behavior;
- a slow client can never alter simulation timing or silently continue from an
  incomplete state; and
- research progress and material location remain exact when a project pauses,
  resumes, loses its Research Ship, or changes facility.

## Decision

### 1. Command ownership and wire conventions

ADR-0003's V1 command list is exhaustive. This ADR adds no second command list.
The generated command schema expands every pseudo-signature in ADR-0003 into a
tagged object with:

- snake_case field names;
- a PascalCase `type` discriminator;
- explicit integer widths and ID newtypes;
- rejection of unknown fields; and
- the shared `CommandEnvelope { id, expected_tick, command }` for both REST and
  WebSocket.

Commands rejected before actor acceptance return the ordinary typed API error
and emit no committed event. Every accepted command eventually receives exactly
one committed `CommandApplied`, `CommandRejected`, or `CommandFailed` event,
including commands that commit immediately while Paused. `CommandFailed` is
limited to accepted actor controls whose infrastructure operation fails, such
as `SaveNow` persistence I/O; replayable gameplay domain failures use
`CommandRejected`. Structural idempotency returns the recorded
acknowledgement/outcome and never emits a duplicate outcome event.

Event objects use the same snake_case/PascalCase convention. Embedded state
entities use the generated API model, not canonical-save byte encoding; the two
represent the same GDD 13 values but are separately schema-tested projections.
All JSON examples in this ADR are valid JSON.

### 2. Common event envelope and ordering

Every retained event has these fields:

```json
{
  "type": "LifecycleChange",
  "event_sequence": 813,
  "tick": 421,
  "from": "Paused",
  "to": "Running",
  "reason": "ResumeCommand"
}
```

- `event_sequence` is unique and strictly increasing within one authenticated
  server session. A checked runtime allocator owns it even while the game is
  Unloaded. Phase 11 stages gameplay batches against that allocator; committed
  state synchronizes `GameState.next_event_sequence` as a persisted lower bound.
  A rolled-back simulation transaction consumes no sequence, while a separately
  committed failure/control outcome does.
- `tick` is `u64 | null`. Gameplay/domain/StateDelta events always use the
  committed game tick. Paused actor transactions use the unchanged tick. Any
  actor-control/lifecycle/runtime event committed while no `GameState` exists uses
  null, including state-replacement start or failure from Unloaded; null is forbidden
  once a prior or newly committed state supplies an integer tick.
- A failed simulation transaction emits no events derived from its rolled-back
  gameplay mutations. The subsequent failure-isolation transaction may emit a
  lifecycle change and `RuntimeError` at the last committed tick.

The exhaustive V1 retained-event discriminator union is:

```text
LifecycleChange | CommandApplied | CommandRejected | CommandFailed |
EntityCreated | EntityRemoved | StateDelta |
BuildComplete | ResearchComplete | SurveyComplete | ArrivalEvent |
GatePhaseCompleted | BottleneckWarning | BottleneckCleared |
SalvageCreated | ResearchPaused | ResearchReassigned | ResearchResumed |
RuntimeError | ResearchProgress | SurveyProgress | ConstructionProgress |
GatePhaseProgress
```

`Hello`, `Subscribe`, `Command`, and `ResyncRequired` are WebSocket control
messages, not retained events, and therefore are outside this union.

Within one committed transaction, events are assigned sequences in this stable
category order:

1. command outcomes by `server_sequence`;
2. entity removals, then creations, by collection order and canonical key;
3. durable domain outcomes by the event-type order in sections 3.6–3.14 and their
   canonical primary key;
4. lifecycle transition, if any;
5. progress notifications by the event-type order in section 4 and their
   canonical primary key; and
6. one `StateDelta`, last, when externally exposed state changed.

The committed `StateDelta` includes the final `next_event_sequence` after all
events in that transaction have been assigned.

#### 2.1 State replacement and event continuity

The runtime event allocator and canonical ring survive successful same-process
`NewGame` and `LoadAutosave`; they are never replaced by `GameState` and never reset
on a timeline rewind. Before publishing a candidate, the actor checked-rebases
`candidate.next_event_sequence` to
`max(runtime_next_event_sequence, candidate.next_event_sequence)`. It then emits the
control outcome; `EntityRemoved { reason: StateReplaced }` for every old-only key;
`EntityCreated` for every new-only key; the lifecycle event; and one complete-
replacement `StateDelta` last, using the stable category/key order in section 2.
Common-key entities do not emit create/remove events even when their values differ.
The delta contains every root field, every entity in the new state as an upsert, and a
tombstone for every old-state entity key absent from the candidate. No ordinary domain
completion event is inferred merely from a before/after difference. A client can
therefore continue one monotonic session stream without confusing an older saved
counter for new history.

On a fresh process, the empty runtime ring sets `oldest_available_sequence` to the
loaded next-sequence lower bound and seeds its allocator from the same value; autoload
before API Ready publishes the initial state/status but emits no retained event of any
kind (no command, lifecycle, entity, or StateDelta event). The externally reported
`latest_event_sequence` is the trusted runtime next sequence minus one (the allocator
is always at least one). Exact polling boundary rules are in section 6. A failed state-
replacement control keeps the prior gameplay state/lifecycle but still commits its
session receipt, event outcome, lifecycle restoration, and advanced event lower bound.
When failure began from Unloaded, those runtime events use `tick = null`.

The loaded envelope hash is verified before rebasing. Fresh-process load equivalence
therefore remains exact. Same-session explicit state replacement intentionally changes
only normalized lifecycle plus receipt/event-cursor bookkeeping before subsequent
gameplay; P1-13/P1-31 test that projection and the complete replacement delta.

### 3. Durable outcome event union

“Durable” means the event is inserted into canonical retained history and may
never be silently coalesced or discarded from a live client queue. The bounded
history still evicts every event normally when it falls behind the newest
50,000-event boundary. Durability does not mean infinite retention.

#### 3.1 LifecycleChange

```text
LifecycleChange {
  from: GameLifecycle,
  to: GameLifecycle,
  reason: NewGameStarted | LoadAutosaveStarted |
          NewGameSucceeded | LoadAutosaveSucceeded |
          NewGameFailedRestored | LoadAutosaveFailedRestored |
          PauseCommand | ResumeCommand |
          AdvanceTicksStarted | AdvanceTicksComplete | AdvanceTicksFailed |
          GateActivation | SimulationFailure
}
```

`reason` is required; it is never conditionally omitted or null.

#### 3.2 CommandApplied

```text
CommandApplied {
  id: string,
  server_sequence: u64,
  effective_tick: u64 | null,
  result: CommandResult
}

CommandResult =
  None |
  BuildOrderCreated { order_id: BuildOrderId } |
  SurveyOrderCreated { order_id: SurveyOrderId } |
  ResearchProjectCreated { tech_id: TechId } |
  ResearchProjectUpdated { tech_id: TechId } |
  GateAssemblyStarted |
  AdvanceTicksCompleted { resulting_tick: u64 }
```

`CommandResult` is the exact GDD 13 union and uses the common internally tagged
`type` representation; its unit `None` variant is `{"type":"None"}`.
`effective_tick` is null only for an accepted state-replacement control begun
while Unloaded. Every replayable gameplay outcome carries an integer tick.

The command-to-result mapping is exhaustive:

| Command | Applied result |
|---|---|
| `QueueBuildShip`, `QueueBuildStation`, `QueueUpgrade`, `QueueDemolishStation` | `BuildOrderCreated` with the allocated order ID |
| `QueueSurvey` | `SurveyOrderCreated` with the allocated order ID |
| `QueueResearch` with no existing project for the technology | `ResearchProjectCreated` with its technology ID |
| `QueueResearch` resuming or reassigning an existing non-complete project | `ResearchProjectUpdated` with its technology ID |
| `BeginGateAssembly` | `GateAssemblyStarted` |
| `AdvanceTicks` | `AdvanceTicksCompleted` with the final committed tick |
| `NewGame`, `LoadAutosave`, `SaveNow`, `Pause`, `Resume`, `CancelBuildOrder`, `ScrapShip`, `SetStationPriority`, `ConfigureBuffer`, `SetProductionRecipe`, `SetMiningTarget`, `PauseResearch`, `CancelSurveyOrder` | `None` |

No applied command may choose a different result variant or omit the explicit
`None` result.

#### 3.3 CommandRejected

```text
CommandRejected {
  id: string,
  server_sequence: u64,
  effective_tick: u64 | null,
  error: CommandRejection
}
```

`code` is the same machine-readable code used by the HTTP error schema.
`details` is the integer-only constrained map defined by GDD 13 and is `{}`
when the error has no structured details.

#### 3.4 CommandFailed

```text
CommandFailed {
  id: string,
  server_sequence: u64,
  effective_tick: u64 | null,
  error: CommandRejection
}
```

This event records the stable `Failed` session receipt defined by ADR-0008. It
does not enter the deterministic replay log. A persistence failure also emits
the corresponding `RuntimeError`; an idempotent retry returns the same failed
receipt without emitting either event again.

#### 3.5 EntityCreated and EntityRemoved

```text
EntityCreated {
  collection: EntityCollection,
  id: string,
  entity: FullEntity
}

EntityRemoved {
  collection: EntityCollection,
  id: string,
  reason: Scrapped | Demolished | Cancelled | Completed | Released | Compacted |
          StateReplaced
}
```

`FullEntity` is the complete generated API representation for the named
collection. The valid collections and canonical IDs are:

| Collection | ID |
|---|---|
| `celestial_bodies` | `BodyId` |
| `stations` | `StationId` |
| `ships` | `ShipId` |
| `research_projects` | `TechId` |
| `survey_orders` | `SurveyOrderId` |
| `build_orders` | `BuildOrderId` |
| `salvage_caches` | `SalvageId` |
| `logistics_reservations` | `ReservationId` |
| `bottleneck_trackers` | `station:<StationId>/resource:<ResourceType>` |
| `gate_build` | the singleton key `gate_site` |

Every insertion into or deletion from an externally exposed entity collection
emits the corresponding durable event. `EntityRemoved` is mandatory even when
the same tombstone appears in `StateDelta`. Permanent salvage caches are not
removed during ordinary V1 play. Terminal records may remain in their
collection; an event is emitted only if an actual collection insertion/deletion
occurs. Successful explicit NewGame/LoadAutosave compares the prior and candidate
key sets exactly as section 2.1 specifies; fresh-process autoload emits no synthetic
entity events because it has no prior published game transaction.

#### 3.6 BuildComplete

```text
BuildComplete {
  order_id: BuildOrderId,
  source_hub_id: StationId,
  target: BuildCompletionTarget
}

BuildCompletionTarget =
  Ship { ship_id: ShipId, role: ShipRole, tier: u8, hub_id: StationId } |
  Station { station_id: StationId, station_type: StationType, tier: u8,
            body_id: BodyId, orbit_ring: u8, slot: u8 } |
  Upgrade { station_id: StationId, from_tier: u8, to_tier: u8 } |
  Demolition { station_id: StationId, recovery_hub_id: StationId,
                salvage_id: SalvageId }
```

#### 3.7 ResearchComplete

```text
ResearchComplete {
  tech_id: TechId,
  facility_id: StationId,
  ticks_completed: u64,
  resources_consumed: Map<ResourceType, u32>
}
```

#### 3.8 SurveyComplete

Emitted at every completed survey-depth milestone. `order_complete` states
whether the milestone also completed the order.

```text
SurveyComplete {
  order_id: SurveyOrderId,
  body_id: BodyId,
  new_depth: u8,
  work_completed: u32,
  total_work: u32,
  order_complete: bool
}
```

#### 3.9 ArrivalEvent

```text
ArrivalEvent { ship_id: ShipId, arrival: ArrivalOutcome }

ArrivalOutcome =
  CargoPickup { reservation_id: ReservationId,
                source: InventorySourceRef,
                resource: ResourceType, amount: u32 } |
  CargoDelivery { reservation_id: ReservationId,
                  destination: InventoryDestinationRef,
                  resource: ResourceType, amount: u32 } |
  Refuel { station_id: StationId, fuel_transferred: u32 } |
  BuildSite { order_id: BuildOrderId, destination: DestinationRef } |
  SurveySite { order_id: SurveyOrderId, body_id: BodyId } |
  ResearchDock { station_id: StationId, tech_id: TechId } |
  RescueTow { hub_id: StationId }
```

The referenced unions use exact tagged shapes:

```text
InventorySourceRef =
  Station { station_id: StationId } |
  Salvage { salvage_id: SalvageId }

InventoryDestinationRef =
  Station { station_id: StationId } |
  BuildOrder { order_id: BuildOrderId } |
  Evacuation { order_id: BuildOrderId } |
  GateSite

DestinationRef =
  Body { body_id: BodyId } |
  Station { station_id: StationId } |
  Salvage { salvage_id: SalvageId } |
  GateSite
```

All variants serialize as objects with their PascalCase `type` discriminator.
A zero-Fuel Refuel transfer still emits `ArrivalEvent` with
`fuel_transferred: 0`.

#### 3.10 GatePhaseCompleted

Emitted only at a phase boundary, including final activation:

```text
GatePhaseCompleted {
  completed_phase: GatePhase,
  next_phase: GatePhase | null,
  progress_work: u32,
  required_work: u32
}
```

Final activation uses `next_phase: null` and is followed by the Won
`LifecycleChange` in the same committed transaction according to the stable
category order above.

#### 3.11 BottleneckWarning and BottleneckCleared

```text
BottleneckWarning {
  station_id: StationId,
  resource: ResourceType,
  consecutive_deficit_ticks: u16
}

BottleneckCleared {
  station_id: StationId,
  resource: ResourceType,
  consecutive_clear_ticks: u16
}
```

The warning value is exactly 300 when first emitted. The clear value is exactly
300 when first emitted.

#### 3.12 SalvageCreated

```text
SalvageCreated {
  salvage_id: SalvageId,
  position: SystemPosition,
  origin: SalvageOrigin
}

SalvageOrigin =
  BuildCancellation { order_id: BuildOrderId } |
  DemolitionEvacuation { order_id: BuildOrderId } |
  ScrapOverflow { ship_id: ShipId }
```

#### 3.13 ResearchPaused, ResearchReassigned, and ResearchResumed

```text
ResearchPaused {
  tech_id: TechId,
  facility_id: StationId | null,
  reason: Manual | NoResearchShip | FacilityUnavailable,
  release_unused: bool,
  ticks_completed: u64
}

ResearchReassigned {
  tech_id: TechId,
  from_facility_id: StationId | null,
  to_facility_id: StationId,
  state: AwaitingMaterials | Ready | Paused,
  pause_reason: NoResearchShip | null,
  ticks_completed: u64,
  resources_consumed: Map<ResourceType, u32>
}

ResearchResumed {
  tech_id: TechId,
  facility_id: StationId,
  from_reason: Manual | NoResearchShip | FacilityUnavailable,
  state: AwaitingMaterials | Ready,
  ticks_completed: u64,
  resources_consumed: Map<ResourceType, u32>
}
```

`ResearchResumed` means an actual transition out of `Paused`; accepting a
resume command that must continue waiting for a Research Ship does not emit it
yet. Reassignment always emits `ResearchReassigned` and emits
`ResearchResumed` additionally only if that same transaction leaves `Paused`.
For `ResearchPaused`, a manual pause reports the command's flag,
`NoResearchShip` reports `release_unused: false`, and detaching demolition
reports `release_unused: true`.

#### 3.14 RuntimeError

```text
RuntimeError {
  severity: Error | Fatal,
  subsystem: Simulation | Persistence,
  code: string,
  message: string,
  details: Map<string, ErrorDetail>
}
```

Persistence failures use `Error` and do not terminate simulation. A simulation
invariant failure uses `Fatal` for that transaction, rolls back its gameplay
changes, and accompanies the failure-isolation lifecycle transition to Paused.
Startup failures occur before retained event history exists and are reported by
startup/API error surfaces instead.

### 4. Coalescible progress event union

Progress events are informational cumulative readings. The canonical retention
ring stores every one. Only a per-client WebSocket queue may replace an older
pending progress event with a newer event having the same coalescing key.

#### 4.1 ResearchProgress

```text
ResearchProgress {
  tech_id: TechId,
  facility_id: StationId,
  ticks_completed: u64,
  total_ticks: u64,
  resources_consumed: Map<ResourceType, u32>
}
```

Coalescing key: `(ResearchProgress, tech_id)`.

#### 4.2 SurveyProgress

```text
SurveyProgress {
  order_id: SurveyOrderId,
  body_id: BodyId,
  work_completed: u32,
  total_work: u32
}
```

Coalescing key: `(SurveyProgress, order_id)`.

#### 4.3 ConstructionProgress

```text
ConstructionProgress {
  order_id: BuildOrderId,
  state: BuildState,
  progress_work: u32,
  total_work: u32
}
```

Coalescing key: `(ConstructionProgress, order_id)`.

#### 4.4 GatePhaseProgress

```text
GatePhaseProgress {
  phase: GatePhase,
  progress_work: u32,
  required_work: u32
}
```

Coalescing key: `(GatePhaseProgress, gate_site)`. A phase boundary also emits
the durable `GatePhaseCompleted` event.

No other event type is coalescible in V1.

### 5. StateDelta uses complete replacements

`StateDelta` is a continuity-required event. It is retained and may not be
coalesced or silently dropped from a live queue. If it cannot be delivered, the
client must resynchronize as defined in section 6.

```text
StateDelta {
  root_changes: RootChanges,
  upserted_entities: EntityReplacement[],
  removed_entities: EntityKey[]
}

EntityReplacement {
  collection: EntityCollection,
  id: string,
  entity: FullEntity
}

EntityKey { collection: EntityCollection, id: string }
```

`root_changes` may contain only these top-level non-entity `GameState` fields:

- `schema_version`;
- `content_version`;
- `lifecycle`;
- `tick`;
- `next_server_sequence`;
- `next_event_sequence`;
- `id_counters`;
- `completed_techs`;
- `rng_state`; and
- `command_log`.

Every present root field is a complete replacement. An absent field is
unchanged. Arrays, sets, maps, nested objects, and tagged variants within a
present root value are replaced in full; there is no recursive merge.

Every `upserted_entities` entry is the complete current API representation of
that entity after commit. It replaces any cached entity with the same
collection/key in full. Every actual collection deletion appears in
`removed_entities` and also has its durable `EntityRemoved` event. The arrays
are ordered by collection order from section 3.5 and canonical key. A key may
not occur in both arrays.

Optional fields are carried inside the complete entity: JSON `null` clears a
nullable field when that field's API schema uses null; omitted optional fields
follow the generated full-entity schema. Map-entry deletion never needs a
special leaf tombstone because the containing entity or root map is replaced
in full.

One StateDelta is emitted for every committed simulation tick, including a
no-op gameplay tick, because `tick` and `next_event_sequence` change. A Paused
actor transaction emits one when it changes externally exposed state. Query-
only operations and commands rejected before actor acceptance do not emit one.
An accepted command that later commits `CommandRejected` does emit StateDelta
because its command-log outcome and event cursor changed. A rolled-back
simulation mutation appears only through the separate committed
failure-isolation state/event transaction.

Client application is unambiguous:

1. Apply events strictly in `event_sequence` order.
2. Replace every present `root_changes` field.
3. Delete every `removed_entities` key.
4. Replace every `upserted_entities` entity in full.
5. Set the cache cursor to the event's sequence only after the whole event
   applies successfully.

Durable entity events are useful independently for filtered subscribers; their
duplication with StateDelta is idempotent.

### 6. Retention, backpressure, and resynchronization

The session-owned canonical event ring contains the newest 50,000 events of
**all** types in exact sequence order and survives game-state replacement. It does
not coalesce. HTTP polling returns canonical events from that ring. At process
startup `oldest_available_sequence` is initialized from the loaded next-sequence
lower bound even though the in-memory ring is empty, so an older cursor cannot
masquerade as retained data. When the ring is nonempty, this value is the first
retained event's sequence; when it is empty, it is the next sequence that would be
allocated. `latest_event_sequence` is always
`next_session_event_sequence - 1`, so a new session beginning at sequence one reports
zero and a fresh process loaded with lower bound 814 reports 813 despite retaining no
events.

For a polling/subscription cursor `since_sequence`:

1. If it is greater than `latest_event_sequence`, reject it with 400
   `InvalidEventCursor` and the current latest value.
2. Otherwise, return 410 `ResyncRequired` exactly when
   `since_sequence + 1 < oldest_available_sequence`; the addition is safe after the
   preceding upper-bound check.
3. Otherwise, return all matching retained events with sequence greater than the
   cursor, subject to the requested page limit.

Thus a cursor exactly one below the oldest retained event is resumable. For a freshly
loaded empty ring at next sequence 814, cursor 813 returns an empty page, while cursor
812 requires resynchronization. This makes the snapshot-then-subscribe loop
terminating and race-free.

HTTP polling requires `since_sequence: u64`. `limit` is optional, defaults to 1,000,
and must be `1..=1,000`. Repeated `event_type` parameters form an optional nonempty
filter of unique exact retained-event discriminators; omitting the parameter selects
all types, while an unknown/duplicate/empty value is a 400 error. The actor clones one
immutable ring view and latest cursor as the request linearization point, then scans
canonical sequence order. It stops at the `limit`th matching event, or scans through
that captured latest cursor when fewer match.

```text
EventPage {
  protocol_version: string,
  events: Event[],
  next_since_sequence: u64,
  oldest_available_sequence: u64,
  latest_event_sequence: u64,
  has_more: bool
}
```

`next_since_sequence` is the last canonical sequence scanned, not merely the last
matching event: it is the last returned event when the limit stops the scan, otherwise
the captured latest cursor (and equals the request cursor if both are zero). `has_more`
is exactly `next_since_sequence < latest_event_sequence`. The next page uses
`next_since_sequence`, so a filter that matches nothing still advances safely. A 410
error has detail keys `oldest_available_sequence` and `latest_event_sequence`; a
future-cursor 400 has `latest_event_sequence`.

Each WebSocket client has a 2,048-message outbound queue:

1. A newer progress notification may replace an older queued progress event
   only when both have the same section-4 coalescing key. The server removes the
   older queued item and appends the newer canonical event at its natural tail;
   it never overwrites an earlier queue slot, so delivered event sequences stay
   increasing. The newer event and its own sequence are sent unchanged; the
   skipped progress sequence is intentionally informational.
2. Durable outcomes and StateDelta are never replaced or dropped.
3. If a required message cannot be enqueued after permitted progress
   coalescence, the server makes a best effort to send the unsequenced
   `ResyncRequired` control message and then closes with application code 4009.
   Failure to enqueue the control message does not delay closure.
4. Queue pressure never blocks the simulation actor or changes tick timing.

The race-free resynchronization loop is:

1. Fetch `GET /state` and record its `latest_event_sequence` cursor.
2. Subscribe/poll with `since_sequence` equal to that cursor.
3. Apply **all** retained events with greater sequence, not only durable ones.
4. If the cursor is already behind retention, repeat from a new snapshot.

Subscription filters are permitted, but a client requesting a complete local
state cache must subscribe to StateDelta and may not infer completeness from a
filtered stream.

### 7. Research resume and reassignment

#### 7.1 Facility validity

A project may target:

- a Hub whose serialized `built_in_research_max_tier` is non-null and at least
  the technology tier; or
- a Research Station, which requires a docked eligible Research Ship before
  progress can become Active.

The field value is authored instance data. V1 Hub upgrades do **not** derive a
new built-in research tier from the Hub tier. Canonical Hub Haven has the unique
authored value `1`; therefore Tier-2 through Tier-4 technologies require a
Research Station unless authored content is explicitly changed and revalidated.

The target must exist, support research, have no different nonterminal project,
and pass technology availability and buffer-capacity validation. A technology
has at most one project globally. All validation and reservation changes are
atomic.

#### 7.2 Material accounting at one facility

`ResearchProject.resources_reserved` describes only unconsumed units physically
reserved at the project's current `station_id`. For each resource:

```text
outstanding = resources_required - resources_consumed
missing = outstanding - resources_reserved
```

Starting or resuming reserves compatible unallocated stock already at the
target, then creates/expands target input-buffer maxima and demand for the
remaining missing amount. It never shrinks another buffer. If capacity cannot
fit the required maxima, the entire command is rejected with the exact missing
capacity and no state change.

Material-state precedence after a successful start/resume/reassignment is:

1. A Research Station without its required docked Research Ship is `Paused`
   with `pause_reason = NoResearchShip`; its material demand remains active.
2. Otherwise, any missing material gives `AwaitingMaterials` with no pause
   reason.
3. Otherwise the project is `Ready` and becomes `Active` on the next eligible
   research phase.

`NoResearchShip` is never a `ResearchState` variant.

#### 7.3 Manual resume at the same facility

`QueueResearch` for a manually paused project at the same facility:

- preserves `ticks_completed`, `resources_consumed`, and every consumption
  remainder;
- preserves unused reservations when the pause used
  `release_unused = false`;
- re-reserves released local stock and rebuilds demand when the pause used
  `release_unused = true`;
- follows the state precedence above; and
- emits `ResearchResumed` only when the transaction actually leaves `Paused`.

Automatic `NoResearchShip` recovery does not need a command. When an eligible
ship docks, the project becomes `AwaitingMaterials` or `Ready` according to its
material state and emits `ResearchResumed`. If the ship later leaves for an
authoritative higher-priority survey, the project returns to `Paused /
NoResearchShip` without releasing its materials and emits `ResearchPaused`.

#### 7.4 Reassignment to a different facility

`QueueResearch` with a different `facility_id` may reassign any non-Complete
project. Before mutation, validate the entire target result, including capacity
and facility occupancy. On success, one actor transaction:

1. stops active consumption;
2. releases every unconsumed reservation in place at the old facility's
   ordinary buffer or Fuel compartment, regardless of the earlier manual-pause
   release flag;
3. clears the old facility's `active_research_id` and sets
   `resources_reserved` to an empty map;
4. releases a docked Research Ship to Idle at that station and releases its
   persistent dock; an en-route `DockForResearch` ship finishes its current
   TravelPlan leg and then idles at that exact endpoint;
5. sets the project's `station_id` to the new facility and sets that facility's
   `active_research_id`;
6. preserves `resources_required`, `resources_consumed`, all consumption
   remainders, `ticks_completed`, `total_ticks`, and
   `created_server_sequence`;
7. reserves only physical target stock and establishes demand for the remaining
   outstanding amount; and
8. applies the state precedence in section 7.2.

No material teleports between facilities. Released old-facility stock remains
ordinary inventory and may later move through normal Cargo logistics. Research
Station ship work is assigned in phase 9 using the existing station-priority,
project-sequence, StationId, route-cost, and ShipId ordering; the command does
not conjure a ship job outside that phase.

The transaction emits `ResearchReassigned`. It also emits
`ResearchResumed` only if it leaves `Paused` immediately; otherwise docking
emits that event later.

#### 7.5 Pause and facility loss

`PauseResearch { release_unused: false }` changes the project to
`Paused / Manual`, retains unused reservations at the same facility, and emits
`ResearchPaused`.

`PauseResearch { release_unused: true }` releases every unconsumed reserved
unit in place, clears `resources_reserved`, changes the project to
`Paused / Manual`, and emits `ResearchPaused`.

When an authoritative mechanic makes a facility unavailable, it must state
whether the unavailability is temporary or detaching. Demolition is detaching:
it releases unused research material in place for evacuation, clears
`station_id` and `active_research_id`, and leaves the project
`Paused / FacilityUnavailable` until explicit reassignment. This ADR does not
infer facility loss merely from station tier or invent upgrade downtime.

### 8. Schema and executable obligations

P1-02a defines and snapshot-tests primitive tagged reference unions; P1-02b
adds the complete serialized entity DTOs used by replacement deltas. P1-12
implements the shared command envelope and outcomes, and P1-14 exports their
REST/OpenAPI schema. P1-22 exercises every research state/material transition
above. P1-31 adds the complete event union, full-replacement StateDelta,
polling, retention, and resynchronization. P1-32 runs the same schemas and
command corpus through WebSocket, including queue pressure and close code 4009.

Required focused cases include:

- deserializing one valid payload for every event and nested union variant;
- rejecting unknown event types in strict server fixtures while generated
  clients tolerate additive fields under protocol-v1 policy;
- applying full entity replacements, nullable fields, root replacements, and
  every entity deletion;
- proving loss of StateDelta forces resynchronization rather than stale cache
  continuation;
- progress coalescing only by exact key while canonical polling remains
  complete;
- retention-boundary races;
- persistence and invariant `RuntimeError` payloads;
- pause keep/release, same-facility resume, no-ship automatic resume,
  reassignment with old and new stock, insufficient target capacity, occupied
  target, en-route Research Ship cleanup, save/load, and replay; and
- rejecting Tier-2+ Hub research under canonical V1 content.

## Consequences

### Positive

- StateDelta has one simple, generated-model-compatible replacement rule.
- A live client either has a continuous cache or is explicitly forced to
  resynchronize; silent divergence is impossible.
- Canonical history remains complete until its documented bounded eviction.
- Progress messages can still be coalesced without changing gameplay state.
- Research reassignment preserves consumed value without teleporting unused
  materials or expanding Hub research capability.

### Negative

- Full entity replacements are larger than leaf patches.
- Every entity deletion emits both a domain event and a StateDelta tombstone.
- Slow clients are disconnected rather than allowed to continue with a gap.
- The exhaustive event union and generated schemas require deliberate additive
  versioning when new gameplay outcomes are introduced.

## Related documents

- ADR-0003 — Command/Query API with WebSocket Streaming
- ADR-0004 — Game Lifecycle State Machine
- ADR-0006 — Canonical Content/State Hashing
- ADR-0007 — Save Envelope
- ADR-0008 — Accepted-Command Persistence
- GDD 4 — Technology Tree
- GDD 12 — Simulation Foundations
- GDD 13 — Data Models
- GDD 14 — Authored Content
- TDD 00 — System Architecture
- TDD 02 — API Protocol
