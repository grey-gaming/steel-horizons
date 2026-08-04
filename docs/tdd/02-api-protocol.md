---
status: Approved
owner: Tech Lead
date: 2026-08-04
---

# API Protocol Design

## Connection and Authentication

The engine binds to loopback and writes owner-only discovery data:

```json
{
  "protocol": "v1",
  "host": "127.0.0.1",
  "port": 4881,
  "token": "base64url-random-session-token",
  "pid": 12345
}
```

Clients read the OS-specific `connection.json` and send `Authorization: Bearer <token>` on HTTP. CLI WS clients use the same upgrade header. Browser WS clients request protocols `steel-horizons.v1` and `bearer.<token>` because browser JavaScript cannot set arbitrary upgrade headers; the server selects only `steel-horizons.v1`. Packaged browser clients must also present an allowed Origin. Missing/invalid credentials return 401/403 before command parsing.

## JSON Conventions

- `snake_case` field names and PascalCase tagged variant names
- UTF-8 JSON
- Integers for all simulation values; no float positions/rates
- IDs are opaque strings
- Unknown fields are rejected on commands and ignored only on forward-compatible event payloads
- Every response includes `protocol_version`

## Queries

### `GET /api/v1/status`

Always available after server Ready:

```json
{
  "protocol_version": "v1",
  "server": "ready",
  "game_state": "Paused",
  "tick": 420,
  "latest_event_sequence": 812,
  "schema_version": 1,
  "content_version": "v1"
}
```

### `GET /api/v1/state`

Returns an immutable committed `GameSnapshot`. Unloaded returns 503. Loading returns 503 plus progress in the error details.

### Collection Queries

- `GET /api/v1/state/ships`
- `GET /api/v1/state/ships/{id}`
- `GET /api/v1/state/stations`
- `GET /api/v1/state/stations/{id}`
- `GET /api/v1/state/bodies`
- `GET /api/v1/state/bodies/{id}`
- `GET /api/v1/state/research`
- `GET /api/v1/state/build-orders`
- `GET /api/v1/content`

Unknown IDs return 404. Content returns immutable recipe, technology, ship, and station definitions for agent reasoning.

## Commands

### `POST /api/v1/command`

```json
{
  "id": "cmd_000123",
  "expected_tick": 420,
  "command": {
    "type": "QueueBuildStation",
    "source_hub_id": "hub_haven",
    "body_id": "planet_haven",
    "orbit_ring": 0,
    "slot": 1,
    "station_type": "Mining",
    "tier": 1
  }
}
```

Success is 202 when accepted into a future running-tick queue and 200 when committed immediately while Paused. A 202 command receives a later `CommandApplied` or `CommandRejected` event after effective-tick validation in server-sequence order:

```json
{
  "protocol_version": "v1",
  "id": "cmd_000123",
  "accepted": true,
  "effective_tick": 421,
  "resulting_tick": null,
  "server_sequence": 813,
  "game_state": "Running"
}
```

`resulting_tick` is populated for synchronously completed operations such as `AdvanceTicks` and null for a command merely queued at a future tick. Reusing the same ID and identical payload returns the original/current recorded result. Reusing it with a different payload returns 409 `IdempotencyConflict`.

The exhaustive V1 command list is owned by ADR-0003 and generated into OpenAPI/JSON Schema. REST and WS deserialize the same Rust `CommandEnvelope`.

## Events

### Polling

`GET /api/v1/events?since_sequence=812&limit=1000` returns committed events with sequence greater than 812. If 812 predates retention, return 410:

```json
{
  "error": {
    "code": "ResyncRequired",
    "message": "Requested event sequence is no longer retained",
    "details": { "oldest_available": 950, "latest": 1200 }
  }
}
```

### WebSocket `/api/v1/stream`

After authentication, the server sends:

```json
{
  "type": "Hello",
  "protocol_version": "v1",
  "game_state": "Paused",
  "tick": 420,
  "latest_event_sequence": 812
}
```

The client then sends one subscription:

```json
{
  "type": "Subscribe",
  "since_sequence": 812,
  "event_types": ["StateDelta", "BuildComplete", "ResearchComplete"]
}
```

Commands use the same envelope wrapped as `{"type":"Command","envelope":...}`.

Example integer-only delta:

```json
{
  "type": "StateDelta",
  "event_sequence": 813,
  "tick": 421,
  "changed_entities": [{
    "entity_type": "ship",
    "id": "ship_003",
    "changes": {
      "travel_plan": { "active_segment": 1, "remaining_distance_milli": 238000 },
      "fuel": 95
    }
  }]
}
```

A queue overflow sends `ResyncRequired` when possible and closes with application code 4009. It never slows simulation ticks.

## Errors

```json
{
  "protocol_version": "v1",
  "error": {
    "code": "InvalidLifecycle",
    "message": "AdvanceTicks requires Paused",
    "details": { "game_state": "Running", "allowed": ["Paused"] }
  }
}
```

| HTTP | Meaning |
|-----:|---------|
| 400 | Malformed/invalid parameters |
| 401 | Missing/invalid token |
| 403 | Disallowed Origin or operation |
| 404 | Entity/endpoint not found |
| 409 | Lifecycle, expected-tick, idempotency, or state conflict |
| 410 | Event history requires resynchronization |
| 422 | Well-formed command violates content/game rule |
| 503 | Game unavailable/unloaded/loading |

## Versioning

The URL major version changes only for breaking protocol changes. Additive fields/events remain within v1. `schema_version` governs saves; `content_version` governs authored data. Compatibility tests cover supported combinations.

## Backpressure and Limits

- Request body: 1 MiB maximum
- `AdvanceTicks`: 1,000 ticks per command (long runs use repeated acknowledged batches)
- Event poll: 1,000 events per response
- WS outbound queue: 2,048 messages per client
- Event retention: newest 50,000 events; tick coverage is workload-dependent
- REST rate limit: 100 commands/sec per token, burst 200

These limits protect the local process without affecting ordinary play or batch tests.

## Related ADRs

- ADR-0003 — Command/Query API with WebSocket Streaming
- ADR-0004 — Game Lifecycle State Machine
