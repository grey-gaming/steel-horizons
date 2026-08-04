---
status: accepted
owner: Tech Lead
date: 2026-08-04
---

# ADR-0012: Complete Command/Event Wire Contracts

## Context

The existing ADRs, GDDs, and TDDs define command envelopes, an event polling/streaming surface, and StateDelta examples, but several specification gaps remain before dependent production increments (P1-22 research, P1-31 complete event surface):

1. No single document exhaustively lists every tagged event type with its JSON payload shape.
2. StateDelta replacement/removal semantics are unspecified — clients need to know whether fields are patched or replaced, and how entity removal is represented.
3. The distinction between durable events (must survive retention) and coalescible events (may be dropped by slow queues) is not defined.
4. Research resume/reassignment across facilities (Hub ↔ Research Station) is not fully specified.

## Decision

### 1. Exhaustive Tagged Event Payloads

Every committed transaction emits zero or more events. Events are serialized as tagged JSON with `type`, `event_sequence`, and `tick` in every payload. All `tick` values reference the tick at which the event was committed.

```json
// Every event includes these common fields
{
  "event_sequence": 813,
  "tick": 421,
  // type-specific fields follow
}
```

#### LifecycleChange

Emitted when `GameLifecycle` transitions. Durable.

```json
{
  "type": "LifecycleChange",
  "event_sequence": 813,
  "tick": 421,
  "from": "Paused",
  "to": "Running",
  "reason": null
}
```

`reason` is present only when the transition is automatic (e.g., `"AdvanceTicksComplete"` for Advancing→Paused, `"GateActivation"` for Running→Won).

#### CommandApplied

Emitted when a previously accepted command passes effective-tick validation and is applied during a tick transaction. Durable.

```json
{
  "type": "CommandApplied",
  "event_sequence": 814,
  "tick": 421,
  "id": "cmd_000123",
  "server_sequence": 813,
  "effective_tick": 421
}
```

#### CommandRejected

Emitted when a previously accepted command fails validation during the tick transaction. Durable.

```json
{
  "type": "CommandRejected",
  "event_sequence": 815,
  "tick": 421,
  "id": "cmd_000123",
  "server_sequence": 814,
  "effective_tick": 421,
  "reason": "InsufficientFuel",
  "details": { "required": 10, "available": 5 }
}
```

`reason` is a machine-readable string code. `details` is an optional JSON object with error-specific fields.

#### EntityCreated

Emitted when a new simulation entity is created. Durable.

```json
{
  "type": "EntityCreated",
  "event_sequence": 816,
  "tick": 421,
  "entity_type": "ship",
  "id": "ship_002",
  "entity": { /* full serialized Ship */ }
}
```

Entity types: `ship`, `station`, `build_order`, `survey_order`, `salvage_cache`, `gate_build`, `research_project`.

The full entity is included so the client can add it to its local state without fetching a full snapshot.

#### EntityRemoved

Emitted when a simulation entity is permanently removed. Durable.

```json
{
  "type": "EntityRemoved",
  "event_sequence": 817,
  "tick": 421,
  "entity_type": "ship",
  "id": "ship_002",
  "reason": "Scrapped"
}
```

`reason` is one of `"Scrapped"`, `"Demolished"`, `"BuildCancelled"`, `"ResearchCompacted"`.

Entity types that support removal: `ship`, `station`, `build_order`, `survey_order`, `salvage_cache`, `gate_build`, `research_project`.

#### StateDelta

Emitted on every tick for entities whose state changed during the tick. Coalescible.

```json
{
  "type": "StateDelta",
  "event_sequence": 818,
  "tick": 422,
  "changed_entities": [
    {
      "entity_type": "ship",
      "id": "ship_001",
      "changes": {
        "fuel": 95,
        "travel_plan": {
          "active_segment": 1,
          "remaining_distance_milli": 238000
        }
      }
    },
    {
      "entity_type": "station",
      "id": "mine_001",
      "changes": {
        "input_buffers": [
          { "resource": "MetalOre", "current": 12, "max": 50 }
        ],
        "mining_targets": [
          { "resource": "MetalOre", "rate_remainder": { "value": 400, "denominator": 1000 } }
        ]
      }
    }
  ],
  "removed_entities": []
}
```

Semantics for `changes`:
- **Scalar fields** (fuel, tick, lifecycle, etc.): the new value replaces the old. The field is present only when it changed.
- **Map fields** (input_buffers, output_buffers, production_slots, mining_targets, deposits, installed_components, cargo, etc.): each element is a complete replacement for that keyed entry. Elements are keyed by `resource` (for buffers/targets/deposits/components) or `slot_index` (for production slots). Missing entries are unchanged. There is no partial-field update within a buffer/target entry — the entire entry is replaced.
- **Nested object fields** (travel_plan, job, gate_build): the changes object contains only the leaf fields that changed, using the same scalar-replacement rule. A field is absent when unchanged.
- **Array fields** (docked_ship_ids, holding_ship_ids, ship_build_queue): the entire array is replaced when any element changes.

`removed_entities` is an array of `{entity_type, id}` pairs. Removed entities are conveyed here rather than as a separate `EntityRemoved` event only when the removal is part of a coalescible tick (e.g., a build order transitions to `Complete` and is removed from the build_orders map on the same tick). Removals that are intrinsically durable events (scrapping, demolition, build cancellation with component return) MUST also emit `EntityRemoved`.

#### BuildComplete

Emitted when a BuildOrder transitions to `Complete`. Durable.

```json
{
  "type": "BuildComplete",
  "event_sequence": 819,
  "tick": 422,
  "order_id": "bo_005",
  "target": {
    "type": "Station",
    "station_id": "mine_001",
    "station_type": "Mining",
    "tier": 1
  },
  "source_hub_id": "hub_haven"
}
```

`target` is a tagged union matching `BuildTarget` semantics.

#### ResearchComplete

Emitted when a ResearchProject transitions to `Complete`. Durable.

```json
{
  "type": "ResearchComplete",
  "event_sequence": 820,
  "tick": 422,
  "tech_id": "sensor_systems",
  "facility_id": "hub_haven"
}
```

#### ResearchProgress

Emitted on ticks where research consumption advances. Coalescible.

```json
{
  "type": "ResearchProgress",
  "event_sequence": 821,
  "tick": 422,
  "tech_id": "sensor_systems",
  "ticks_completed": 42,
  "resources_consumed": { "Metals": 2, "SiliconWafers": 3 }
}
```

#### SurveyComplete

Emitted when a survey reaches a depth milestone or completes. Durable.

```json
{
  "type": "SurveyComplete",
  "event_sequence": 822,
  "tick": 422,
  "order_id": "so_001",
  "body_id": "planet_pyre",
  "new_depth": 1,
  "total_work": 300
}
```

#### ArrivalEvent

Emitted when a ship arrives at a destination and transfers cargo or completes a job milestone. Durable.

```json
{
  "type": "ArrivalEvent",
  "event_sequence": 823,
  "tick": 422,
  "ship_id": "ship_001",
  "destination": { "type": "Station", "station_id": "refinery_001" },
  "job_type": "Transport",
  "transferred": { "resource": "MetalOre", "amount": 10 }
}
```

`destination` is a `DestinationRef` tagged union. `transferred` is present for cargo delivery events and omitted for Refuel/Build/Rescue arrivals that do not transfer cargo.

#### GatePhaseProgress

Emitted when Gate assembly advances a phase or completes. Durable.

```json
{
  "type": "GatePhaseProgress",
  "event_sequence": 824,
  "tick": 422,
  "phase": "FrameAssembly",
  "progress_work": 42,
  "phase_complete": false
}
```

`phase_complete` is `true` when the phase work reaches its threshold and the phase transitions.

#### BottleneckWarning / BottleneckCleared

Emitted when a station/resource pair crosses the bottleneck threshold. Durable.

```json
{
  "type": "BottleneckWarning",
  "event_sequence": 825,
  "tick": 422,
  "station_id": "refinery_001",
  "resource": "MetalOre",
  "deliveries_deficit_ticks": 50
}

{
  "type": "BottleneckCleared",
  "event_sequence": 826,
  "tick": 422,
  "station_id": "refinery_001",
  "resource": "MetalOre"
}
```

#### SalvageCreated

Emitted when a salvage cache is created (build cancellation, demolition evacuation, scrapping overflow). Durable.

```json
{
  "type": "SalvageCreated",
  "event_sequence": 827,
  "tick": 422,
  "salvage_id": "salvage_001",
  "position": { "lane_id": "Habitable", "radius_units": 1200, "angle_milli": 0 },
  "origin": { "type": "BuildCancellation", "order_id": "bo_003" }
}
```

`origin` is a tagged union: `BuildCancellation`, `DemolitionEvacuation`, `ScrapOverflow`.

#### ResearchResumed

Emitted when a paused research project is resumed or reassigned to a different facility. Durable.

```json
{
  "type": "ResearchResumed",
  "event_sequence": 828,
  "tick": 422,
  "tech_id": "sensor_systems",
  "facility_id": "research_station_001",
  "ticks_completed": 42,
  "resources_consumed": { "Metals": 2 }
}
```

---

### 2. StateDelta Replacement / Removal Semantics

#### 2.1 Replacement Rules

StateDelta uses **replacement** semantics, not recursive patching:

| Field type | Rule |
|---|---|
| Scalar (`fuel`, `tick`, `priority`, etc.) | Field value replaces the old value atomically |
| Map (`input_buffers`, `production_slots`, `mining_targets`, `deposits`, `installed_components`, `cargo`) | Each entry in the array is a complete replacement for that key. Entries are keyed by `resource` or `slot_index`. Missing entries are unchanged. The entry replaces the entire buffer/slot/target, not individual sub-fields |
| Nested object (`travel_plan`, `job`, `gate_build`) | The changes object contains only the leaf fields that changed, using scalar-replacement rules for each leaf. The unchanged leaf fields are absent |
| Array (`docked_ship_ids`, `holding_ship_ids`, `ship_build_queue`) | The entire array is replaced when any element changes. Present → replaces; absent → unchanged |
| Entity-typed map (`stations`, `ships`, `build_orders`, etc.) | Not included in StateDelta. Entity creation/removal uses `EntityCreated`/`EntityRemoved`/`removed_entities` instead |

#### 2.2 Entity Removal in StateDelta

`removed_entities` entries are `{entity_type, id}` pairs. Valid entity types: `ship`, `station`, `build_order`, `survey_order`, `salvage_cache`, `research_project`.

Removals that are also intrinsically durable (scrapping, demolition, build cancellation with component return) MUST additionally emit a separate `EntityRemoved` event. Removals that are ordinary lifecycle transitions (build order reaches Complete and is removed from the map) appear only in `removed_entities`.

#### 2.3 Client Application

A client maintaining a local cache applies deltas as follows:

1. For each `changed_entities` entry: locate the entity by `entity_type` + `id`, then for each field in `changes`, replace the stored value entirely with the new value.
2. For each `removed_entities` entry: delete the entity from the local cache.
3. For each durable event (`EntityCreated`, `EntityRemoved`): add/delete the entity regardless of any concurrent delta.

Deltas may be coalesced by dropping intermediate ones and keeping only the latest per tick. Durable events must never be coalesced.

---

### 3. Durable vs Coalescible Events

#### 3.1 Durable Events (must survive retention ring)

These events represent committed gameplay outcomes and must survive the 50,000-event retention ring. They may not be coalesced or dropped by slow-client queues.

| Event | Rationale |
|---|---|
| `LifecycleChange` | Game state machine transitions |
| `CommandApplied` | Proof of command execution |
| `CommandRejected` | Proof of command rejection |
| `EntityCreated` | Entity existence |
| `EntityRemoved` | Entity deletion (scrapped, demolished, cancelled) |
| `BuildComplete` | Build order completion |
| `ResearchComplete` | Technology unlock |
| `SurveyComplete` | Survey depth milestone |
| `ArrivalEvent` | Cargo transfer, job completion, refuel delivery |
| `GatePhaseProgress` | Gate assembly progress (phase boundaries) |
| `BottleneckWarning` / `BottleneckCleared` | Bottleneck state transitions |
| `SalvageCreated` | Salvage cache creation |
| `ResearchResumed` | Research resume/reassignment |

#### 3.2 Coalescible Events (may be dropped by slow queues)

These events represent transient progress updates. A slow WebSocket client may drop intermediate values and receive only the latest per tick.

| Event | Rationale |
|---|---|
| `StateDelta` | Per-tick state changes; only the latest tick matters |
| `ResearchProgress` | Intermediate consumption ticks |
| `SurveyProgress` | Intermediate scan ticks |
| `ConstructionProgress` | Intermediate build work ticks |

#### 3.3 Coalescence Rules

- A per-client queue that overflows (2048 messages) sends `ResyncRequired` rather than dropping durable events.
- Coalescible events for the same tick may be replaced by a later event for that tick in the same queue.
- Durable events for a tick must not be dropped; if the queue is full, the connection closes with 4009.
- After resynchronization, the client fetches a fresh snapshot and replays only durable events from `since_sequence`.

---

### 4. Research Resume / Reassignment Across Facilities

#### 4.1 Definitions

A ResearchProject may be hosted at either:
- A **Hub** with `built_in_research_max_tier` ≥ the tech tier (Hub Haven has `built_in_research_max_tier = 1`). Hub research does not require a docked Research Ship.
- A **Research Station** with a docked Research Ship. Any Research Station can run any available tech tier.

#### 4.2 Resume

When a Paused research project is resumed (via `QueueResearch` on the same tech ID with the same or different facility):

1. If the new facility differs from the current `station_id`, the project is reassigned (see §4.3).
2. The project transitions from `Paused` to `Ready`, then `Active` as materials become available.
3. Progress (`ticks_completed`, `resources_consumed`, `consumption_remainders`) is preserved.
4. Reserved resources remain reserved for the project. If `release_unused` was set during the pause, resources were released in place and must be re-reserved.
5. The `ResearchResumed` event is emitted with the current `ticks_completed` and `resources_consumed`.

#### 4.3 Reassignment

Reassignment occurs when `QueueResearch` is called with a `facility_id` that differs from the project's current `station_id`:

1. If the current project state is `Active`, it is first paused automatically (resources released in place to the old facility).
2. If the current project has docked Research Ship reservations, those are released. The Research Ship becomes Idle.
3. The project's `station_id` is set to the new facility.
4. If the new facility is a Research Station, a `DockForResearch` job is created for an eligible Research Ship. The project state becomes `NoResearchShip` until the ship docks.
5. If the new facility is a Hub with sufficient `built_in_research_max_tier`, no ship job is needed; the project transitions to `Ready` then `Active`.
6. The `ResearchResumed` event is emitted.
7. Progress and consumed-resource credit are preserved. The resources_required and resources_reserved maps remain unchanged except for any release-on-pause side effects.

#### 4.4 Pause with Release

`PauseResearch { tech_id, release_unused: true }`:
- Reserved resources that have not yet been consumed are released to the facility's general buffers (or Fuel compartment for Fuel inputs).
- The project enters `Paused` with `pause_reason = Manual`.
- If the project was `Active`, consumption stops. Completed ticks remain credited.

`PauseResearch { tech_id, release_unused: false }`:
- Reserved resources remain reserved.
- The project enters `Paused` with `pause_reason = Manual`.
- If the project was `Active`, consumption stops but reserved materials are not released.

#### 4.5 Automatic Pause Reasons

| `pause_reason` | Trigger | Resume behavior |
|---|---|---|
| `Manual` | Explicit `PauseResearch` command | Requires explicit `QueueResearch` |
| `NoResearchShip` | Research Station project with no docked Research Ship | Automatic when a Research Ship docks |
| `FacilityUnavailable` | Facility destroyed/demolished or upgraded while Active | Requires explicit `QueueResearch` with reassignment |

#### 4.6 Hub Built-In Research

Hub Haven (and future Hub tiers) have `built_in_research_max_tier` which limits the maximum tech tier they can research without a Research Ship:

- Hub T1: `built_in_research_max_tier = 1` (can run Tier-1 techs only)
- Hub T2: `built_in_research_max_tier = 2`
- Hub T3: `built_in_research_max_tier = 3`
- Hub T4: `built_in_research_max_tier = 4` (can run any tech)

`QueueResearch` that targets a Hub for a tech whose tier exceeds `built_in_research_max_tier` is rejected. The command must target a Research Station instead.

#### 4.7 Event Coverage

Every pause, resume, or reassignment generates at least one durable event:
- `PauseResearch` → `CommandApplied` (durable, includes the tech_id and release flag in the command record)
- Resume/reassign → `ResearchResumed` (durable)
- Research Station docks a Research Ship for an active project → `ResearchResumed` (durable, NoResearchShip→Ready transition)

---

## Consequences

### Positive

- Clients can implement a complete state cache from durable events + StateDelta without fetching full snapshots on every tick.
- Coalescence rules prevent slow clients from unbounded queuing while preserving every gameplay-significant event.
- Research reassignment has a well-defined event trail and no lost progress.
- Every event shape is now documented alongside the existing command catalog.

### Negative

- The `StateDelta` `changes` object for nested fields (travel_plan, job) requires recursive leaf-field replacement rather than simple field overwrite — slightly more complex for clients.
- Research reassignment creates an extra `ResearchResumed` event that earlier drafts did not define.

### Mitigations

- Document the recursive leaf-field rule with examples.
- The `ResearchResumed` event is structurally simple and maps directly to the reassignment code path.

## Related Documents

- ADR-0003 — Command/Query API with WebSocket Streaming (command catalog, event polling)
- ADR-0004 — Game Lifecycle State Machine (lifecycle transitions)
- ADR-0006 — Canonical Content/State Hashing (hash scope)
- ADR-0007 — Save Envelope Format (event sequence serialization)
- ADR-0008 — Accepted-Command Persistence (command record lifecycle)
- GDD 12 — Simulation Foundations (tick phases, research, persistence)
- GDD 13 — Data Models (serialized state shapes, ResearchProject, ResearchState)
- TDD 00 — System Architecture (actor transactions, event broadcast)
- TDD 02 — API Protocol (Hello/Subscribe/Command, StateDelta example, rate limits)
