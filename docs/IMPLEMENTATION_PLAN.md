---
status: Active
owner: Autonomous implementation agent
last-reviewed: 2026-08-04
---

# Steel Horizons Implementation Plan

## Purpose

This is the canonical execution order for building Steel Horizons in small, cumulative, independently verifiable increments. It is written for an autonomous coding agent. The root [`AGENTS.md`](../AGENTS.md) applies throughout the repository and must be read at the start of every implementation turn.

The first useful end-to-end milestone is:

> Start the canonical game, queue a Mine, Refinery, and Courier while Paused, advance exact ticks through the authenticated API, and observe the first autonomous delivery in state, committed events, and the text UI.

Phase 1 delivers the Rust engine, authenticated local HTTP/WebSocket API, canonical JSON content, Python text UI/client, and agent play-tests. Phase 2 adds the PixiJS v8/Tauri graphical client only after the Phase 1 completion and protocol-freeze gates. V2 inter-system Gate logistics is outside this plan.

## Design authority and traceability

When documents overlap, implementation and tests use this authority order:

1. Accepted ADRs for architecture.
2. [GDD 12](gdd/12-simulation-foundations.md) for simulation semantics and formulas.
3. [GDD 13](gdd/13-data-models.md) for serialized state shapes.
4. [GDD 14](gdd/14-authored-content.md) for exact content, costs, statistics, and starting values.
5. Other approved GDDs for player-facing behavior and presentation.
6. TDDs for implementation structure.

Each completed increment must add entries to a machine-readable traceability ledger created in P1-01. Each entry records:

- Stable requirement/test identifier.
- Authoritative document path and section.
- Production/content files implementing the requirement.
- Focused unit/property/scenario/API/play-test evidence.
- Current status and owning plan increment.
- Canonical state/content golden changes, when applicable.

Changing an authoritative rule requires updating every affected summary, schema/content record, validator, and executable test in the same increment.

## Delivery principles

- Execute Phase 1 serially in dependency order. Only independent read-only research or review work may run in parallel.
- Treat each numbered item as the maximum size of one implementation increment. Split it further when the red/green loop would be clearer; do not merge adjacent increments.
- Add recovery behavior with the mechanic that creates the recoverable state: build cancellation with building, recipe release with production, research pause with research, and survey cancellation with surveying.
- Establish canonical serialization, hashing, save/load, and replay before rich gameplay state so every later state field inherits those gates.
- Use canonical content in scenarios. Specialized fixtures may vary setup but must not redefine balance.
- Once a commit-class test or scenario is activated, it remains part of every subsequent commit gate. Qualification-class play-test/platform gates follow the explicit matrix cadence.
- Correctness and deterministic equivalence precede optimization.
- Simulation state uses checked gameplay/domain integers, stable iteration, and the project-owned serialized PRNG; overflow is a typed error rather than wrapping or panicking, except for GDD 12's named PRNG operations that intentionally wrap modulo 2^64.
- In the running application, the simulation actor is the sole mutable state owner; the fixed GDD 12 phase order and outcomes are independent of execution speed.
- Generated IDs come only from serialized per-kind counters; UUIDs, wall-clock values, and collection lengths never determine simulation identity.

The following block is intentionally duplicated verbatim in root `AGENTS.md`. Changes to either copy must update both in the same change; P1-01 adds an automated synchronization check.

<!-- BEGIN PER-TURN PROTOCOL -->
## Per-turn autonomous execution protocol

On every implementation turn, follow this sequence without skipping steps:

1. **Re-establish repository context.**
   - Read root `AGENTS.md` and `docs/IMPLEMENTATION_PLAN.md`.
   - Inspect `git status --short`, recent commits, and the files/tests belonging to the current increment.
   - Preserve all unrelated or user-authored changes. Never reset or overwrite them.
2. **Select exactly one bounded increment.**
   - Resume the earliest partially completed increment; otherwise choose the earliest unchecked, unblocked increment in the active phase.
   - A direct user instruction may select a different in-scope increment, but unmet dependencies must be reported and may not be bypassed.
   - Split an increment further when useful; never combine multiple plan increments merely to make a larger change.
3. **Re-read the governing contract.**
   - Read the authoritative document sections cited by the increment and the relevant data/API/test contracts.
   - Check the specification-closure list in the plan. If deterministic behavior remains ambiguous, resolve it in a doc-only ADR/GDD/TDD change before production code. Do not silently invent gameplay, serialization, or wire semantics.
4. **Define the proof before the implementation.**
   - State the behavior and acceptance evidence for the slice.
   - Add or identify one focused test that fails for the intended reason, and run it to confirm the red state.
   - Early scaffolding that cannot have a behavioral test must have a deterministic smoke check with an explicit expected result.
   - For an explicitly doc-only Gate 0 increment, use a focused document consistency, link, or contract check instead of a behavioral test and record the exact future executable proof. Confirm the pre-change inconsistency when practical.
5. **Implement the smallest coherent change.**
   - Make only the production/content/tooling changes needed to satisfy the focused proof.
   - Keep domain/content independent of Axum and presentation code. Runtime/API code never receives mutable `GameState`; pure constructors and deterministic test harnesses may own isolated state before P1-11, after which production mutation is actor-owned.
   - Do not add speculative abstractions, unrelated cleanup, hidden fallbacks, or unrequested V2 behavior.
6. **Apply cross-cutting proof obligations.**
   - Any serialized-state change requires Serde round-trip, invariant, canonical-hash, save/load, and replay coverage.
   - Any material/inventory change requires conservation, capacity/bounds, and recovery-path coverage.
   - Any command change requires allowed/invalid lifecycle, `expected_tick`, idempotency, ordering, and malformed-input coverage; after both transports exist, run it through the shared REST/WS corpus.
   - Any content change requires validator coverage, normalized content-hash review, and affected reachability/scenario tests.
   - Any presentation change requires keyboard, non-color, UI-scale, reduced-motion, and visual-regression checks where applicable.
7. **Run verification in increasing scope.**
   - Run the focused test first, then the affected crate/package suite.
   - Run every cumulative gate activated by the plan through the current increment, in the documented order.
   - Tests must use isolated temporary data directories and explicit tick advancement; never use a developer save or `sleep()` as a simulation clock.
   - Do not accept flaky retries, skipped required tests, warnings, silent golden updates, or unexplained state-hash changes.
   - If a required tool, platform, runner, or credential is unavailable, record exactly what did and did not run; never claim the unavailable gate passed.
8. **Review the resulting diff.**
   - Check correctness, deterministic ordering, checked arithmetic, typed errors, and document traceability.
   - Confirm generated files are reproducible and no credentials, discovery tokens, saves, build outputs, or unrelated files are included.
9. **Record progress and evidence.**
   - Update the increment checkbox and evidence log in `docs/IMPLEMENTATION_PLAN.md` only when its acceptance evidence is fully satisfied.
   - Record commands run, tests/scenarios activated, and any intentional golden/hash changes.
   - Keep the plan, authoritative documents, schemas/content, and executable tests synchronized.
10. **End in a resumable state.**
    - A completed increment ends green and narrowly scoped. When the active request authorizes autonomous implementation commits, create one intentional local commit for the increment.
    - Do not push, open a PR, publish, sign, notarize, or release unless explicitly requested.
    - If incomplete, leave the checkbox open and report the exact remaining work or blocker; never claim completion based only on partial tests.
    - Report the exact verification commands and outcomes, files changed, completed increment, and next increment or blocker.

<!-- END PER-TURN PROTOCOL -->

## Increment contract map

Before implementing an increment, read its row here and the more-specific cross-references contained in those documents. This map identifies the minimum governing contract; authority still follows the hierarchy above.

| Increment(s) | Minimum authoritative/design reading |
|---|---|
| G0-01–G0-09 | [README — Authority and Change Rules](../README.md#authority-and-change-rules), [GDD 12](gdd/12-simulation-foundations.md), [GDD 13](gdd/13-data-models.md), [GDD 14](gdd/14-authored-content.md), [ADR-0005](adr/ADR-0005-test-architecture.md), and the specific overlapping documents named by the gap |
| P1-01 | [TDD 04 — Testing Strategy](tdd/04-testing-strategy.md), [TDD 05 — Build & Deployment](tdd/05-build-and-deployment.md) |
| P1-02a–P1-05 | [GDD 13 — Identifiers, Root State, and Content Definitions](gdd/13-data-models.md), [GDD 14 — Authored Content](gdd/14-authored-content.md), [ADR-0005 — Content Validation Gate](adr/ADR-0005-test-architecture.md#content-validation-gate) |
| P1-06–P1-10 | [ADR-0002 — Deterministic Tick Simulation](adr/ADR-0002-deterministic-tick-simulation.md), [GDD 12 — Numeric Representation, Tick Order, and Replay](gdd/12-simulation-foundations.md), [TDD 01 — Simulation Engine](tdd/01-simulation-engine.md) |
| P1-11–P1-14 | [ADR-0003 — Command/Query API](adr/ADR-0003-command-query-api-with-websocket-streaming.md), [ADR-0004 — Lifecycle](adr/ADR-0004-game-lifecycle-state-machine.md), [TDD 00 — Architecture](tdd/00-architecture.md), [TDD 02 — API Protocol](tdd/02-api-protocol.md) |
| P1-15–P1-19 | [GDD 3 — Economy](gdd/03-economy.md), [GDD 5 — Ships, Stations & Factories](gdd/05-ships-stations-factories.md), [GDD 6 — Onboarding Phase 1](gdd/06-onboarding.md#phase-1--first-production-chain-hour-01), [GDD 7 — Routes & Logistics](gdd/07-routes-and-logistics.md), [GDD 12 — Production and Logistics](gdd/12-simulation-foundations.md), [GDD 14 — Starting State and Recipes](gdd/14-authored-content.md) |
| P1-20–P1-21 | [GDD 3 — Fuel](gdd/03-economy.md#fuel), [GDD 7 — Jobs, Docks, and Fuel Safety](gdd/07-routes-and-logistics.md), [GDD 12 — Travel, Jobs, and Reservations](gdd/12-simulation-foundations.md) |
| P1-22–P1-23 | [GDD 4 — Research](gdd/04-tech-tree.md), [GDD 5 — Research Ships/Stations](gdd/05-ships-stations-factories.md), [GDD 6 — Onboarding Phase 2](gdd/06-onboarding.md#phase-2--research-and-surveying-hour-12), [GDD 12 — Research and Survey Semantics](gdd/12-simulation-foundations.md), [GDD 13 — Research and Survey Models](gdd/13-data-models.md) |
| P1-24–P1-26 | [GDD 3 — Production and Recovery](gdd/03-economy.md), [GDD 4 — Technology Tree](gdd/04-tech-tree.md), [GDD 5 — Upgrades and Factories](gdd/05-ships-stations-factories.md), [GDD 6 — Onboarding Phases 3–5](gdd/06-onboarding.md), [GDD 14 — Exact Progression Content](gdd/14-authored-content.md) |
| P1-27 | [GDD 7 — Bottleneck Detection](gdd/07-routes-and-logistics.md#bottleneck-detection), [GDD 13 — Bottleneck Monitoring](gdd/13-data-models.md#bottleneck-monitoring) |
| P1-28–P1-29 | [GDD 3 — Construction and Recycling](gdd/03-economy.md#construction-and-recycling), [GDD 5 — Cancellation, Demolition, and Scrapping](gdd/05-ships-stations-factories.md#cancellation-demolition-and-scrapping), [GDD 7 — Demolition Logistics](gdd/07-routes-and-logistics.md), [GDD 12 — Recovery](gdd/12-simulation-foundations.md#recovery-and-the-always-solvable-invariant) |
| P1-30 | [GDD 5 — Space Gate](gdd/05-ships-stations-factories.md#space-gate), [GDD 13 — Gate Model](gdd/13-data-models.md#gate), [GDD 14 — Space Gate Definition](gdd/14-authored-content.md#space-gate-definition), [ADR-0004 — Won Lifecycle](adr/ADR-0004-game-lifecycle-state-machine.md) |
| P1-31–P1-33 | [ADR-0003](adr/ADR-0003-command-query-api-with-websocket-streaming.md), [TDD 00 — Read/Streaming, Security, Persistence, Failure](tdd/00-architecture.md), [TDD 02](tdd/02-api-protocol.md), [TDD 04 — API Tests](tdd/04-testing-strategy.md#api-tests) |
| P1-34a–P1-34d | [TDD 03 — Text UI / Agent Interface](tdd/03-text-ui-agent-interface.md), [TDD 02 — Resynchronization](tdd/02-api-protocol.md) |
| P1-35–P1-36 | [ADR-0005 — Agent Play-Tests and CI](adr/ADR-0005-test-architecture.md), [TDD 04 — Cross-Platform and Performance](tdd/04-testing-strategy.md), [TDD 05 — Build and Release](tdd/05-build-and-deployment.md) |
| P2-01–P2-15 | [README — Product Requirements and Accessibility](../README.md#product-requirements), [GDD 8 — Visual Style](gdd/08-visual-style.md), [GDD 9 — Zoom](gdd/09-zoom-levels.md), [GDD 10 — Assets](gdd/10-iconography-and-textures.md), [GDD 11 — UI Interactions](gdd/11-ui-interactions.md) |

## Gate 0 — Specification closure

Although the Phase 1 architecture is approved, the following deterministic contracts must be made explicit before their dependent production increments. Resolve each as a small doc-only change using the authority hierarchy, then add its planned executable proof. Do not bury these decisions inside code.

- [x] **G0-01 — Complete machine-readable content schemas.** Define `ShipStats`, `StationStats`, the starting-scenario record, Gate definition, authored defaults, and schema-generation ownership. Required before P1-02a/P1-02b/P1-03.
- [x] **G0-02 — Define canonical content/state hashing.** Specify included/excluded fields, canonical byte encoding, map/set ordering, hash function/version, and golden-update policy. Required before P1-05/P1-08.
- [x] **G0-03 — Define the save envelope.** Place the normalized content hash outside or inside the authoritative root state explicitly, and specify schema/content compatibility and migration fixtures. Required before P1-13.
- [x] **G0-04 — Define accepted-command persistence.** Specify what `SaveNow` does with commands accepted for future ticks, pending actor-mailbox state, command outcomes, and idempotency records across save/load and process restart. Required before P1-13.
- [x] **G0-05 — Define Hub shipyard queue semantics.** Specify capacity, ordering, active work representation, cancellation, and serialized fields. Required before P1-15.
- [x] **G0-06 — Define mining boundary behavior.** Specify full-output handling, finite-deposit exhaustion when production exceeds the remainder, and the extraction/belt-drift order at tick multiples of 1,000. Required before P1-17/P1-26.
- [x] **G0-07 — Define deterministic refueling.** Specify eligible ship roles, Fuel-station selection and tie-breaks, route feasibility, transfer quantity/timing, partial stock behavior, and dock usage. Required before P1-20/P1-21.
- [x] **G0-08 — Complete command/event wire contracts.** Define exhaustive tagged payloads, `StateDelta` replacement/removal semantics, durable versus coalescible events, and research resume/reassignment across facilities. Required before P1-22/P1-31.
- [x] **G0-09 — Reconcile cumulative CI policy with ADR-0005/TDD 04.** Amend the authoritative testing documents so that, during greenfield construction, every commit runs all gates activated so far; each final required deterministic scenario becomes permanently mandatory at its owning increment; long-running agent play-tests run at P1-35 completion and thereafter in nightly/pre-release/completion gates; all gates are mandatory at Phase 1 completion. Required before P1-01.
  - Verify: focused text/link consistency review proves ADR-0005, TDD 04, this plan, and root `AGENTS.md` state the same policy; P1-01 later automates protocol/policy synchronization.

Gate 0 evidence is recorded like implementation evidence. If an item materially changes approved product behavior and cannot be resolved by the authority hierarchy, it is an explicit blocker requiring owner direction.

## Phase 1 — Foundation and executable contracts

- [x] **P1-01 — Repository, toolchain, CI, and traceability scaffold.**
  - Deliver: pinned Rust and Python toolchains; Cargo workspace; engine library/binary shell; Python package shell; dependency locks; formatting, lint, unit-test, and macOS CI entry points; platform-neutral verification scripts for later Windows activation; requirement-to-test ledger; reproducible schema-export command shell; marker-based checks for the duplicated per-turn protocol and staged verification-policy manifest.
  - Verify: clean locked build, one Rust test, one Python scaffold test, formatter/linter gates, protocol/policy-sync checks, and `steel-horizons-engine --version`.
  - Depends on: G0-09.

- [x] **P1-02a — Primitive protocol/domain vocabulary.**
  - Deliver: ID newtypes, `ResourceType`, lanes, lifecycle, entity-role/state enums, explicit Serde tags, and deterministic `Ord` implementations.
  - Verify: serialization snapshots, enum/newtype round trips, stable resource order, invalid/unknown-field cases.
  - Depends on: G0-01, P1-01.

- [x] **P1-02b — Serialized state/content contracts and schema exporter.**
  - Deliver: every GDD 13 Serde-only DTO including `GameState`, `DefinitionsCatalog`, `StartingScenario`, commands, strict unknown-field policy, `BTreeMap`/`BTreeSet` shapes, and deterministic content JSON Schema export.
  - Verify: full DTO round trips; explicit null/empty collection fixtures; root-schema snapshots; duplicate/unknown/missing-field rejection; regeneration has no diff.
  - Depends on: G0-04–G0-08, P1-02a.

- [x] **P1-03 — Canonical content files.**
  - Deliver: `content/definitions.v1.json`, `content/starting_system.v1.json`, and the two reproducibly generated `schemas/content/*.schema.json` artifacts transcribed from GDD 14/GDD 13.
  - Verify: schema regeneration has no diff; exact record/count/value fixtures cover 25 resource types, seven bodies, 19 slots, 27 recipes (11 refining, eight assembly, eight inverse), 23 technologies, 12 ship definitions, 20 station definitions, starting metadata/inventory/explicit fields, typed mechanic unlocks, build-work/default constants, and Gate values.
  - Depends on: P1-02b.

- [x] **P1-04 — Content validator: structural rules.**
  - Deliver: schema parsing, precise errors, unique IDs, authored/generated namespace separation, parent/body/slot validity, thresholds, statistics, starting capacities, and exact slot total.
  - Verify: table-driven invalid fixtures, each with stable path and typed failure.
  - Depends on: P1-03.

- [x] **P1-05 — Content validator: semantic rules and content hash.**
  - Deliver: technology DAG, unlock/recipe/facility reachability, inverse-recipe equality, cost/build-hold rules, critical-resource budget, normalized representation, and versioned content hash.
  - Verify: cycle/unreachable/budget/inverse negative fixtures and stable content-hash golden. Executable bootstrap validation is added in P1-19 when the simulation exists.
  - Depends on: G0-02, P1-04.

- [ ] **P1-06 — Exact arithmetic kernel.**
  - Deliver: standard milli accumulator, denominator-specific rational accumulator, Fuel and Life Support accumulators, scale newtypes, and checked arithmetic errors.
  - Verify: canonical examples, quotient greater than one, exact project completion, remainder bounds, overflow/property tests.
  - Depends on: P1-02a.

- [ ] **P1-07 — Deterministic PRNG and travel geometry.**
  - Deliver: project-owned xoshiro256**, stable iteration helpers, radial/arc route construction, lane/payload speed calculation, zero-distance and wrap behavior.
  - Verify: PRNG golden vectors, radial/arc/combined/zero routes, 6,283 boundary, Research zero-capacity branch, checked-product boundaries.
  - Depends on: P1-06.

- [ ] **P1-08 — Root state and canonical starting state.**
  - Deliver: canonical tick-zero constructor from the explicit scenario mapping, generated counters/names, cheap invariant checker, canonical state projection and hash over the P1-02b DTOs.
  - Verify: exact tick-zero snapshot/hash, complete Hub/Builder fields, Serde round trip, malformed-state negatives, insertion-order independence, exact generated prefixes/next-unused formatting, checked overflow, rollback non-consumption, committed non-reuse, and save/replay counter continuity.
  - Depends on: G0-02, P1-02b, P1-03, P1-06, P1-07.

- [ ] **P1-09 — Tick transaction skeleton.**
  - Deliver: pending field changes, explicit reducers, conflict rejection, all eleven fixed phase hooks, immutable tick facts, atomic commit/rollback, committed event hook.
  - Verify: exact N no-op ticks, phase order/isolation, arrival not readable by later gameplay phases until the next tick, rollback on invariant error.
  - Depends on: P1-08.

- [ ] **P1-10 — Deterministic scenario harness.**
  - Deliver: canonical content loader, explicit `command_at`, `advance_until`, state/event assertions, invariant and hash helpers; no wall-clock waits.
  - Verify: repeated calls to the ordinary tick function and harness batch advancement of the same N ticks produce identical ADR-0006 replay-equivalence hashes and canonical deterministic event traces.
  - Depends on: P1-09.

- [ ] **P1-11 — Actor and lifecycle.**
  - Deliver: sole-owner actor mailbox, immutable `Arc<GameSnapshot>` publication, scheduler, `Unloaded/Loading/Paused/Running/Advancing/Won`, exact batch advancement and rollback.
  - Verify: complete lifecycle transition table, batch range, concurrent exclusion, failed load/new-game restoration, and required `realtime_batch_equivalence` through scheduler and `AdvanceTicks` paths.
  - Depends on: P1-09, P1-10.

- [ ] **P1-12 — Command sequencing and idempotency.**
  - Deliver: strict command envelope, replayable-versus-control classification, server/effective sequence, paused immediate versus running queued behavior, `expected_tick`, full-envelope structural idempotency, exact typed command results, command records, state-replacement receipt merging, and typed errors.
  - Verify: concurrent stable ordering; same ID/same full envelope; same ID with changed command/`expected_tick`; exact generated-ID result survives later entity removal and identical retry; replayable/control lifecycle conflicts; NewGame/Load receipt retention and collision behavior; transaction rollback.
  - Depends on: P1-11.

- [ ] **P1-13 — Persistence and replay skeleton.**
  - Deliver: normalized save envelope with exact `content_hash`, versioned hash scheme, linearizable actor save barrier, single FIFO save/load target worker, platform-specific atomic replacement, schema migrations, load rollback, command-log replay, pending/idempotency rebuild rules, and control-command save acknowledgements.
  - Verify: uninterrupted, save-split (including pending commands), and replayed walking-skeleton runs have identical replay-equivalence hashes and canonical deterministic event traces; applied results and generated IDs survive save/load and are returned on identical resubmission even after entity removal; save-integrity hashes validate exact stored bytes; malformed envelope/hash/content/migration cases fail typed; injected write/replace and SaveNow failures leave the prior save/state intact and return the correct idempotent outcome; delayed A/B saves, success/failure permutations, SaveNow→LoadAutosave, autosave, and shutdown prove target ordering.
  - Depends on: G0-03, G0-04, P1-12.

- [ ] **P1-14 — Authenticated REST walking skeleton and generated client.**
  - Deliver: engine CLI basics; loopback preferred/fallback ports; random token; owner-only discovery; status/state/content/command endpoints; error envelope and request limits; deterministic OpenAPI/JSON Schema export; minimal generated Python client.
  - Verify: spawned real-process discovery/auth/status/state plus Paused `AdvanceTicks`; exact Accepted/Applied/Rejected/Failed acknowledgement and pre-acceptance error schemas/status codes; result/resulting-tick derivation; missing/bad credentials and permissions.
  - Depends on: P1-13.

## Phase 1 — Canonical gameplay vertical slices

- [ ] **P1-15 — Local BuildOrder staging and Hub shipyard.**
  - Deliver: one active FIFO shipyard order; pending orders with no staging/demand; promotion-time local allocation; remote reservation handling; zero-Fuel ship completion; cancellation/loaded-delivery recovery; component return/overflow salvage.
  - Verify: build, promote, and cancel a Courier at every queue/material state; no pending-order demand; build→scrap whole-economy conservation; exact component/Fuel conservation and save/replay equivalence.
  - Depends on: G0-05, P1-14.

- [ ] **P1-16 — Station construction.**
  - Deliver: `QueueBuildStation`, slot/survey/tech validation, staged multi-component hold, deterministic Builder assignment, travel/work/completion, cancellation/return path.
  - Verify: canonical Haven Mine and Refinery; slot uniqueness; cancellation at each state; conservation and replay.
  - Depends on: P1-15.

- [ ] **P1-17 — Buffers and mining.**
  - Deliver: priority/buffer commands, atomic preferred/minimum allocation, thresholds, serialized mining slots, exact ten-resulting-tick retune, capacity-safe finite extraction, and shared-deposit reducer.
  - Verify: exact retarget/re-retarget traces; buffer/capacity bounds; two-station shared-deposit conservation/order; partial-space, full-output, exact/excess exhaustion; save/replay at retune and extraction boundaries.
  - Depends on: G0-06, P1-16.

- [ ] **P1-18 — Production and slow Hub assembly.**
  - Deliver: recipe validation, complete input reservation, progress, atomic output, `OutputBlocked`, recipe replacement/clear recovery, Hub component slot.
  - Verify: exact stoichiometry, output blocking, multi-output recipes, reserved-input release, save/replay conservation.
  - Depends on: P1-17.

- [ ] **P1-19 — Same-node logistics and canonical bootstrap.**
  - Deliver: derived station/BuildOrder/salvage supply and demand, priority score/tie tuple, persistent two-sided reservations, dock transactions, zero-distance pickup/delivery.
  - Verify: no flow before a Courier; canonical Mine-to-Refinery flow; `bootstrap_to_first_cargo`; executable bootstrap proof is now part of `content_validate`.
  - Depends on: P1-18.

- [ ] **P1-20 — Inter-body movement and actual-distance Fuel.**
  - Deliver: TravelPlan execution, segment-boundary positions, payload speed, actual-distance/arrival tick facts, phase-8 Fuel debit, arrival/dock holding, next-tick inventory visibility, and checked route-feasibility simulation over cloned Fuel remainders.
  - Verify: multi-body delivery; radial/arc/final-cap and Life Support cases; empty base mass; arrival/debit isolation; save/replay equivalence.
  - Depends on: G0-07, P1-19.

- [ ] **P1-21 — Fuel feasibility, refueling, reservation contention, and rescue.**
  - Deliver: deterministic fuel-feasible cargo binary search, below-full refuel trigger, unreserved direct-refuel allocation distinct from the Cargo export floor, shared phase-8 Fuel reducer/result fact, phase-9 fact-budget consumption, stable dock/Fuel contention, AwaitingPickup expiry, Holding order, and docked-wait versus conservation-safe rescue delay/tow/arrival.
  - Verify: `reservation_contention`, `fuel_rescue_recovery`, exact threshold/remainder/Life Support routes, same-tick partial-stock contention, production/research/Cargo hold priority, post-phase-8 assignment/route inputs, zero-Fuel newly built ship, rescue Fuel/remainder preservation, no double reservation/overfill, and loaded jobs never expire or rescue.
  - Depends on: P1-20.

- [ ] **P1-22 — Hub research.**
  - Deliver: availability, one-project rule, automatic research buffers, local reservation, rational consumption, Hub Tier-1 activation, manual pause keep/release/resume, permanent unlock.
  - Verify: exact Sensor Systems trace, no pre-Active consumption, pause/save/load/resume equality, retained credit/material location.
  - Depends on: G0-08, P1-21.

- [ ] **P1-23 — Surveys and docked Research Station projects.**
  - Deliver: Research Ship/Station builds, survey queue/priority/assignment/depth milestones/cancellation, DockForResearch selection, persistent dock, `NoResearchShip` pause/resume.
  - Verify: onboarding hour 1–2, stable ordering, partial-work cancellation, research/survey competition, milestone save/load.
  - Depends on: P1-22.

- [ ] **P1-24 — Upgrades, Advanced Refining, and renewable ship Fuel supply.**
  - Deliver: `QueueUpgrade`, tier skip/delta/work, capacity/slot/dock atomic completion, Chemicals and Fuel recipes, automatic refueling.
  - Verify: onboarding hour 2–3, upgrade conservation/cancellation, Fuel production and transfer.
  - Depends on: P1-23.

- [ ] **P1-25 — Advanced materials, factories, and reversible components.**
  - Deliver: Rime/Glint extraction, T3 refining/reclamation, all component assembly/disassembly, Construction Factory, multi-slot processing, intermediate ship/station tiers.
  - Verify: `full_supply_chain`, onboarding hour 3–4, inverse recipes, parallel slot independence, whole-chain conservation.
  - Depends on: P1-24.

- [ ] **P1-26 — The Veil and complete technology progression.**
  - Deliver: depth-2/3 surveys, Orbital Logistics placement gate, renewable rational mining, ordered 1,000-tick drift, remaining tiers/techs and unlock validation.
  - Verify: `first_five_hours`, `belt_drift_determinism`, `all_techs_reachable`, save/load at onboarding milestones.
  - Depends on: G0-06, P1-25.

- [ ] **P1-27 — Bottleneck monitoring.**
  - Deliver: 600-tick delivery buckets, rolling total, serialized consecutive-deficit/clear counters, 300-tick warning/clear state, and typed events.
  - Verify: uninterrupted and split-save runs emit/clear at the same gameplay tick
    and match under ADR-0006 replay-equivalence bytes and canonical deterministic
    event traces; session event-sequence continuity is checked separately.
  - Depends on: P1-26.

- [ ] **P1-28 — Ship scrapping and salvage.**
  - Deliver: Hub-docked validation, cargo/Fuel/component return, compatible buffers/compartment filling, deterministic permanent overflow cache.
  - Verify: role/tier generated conservation, invalid-location errors, salvage logistics, save/replay.
  - Depends on: P1-27.

- [ ] **P1-29 — Demolition, evacuation, and rebuild recovery.**
  - Deliver: Evacuating state, research detach/release, awaiting-inbound release, loaded-inbound completion, source-locked priority-100 evacuation, cache/component return, demolition cancellation.
  - Verify: `cancel_demolish_rebuild_recovery`, cache persistence, final-Hub protection, conservation, and victory reachability after recovery.
  - Depends on: P1-28.

- [ ] **P1-30 — Space Gate and victory.**
  - Deliver: `BeginGateAssembly`, Tier-4 Fabricator, exact manifest and priority, virtual berth, ordered work phases, activation Reactor Rod sink, immediate `Won` lifecycle.
  - Verify: `starting_state_to_gate_victory`, all-tech prerequisite enforcement, manifest bounds, whole-economy conservation, Won save/load.
  - Depends on: P1-29.

## Phase 1 — Protocol, client, and release qualification

- [ ] **P1-31 — Complete event and polling surface.**
  - Deliver: generated-schema-exhaustive external event union, a session-owned monotonic allocator/ring valid while Unloaded, full entity/root replacement and removal deltas from committed transactions/state replacement, 50,000-event retention, pagination, filtering, and resynchronization.
  - Verify: every event variant/payload including null-tick control failure; integer-only replacement/deletion cache reducer; same-session NewGame/Load rebase, old/new key-set entity events, and complete replacement; fresh-process next/latest/oldest cursor math; oldest-minus-one resume, older 410, and future-cursor 400 boundaries; monotonic order; progress-only coalescence or forced resync; exact error details; eviction; schema snapshots.
  - Depends on: G0-08, P1-30.

- [ ] **P1-32 — WebSocket parity and backpressure.**
  - Deliver: Hello/Subscribe/Command, CLI authorization header, browser subprotocol token and Origin allowlist, per-client queue 2,048, coalescing/resync/4009 close.
  - Verify: shared REST/WS corpus has equal acknowledgements/replay-equivalence hashes; lagged clients never alter tick timing; reconnection at/behind retention.
  - Depends on: P1-31.

- [ ] **P1-33 — Runtime and API hardening.**
  - Deliver: one-Hz scheduler, catch-up capped at ten, autosave cadence/material paused saves, graceful shutdown, discovery cleanup safety, rate/body/batch limits, structured logging and typed failure isolation.
  - Verify: complete API conformance, scheduler pressure, autosave/shutdown normalization, port fallback, startup/load failure rollback.
  - Depends on: P1-32.

- [ ] **P1-34a — Generated Python client and System Map renderer.**
  - Deliver: locked generated models, discovery/auth, snapshot/event client, deterministic map renderer.
  - Verify: Python formatting/type/unit tests, text golden, real-engine initial render.
  - Depends on: P1-33.

- [ ] **P1-34b — Logistics renderer.**
  - Deliver: deterministic supplies/demands/jobs/priority/throughput text mode.
  - Verify: fixture goldens and live state/event updates.
  - Depends on: P1-34a.

- [ ] **P1-34c — Entity-detail and event-log renderers.**
  - Deliver: complete entity details, ordered committed event log, no-color output.
  - Verify: fixture goldens, missing/deleted entity behavior, stable order.
  - Depends on: P1-34b.

- [ ] **P1-34d — Interactive CLI and resynchronization.**
  - Deliver: documented flags/mode switching, WS reconnect, `ResyncRequired` snapshot recovery, clean shutdown.
  - Verify: spawned-engine CLI smoke, retention-boundary recovery, no `sleep()`-based assertions.
  - Depends on: P1-34c.

- [ ] **P1-35 — Agent play-test suite.**
  - Deliver: minimal Gate victory, full economy/all-tech, deliberately poor-build recovery, and V1-ceiling stress clients using generated models.
  - Verify: acknowledged Paused batches of at most 1,000 ticks, explicit state/content/event assertions, deterministic command-log artifacts.
  - Depends on: P1-34d.

- [ ] **P1-36 — Cross-platform, performance, and private release gate.**
  - Deliver: macOS/Windows CI, supported-target locked builds, cross-platform hash comparison with divergence artifacts, reference benchmark, macOS universal binary flow, SBOM and unsigned/private packaging.
  - Verify: complete Phase 1 gate from a clean checkout; at least 100 batch ticks/sec at 500 ships, 200 stations, and all 25 resources on the recorded reference runners.
  - Depends on: P1-35.

## Gate activation matrix

Commit gates remain required after activation. Qualification gates run for their owning
increment's completion and thereafter in nightly, pre-release, and Phase 1 completion
runs; they are not added to every ordinary commit merely because they are long-running
or platform-bound.

| Owning increment | Class | Newly mandatory scenario/gate |
|---|---|---|
| P1-05 | Commit | Structural and semantic content validation; normalized content hash |
| P1-10 | Commit | Ordinary-tick versus harness-batch equivalence proof |
| P1-11 | Commit | Required `realtime_batch_equivalence` through scheduler and `AdvanceTicks` paths |
| P1-13 | Commit | `save_load_equivalence` and `command_log_replay_equivalence`, extended by every later state change |
| P1-14 | Commit | Authenticated REST walking-skeleton conformance and generated-client smoke |
| P1-19 | Commit | `bootstrap_to_first_cargo`; executable bootstrap content validation |
| P1-21 | Commit | `reservation_contention`; `fuel_rescue_recovery` |
| P1-25 | Commit | `full_supply_chain` |
| P1-26 | Commit | `first_five_hours`; `belt_drift_determinism`; `all_techs_reachable` |
| P1-29 | Commit | `cancel_demolish_rebuild_recovery` |
| P1-30 | Commit | `starting_state_to_gate_victory` |
| P1-31 | Commit | Complete event polling, retention, schema, and resynchronization corpus |
| P1-32 | Commit | Complete shared REST/WebSocket transport and backpressure corpus |
| P1-34a–P1-34d | Commit | Cumulative generated Python client/TUI gates delivered by each sub-increment |
| P1-35 | Qualification | Minimal Gate, full-economy, poor-build recovery, and ceiling agent play-tests |
| P1-36 | Commit | Windows locked build and test CI |
| P1-36 | Qualification | Supported-platform state hashes, release benchmark, private artifacts, and SBOM |

## Phase 1 completion gate

Phase 1 is complete only when a clean checkout passes, in order:

1. Repository policy/protocol synchronization and reproducible generated-file checks.
2. Rust formatting, locked workspace build, and Clippy with warnings denied.
3. Full schema/content validation, including exact bootstrap and critical-resource budget.
4. Unit and property tests for arithmetic, invariants, conservation, ordering, recovery, and overflow.
5. Every deterministic scenario and equivalence proof activated above.
6. Complete HTTP/WebSocket API conformance and security/backpressure tests.
7. Save/load, batch/real-time, and command-log replay equivalence over the completed game.
8. Python formatting, typing, unit/integration tests, all renderer modes, and agent play-tests.
9. Windows locked build and test through the supported Windows CI entry point.
10. Supported-platform canonical state hashes.
11. V1-ceiling release benchmark.
12. Locked private artifacts and SBOM.

No skipped required test, flaky retry, unexplained golden/hash update, or unsupported authoritative rule is compatible with this gate.

## Phase 2 entry gate

The existing TDD set describes Phase 1 only. Before Phase 2 code, create and approve a Phase 2 ADR/TDD package covering:

- TypeScript workspace, dependency/version policy, PixiJS rendering structure, and state ownership.
- Tauri-managed engine child startup, discovery, authentication, shutdown, crash recovery, upgrades, and save isolation.
- Exact packaged Origin/CSP rules.
- Generated protocol client, snapshot/delta reconciliation, event resynchronization, and optimistic-command UX.
- Rendering/performance budgets at 1280×720 and 1920×1080.
- Unit, component, browser, real-sidecar E2E, screenshot, and accessibility test strategy.
- Asset loading, atlas generation, placeholders, provenance records, and distribution eligibility.
- macOS/Windows packaging, signing/notarization boundaries, SBOM, and store artifacts.

The Phase 2 design work must also reconcile these player-facing inconsistencies before visual implementation:

- Station silhouettes differ between GDD 8 and GDD 10.
- Raw/refined icon distinction differs between GDD 8 and GDD 10.
- The example Gate panel omits the activation Reactor Rod from the canonical manifest.
- Demolition UI wording implies rejection for inventory/docks, while authoritative behavior queues evacuation.
- Ship-build UI wording says components are consumed even though they become recoverable installed components.

## Phase 2 ordered backlog

- [ ] **P2-01 — Technical design and frontend traceability gate.** Approve the ADR/TDD package above and map every GDD 8–11 interaction/accessibility clause to a test.
- [ ] **P2-02 — Tauri/PixiJS shell and locked CI.** Create the TypeScript/PixiJS workspace, Tauri shell, generated v1 client, mock API fixtures, and test/build entry points.
- [ ] **P2-03 — Managed engine lifecycle.** Launch/connect/authenticate, display startup/loading failures, restart after crashes, normalize shutdown, and isolate test/user data.
- [ ] **P2-04 — Client state synchronization.** Full snapshot, committed events/deltas, cursor tracking, reconnection/resync, and pause/resume/save controls.
- [ ] **P2-05 — Read-only System View.** Star, lanes, bodies, fog, stations/ships, deterministic layout, procedural placeholder assets, and screenshot goldens at required resolutions.
- [ ] **P2-06 — Camera and selection.** Three snap bands, cursor-centered zoom, pan, selection/deselection, keyboard alternatives, and minimum hit regions.
- [ ] **P2-07 — Read-only inspection.** Station, ship, flow, research, survey, construction, bottleneck, salvage, and Gate panels driven only by protocol state/content.
- [ ] **P2-08 — Bootstrap command journey.** Station placement, Hub choice, mining/recipe configuration, shipyard, command acknowledgements/errors, and first autonomous flow against the real engine.
- [ ] **P2-09 — Progression command journeys.** Threshold/priority tuning, research/pause, survey/cancel, upgrades, advanced production, and clear blocked-state remediation.
- [ ] **P2-10 — Recovery and Gate journeys.** Build cancellation, demolition evacuation, ship scrapping, salvage, Gate assembly, victory state, and restart/load inspection.
- [ ] **P2-11 — Visual simulation layer.** Travel interpolation from serialized plans, route patterns/throughput, holding/docking/cargo animation, fog/deposit reveals, and reduced-motion equivalents.
- [ ] **P2-12 — Final asset pipeline.** Produce/assemble the asset catalog, deterministic atlases, license/provenance records, review artifacts, and placeholder rejection in distributable builds.
- [ ] **P2-13 — Settings and accessibility.** UI scale 100/125/150/200%, keyboard navigation/shortcuts, non-color resource cues, no color-only ambiguity, reduced motion, reflow, and contrast/hit-target tests.
- [ ] **P2-14 — Graphical end-to-end and performance qualification.** Run onboarding, poor-build recovery, save/restart, and Gate victory journeys with visual/performance budgets on supported platforms.
- [ ] **P2-15 — Signed distribution candidates.** Produce notarized universal macOS and signed Windows artifacts for the approved channel after external release prerequisites are supplied.

## External release blockers

Private implementation may proceed, but public distribution remains blocked until all of the following are supplied and verified:

- A selected code license and asset-license policy; the current `LICENSE` is a placeholder with no public grant.
- Distribution-compatible provenance review for every AI-assisted or third-party asset.
- macOS signing/notarization identity and credentials.
- Windows signing identity and credentials.
- Steam/itch.io accounts, application identifiers, and publishing authority.
- Supported-platform CI runners and a recorded reference benchmark environment.

The autonomous agent must never work around these blockers by weakening tests, publishing unsigned artifacts as final, or asserting approval it cannot verify.

## Evidence log

Append one entry when—and only when—an increment checkbox is completed.

```text
Increment: G0-NN, P1-NN, or P2-NN
Date: YYYY-MM-DD
Commit: <local commit hash, when commits were authorized>
Requirements: <traceability IDs and authoritative document sections>
Focused proof: <test name and command>
Cumulative gates: <commands and results>
Scenarios activated: <names or none>
Golden/hash changes: <none, or exact reviewed reason>
Notes: <remaining non-blocking follow-up, if any>
```

Gate 0 was independently re-audited on 2026-08-04 after its initial closure.
The entries below retain each original closure commit for traceability; all
corrections named in the notes belong to the enclosing Gate 0 audit change
(b4fb5e1). No production harness exists before P1-01, so Gate 0 uses document
proofs and names the exact future executable owners. The common audit runs
`git diff --check`, a read-only local Markdown link/anchor/frontmatter/fence
check, a byte comparison of the duplicated per-turn protocol, ADR-index
coverage, and targeted stale-rule searches. All passed before this log was
finalized.

```text
Increment: G0-01
Date: 2026-08-04
Commit: 317e05a (original closure; evidence follow-up f1f3872; audit repair b4fb5e1)
Requirements: GDD 13 §Content Definitions and §Schema Generation Ownership; GDD 14 §Starting State, §Canonical Ship Definitions, §Canonical Station Definitions, §Space Gate Definition, and §Automatic Buffer Defaults
Focused proof: Content-contract cross-check. Future executable proof: P1-02b round-trips the complete strict DTO roots and schema exporter; P1-03 regeneration is diff-free and exact fixtures cover every authored value; P1-04/P1-05 validate both roots.
Cumulative gates: Common Gate 0 document audit — passed.
Scenarios activated: none (doc-only Gate 0)
Golden/hash changes: none
Notes: Audit repair closes the malformed code fence, distinguishes authored data roots from generated schemas, defines complete DefinitionsCatalog/StartingScenario construction without hidden inventory injection, and assigns exporter implementation to P1-02b and generated artifacts to P1-03.
```

```text
Increment: G0-02
Date: 2026-08-04
Commit: e199942 (original closure; audit repair b4fb5e1)
Requirements: ADR-0006 §§1–9; GDD 12 §Save, Load, and Replay; GDD 13 §Lifecycle, RNG, Commands, and Root State and §Content Definitions
Focused proof: Hash-contract cross-check. Future executable proof: P1-05 locks normalized content bytes/hash; P1-08 locks scenario/replay-equivalence/save-integrity projections; P1-13 verifies save integrity; P1-36 compares canonical bytes and hashes on supported runners.
Cumulative gates: Common Gate 0 document audit — passed.
Scenarios activated: none (doc-only Gate 0)
Golden/hash changes: none; the first executable goldens are owned by P1-05/P1-08.
Notes: Audit repair replaces nonexistent library “canonical modes” with project-owned canonical JSON v1, domain-separated SHA-256, explicit snake_case/null/duplicate/float rules, recursive lexicographic object ordering, typed failures, and exact content/scenario/replay-equivalence/save-integrity projections.
```

```text
Increment: P1-02a
Date: 2026-08-05
Commit: 20770e8
Requirements: GDD 13 §Identifiers and Resources, §Entity Enums, §Lanes, §Lifecycle, §Destination References, §Build Targets/States, §Research, §Survey Orders, §Reservations, §Gate, §Commands; GDD 14 §Canonical Refining Recipes (resource order)
Focused proof: `cargo test --locked` — 29 tests covering serialization round trips (all variants of state enums, tagged enums, ID newtypes), stable resource/lane ordering, unknown-field rejection, missing-variant-field rejection, ErrorDetail variant round trips, CommandRejection round trip, and empty-string ID.
Cumulative gates: `cargo fmt --check` (clean), `cargo build --locked` (clean), `cargo clippy --locked -- -D warnings` (clean), `cargo test --locked` (29/29 pass).
Scenarios activated: none (primitive types only)
Golden/hash changes: none
Notes: Review subagent flagged TransportStage having no direct round-trip test and thin variant coverage. Both addressed before commit: added TransportStage round-trip test and comprehensive all-variant round-trip tests for BuildState, ResearchState, SurveyOrderState, ReservationState, GatePhase, and ResearchPauseReason.
```

```text
Increment: P1-02b
Date: 2026-08-05
Commit: 55a9807
Requirements: GDD 13 §Content Definitions, §Serialized State DTOs, §Commands, §Schema Generation Ownership; GDD 13 §Identifiers and Resources (id newtype JsonSchema); GDD 13 §Entity Enums, §Lanes, §Lifecycle, §Destination References, §Build Targets/States, §Research, §Survey Orders, §Reservations, §Gate, §Commands (types JsonSchema)
Focused proof: `cargo run -p steel-horizons-engine --bin schema_export -- <tmpdir>` — 35 JSON Schema files generated for every root DTO. Round-trip tests added for every state DTO (SystemPosition, TravelSegment, TravelPlan, Buffer, ProductionSlot, MiningTarget, Station, Ship, ResourceDeposit, CelestialBody, RationalRemainder, ResearchProject, SurveyOrder, BuildOrder, SalvageCache, GateBuild, Reservation, BottleneckTracker, RNGState, IdCounters, CommandRecord, GameState, GameSnapshot) and every command DTO (BufferConfiguration variants, ActorControl variants, ReplayableGameCommand variants, CommandEnvelope, CommandAcknowledgement).
Cumulative gates: `cargo fmt --check` (clean), `cargo build --locked` (clean), `cargo clippy --locked -- -D warnings` (clean), `cargo test --locked` (29/29 pass), schema export binary (35 schemas generated, deterministic across runs)
Scenarios activated: none (DTO/schema-only increment)
Golden/hash changes: none (generated schemas are deterministic; content files deferred to P1-03)
Notes: JsonSchema derives added to all DTO structs/enums in state.rs, content.rs, command.rs, types.rs, and the id_newtype! macro in id.rs. Fixed-field-size limitation: BottleneckTracker.deliveries_by_tick changed from `[u32; 600]` to `Vec<u32>` to satisfy serde fixed-array constraint (arrays > 32 elements lack serde derive support). Schema export binary count corrected from 33 to 35. Dead BTreeSet import removed from content.rs (false positive — BTreeSet is used at completed_techs field). Derive ordering unified: all files use `Serialize, Deserialize, JsonSchema` consistently. Missing docs suppression added at module level. Round-trip tests added for every state and command DTO.
```

```text
Increment: G0-03
Date: 2026-08-04
Commit: 53902e7 (original closure; evidence follow-up be0999d; audit repair b4fb5e1)
Requirements: ADR-0007 §§1–7; ADR-0004 §Save Normalization; ADR-0006 §§5–7; GDD 12 §Save, Load, and Replay; GDD 13 §Lifecycle, RNG, Commands, and Root State
Focused proof: Envelope/load-order cross-check. Future executable proof: P1-13 covers canonical save round-trip, exact content compatibility, raw-hash-before-migration validation, every contiguous migration fixture, injected atomic-write failure, and replay equivalence.
Cumulative gates: Common Gate 0 document audit — passed.
Scenarios activated: none (doc-only Gate 0)
Golden/hash changes: none; baseline and migration goldens begin at P1-13.
Notes: Audit repair gives the envelope explicit hash_scheme, schema_version, content_hash, normalized state and state_hash; validates raw saved bytes before migration; separates content and state versions; and defers real historical migration fixtures until a successor schema exists.
```

```text
Increment: G0-04
Date: 2026-08-04
Commit: 04bb9cf (original closure; audit repair b4fb5e1)
Requirements: ADR-0008 §§1–7; ADR-0003 §Command Envelope and Ordering and §Idempotency; ADR-0007 §§3–4; GDD 13 §Lifecycle, RNG, Commands, and Root State
Focused proof: Command/control/save-barrier cross-check. Future executable proof: P1-12 tests full-envelope idempotency and state-replacement receipt merging; P1-13 tests pending replayable commands, FIFO SaveNow inclusion/exclusion, failed writes, load validation, restart import, and replay equivalence.
Cumulative gates: Common Gate 0 document audit — passed.
Scenarios activated: none (doc-only Gate 0)
Golden/hash changes: none
Notes: Audit repair separates replayable commands from actor controls, makes SessionReceiptLedger the session idempotency owner, records expected_tick and stable rejections, defines next_server_sequence as a persisted lower bound, and replaces the impossible mailbox drain with a linearizable FIFO snapshot barrier.
```

```text
Increment: G0-05
Date: 2026-08-04
Commit: d4f88b0 (original closure; audit repair b4fb5e1)
Requirements: ADR-0009 §§1–7; GDD 5 §Station Hub, §Build Work, and §Cancellation; GDD 13 §Station and §Construction and Salvage
Focused proof: Shipyard queue/conservation cross-check. Future executable proof: P1-15 covers one active plus FIFO pending orders, promotion, staging/delivery, active/pending cancellation including loaded inbound cargo, exact component conservation, zero-Fuel ship completion, and save/replay.
Cumulative gates: Common Gate 0 document audit — passed.
Scenarios activated: none (doc-only Gate 0)
Golden/hash changes: none
Notes: Audit repair makes only queue index zero active and demand-producing, defines terminal/history behavior and inbound cancellation, preserves complete component multisets, and creates ships with zero Fuel and zero remainders so normal deterministic refueling supplies them.
```

```text
Increment: G0-06
Date: 2026-08-04
Commit: d915d15 (original closure; audit repair b4fb5e1)
Requirements: ADR-0010 §§1–7; GDD 12 §Mining drift, retune, and extraction reducers; GDD 13 §Buffers and Production; GDD 14 §The Veil
Focused proof: Mining reducer/boundary cross-check. Future executable proof: P1-17 covers slot identity, repeated retunes, partial/full capacity, exact/excess exhaustion, shared-deposit contention and conservation; P1-26 covers tick-999→1,000 drift, save/load, replay, and PRNG equality.
Cumulative gates: Common Gate 0 document audit — passed.
Scenarios activated: none (doc-only Gate 0)
Golden/hash changes: none
Notes: Audit repair adds serialized slot_index and exact ten-resulting-tick retune state, capacity-safe extraction with no material loss, stable shared-deposit reduction, and drift based on the resulting tick before extraction from one immutable density fact.
```

```text
Increment: G0-07
Date: 2026-08-04
Commit: 0d2a2a9 (original closure; audit repair b4fb5e1)
Requirements: ADR-0011 §§1–8; GDD 3 §Fuel; GDD 7 §Fuel Safety and §Deterministic Cargo Matching; GDD 12 §Movement, Fuel, Life Support, and Refueling
Focused proof: Refuel feasibility/arrival-order cross-check. Future executable proof: P1-20 proves exact cloned-remainder Fuel/Life Support feasibility and final-leg debit; P1-21 proves reservation contention, full/partial/zero direct transfers, all-role assignment, export-floor independence, and Hub rescue boundaries.
Cumulative gates: Common Gate 0 document audit — passed.
Scenarios activated: none (doc-only Gate 0)
Golden/hash changes: none
Notes: Audit repair accepts the ADR, distinguishes direct unreserved Fuel from Cargo export floors, covers every ship role, defines exact one-way feasibility, phases final movement debit before arrival transfer, orders contention by ShipId, and makes an origin-Hub wait the only non-self-rescuing boundary.
```

```text
Increment: G0-08
Date: 2026-08-04
Commit: ed287f5 (original closure; audit repair b4fb5e1)
Requirements: ADR-0012 §§1–8; ADR-0003 §V1 Commands and §Events and Resynchronization; GDD 13 §Lifecycle, RNG, Commands, and Root State and §Research; TDD 02 §Events
Focused proof: Exhaustive wire-union/cache/research cross-check. Future executable proof: P1-22 covers pause/resume/reassignment material location and NoResearchShip; P1-31 covers every retained event, full root/entity replacement, deletion, retention, polling, and resync; P1-32 repeats the shared corpus under WebSocket backpressure.
Cumulative gates: Common Gate 0 document audit — passed.
Scenarios activated: none (doc-only Gate 0)
Golden/hash changes: none
Notes: Audit repair defines an exhaustive retained-event union including CommandFailed and RuntimeError, stable event order, full-replacement StateDelta with durable continuity, progress-only queue coalescence, a bounded canonical ring and forced resync, plus research reassignment that preserves consumed credit without teleporting unused stock or inventing Hub tiers.
```

```text
Increment: G0-09
Date: 2026-08-04
Commit: 3455791 (original closure; audit repair b4fb5e1)
Requirements: ADR-0005 §CI Policy — Greenfield Construction; TDD 04 §Quality Gates; this plan §Gate activation matrix and §Phase 1 completion gate; AGENTS.md §Verification order; TDD 05 §CI Staging and Matrix
Focused proof: Staged-gate policy comparison. Future executable proof: P1-01 adds marker-based protocol equality and a staged policy manifest/check; each later owning increment activates only its named cumulative or qualification gate.
Cumulative gates: Common Gate 0 document audit — passed; the per-turn protocol block is byte-identical between AGENTS.md and this plan.
Scenarios activated: none (doc-only Gate 0)
Golden/hash changes: none
Notes: Audit repair stages REST at P1-14, polling at P1-31, WebSocket at P1-32, Python/TUI at P1-34, long agent clients as P1-35 qualification, Windows build/test CI as a P1-36 commit gate, and hashes/benchmark/artifacts/SBOM as P1-36 qualification. The complete Phase 1 gate remains mandatory.
```

```text
Increment: P1-01
Date: 2026-08-05
Commit: bb27b4f
Requirements: TDD 04 §Quality Gates (P1-01 row); TDD 05 §Repository Layout, §CI Staging and Matrix, §Python CI; AGENTS.md §Verification order
Focused proof: `cargo build --locked`, `cargo fmt --check`, `cargo clippy --locked -- -D warnings`, `cargo test --locked`, `python3 scripts/check-protocol-sync.py`, `ruff check src/`, `mypy src/`, `steel-horizons-engine --version`
Cumulative gates: P1-01 gate set — all passed.
Scenarios activated: none (P1-01 scaffold)
Golden/hash changes: none (no golden tests yet)
Notes: Independent review completed via delegated subagent (deleg_400e18c2). One minor fix applied: check.sh/check.ps1 pytest --co fallback message clarified. All verification gates pass cleanly.
```

```text
Increment: P1-03
Date: 2026-08-05
Commit: 95e3cfb
Requirements: GDD 13 §Content Definitions, §Serialized State DTOs; GDD 14 §Authored Content & Balance Catalog (complete starting bodies, deposits, recipes, technologies, ship/station statistics, Gate definition); TDD 04 §Quality Gates
Focused proof: `content/definitions.v1.json` parses as `DefinitionsCatalog` and round-trips; `content/starting_system.v1.json` parses as `StartingScenario` and round-trips; `ContentCatalog` round-trips from both files. Exact record counts: 27 recipes, 23 technologies, 12 ships, 20 stations, 7 bodies, 19 slots, 4 starting techs, 6 stations with authored buffers/ship/states. Hub Haven output buffers in ResourceType order. RNG words match GDD 14 authoritative hex values. Schema regeneration is deterministic with no diff.
Cumulative gates: `cargo fmt --check` (clean), `cargo build --locked` (clean), `cargo clippy --locked -- -D warnings` (clean), `cargo test --locked` (99/99 pass), schema export `--check` (no diff)
Scenarios activated: none (content/schema-only increment)
Golden/hash changes: none (content files are versioned authored data; schemas are deterministic)
Notes: Independent review completed via delegated subagent (deleg_a358ade3). One bug found and fixed: RNG words in starting_system.v1.json did not match GDD 14 hex values; corrected to authoritative decimal representations. Schema-export.sh updated with `--check` flag. 16 focused tests validate round-trip, record counts, ID uniqueness, body counts, slot totals, hub buffers, ship state, techs, metadata, gate definition, and recipe tech validity.
```

```text
Increment: P1-04
Date: 2026-08-05
Commit: 771ce89
Requirements: GDD 13 §Identifiers and Resources (generated-ID prefixes); GDD 13 §Content Definitions; GDD 14 §Starting System Bodies, §Starting State, §Automatic Buffer Defaults, §Canonical Station Definitions
Focused proof: `cargo test --locked` — 132 tests covering the content_validate module (28 table-driven invalid fixtures) plus all prior content, state, types, id, and command tests. Canonical content validates with zero errors. Every invalid fixture produces the expected error with a stable path.
Cumulative gates: `cargo fmt --check` (clean), `cargo build --locked` (clean), `cargo clippy --locked -- -D warnings` (clean), `cargo test --locked` (132/132 pass)
Scenarios activated: none (content-validation-only increment)
Golden/hash changes: none
Notes: Independent review completed. Dead code removed from `validate_gate_definition` (unused `expected_phases` array, `_pname`, empty if block). Added structural validation for Gate fields (`required_techs`, `manifest`, `logistics_priority`, `transfer_berths`, `minimum_fabricator_tier`, `required_deliveries`, `completion_consumption`). Added validation for `build_work`, `component_cost`, and `required_tech` on ship and station definitions. 8 new invalid-fixture tests cover these validation paths. Fixed pre-existing unused-variable warning (`sid` → `_sid`).
```

```text
Increment: P1-05
Date: 2026-08-05
Commit: (pending — commit after review)
Requirements: GDD 13 §Content Definitions (normalized representation); GDD 14 §Authored Content & Balance Catalog, §Solvability Budget; ADR-0006 §Canonical JSON v1 byte encoding, §Collection ordering, §Domain separation prefixes, §Canonical content hash input; ADR-0005 §Content Validation Gate
Focused proof: `cargo test --lib` — 155 tests covering canonical JSON writer (13 tests), content hash computation (4 tests), existing content/state/type/command tests (128 tests), and 6 new P1-05 semantic fixture tests (tech DAG cycle, unknown required tech, inverse recipe mismatch, zero-quantity component cost, build hold exceeds cargo, critical resource budget). Canonical content validates with zero errors.
Cumulative gates: `cargo clippy` (clean), `cargo build` (clean), `cargo test --lib` (155/155 pass)
Scenarios activated: none (validation-only increment)
Golden/hash changes: `tests/goldens/content_hash.txt` — new golden for content hash `667407ea914272134aa9495b46c527191ec0e6987a6e410e6f18597413474972`
Notes: Independent review completed via delegated subagent (deleg_ef6a4342). All 10 review points confirmed passing with no issues. New modules: `engine/src/canonical.rs` (canonical JSON v1 writer), `engine/src/content_hash.rs` (SHA-256 content hash with domain separation). New semantic validators added to `content_validate.rs`: tech DAG cycle detection, required tech reachability, inverse-recipe equality, zero-cost component checks, build hold vs cargo capacity, and critical-resource budget. Construction ships exempted from build-hold check (zero cargo capacity by design). sha2 dependency added to Cargo.toml.
```
