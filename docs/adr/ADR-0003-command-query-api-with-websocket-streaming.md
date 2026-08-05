---
status: accepted
owner: Tech Lead
date: 2026-08-04
---

# ADR-0003: Command/Query API with WebSocket Streaming

## Context

The text UI, agent play-tests, and future PixiJS client need synchronous commands, consistent snapshots, and live committed events. The same interface must preserve deterministic command ordering, support lifecycle operations, recover from dropped events, and prevent arbitrary browser pages from mutating a localhost game.

## Decision

We will expose versioned HTTP query/command endpoints and a WebSocket event/command stream. Both transports use one command envelope and submit to the simulation actor; handlers never mutate state directly.

### Endpoints

- `GET /api/v1/status`
- `GET /api/v1/state`
- `GET /api/v1/state/{collection}`
- `GET /api/v1/state/{collection}/{id}`
- `GET /api/v1/content`
- `GET /api/v1/events?since_sequence=N&limit=L&event_type=T`
- `POST /api/v1/command`
- `WS /api/v1/stream`

### Command Envelope and Ordering

```json
{
  "id": "cmd_000123",
  "expected_tick": 420,
  "command": {
    "type": "SetStationPriority",
    "station_id": "refinery_alpha",
    "priority": 80
  }
}
```

- `id` is a mandatory nonempty UTF-8 string and idempotent for the server session.
- `expected_tick` is required but nullable optimistic concurrency. `null` opts out;
  an integer mismatch returns 409 with the current tick.
- The actor assigns `server_sequence` monotonically.
- While Running, gameplay/configuration commands are accepted for the next tick, then revalidated/applied in sequence order; their committed outcome is a later event. Lifecycle commands (`Pause`, `SaveNow`, `NewGame`, `LoadAutosave`) execute as actor transactions at the next safe commit boundary without manufacturing an extra simulation tick.
- While Paused, planning commands apply immediately as serialized actor transactions without advancing the tick.
- Every receipt records command ID, effective boundary/tick, and server sequence; replayable commands additionally persist them in the game command log.

ADR-0008 divides this exhaustive command union into replayable game commands and
actor controls (`NewGame`, `LoadAutosave`, `SaveNow`, `Pause`, `Resume`, and
`AdvanceTicks`). Only the replayable subset enters `GameState.command_log`; controls
use the actor's runtime session receipt ledger and are never executed by command-log
replay. Both subsets share the same strict envelope, structural idempotency comparison,
and monotonic runtime receipt order, so gaps from control receipts are valid in the
persisted replayable sequence. `SaveNow` is a FIFO committed-state barrier and is
acknowledged only after the atomic save succeeds or fails.

### V1 Commands

The shared strict envelope and complete language-neutral payload schema are:

```text
struct CommandEnvelope {
  id: string              // nonempty UTF-8
  expected_tick: u64 | null // required JSON member; null explicitly opts out
  command: Command
}

enum BufferConfiguration { // internally tagged by "kind"
  Input {
    resource: ResourceType,       // Fuel forbidden
    max: u32,
    demand_threshold: u8
  },
  Output {
    resource: ResourceType,       // Fuel forbidden
    max: u32,
    export_threshold: u8
  },
  Fuel {
    demand_threshold: u8,
    export_threshold: u8
  }
}

enum Command { // internally tagged by "type"
  NewGame { scenario_id: ScenarioId }
  LoadAutosave
  SaveNow
  Pause
  Resume
  AdvanceTicks { count: u16 }

  QueueBuildShip { hub_id: StationId, role: ShipRole, tier: u8 }
  QueueBuildStation {
    source_hub_id: StationId,
    body_id: BodyId,
    orbit_ring: u8,
    slot: u8,
    station_type: StationType,
    tier: u8
  }
  QueueUpgrade {
    source_hub_id: StationId,
    station_id: StationId,
    target_tier: u8
  }
  CancelBuildOrder { order_id: BuildOrderId }
  QueueDemolishStation {
    station_id: StationId,
    recovery_hub_id: StationId
  }
  ScrapShip { ship_id: ShipId }
  BeginGateAssembly { fabricator_ship_id: ShipId }

  SetStationPriority { station_id: StationId, priority: u8 }
  ConfigureBuffer {
    station_id: StationId,
    configuration: BufferConfiguration
  }
  SetProductionRecipe {
    station_id: StationId,
    slot_index: u8,
    recipe_id: RecipeId | null
  }
  SetMiningTarget {
    station_id: StationId,
    slot_index: u8,
    resource: ResourceType
  }
  QueueResearch { facility_id: StationId, tech_id: TechId }
  PauseResearch { tech_id: TechId, release_unused: bool }
  QueueSurvey { body_id: BodyId, target_depth: u8, priority: u8 }
  CancelSurveyOrder { order_id: SurveyOrderId }
}
```

`LoadAutosave`, `SaveNow`, `Pause`, and `Resume` serialize as objects containing
only their `type` discriminator. Every object in the envelope, command, and nested
buffer union rejects unknown fields. The schema bounds `AdvanceTicks.count` to
`1..=1,000`, priorities and percentages to `0..=100`, and tiers/depths/slot indices
to their content-validated ranges.

An integer `expected_tick` is compared with the committed tick at actor dequeue. If no
`GameState` exists, it receives the pre-acceptance 409 `ExpectedTickUnavailable` with
`current_tick: null`; clients starting/loading from Unloaded must send null. A
same-process NewGame/LoadAutosave from a stable game compares against the retained
game's current tick before entering Loading.

For an Input configuration, `export_threshold` remains the required serialized
inactive value zero; for Output, `demand_threshold` remains zero. A Fuel
configuration changes both active thresholds subject to demand less than or equal to
export, while Fuel `resource` and `max` are immutable and therefore absent from that
variant. General `max = 0` is valid only when current stock and every applicable
inbound/outbound/production/research reservation are zero; it retains an explicit
zero-capacity buffer entry rather than introducing deletion/default behavior.
`SetProductionRecipe.recipe_id = null` clears the selected slot under GDD 12's exact
state rule: Idle/AwaitingInputs clears directly; Processing atomically releases the
complete reserved-input map and resets progress before clearing; OutputBlocked rejects
until its completed batch transfers.

`ReplayableGameCommand` is this exact union excluding `NewGame`, `LoadAutosave`,
`SaveNow`, `Pause`, `Resume`, and `AdvanceTicks`. ADR-0008 owns the persistence and
receipt consequences of that split. There is no duplicate `PlaceStation` command:
placement always creates a BuildOrder.

### Response

Accepted envelopes return this exact acknowledgement shape (`accepted` is the
literal `true`):

```text
struct CommandAcknowledgement {
  protocol_version: string
  id: string
  accepted: true
  status: Accepted | Applied | Rejected | Failed
  effective_tick: u64 | null
  resulting_tick: u64 | null
  server_sequence: u64
  game_state: GameLifecycle
  result: CommandResult | null
  error: CommandRejection | null
}
```

```json
{
  "protocol_version": "v1",
  "id": "cmd_000123",
  "accepted": true,
  "status": "Accepted",
  "effective_tick": 421,
  "resulting_tick": null,
  "server_sequence": 812,
  "game_state": "Running",
  "result": null,
  "error": null
}
```

`202` carries `Accepted` for a command queued at a future running tick. `200`
carries `Applied` for a command committed during the request. If an accepted
command reaches a domain rejection during the request, `409` or `422` carries a
`Rejected` acknowledgement according to the error class; an accepted control
whose infrastructure operation fails returns `500` with `Failed`. A command
rejected before actor acceptance instead returns the ordinary typed API error
from TDD 02 and no acknowledgement or committed outcome event. The runtime
ledger still records that pre-acceptance rejection so an exact retry cannot evade
`expected_tick`, lifecycle, or idempotency validation.

`result` is non-null only for `Applied`, including the explicit tagged
`{"type":"None"}` result. `error` is non-null only for `Rejected` or `Failed`.
`resulting_tick` is derived only from
`{"type":"AdvanceTicksCompleted","resulting_tick":N}` and otherwise is null.
`effective_tick` may be null only for an accepted state-replacement control begun
while Unloaded; replayable commands always carry an integer. `game_state` is the
actor's current lifecycle when the response is serialized and is advisory rather
than part of the persisted command result. Clients fetch a snapshot when needed;
responses never serialize the complete `GameState`.

An identical retry returns the receipt's current recorded status and exact stored
result or error without starting work or emitting another event. Thus a prior
`202 Accepted` may become `200 Applied` on a later retry. A changed envelope with
the same ID always returns `409 IdempotencyConflict`.

### Events and Resynchronization

Every event has a session-monotonic `event_sequence`, nullable committed `tick`, and a tagged payload. `tick` is null only for accepted control/runtime outcomes while Unloaded. Event kinds include lifecycle changes, command results, state deltas, builds, research, arrivals, survey completion, bottlenecks, salvage, and Gate completion.

The server retains the newest 50,000 session events; its runtime ring/counter survives
same-process NewGame/LoadAutosave. A loaded state provides only a persisted lower
bound, which is rebased before a complete-replacement StateDelta. Fresh-process load
seeds the empty ring's next sequence and oldest-available boundary. ADR-0012's exact
rule permits `since_sequence = oldest_available - 1`, returns 410 only for an older
cursor, and rejects a cursor beyond the runtime latest value with 400. The client then
fetches `GET /state` and resumes from the snapshot's runtime latest event sequence.

A slow WebSocket client never changes tick rate. Its bounded queue may coalesce only
the named cumulative progress events in ADR-0012; `StateDelta`, command outcomes,
entity changes, and other durable events are never coalesced or dropped. If required
continuity cannot be queued, the server sends `ResyncRequired` when possible and
terminates the stream. Canonical retained history remains complete until its bounded
retention boundary.

### Local Security

- Bind to loopback only by default.
- Generate a random session bearer token at startup and write `{port, token}` to the user-data `connection.json` with owner-only permissions.
- Require the token for REST and WS. CLI clients may use the upgrade `Authorization` header; browser clients pass it in the `bearer.<token>` WebSocket subprotocol because the browser WebSocket API cannot set arbitrary headers.
- Validate browser `Origin` against the packaged PixiJS/Tauri origins; CLI clients omit Origin and authenticate by token.
- Disable permissive CORS by default. `--insecure-no-auth` is an explicit development-only flag and prints a warning.

## Consequences

### Positive

- All clients share one stable, replayable command system.
- Lost/dropped deltas have a defined recovery path.
- Localhost is not treated as an authentication boundary.
- Full snapshots are decoupled from lightweight command acknowledgements.

### Negative

- Two transports and session discovery require integration tests.
- Event retention consumes bounded memory/disk space.
- Browser UI packaging must declare an Origin allowlist.

### Mitigations

- Generate JSON Schema/OpenAPI from shared Rust types.
- Run the same command conformance corpus through REST and WS.
- Test idempotency, expected-tick conflicts, reconnection at/behind retention, slow consumers, and unauthorized browser origins.

## Related ADRs

- ADR-0001 — Rust Simulation Engine with HTTP/WS API
- ADR-0002 — Deterministic Tick Simulation
- ADR-0004 — Game Lifecycle State Machine
