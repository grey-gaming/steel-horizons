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

As its owning increments activate each rule, `content_validate` checks canonical
JSON mirrored from GDD 14:

- Schema conformance and unique IDs
- Acyclic, reachable technology prerequisites
- Recipe technologies and facility tiers
- Starting inventory and buffer capacity
- Exact 19 authored slots and valid moon parents/belt fields
- Bootstrap reachability to the first Cargo Ship (activated at P1-19, when the
  simulation proof exists)
- Expanded critical-resource budget for all research plus the Gate
- Every command-referenced entity/tier/recipe definition

Content JSON is the source of truth used by runtime. Scenario fixtures reference it; fixtures do not redefine canonical balance.

## Tier 1 — Unit and Property Tests

Inline Rust tests cover pure mechanics:

- Fixed-scale and denominator-specific accumulators
- Actual-distance fuel calculation and final-segment capping
- Refuel feasibility equality with actual movement for a fixed technology set,
  including serialized remainders and Life Support, plus monotonic safety for a
  mid-route permanent unlock
- Direct-refuel availability, ordered arrival reduction, partial transfer, and
  Cargo Fuel-reservation protection
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

These equivalence scenarios compare ADR-0006 replay-equivalence bytes and its
canonical deterministic event trace. Runtime actor-control receipts, stream sequence
numbers, lifecycle-only control events, and cursor rebasing have their own protocol
tests and cannot be mistaken for replayable gameplay differences.

Golden snapshots include schema/content versions and are reviewed like code. Tests prefer invariants over broad snapshots when only a small result matters.

## Tier 3 — API Integration and Conformance

A real server on a random loopback port is exercised through each transport as
it becomes available: REST at P1-14, complete event polling at P1-31, and
WebSocket parity at P1-32. Once P1-32 completes, the same command corpus must
produce equivalent acknowledgements and committed state through both
transports.

Coverage includes:

- Every lifecycle state and command
- Idempotent command IDs and expected-tick conflicts
- Single-actor ordering under concurrent clients
- Integer-only snapshots/deltas
- Authentication, Origin validation, and connection discovery permissions
- Exhaustive event-union and full-replacement StateDelta schemas
- Event replay at the retention boundary and 410/`ResyncRequired`
- Progress-only coalescence and forced resync/4009 when StateDelta continuity
  cannot be delivered
- Slow WebSocket clients without tick-rate coupling
- Graceful shutdown and save normalization

API schemas are snapshot-tested and generated examples are deserialized by the client library.

## Tier 4 — Agent Play-Tests

Python scripts discover the running server through `connection.json`, authenticate, inspect canonical content, and use explicit `AdvanceTicks` while Paused. They never rely on `sleep()` to infer simulation progress.

The following long-running scripts are required to complete P1-35 and then run
nightly, pre-release, and at the Phase 1 completion gate. They are an explicit
exception to the ordinary every-commit cumulative policy. Fast deterministic
unit/schema tests and reusable fixtures introduced by P1-35 remain ordinary
cumulative gates:

- Minimal Gate victory
- Full-economy/all-tech run
- Recovery after deliberately poor builds
- V1-ceiling stress run

## CI Policy — Greenfield Construction (Phase 1)

During Phase 1 development, every commit runs all gates whose owning increment
has completed, except for the explicitly long-running P1-35 play-test clients.
An ordinary gate remains mandatory after activation. The owning infrastructure
activates in this order:

| Owning increment | Gate or qualification check activated |
|---|---|
| P1-01 | Locked Rust build, formatting, Clippy with warnings denied, Rust smoke/unit tests, Python package formatting/typing/unit smoke, protocol/policy-sync checks, and macOS CI |
| P1-03–P1-05 | Exact content fixtures, then structural validation, then semantic validation and normalized content hash as each validator layer exists |
| Every behavioral increment | Its focused unit/property tests and every previously activated unit/property suite |
| P1-10 onward | Deterministic scenarios exactly as listed in the implementation plan's gate activation matrix; an unowned future scenario is not required |
| P1-13 | Save/load and command-log replay equivalence, extended by every later serialized-state change |
| P1-14 | Authenticated REST walking-skeleton conformance and focused generated-client smoke tests; WebSocket is not yet required |
| P1-31 | Complete HTTP event-polling, retention, event-schema, and resynchronization conformance |
| P1-32 | WebSocket authentication, shared REST/WS command corpus, backpressure, reconnect, and parity conformance |
| P1-33 | Runtime hardening, scheduler, autosave, shutdown, rate/body-limit, and failure-isolation conformance |
| P1-34a–P1-34d | The cumulative generated Python client/TUI formatting, typing, unit, renderer, integration, and resynchronization gates delivered by each sub-increment |
| P1-35 | Fast deterministic tests/fixtures become cumulative; all four long-running Python agent play-tests are required for P1-35 completion and thereafter nightly, pre-release, and at Phase 1 completion, not on every ordinary commit |
| P1-36 ordinary gate | Windows locked build and test CI |
| P1-36 qualification | Supported-platform state-hash comparison, V1-ceiling benchmark, locked private artifacts, and SBOM; required to complete P1-36 and thereafter in qualification runs, not on every ordinary commit |

The increasing-scope order on a commit is:

1. repository synchronization and generated-file checks;
2. Rust formatting, locked build, and Clippy with warnings denied;
3. active content/schema validation;
4. unit and property tests, including the active Python package tests where
   applicable;
5. every activated deterministic scenario;
6. persistence/replay equivalence once P1-13 exists;
7. the active REST, polling, WebSocket, and runtime conformance layers in their
   activation order;
8. the cumulative Python client/TUI gate once P1-34a exists; and
9. Windows locked build and test CI once P1-36 exists.

The P1-35 qualification invocation runs the four long agent play-tests after
the ordinary active gates. Subsequent ordinary commits do not repeat those four
clients; nightly, pre-release, and Phase 1 completion runs do.

The P1-36 qualification invocation then runs supported-platform canonical
state-hash comparison, the V1-ceiling benchmark, private packaging, and SBOM
generation. Those checks are required to complete P1-36 and in later nightly,
pre-release, and Phase 1 completion runs, but are not repeated on every
ordinary commit.

macOS CI begins at P1-01 and runs every Phase 1 commit. P1-01 establishes
platform-neutral repository commands and documents the future Windows target,
but the Windows CI job is added only at P1-36. Nightly jobs may run a larger
matrix or repeat long gates, but only gates whose owning increment is complete
may be claimed. Cross-platform hashes and the V1-ceiling benchmark cannot run
before P1-36; pre-release after Phase 1 completion runs the full completion
gate.

At Phase 1 completion, all gates listed in the implementation plan's Phase 1
completion gate are mandatory.

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
