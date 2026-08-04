---
status: Approved
owner: Tech Lead
date: 2026-08-04
---

# Simulation Engine Design

## Workspace Layout

```text
steel-horizons/
├── Cargo.toml
├── engine/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── main.rs
│   │   ├── actor.rs
│   │   ├── lifecycle.rs
│   │   ├── command.rs
│   │   ├── content/
│   │   ├── model/
│   │   ├── simulation/
│   │   │   ├── tick.rs
│   │   │   ├── arithmetic.rs
│   │   │   ├── travel.rs
│   │   │   ├── logistics.rs
│   │   │   └── phases/
│   │   ├── api/
│   │   ├── persistence/
│   │   └── rng.rs
│   └── tests/
│       ├── scenarios.rs
│       └── api.rs
├── content/
│   ├── definitions.v1.json
│   └── starting_system.v1.json
├── text-ui/
│   └── pyproject.toml
└── tests/
    ├── fixtures/
    └── playtest/
```

`engine` builds both a library and `steel-horizons-engine` binary. The Python text UI is not a Cargo workspace member.

## Simulation Actor

```rust
pub struct SimulationActor {
    state: GameState,
    content: Arc<ContentCatalog>,
    pending_commands: BTreeMap<u64, Vec<SequencedCommand>>,
    event_store: EventStore,
    snapshot_tx: watch::Sender<Arc<GameSnapshot>>,
}
```

The actor mailbox accepts `SubmitCommand`, `SchedulerTick`, `GetSnapshot`, and `Shutdown`. Only actor methods receive `&mut GameState`. HTTP/WS tasks wait on oneshot acknowledgements and consume immutable snapshots/events.

## Arithmetic Types

### Standard Rate Accumulator

```rust
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct MilliRemainder(u32); // invariant: < 1000

impl MilliRemainder {
    pub fn add_increment(&mut self, increment: u32) -> Result<u32, ArithmeticError> {
        let total = self.0.checked_add(increment).ok_or(ArithmeticError::Overflow)?;
        let whole = total / 1000;
        self.0 = total % 1000;
        Ok(whole)
    }

    pub fn increment_for(quantity: u32, cycle_ticks: u32) -> Result<u32, RateError> {
        let scaled = quantity.checked_mul(1000).ok_or(RateError::Overflow)?;
        if cycle_ticks == 0 || scaled % cycle_ticks != 0 {
            return Err(RateError::UseRationalAccumulator);
        }
        Ok(scaled / cycle_ticks)
    }
}
```

For one unit per ten ticks, `increment_for(1, 10) == 100`; ten calls to `add_increment(100)` produce exactly one unit.

### Rational Research Consumption

```rust
pub fn consume_tick(
    remainder: &mut u64,
    total_required: u64,
    total_ticks: u64,
) -> Result<u64, ArithmeticError> {
    let total = remainder.checked_add(total_required).ok_or(ArithmeticError::Overflow)?;
    let consumed = total / total_ticks;
    *remainder = total % total_ticks;
    Ok(consumed)
}
```

Each resource owns its remainder. Completion asserts consumed total equals required total and every remainder is zero.

### Fuel

```rust
pub fn consume_fuel(
    remainder: &mut u64,
    efficiency_remainder: &mut u8,
    actual_distance_milli: u64,
    base_mass: u64,
    payload_amount: u64,
    life_support_discount: bool,
) -> Result<u64, ArithmeticError> {
    let mass = base_mass.checked_add(payload_amount).ok_or(ArithmeticError::Overflow)?;
    let raw = actual_distance_milli.checked_mul(mass).ok_or(ArithmeticError::Overflow)?;
    let increment = if life_support_discount {
        let discounted = raw.checked_mul(4).ok_or(ArithmeticError::Overflow)?
            .checked_add(u64::from(*efficiency_remainder)).ok_or(ArithmeticError::Overflow)?;
        *efficiency_remainder = (discounted % 5) as u8;
        discounted / 5
    } else {
        raw
    };
    let total = remainder.checked_add(increment).ok_or(ArithmeticError::Overflow)?;
    let consumed = total / 10_000_000;
    *remainder = total % 10_000_000;
    Ok(consumed)
}
```

The movement phase records actual capped movement per ship; the fuel phase consumes from that record.

## Tick Implementation

```rust
pub fn advance_one_tick(&mut self) -> Result<CommittedTick, SimulationError> {
    let tick = self.state.tick;
    let mut tx = TickTransaction::new(tick);

    self.apply_scheduled_commands(&mut tx)?;
    self.move_ships(&mut tx)?;
    self.process_stations(&mut tx)?;
    self.extract_mining(&mut tx)?;
    self.advance_construction(&mut tx)?;
    self.advance_surveys(&mut tx)?;
    self.advance_research(&mut tx)?;
    self.consume_fuel_from_actual_movement(&mut tx)?;
    self.rebuild_logistics_and_assign_jobs(&mut tx)?;
    self.update_bottlenecks(&mut tx)?;
    self.check_victory(&mut tx)?;

    self.commit(tx)
}
```

`TickTransaction` rejects conflicting writes to the same field unless an explicit reducer is registered. Phases read the committed snapshot plus named immutable tick facts; for example, movement emits actual-distance facts consumed by the later fuel phase. Staged gameplay mutations never become an implicit input to a later phase.

## Travel Plans

`TravelPlan::between(source, destination)` records its origin and emits a radial segment when radii differ and a destination-lane arc when angles differ. Every segment stores total/remaining milli-distance; arcs also store direction. Each movement tick:

1. Compute the segment's integer effective speed.
2. Move `min(speed, remaining)`.
3. Record actual movement for fuel.
4. Complete zero-distance/final segments in a bounded loop.
5. Run the arrival transaction once.

`Ship.position` advances only at exact segment boundaries; presentation interpolates from the serialized plan. Expired/cancelled empty jobs finish the current leg before reassignment. `TAU_MILLI = 6283` makes the shorter integer arc unique. Bodies remain fixed simulation nodes.

## Production

Finite-deposit mining uses `MilliRemainder`; renewable belt density uses a denominator-specific remainder of `cycle_ticks * baseline_density`. Refining and construction slots reserve complete input maps and advance integer cycle ticks. OutputBlocked slots hold a completed output map and do not reserve another batch.

Resource conservation covers deposits, buffers, ship cargo, production reservations, research reservations/credit, build orders, reservations, salvage, Gate state, and intentionally consumed research/Gate activation.

## Logistics

The logistics phase:

1. Rebuilds source-locked demolition evacuation pairs, then ordinary entries from sorted Station/Resource keys, Salvage/Resource keys, BuildOrder staging manifests, and the active Gate manifest.
2. Subtracts active persistent reservations.
3. Sorts idle Cargo Ship IDs.
4. Scores all compatible, fuel-feasible pairs using GDD 12.
5. Applies the complete tie-break tuple using canonical tagged source/destination keys.
6. Creates each reservation immediately before scoring the next ship.

Construction, survey, and research-docking queues use their separate deterministic orderings. Survey orders and ResearchProjects carry their creation server sequence, and no hash-map iteration may influence selection.

## Recovery

Component recipes are stored on completed entities as `installed_components`. Cancellation moves reserved/delivered components back to the source. Demolition first enters `Evacuating`; once the station is empty, its Construction Ship returns installed components to the recovery Hub or creates a non-decaying cache at that Hub. Scrapping is valid only at the ship's current Hub, unloads cargo/Fuel first, and returns components to that Hub or Hub-side salvage.

Research Pause never deletes the project or credited value. With `release_unused`, it also changes the current facility buffer entries from research-reserved to ordinary inventory in place; no inventory location changes.

## Content and Invariants

`ContentCatalog::load` validates GDD 14's mirrored JSON before constructing a game. Cross-record checks include graph acyclicity, recipe reachability, entity costs, exact slot count, starting-buffer capacity, solvability budget, and bootstrap commands.

Runtime commits assert cheap invariants such as buffer bounds, unique slot occupancy, reservation bounds, normalized angles, valid lifecycle, and nonzero RNG state. Expensive whole-economy checks run in tests/debug builds.

Generated IDs come only from serialized per-kind counters and use `kind_generated_%08d`. Authored IDs are validated not to use that namespace. Random UUIDs, wall-clock values, and collection lengths never determine simulation IDs.

## Persistence

The actor clones a normalized save snapshot at a commit boundary and sends it to the persistence task. Saves include all rational remainders, reservations, salvage, bottleneck windows, next event sequence, RNG words, and command-log ordering. The event-retention ring, supply/demand tables, and graphical state are rebuilt/reset; stale event cursors receive `ResyncRequired`.

## Performance

The implementation avoids cloning full state per tick. `TickTransaction` records field-level changes, while published snapshots use structural sharing where profiling shows value. Correctness precedes optimization; the release stress gate is at least 100 batch ticks/sec at 500 ships, 200 stations, and 25 ResourceType values.

## Related ADRs

- ADR-0002 — Deterministic Tick Simulation
- ADR-0004 — Game Lifecycle State Machine
- ADR-0005 — Test Architecture
