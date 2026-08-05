---
status: Approved
owner: Tech Lead
date: 2026-08-04
---

# Testing Strategy

## Quality Gates

### Phase 1 Greenfield Construction

During Phase 1 development, every commit runs all gates whose owning increment
has completed, except for the explicitly long-running P1-35 agent play-test
clients. Ordinary activated gates remain cumulative. Infrastructure and
qualification checks are staged as follows:

| Owning increment | Gate or qualification check activated |
|---|---|
| P1-01 | Locked Rust build, formatting, Clippy with warnings denied, Rust smoke/unit tests, Python package formatting/typing/unit smoke, protocol/policy-sync checks, and macOS CI |
| P1-03–P1-05 | Exact content fixtures, structural validation, then semantic validation and normalized content hash |
| Every behavioral increment | Its focused unit/property tests plus every previously activated unit/property suite |
| P1-10 onward | Only the deterministic scenarios activated by the implementation plan's gate activation matrix |
| P1-13 | Save/load and command-log replay equivalence, extended by every later serialized-state change |
| P1-14 | Authenticated REST walking-skeleton conformance and focused generated-client smoke tests; no WebSocket requirement yet |
| P1-31 | HTTP event polling, retention, exhaustive event schema, and resynchronization conformance |
| P1-32 | WebSocket authentication, shared REST/WS corpus, backpressure, reconnect, and parity conformance |
| P1-33 | Scheduler/runtime hardening, autosave, shutdown, limits, and failure-isolation conformance |
| P1-34a–P1-34d | Cumulative generated Python client/TUI formatting, typing, unit, renderer, integration, and resynchronization gates |
| P1-35 | Fast deterministic tests/fixtures become cumulative; all four long-running Python agent play-tests are required for P1-35 completion and thereafter nightly, pre-release, and at Phase 1 completion, not on every ordinary commit |
| P1-36 ordinary gate | Windows locked build and test CI |
| P1-36 qualification | Supported-platform state hashes, V1-ceiling benchmark, locked private artifacts, and SBOM; required to complete P1-36 and thereafter in qualification runs, not on every ordinary commit |

Run active gates in increasing scope: repository/generated-file checks; Rust
format/build/Clippy; content validation; unit/property and active Python package
tests; activated scenarios; persistence/replay; active REST/polling/WS/runtime
conformance; the P1-34 Python client/TUI gate; and finally the P1-36
Windows locked build and test gate.

The P1-35 qualification invocation runs the four long agent play-tests after
the ordinary active gates. Subsequent ordinary commits do not repeat those four
clients; nightly, pre-release, and Phase 1 completion runs do.

The P1-36 qualification invocation then adds supported-platform canonical
state hashes, the V1-ceiling benchmark, private artifacts, and the SBOM. These
checks are required to complete P1-36 and in later qualification runs, not on
every ordinary commit.

macOS CI starts at P1-01 and runs every Phase 1 commit. P1-01 provides
platform-neutral commands and records the future Windows target; the Windows CI
job is added only at P1-36. A nightly job may repeat long active gates, but it
must not claim a future unimplemented gate.

### Phase 1 Completion Gate

At Phase 1 completion, the full gate defined in the implementation plan is mandatory, in order:

1. Repository policy/protocol synchronization and reproducible generated-file checks.
2. Rust formatting, locked workspace build, and Clippy with warnings denied.
3. Full schema/content validation, including exact bootstrap and critical-resource budget.
4. Unit and property tests for arithmetic, invariants, conservation, ordering, recovery, and overflow.
5. Every deterministic scenario and equivalence proof activated by the implementation plan's gate activation matrix.
6. Complete HTTP/WebSocket API conformance and security/backpressure tests.
7. Save/load, batch/real-time, and command-log replay equivalence over the completed game.
8. Python formatting, typing, unit/integration tests, all renderer modes, and agent play-tests.
9. Windows locked build and test through the supported Windows CI entry point.
10. Supported-platform canonical state hashes.
11. V1-ceiling release benchmark.
12. Locked private artifacts and SBOM.

After P1-35 completes, nightly and pre-release runs include its four agent
play-tests. After P1-36 completes, those runs also include the cross-platform
hash and benchmark gates. A Phase 1 pre-release always runs the complete gate
above.

## Content Validation

`cargo run -p steel-horizons-engine --bin content_validate -- content/` validates the exact GDD 14 JSON. Its structural and semantic checks activate at P1-04/P1-05. It fails on graph cycles, unreachable recipe requirements, duplicate IDs, invalid authored schema, wrong slot totals, or insufficient critical-resource budget. The executable bootstrap sequence joins this same gate at P1-19, when the required simulation exists.

The validator exports a deterministic normalized content hash used in saves and golden tests.

## Unit Tests

### Correct Standard Accumulation

```rust
#[test]
fn one_unit_per_ten_ticks_is_exact() {
    let increment = MilliRemainder::increment_for(1, 10).unwrap();
    assert_eq!(increment, 100);

    let mut rem = MilliRemainder::default();
    let produced: u32 = (0..1_000)
        .map(|_| rem.add_increment(increment).unwrap())
        .sum();

    assert_eq!(produced, 100);
    assert_eq!(rem.value(), 0);
}
```

Tests also use increments above 1,000 and assert the quotient can exceed one; no implementation may subtract the threshold only once.

### Exact Research Consumption

For each authored technology/resource, run its complete duration and assert:

- Sum consumed equals required exactly.
- Remainder is zero on completion.
- Pause/save/load/resume produces the same consumption trace.
- No resource is consumed before Ready/Active.

### Movement and Fuel

Tests cover radial-only, arc-only, combined, the 6,283 wrap boundary,
zero-distance, Cargo/Construction payload penalties, the Research zero-capacity
branch, lane rationals, rescue tow, final-segment capping, and base-mass Fuel for
empty ships. Refuel feasibility must equal actual movement when Fuel and Life
Support remainders are nonzero and the technology set is unchanged; a
mid-route permanent Life Support unlock may only reduce the actual debit.
Arrival tests charge the final leg before
transfer, protect AwaitingPickup Fuel reservations, order contending arrivals by
ShipId, distinguish Cargo export floors from direct refueling, and cover full,
partial, zero, and same-station transfers. A 3,000 milli-speed ship moves 30,000
milli-units over ten unobstructed baseline ticks—not ten units caused by a
single subtraction.

### Properties and Invariants

Generated tests cover:

- Every accumulator remainder stays below its denominator.
- Conservation across deposits, buffers, cargo, reservations, production, research, builds, salvage, and sinks.
- BuildOrder staging, multi-component build holds, Gate delivery, and source-locked demolition evacuation conserve exact maps.
- Survey-order and automatic research-docking assignment obey stable ordering.
- Automatic buffer allocation is atomic and never exceeds station capacity.
- Scores and tie-breaks are independent of input map insertion order.
- Cancelling/demolishing/scrapping preserves component multisets.
- Checked arithmetic returns typed errors near bounds.
- xoshiro256** matches fixed golden vectors.

## Scenario Harness

```rust
let mut scenario = ScenarioRunner::from_starting_system()?;
scenario.command_at(0, queue_haven_mine())?;
scenario.advance_until(|s| s.station_exists("mine_haven_1"), 500)?;
scenario.command(queue_haven_refinery())?;
scenario.command(queue_first_courier())?;
scenario.advance_until(|s| s.has_ship_role(ShipRole::Cargo), 1_000)?;
scenario.assert_invariants()?;
```

Commands are scheduled explicitly; the harness never sleeps.

### CI-Required Scenarios

| Scenario | Principal assertion |
|----------|---------------------|
| `bootstrap_to_first_cargo` | Exact onboarding sequence is reachable from authored tick 0 |
| `first_five_hours` | Every prescribed recipe/build/research is legal and reachable |
| `full_supply_chain` | Raw → refined → component conservation and output |
| `all_techs_reachable` | Every technology can complete from authored resources |
| `starting_state_to_gate_victory` | Full victory from the actual starting fixture |
| `cancel_demolish_rebuild_recovery` | Poor reversible choices do not remove victory reachability |
| `fuel_rescue_recovery` | Zero reachable ordinary Fuel returns ship safely |
| `reservation_contention` | No double-booking/overfill under many idle ships |
| `belt_drift_determinism` | Cross-run PRNG/state equality |
| `realtime_batch_equivalence` | N scheduler ticks equal `AdvanceTicks(N)` under replay-equivalence bytes/trace |
| `save_load_equivalence` | Split run equals uninterrupted run under replay-equivalence bytes/trace |
| `command_log_replay_equivalence` | Replayed log bytes/trace equal the original |

Scenario fixtures provide only setup variations; canonical content remains under `content/`.

Every equivalence row compares ADR-0006 replay-equivalence bytes plus the canonical
deterministic event trace. Session control receipts, raw event sequence values, and
cursor rebasing are verified by the API corpus instead of polluting gameplay equality.

## API Tests

API tests bind a random loopback port and use the discovery/token flow. P1-14
starts with REST-only walking-skeleton cases, P1-31 adds event polling, and
P1-32 runs the shared command corpus once through REST and once through WS, then
compares state hashes and acknowledgement metadata.

```rust
#[tokio::test]
async fn paused_advance_ticks_is_exact() {
    let server = TestServer::starting_system().await?;
    let before = server.get_state().await?;

    let ack = server.command(envelope("advance-5", json!({
        "type": "AdvanceTicks", "count": 5
    }))).await?;
    assert_eq!(ack.status, ReceiptStatus::Applied);
    assert_eq!(ack.effective_tick, Some(before.tick));
    assert_eq!(ack.result, Some(CommandResult::AdvanceTicksCompleted {
        resulting_tick: before.tick + 5,
    }));
    assert_eq!(ack.resulting_tick, Some(before.tick + 5));

    let after = server.get_state().await?;
    assert_eq!(after.tick, before.tick + 5);
    assert_eq!(after.lifecycle, GameLifecycle::Paused);
}
```

`Resume` tests assert only the lifecycle acknowledgement; they do not assume a tick races ahead before the HTTP response.

Required protocol cases include authorization, Origin denial, idempotency,
expected-tick conflicts, lifecycle 409s, exact acknowledgement/result/error
shapes and status mappings, persisted generated-ID retry results, the exhaustive event union,
integer-only full-replacement StateDelta, concurrent sequence ordering,
retention-boundary resynchronization, permitted progress coalescence,
forced resync/4009 on continuity loss, lagged WS clients, save normalization,
and graceful shutdown. Each case joins the cumulative gate at its owning
P1-14/P1-31/P1-32/P1-33 increment.

## Agent Play-Tests

P1-35's four long-running Python clients are required to complete that
increment and then run nightly, pre-release, and at the Phase 1 completion
gate. They are not ordinary every-commit gates. Fast deterministic unit/schema
tests and reusable fixtures introduced by P1-35 remain cumulative. The clients:

1. Discover and authenticate.
2. Start/load the canonical scenario.
3. Keep the lifecycle Paused while planning.
4. Send explicit batches and wait for acknowledgements.
5. Assert content/state/events using generated models.

No test uses `time.sleep()` as a proxy for game ticks.

## Cross-Platform Determinism

P1-36 activates this gate. The same command-log fixture runs on macOS ARM,
macOS x86, and Windows x86. CI compares ADR-0006 replay-equivalence canonical bytes
and hashes. A mismatch uploads both states and the first divergent canonical
deterministic event; runtime session cursors are not cross-process identity.

## Performance Benchmarks

P1-36 activates the reference stress fixture containing 500 ships, 200
stations, and all 25 ResourceType values. Release builds must sustain at least
100 batch ticks/sec at p95 on reference CI runners. Benchmarks report allocation
count, tick duration by phase, snapshot serialization time, and event-queue
pressure.

## TDD Workflow

1. Add/identify the failing unit, scenario, content, or protocol test.
2. Confirm it fails for the intended reason.
3. Implement the smallest behavioral change.
4. Run the focused and full applicable suites.
5. Refactor while keeping deterministic state hashes stable or intentionally updating reviewed goldens.

## Related ADRs

- ADR-0005 — Test Architecture
