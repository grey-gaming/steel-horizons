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
- `GET /api/v1/events?since_sequence=N`
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

- `id` is mandatory and idempotent for the server session.
- `expected_tick` is optional optimistic concurrency. A mismatch returns 409 with current tick.
- The actor assigns `server_sequence` monotonically.
- While Running, gameplay/configuration commands are accepted for the next tick, then revalidated/applied in sequence order; their committed outcome is a later event. Lifecycle commands (`Pause`, `SaveNow`, `NewGame`, `LoadAutosave`) execute as actor transactions at the next safe commit boundary without manufacturing an extra simulation tick.
- While Paused, planning commands apply immediately as serialized actor transactions without advancing the tick.
- The accepted result records command ID, effective tick, and server sequence for replay.

### V1 Commands

Lifecycle and execution:

- `NewGame { scenario_id }`
- `LoadAutosave`
- `SaveNow`
- `Pause`
- `Resume`
- `AdvanceTicks { count }` — Paused agent/test sessions only

Construction and recovery:

- `QueueBuildShip { hub_id, role, tier }`
- `QueueBuildStation { source_hub_id, body_id, orbit_ring, slot, station_type, tier }`
- `QueueUpgrade { source_hub_id, station_id, target_tier }`
- `CancelBuildOrder { order_id }`
- `QueueDemolishStation { station_id, recovery_hub_id }`
- `ScrapShip { ship_id }` — valid only while docked at a Hub; that Hub is the recovery destination
- `BeginGateAssembly { fabricator_ship_id }` — validates and commits an idle Tier-4 Fabricator to the fixed Gate site

Configuration and progression:

- `SetStationPriority { station_id, priority }`
- `ConfigureBuffer { station_id, direction, resource, max, threshold }`
- `SetProductionRecipe { station_id, slot, recipe_id }`
- `SetMiningTarget { station_id, slot, resource }`
- `QueueResearch { facility_id, tech_id }`
- `PauseResearch { tech_id, release_unused }`
- `QueueSurvey { body_id, target_depth, priority }`
- `CancelSurveyOrder { order_id }`

There is no duplicate `PlaceStation` command: placement always creates a BuildOrder.

### Response

```json
{
  "id": "cmd_000123",
  "accepted": true,
  "effective_tick": 421,
  "resulting_tick": null,
  "server_sequence": 812,
  "game_state": "Running"
}
```

`resulting_tick` is present for synchronously completed operations such as paused `AdvanceTicks`; otherwise it is null. Clients fetch a snapshot when needed; command responses do not serialize the entire `GameState` by default.

### Events and Resynchronization

Every event has `event_sequence`, committed `tick`, and a tagged payload. Event kinds include lifecycle changes, command results, state deltas, builds, research, arrivals, survey completion, bottlenecks, salvage, and Gate completion.

The server retains the newest 50,000 events; tick coverage is workload-dependent. If `since_sequence` predates retention, HTTP returns 410 `ResyncRequired`; WS sends the same signal. The client then fetches `GET /state` and resumes from the snapshot's latest event sequence.

A slow WebSocket client never changes tick rate. Its bounded queue may coalesce position deltas or terminate with `ResyncRequired`; durable gameplay events remain available through retained history until the retention boundary.

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
