---
status: Approved
owner: Tech Lead
date: 2026-08-04
---

# System Architecture

## Overview

Steel Horizons uses a Rust engine process with a single-owner simulation actor and a loopback HTTP/WebSocket boundary. Phase 1 consumers are a Python text UI and agent play-tests. Phase 2 packages a PixiJS v8 graphical client in Tauri and launches/connects to the same engine.

```text
Python TUI ----------- HTTP/WS -----------+
Agent play-tests ----- HTTP/WS -----------+--> API server
PixiJS/Tauri UI ------ HTTP/WS (Phase 2) -+       |
                                                  v
                                        Command/query gateway
                                                  |
                                    +-------------+-------------+
                                    | Simulation actor          |
                                    | lifecycle + command queue |
                                    | tick phases + GameState   |
                                    +-------------+-------------+
                                                  |
                                      committed event broadcaster
```

## Process Model

`steel-horizons-engine` is one native process containing:

1. Axum HTTP/WS server bound to loopback
2. Authentication/session-discovery layer
3. Simulation actor task—the only mutable `GameState` owner
4. One-second scheduler and paused batch executor
5. Bounded event retention/broadcast
6. Atomic persistence service

The process uses compiled Rust dependencies but requires no separately installed runtime or external service.

Phase 2 launches the engine as a managed child process. Tauri/PixiJS is the only graphical stack; SwiftUI, C#, 3D, and mesh rendering are outside V1.

## Ownership and Dependency Direction

```text
API transport -> command/query gateway -> simulation actor -> domain/content
tick scheduler --------------------------> simulation actor
persistence <----------------------------- simulation actor snapshots
event retention <------------------------- committed actor events
clients -> generated protocol types only
```

Rules:

- Domain/content modules depend on neither Axum nor presentation code.
- API handlers cannot obtain mutable `GameState` references.
- Commands and scheduled ticks enter one actor mailbox.
- Queries read immutable committed snapshots published by the actor.
- Persistence receives a consistent actor snapshot, never a live mutable tree.
- Events are produced from committed changes, not by diffing two unlocked states.

## Actor Transactions

### Running Command

```text
client -> authenticated command -> validate envelope
       -> actor assigns server_sequence and effective next tick
       -> return 202 Accepted to request transport
       -> tick applies queued commands in sequence order
       -> commit -> CommandApplied/CommandRejected event
```

### Paused Planning Command

```text
client -> gateway -> actor transaction at current tick
       -> validate + apply + commit without advancing tick
       -> record command sequence -> acknowledgement/event
```

### Batch Advancement

```text
AdvanceTicks(N) in Paused -> lifecycle Advancing
                           -> execute N ordinary tick transactions
                           -> lifecycle Paused
                           -> return final acknowledgement
```

No player command is accepted during Advancing.

## Read and Streaming Flow

The actor publishes an immutable `Arc<GameSnapshot>` after each commit. Queries clone the `Arc`, so a slow serializer cannot block mutation. Each snapshot contains `tick`, `latest_event_sequence`, lifecycle, schema/content versions, and complete state.

Committed transactions emit typed events to:

- A bounded ring containing the newest 50,000 events
- Per-client bounded WebSocket queues
- Structured logs/metrics

Slow-client handling never changes scheduler speed. A lagged client receives `ResyncRequired` and fetches a fresh snapshot.

## Security and Discovery

Default startup:

1. Bind `127.0.0.1:4880`; if unavailable, scan 4881–4890.
2. Generate a random bearer token.
3. Atomically write owner-only `connection.json` containing port, token, PID, and protocol version.
4. Require the token on every HTTP request and WS upgrade.
5. Validate browser Origin against the packaged Tauri allowlist; permissive CORS is disabled.

Clients read discovery data rather than assuming port 4880. Shutdown removes the file only if its PID/token still match the current process.

## Persistence

The actor creates a normalized save snapshot. The persistence service writes to a same-directory temporary file, syncs it, renames it, and syncs the directory where supported. Saves include the next event sequence and bottleneck rolling windows but not the runtime event-retention ring. Saves from Running/Advancing load Paused; Won remains Won.

Autosave occurs every 300 committed ticks and after material Paused-state commands. Persistence failures emit durable errors without terminating the simulation.

## Content Loading

Versioned JSON under `content/` contains the exact GDD 14 definitions and starting scenario. Startup runs schema, graph, recipe, budget, and bootstrap validation before the server reports Ready. Invalid content is a fatal startup error with a precise path/message.

## Configuration

| Parameter | Default |
|-----------|---------|
| Host | `127.0.0.1` |
| Preferred port | `4880` |
| Fallback range | `4881..4890` |
| User data | OS application-support directory via `dirs` |
| Autosave | `save.json` |
| Discovery | `connection.json` (owner-only) |
| Scheduler | 1 tick/sec |
| Event retention | newest 50,000 events; tick coverage is workload-dependent |
| Initial scenario | `starting_system` |

## Failure Strategy

- Invalid commands return typed client errors and do not mutate state.
- Simulation invariant failures abort the current transaction, transition Running/Advancing to Paused, and emit a high-severity error.
- Panics are bugs; production paths return typed errors.
- Startup content errors are fatal before Ready. A game load/new-scenario error restores the prior immutable state/lifecycle, or returns to Unloaded only when no prior game existed.
- API/client serialization errors affect only the request/client queue.

## Related ADRs

- ADR-0001 — Rust Simulation Engine with HTTP/WS API
- ADR-0002 — Deterministic Tick Simulation
- ADR-0003 — Command/Query API with WebSocket Streaming
- ADR-0004 — Game Lifecycle State Machine
