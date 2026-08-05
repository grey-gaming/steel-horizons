# Architecture Decision Records (ADRs)

This directory holds Architecture Decision Records for Steel Horizons.
Each ADR documents a key architectural choice — context, decision, and consequences.

Naming convention: `ADR-NNNN-title-with-hyphens.md`
Status: proposed | accepted | deprecated | superseded

## ADRs

| ADR | Title | Status | Description |
|-----|-------|--------|-------------|
| ADR-0001 | Rust Simulation Engine with HTTP/WS API | accepted | Core tech stack decision — Rust engine, local HTTP/WS API server |
| ADR-0002 | Deterministic Tick Simulation with Integer Arithmetic | accepted | Simulation model — discrete ticks, integer math, no floating-point |
| ADR-0003 | Command/Query API with WebSocket Streaming | accepted | API protocol — dual REST + WebSocket for sync commands and streaming |
| ADR-0004 | Game Lifecycle State Machine | accepted | Game lifecycle — explicit state machine for Unloaded → Won |
| ADR-0005 | Test Architecture — TDD, Simulation Tests, API Tests | accepted | Four-tier testing strategy for TDD workflow |
| ADR-0006 | Canonical Content/State Hashing | accepted | Versioned canonical JSON and SHA-256 content/scenario/replay-equivalence/save-integrity fingerprints |
| ADR-0007 | Save Envelope Format, Content Hash Placement, and Migration Fixtures | accepted | Exact-content save envelope, integrity checks, atomic replacement, and migrations |
| ADR-0008 | Accepted-Command Persistence | accepted | Save barriers, replayable command records, controls, idempotency, and load rebuilds |
| ADR-0009 | Hub Shipyard Queue Semantics | accepted | Single-active FIFO ship construction, staging, cancellation, and conservation |
| ADR-0010 | Mining Boundary Behavior | accepted | Capacity-safe extraction, shared-deposit reduction, retuning, exhaustion, and belt drift |
| ADR-0011 | Deterministic Refueling | accepted | Cross-role refueling, exact reachability, Fuel allocation, dock contention, and rescue |
| ADR-0012 | Complete Command/Event Wire Contracts | accepted | Exhaustive external events, state replacement/removal, backpressure, and research reassignment |
