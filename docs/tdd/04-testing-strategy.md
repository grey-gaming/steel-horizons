---
status: Approved
owner: Tech Lead
date: 2026-08-04
---

# Testing Strategy

## Quality Gates

### Phase 1 Greenfield Construction

During Phase 1 development, every commit runs all gates activated so far, in order:

1. Rust formatting and Clippy with warnings denied.
2. Content/schema validation.
3. Unit/property tests.
4. Every deterministic scenario activated so far (see the scenario activation matrix in the implementation plan). Scenarios whose owning increment has not yet completed are not required.
5. HTTP/WS API conformance tests — once the API transport exists (P1-14), these become mandatory on every commit.
6. Save/load and command-log replay equivalence — once persistence exists (P1-13), these become mandatory on every commit.

Platform CI: macOS runs every commit during Phase 1. Windows CI is added at P1-36.

### Phase 1 Completion Gate

At Phase 1 completion, the full gate defined in the implementation plan is mandatory, in order:

1. Rust formatting and Clippy with warnings denied.
2. Full schema/content validation, including exact bootstrap and critical-resource budget.
3. Unit and property tests for arithmetic, invariants, conservation, ordering, recovery, and overflow.
4. All twelve deterministic scenarios.
5. Complete HTTP/WebSocket API conformance and security/backpressure tests.
6. Save/load, batch/real-time, and command-log replay equivalence over the completed game.
7. Python formatting, typing, unit/integration tests, all renderer modes, and agent play-tests.
8. Supported-platform canonical state hashes.
9. V1-ceiling release benchmark.
10. Locked private artifacts and SBOM.

Nightly/pre-release additionally runs Python agent play-tests, cross-platform golden-state comparison, and V1-ceiling benchmarks.

## Content Validation

`cargo run -p steel-horizons-engine --bin content_validate -- content/` validates the exact GDD 14 JSON. It fails on graph cycles, unreachable recipe requirements, duplicate IDs, invalid authored schema, wrong slot totals, insufficient critical-resource budget, or a bootstrap sequence that cannot build the first Cargo Ship.

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

Tests cover radial-only, arc-only, combined, the 6,283 wrap boundary, zero-distance, Cargo/Construction payload penalties, the Research zero-capacity branch, lane rationals, rescue tow, final-segment capping, and base-mass fuel for empty ships. A 3,000 milli-speed ship moves 30,000 milli-units over ten unobstructed baseline ticks—not ten units caused by a single subtraction.

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
| `realtime_batch_equivalence` | N scheduler ticks equal `AdvanceTicks(N)` |
| `save_load_equivalence` | Split run equals uninterrupted run |
| `command_log_replay_equivalence` | Replayed log state hash equals original |

Scenario fixtures provide only setup variations; canonical content remains under `content/`.

## API Tests

API tests bind a random loopback port and use the discovery/token flow. A shared command corpus is run once through REST and once through WS, then state hashes and acknowledgement metadata are compared.

```rust
#[tokio::test]
async fn paused_advance_ticks_is_exact() {
    let server = TestServer::starting_system().await?;
    let before = server.get_state().await?;

    let ack = server.command(envelope("advance-5", json!({
        "type": "AdvanceTicks", "count": 5
    }))).await?;
    assert_eq!(ack.effective_tick, before.tick);
    assert_eq!(ack.resulting_tick, Some(before.tick + 5));

    let after = server.get_state().await?;
    assert_eq!(after.tick, before.tick + 5);
    assert_eq!(after.lifecycle, GameLifecycle::Paused);
}
```

`Resume` tests assert only the lifecycle acknowledgement; they do not assume a tick races ahead before the HTTP response.

Required protocol cases include authorization, Origin denial, idempotency, expected-tick conflicts, lifecycle 409s, integer delta schema, concurrent sequence ordering, event retention/resync, lagged WS clients, save normalization, and graceful shutdown.

## Agent Play-Tests

Python clients:

1. Discover and authenticate.
2. Start/load the canonical scenario.
3. Keep the lifecycle Paused while planning.
4. Send explicit batches and wait for acknowledgements.
5. Assert content/state/events using generated models.

No test uses `time.sleep()` as a proxy for game ticks.

## Cross-Platform Determinism

The same command-log fixture runs on macOS ARM, macOS x86, and Windows x86. CI compares a canonical state hash that excludes presentation data and normalizes map ordering. A mismatch uploads both states and the first divergent event sequence.

## Performance Benchmarks

The reference stress fixture contains 500 ships, 200 stations, and all 25 ResourceType values. Release builds must sustain at least 100 batch ticks/sec at p95 on reference CI runners. Benchmarks report allocation count, tick duration by phase, snapshot serialization time, and event-queue pressure.

## TDD Workflow

1. Add/identify the failing unit, scenario, content, or protocol test.
2. Confirm it fails for the intended reason.
3. Implement the smallest behavioral change.
4. Run the focused and full applicable suites.
5. Refactor while keeping deterministic state hashes stable or intentionally updating reviewed goldens.

## Related ADRs

- ADR-0005 — Test Architecture
