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
| ADR-0006 | Canonical Content/State Hashing | accepted | SHA-256, canonical JSON serialization, BTreeMap ordering, golden-update policy |
