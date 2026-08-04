---
status: accepted
owner: Tech Lead
date: 2026-08-04
---

# ADR-0001: Rust Simulation Engine with HTTP/WS API Server

## Context

Steel Horizons is a desktop game targeting macOS (primary) and Windows (secondary). The build is split into two phases:

- **Phase 1:** Simulation engine + API/streaming interface + text-only UI (for agent play-testing)
- **Phase 2:** Full graphical UI/UX

Phase 1 requires a cross-platform simulation engine that can be consumed by both an AI agent (for automated play-testing) and a future native UI. The engine must expose a programmatic interface that supports both synchronous commands and real-time state streaming.

## Decision

We will implement the simulation engine in **Rust** and expose it as a **local HTTP/WS API server**.

### Rationale

1. **Rust** is chosen because:
   - Cross-platform by nature — compiles to native binaries on macOS, Windows, and Linux with no runtime dependency
   - Memory safety without garbage collection — critical for deterministic tick simulation with complex state
   - Performance — targets 500 ships, 200 stations, all 25 V1 resource/component types, and at least 100 batch ticks/second in the reference release fixture
   - Cross-platform packaging — the same binary can run beside the PixiJS v8 graphical client in a Tauri desktop bundle on macOS and Windows
   - Strong type system — the data models (ResourceType, Ship, Station, etc.) map naturally to Rust enums and structs
   - Ecosystem — mature HTTP/WS libraries (axum, tokio-tungstenite), excellent testing support

2. **HTTP/WS API server** is chosen because:
   - The text UI and AI agent are separate consumers — they need a network-addressable interface
   - HTTP REST provides synchronous command/query semantics (place station, get state)
   - WebSocket provides streaming state deltas tick-by-tick
   - Both consumers (text UI and agent) can connect independently using standard HTTP clients
   - No coupling between simulation and UI — the simulation can run as a background process or be embedded
   - The API surface is testable with standard HTTP testing tools

### Consequences

- **Positive:**
  - Clear separation of concerns: simulation engine has zero UI coupling
  - AI agent can interact using standard curl/HTTP or WebSocket client — no custom protocol needed
  - Text UI is a thin client — complexity lives in the API server
  - The Phase 2 PixiJS/Tauri UI reuses the same API without changes to simulation rules
  - The simulation can run in-process or as a separate child process

- **Negative:**
  - Serialization overhead for state transfers — full state snapshots can be large (~500KB+ at scale)
  - The API server adds startup complexity — need to manage server lifecycle, port allocation, error handling
  - Latency between command and visible state change includes serialization + network round-trip (though localhost is negligible)
  - Need to order concurrent API consumers and protect the mutating localhost interface

- **Mitigations:**
  - State deltas (tick-by-tick changes) over WebSocket avoid full-state serialization every tick
  - Full state snapshots are available on demand via REST for client initialization
  - A single-owner simulation actor serializes ticks, lifecycle transitions, and commands
  - Session-token authentication and browser Origin validation protect the loopback API
  - Server lifecycle is managed via Rust's graceful shutdown patterns (tokio signal handling)
  - Port binding with fallback and user-configurable

## Related ADRs

- ADR-0002 (Deterministic Tick Simulation)
- ADR-0003 (Command/Query API with WebSocket Streaming)
