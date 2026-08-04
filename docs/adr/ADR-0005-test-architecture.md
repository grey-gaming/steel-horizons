---
status: accepted
owner: Tech Lead
date: 2026-08-04
---

# ADR-0005: Test Architecture — TDD, Scenarios, API, and Play-Tests

## Context

Steel Horizons combines deterministic arithmetic, finite authored resources, an unlock graph, autonomous matching, persistence, and two API transports. Unit coverage alone cannot prove that the starting state is playable, the economy remains solvable, or clients observe the same command semantics.

## Decision

We will use four test tiers plus mandatory content validation. TDD applies to behavioral production code: every mechanic begins with a failing test or conformance case, followed by the minimum implementation and refactor.

## Content Validation Gate

Before simulation tests run, `content_validate` checks canonical JSON mirrored from GDD 14:

- Schema conformance and unique IDs
- Acyclic, reachable technology prerequisites
- Recipe technologies and facility tiers
- Starting inventory and buffer capacity
- Exact 19 authored slots and valid moon parents/belt fields
- Bootstrap reachability to the first Cargo Ship
- Expanded critical-resource budget for all research plus the Gate
- Every command-referenced entity/tier/recipe definition

Content JSON is the source of truth used by runtime. Scenario fixtures reference it; fixtures do not redefine canonical balance.

## Tier 1 — Unit and Property Tests

Inline Rust tests cover pure mechanics:

- Fixed-scale and denominator-specific accumulators
- Actual-distance fuel calculation and final-segment capping
- Radial/arc travel-plan construction
- Cargo/Construction payload penalties, Research zero-capacity handling, and lane multipliers
- Recipe reservation and atomic completion
- Research exact consumption
- Buffer thresholds and capacity
- Logistics score and full tie-break order
- Reservation lifecycle
- xoshiro256** golden vectors
- Component recovery and salvage
- BuildOrder staging, Gate delivery, demolition evacuation, and automatic buffer allocation
- Survey-order and research-docking assignment

Property tests assert conservation, bounds, determinism, and overflow handling over generated inputs.

## Tier 2 — Deterministic Scenario Tests

The shared `ScenarioRunner` loads canonical content plus a small state fixture, applies commands at explicit ticks, advances batches, and asserts committed state/events.

CI-required scenarios include:

- `bootstrap_to_first_cargo`
- `first_five_hours`
- `full_supply_chain`
- `all_techs_reachable`
- `starting_state_to_gate_victory`
- `cancel_demolish_rebuild_recovery`
- `fuel_rescue_recovery`
- `reservation_contention`
- `belt_drift_determinism`
- `realtime_batch_equivalence`
- `save_load_equivalence`
- `command_log_replay_equivalence`

Golden snapshots include schema/content versions and are reviewed like code. Tests prefer invariants over broad snapshots when only a small result matters.

## Tier 3 — API Integration and Conformance

A real server on a random loopback port is exercised through HTTP and WebSocket. The same command corpus must produce equivalent acknowledgements and committed state through both transports.

Coverage includes:

- Every lifecycle state and command
- Idempotent command IDs and expected-tick conflicts
- Single-actor ordering under concurrent clients
- Integer-only snapshots/deltas
- Authentication, Origin validation, and connection discovery permissions
- Event replay at the retention boundary and 410/`ResyncRequired`
- Slow WebSocket clients without tick-rate coupling
- Graceful shutdown and save normalization

API schemas are snapshot-tested and generated examples are deserialized by the client library.

## Tier 4 — Agent Play-Tests

Python scripts discover the running server through `connection.json`, authenticate, inspect canonical content, and use explicit `AdvanceTicks` while Paused. They never rely on `sleep()` to infer simulation progress.

Required pre-release scripts:

- Minimal Gate victory
- Full-economy/all-tech run
- Recovery after deliberately poor builds
- V1-ceiling stress run

## CI Policy — Greenfield Construction (Phase 1)

During Phase 1 development, every commit runs all gates activated so far, in order:

1. Rust formatting and Clippy with warnings denied.
2. Content/schema validation.
3. Unit/property tests.
4. Every deterministic scenario activated so far per the scenario activation matrix in the implementation plan. Scenarios whose owning increment has not yet completed are not required.
5. HTTP/WS API conformance tests — once the API transport exists (P1-14), these become mandatory on every commit.
6. Save/load and command-log replay equivalence — once persistence exists (P1-13), these become mandatory on every commit.

Platform CI: macOS runs every commit throughout Phase 1. Windows CI is added at P1-36. Cross-platform state-hash comparison runs nightly and before release.

At Phase 1 completion, all gates listed in the Phase 1 completion gate section of the implementation plan are mandatory.

A flaky deterministic test is a product defect; retries are not an accepted mitigation.

## Consequences

- Gameplay promises such as “always solvable” become executable constraints.
- Content and schema drift fail before implementation behavior becomes ambiguous.
- Long scenarios cost more to maintain, but batch advancement keeps them independent of wall-clock time.
- Cross-transport conformance prevents REST and WS clients from becoming different games.

## Related ADRs

- ADR-0002 — Deterministic Tick Simulation
- ADR-0003 — Command/Query API with WebSocket Streaming
- ADR-0004 — Game Lifecycle State Machine
