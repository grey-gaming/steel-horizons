---
status: accepted
owner: Tech Lead
date: 2026-08-04
---

# ADR-0004: Game Lifecycle State Machine

## Context

The API exists before a scenario loads and supports ordinary real-time play, paused planning, deterministic batch advancement, autosave loading, victory inspection, and starting over. Commands must have unambiguous validity and transitions must be serialized with simulation mutation.

## Decision

The simulation actor owns this lifecycle:

```text
Unloaded --NewGame/LoadAutosave--> Loading --success--> Paused or saved Won
                                      |--failure-----> Unloaded

Paused <--Pause/Resume--> Running
Paused --AdvanceTicks(N)--> Advancing --complete/error--> Paused
Paused/Running --GateComplete--> Won
Paused/Running/Won --NewGame/LoadAutosave--> Loading
                         Loading --failure--> prior state/snapshot
```

### States and Commands

| State | Meaning | Allowed commands |
|-------|---------|------------------|
| **Unloaded** | API ready; no scenario | `NewGame`, `LoadAutosave` |
| **Loading** | Validating content and constructing/deserializing state | none; status/query only |
| **Paused** | Tick timer stopped; planning transactions allowed | all gameplay/configuration commands, `Resume`, `AdvanceTicks`, `SaveNow`, `NewGame`, `LoadAutosave` |
| **Running** | One tick per real second | all gameplay/configuration commands, `Pause`, `SaveNow`, `NewGame`, `LoadAutosave` |
| **Advancing** | Exactly N normal ticks executing without wall-clock wait | no commands; queries/status may report progress |
| **Won** | Gate activated; simulation stopped | queries, `SaveNow`, `NewGame`, `LoadAutosave` |

`AdvanceTicks` is unavailable while Running and is intended for agent/test clients. Batch errors return to Paused at the last fully committed tick.

Loading is atomic. When `NewGame`/`LoadAutosave` begins from Paused, Running, or Won, the actor retains the prior immutable state. Validation/deserialization failure restores that exact state and lifecycle; only a load begun from Unloaded fails back to Unloaded. A successful load/new game replaces state and enters the loaded lifecycle (normally Paused; a Won autosave remains Won).

### Save Normalization

Saves made from Running or Advancing serialize a load lifecycle of Paused. A Won save loads as Won. Unloaded and Loading are never saved.

### Atomicity

Every transition occurs in the single simulation actor. Concurrent requests receive server-sequence numbers and cannot create two batch advances, mutate state during Loading/Advancing, or race Pause against a partial tick.

Invalid lifecycle commands return 409 with current state and the allowed-state set. Queries against Unloaded return 503; Loading returns 202 status with progress and 503 for game-state queries.

### API Server Lifecycle

```text
Starting -> Ready -> ShuttingDown -> Terminated
```

- Starting binds loopback, creates connection discovery/token data, loads content definitions, and creates an Unloaded simulation actor. Unless `--no-autoload` is set, it then loads the configured starting scenario to Paused before reporting Ready.
- Ready accepts authenticated connections regardless of game lifecycle.
- ShuttingDown stops acceptance, finishes or rolls back the active actor transaction, saves Paused/Running/Won state, drains bounded acknowledgements, and removes connection discovery data.

## Consequences

- Clients can reason about every command's validity.
- Batch advancement and real-time scheduling share identical tick code.
- Load failure is recoverable without restarting the server or losing the currently loaded game.
- The actor and explicit transition table eliminate lifecycle mutex races.

## Related ADRs

- ADR-0001 — Rust Simulation Engine with HTTP/WS API
- ADR-0002 — Deterministic Tick Simulation
- ADR-0003 — Command/Query API with WebSocket Streaming
