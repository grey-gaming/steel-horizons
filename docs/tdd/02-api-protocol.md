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
  "content_version": "v1",
  "loading": null
}
```

While Unloaded, `game_state` is the literal `"Unloaded"`; `tick`,
`schema_version`, and `content_version` are null. `latest_event_sequence` is always
the trusted runtime next sequence minus one and remains available without a game.
While Loading, `game_state` is `"Loading"`, those same game fields are null, and
`loading` is this required object:

```text
LoadingStatus {
  operation: NewGame | LoadAutosave | StartupAutoload
  stage: ValidatingContent | ConstructingScenario | ReadingSave |
         VerifyingEnvelope | Migrating | ValidatingState | Publishing
}
```

`loading` is null in every other lifecycle. Stages are monotonic for one operation;
inapplicable stages are skipped. Status is advisory runtime progress and is not
serialized into `GameState`.

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

An accepted envelope uses ADR-0003's exact `CommandAcknowledgement` shape. `202`
means it is queued at a future running tick; `200` means it committed during the
request. A queued command receives a later `CommandApplied` or `CommandRejected`
event after effective-tick validation in server-sequence order:

```json
{
  "protocol_version": "v1",
  "id": "cmd_000123",
  "accepted": true,
  "status": "Accepted",
  "effective_tick": 421,
  "resulting_tick": null,
  "server_sequence": 813,
  "game_state": "Running",
  "result": null,
  "error": null
}
```

`status` is exactly `Accepted`, `Applied`, `Rejected`, or `Failed`; internal
`Processing` is never exposed. `Applied` has a non-null typed `result` (including
`{"type":"None"}`) and null `error`. `Rejected`/`Failed` have null result and a
non-null typed error. `resulting_tick` is derived only from an
`AdvanceTicksCompleted` result. An accepted immediate rejection returns this
acknowledgement with 409/422; an accepted infrastructure failure returns it with
500. A rejection before actor acceptance returns the ordinary typed error shape
instead. Exact retries return the current recorded receipt without another effect
or event, while a changed envelope returns 409 `IdempotencyConflict`.

Command `id` is nonempty UTF-8. With no loaded game, an integer `expected_tick`
returns pre-acceptance 409 `ExpectedTickUnavailable` with `current_tick: null`; use
null for NewGame/LoadAutosave from Unloaded.

`expected_tick` is a required member; `null` explicitly opts out of the check and
omission is malformed. Structural equality includes its nullable value and the complete
tagged command. ADR-0008 classifies actor controls separately from replayable game
commands, but both use this envelope and the runtime session receipt ledger. Control
commands never enter deterministic command-log replay; `SaveNow` is acknowledged only
after its atomic persistence operation succeeds or fails.

The exhaustive V1 command list is owned by ADR-0003 and generated into OpenAPI/JSON Schema. REST and WS deserialize the same Rust `CommandEnvelope`.

## Events

### Polling

`GET /api/v1/events?since_sequence=812&limit=1000&event_type=StateDelta` is available in every game
lifecycle and returns committed session events with sequence greater than 812. The
runtime ring/counter survives NewGame/LoadAutosave. Let `oldest_available` be the
first retained sequence, or the next sequence when the ring is empty. A cursor equal
to `oldest_available - 1` is valid; return 410 only when
`since_sequence + 1 < oldest_available`. A cursor greater than `latest` returns 400
`InvalidEventCursor`. For an older cursor, return:

```json
{
  "protocol_version": "v1",
  "error": {
    "code": "ResyncRequired",
    "message": "Requested event sequence is no longer retained",
    "details": {
      "oldest_available_sequence": 950,
      "latest_event_sequence": 1200
    }
  }
}
```

For example, a fresh process loaded with next sequence 814 reports `latest = 813`
and `oldest_available = 814`: polling from 813 returns an empty page, while polling
from 812 returns 410. This is the same boundary used by WebSocket subscription.

`since_sequence` is required. `limit` defaults to 1,000 and is bounded to
`1..=1,000`. A repeated `event_type` parameter is the optional unique discriminator
filter; omit it for all events. Unknown, empty, or duplicate filter values return 400.
The exact successful response is:

```json
{
  "protocol_version": "v1",
  "events": [],
  "next_since_sequence": 813,
  "oldest_available_sequence": 814,
  "latest_event_sequence": 813,
  "has_more": false
}
```

The response is linearized against one immutable ring view. Pagination scans canonical
sequence order; `next_since_sequence` is the last scanned sequence, including filtered-
out events, and the next request uses it verbatim. `has_more` is true exactly when that
cursor is below the captured latest value. ADR-0012 owns the full scan/limit rule.

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

When Unloaded, Hello uses `game_state: "Unloaded"`, `tick: null`, and the runtime
cursor. Event `tick` is nullable only for actor-control/runtime outcomes emitted when
no game exists; gameplay events and StateDelta always carry an integer tick.

The client then sends one subscription:

```json
{
  "type": "Subscribe",
  "since_sequence": 812,
  "event_types": ["StateDelta", "BuildComplete", "ResearchComplete"]
}
```

Commands use the same envelope wrapped as `{"type":"Command","envelope":...}`.

Example integer-only full-replacement delta for a no-op gameplay tick:

```json
{
  "type": "StateDelta",
  "event_sequence": 813,
  "tick": 421,
  "root_changes": {
    "tick": 421,
    "next_event_sequence": 814
  },
  "upserted_entities": [],
  "removed_entities": []
}
```

Each upsert contains the complete generated representation of one entity; each root
member present is replaced in full, and every actual deletion has an explicit key plus
a durable `EntityRemoved` event. There are no recursive leaf patches. `StateDelta` is
continuity-required and is never coalesced or silently dropped. Only the cumulative
progress events named by ADR-0012 may coalesce in a per-client queue; canonical polling
history retains every event.

Successful same-session NewGame/LoadAutosave emits a complete-replacement
StateDelta: every root field is present, every new-state entity is upserted, and every
old-only entity is removed. The loaded event lower bound is rebased to the runtime
session allocator first, so the existing ring/cursor remains monotonic. A fresh
process instead seeds its empty-ring floor from the saved lower bound; a stale client
must fetch a snapshot.

A queue overflow first coalesces only eligible progress readings. If a required event
still cannot be queued, the server makes a best effort to send `ResyncRequired` and
closes with application code 4009. It never slows simulation ticks. Resynchronization
fetches a fresh snapshot, records its cursor, and then applies all retained events after
that cursor.

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

Every error `details` member uses GDD 13's closed `Map<string, ErrorDetail>`:
integer/string/boolean/null or nonempty homogeneous string/integer arrays only. Opaque
objects, floats, mixed arrays, and ambiguous empty arrays are rejected by generated
schemas.

| HTTP | Meaning |
|-----:|---------|
| 400 | Malformed/invalid parameters |
| 401 | Missing/invalid token |
| 403 | Disallowed Origin or operation |
| 404 | Entity/endpoint not found |
| 409 | Lifecycle, expected-tick, idempotency, or state conflict |
| 410 | Event history requires resynchronization |
| 422 | Well-formed command violates content/game rule |
| 500 | Accepted control failed during an infrastructure operation, or an unhandled server fault |
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
