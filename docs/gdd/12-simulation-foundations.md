---
status: Approved
owner: Tech Lead
last-reviewed: 2026-08-04
---

# Simulation Foundations — Tick Rate & Game Loop

## Authoritative Scope

This document owns tick semantics, numeric representation, phase order, travel, reservations, persistence, and performance rules. Exact recipes, technology costs, entity statistics, and authored starting values are owned by [14-authored-content.md](./14-authored-content.md). Core serialized shapes are owned by [13-data-models.md](./13-data-models.md).

## Tick and Time Modes

One simulation tick equals one second of game time. The graphical V1 game advances at exactly one tick per real-time second; Pause freezes all simulation phases while leaving planning commands and inspection available.

There are two non-player execution modes:

- **Paused batch advancement:** `AdvanceTicks(N)` is available to tests and agent clients while Paused. It transitions the lifecycle to Advancing, executes exactly N ticks without wall-clock delay, then returns to Paused.
- **Scheduler catch-up:** after a short frame or event-loop stall, the scheduler may process at most 10 accumulated ticks before yielding. A longer stall slows wall-clock progress; it never skips or merges ticks.

There is no player-facing variable tick rate, offline progress, or background simulation in V1. Batch mode changes execution speed only, never the resulting game state.

## Integer Numeric Representation

Simulation state contains no floating-point values. Three explicit accumulator forms are used:

### Fixed-Scale Values

Travel distances and movement use thousandths while exact gameplay nodes use authored integer radii:

- Angles: milli-radians, `0..6282` (`i32`)
- Body/station/Gate node radii: whole authored units (`u32`)
- Travel-segment distance/progress: milli-units (`u64`)
- Speeds: milli-units per tick (`u32`)
- Percent thresholds: integers `0..100` (`u8`)

Clients may format these integers for display, but API snapshots and deltas transmit the integer values.

### Standard Rate Accumulator

Mining uses rates whose cycle length divides 1,000. For `quantity` units per `cycle_ticks`:

```text
increment = quantity * 1000 / cycle_ticks
remainder += increment
produced = remainder / 1000
remainder = remainder % 1000
```

Example: 1 unit per 10 ticks adds 100 each tick and produces exactly 1 unit every 10 ticks. The quotient operation is required; implementations must not subtract only once.

### Denominator-Specific Rational Accumulator

Research consumption may use durations that do not divide 1,000. Each resource therefore tracks a numerator remainder with the project duration as its denominator:

```text
remainder += total_required
consumed_this_tick = remainder / total_ticks
remainder = remainder % total_ticks
```

At completion, exactly `total_required` units have been consumed. The remainder and denominator are serialized.

### Fuel Accumulator

Fuel depends on actual movement during the tick, including a ship's base mass so empty ships still consume fuel:

```text
mass_units = base_mass + payload_amount
raw_charge = actual_distance_moved_milli * mass_units
if LifeSupport is complete and segment.life_support_eligible:
    discounted = raw_charge * 4 + fuel_efficiency_remainder
    charge = discounted / 5
    fuel_efficiency_remainder = discounted % 5
else:
    charge = raw_charge
fuel_remainder += charge
fuel_consumed = fuel_remainder / 10_000_000
fuel_remainder = fuel_remainder % 10_000_000
```

The 10,000,000 denominator is the V1 balance constant: equivalently, one Fuel per 10,000 unit-mass-distance. Life Support's 4/5 factor is itself accumulated exactly. An arc is eligible when its destination lane is Outer/Fringe; a radial burn is eligible when either endpoint lane is Outer/Fringe. `actual_distance_moved_milli` is capped by the remaining route segment, so the final arrival tick never charges for distance not traveled.

## Tick Transaction and Phase Order

The simulation actor is the only owner allowed to mutate `GameState`. At each tick it reads the committed tick-N state, records changes in a pending transaction, and commits once as tick N+1. Gameplay commands scheduled for the tick are applied in their recorded sequence immediately before phase 1. Lifecycle controls execute at safe commit boundaries and do not create synthetic ticks.

Phases execute in this immutable order:

1. **Apply scheduled commands**
2. **Ship movement, dock admission, and arrival transactions/facts**
3. **Station processing and completed production cycles**
4. **Mining drift, retune, and extraction reducers**
5. **Construction progress**
6. **Survey progress**
7. **Research progress and rational resource consumption**
8. **Fuel debit and admitted refuel transfers from phase-2 facts**
9. **Logistics table rebuild and deterministic job assignment**
10. **Bottleneck monitoring**
11. **Victory check, atomic commit, and event emission**

Gameplay state is read from the committed tick-N snapshot; inventory or entity mutations staged by one phase are not visible to later phases until commit. The sole cross-phase inputs are explicit immutable tick facts—for example, phase 8 receives phase-2 actual-distance/arrival facts plus Fuel debit/hold/release facts from phases 1, 2, 3, and 7; phase 9 receives the resulting post-phase-8 ship Fuel and unreserved station-Fuel budgets; phase 4 extraction consumes its own checked drift-density facts; and phase 11 receives the pending transaction for validation and event construction. Named reducers combine competing writes such as shared-deposit extraction and station/ship Fuel transfers. Cargo or production that credits Fuel during the tick becomes committed at the end and is first available to station processing or refueling on the following tick.

## Production Cycles

### Mining

Finite-deposit mining uses the standard rate accumulator and transfers only whole units that fit in the configured output buffer. A full buffer stalls without advancing the accumulator. With partial space, potential whole-unit throughput above the space is discarded for that tick, but a finite deposit is decremented by exactly the units actually stored—never by discarded throughput. Targets sharing one body/resource deposit submit intents reduced by Station ID then serialized mining-slot index against one phase-local remaining budget. Exhausting the deposit resets every configured target's fractional credit for it.

Renewable belt mining uses a denominator-specific accumulator because density ratios need not divide 1,000: each tick adds `base_quantity * current_density` with denominator `cycle_ticks * baseline_density`. Belt deposits are not decremented. For a transaction from tick N to `resulting_tick = N + 1`, a multiple-of-1,000 drift is computed first in Body ID/ResourceType order, published as an immutable density fact, and used by that transaction's extraction. Mining slots and ten-tick retune countdowns are serialized; a retune suppresses exactly ten simulation ticks and resets its accumulator as specified by ADR-0010.

### Refining and Component Assembly

Recipes use integer inputs and outputs. A production slot behaves as follows:

1. At cycle start, reserve the complete recipe inputs in the station.
2. Advance `progress_ticks` once per active tick.
3. At `cycle_ticks`, consume all reserved inputs atomically and add the complete integer output quantity.
4. If output capacity is unavailable, hold the completed batch until capacity exists.

This prevents partial recipe consumption and preserves stoichiometry. The progress bar is smooth; inventory transfer occurs at the cycle boundary. Parallel throughput means independent production slots, not fractional input destruction.

`Fuel` is a special storage location, not a special arithmetic type: any recipe/research input of Fuel reserves from that station's Fuel compartment, and `make_fuel` transfers its output there. Cargo Ships transport Fuel between station Fuel compartments or salvage. Authored compartment maxima never count against general cargo.

`SetProductionRecipe` on Idle/AwaitingInputs replaces the configuration; on Processing it first releases the complete reserved input map and resets progress; on OutputBlocked it is rejected until the completed output transfers. Clearing a slot uses `recipe_id = null` and follows the same rules.

Recipe and mining commands apply GDD 14's automatic buffer allocation before changing the configuration. Allocation never shrinks an existing maximum or deletes stock. If the minimum buffer set cannot fit, the whole command is rejected atomically.

## Deterministic Travel

Celestial bodies are fixed gameplay nodes in V1. Cosmetic orbital animation does not alter simulation positions or travel time. Stations use their parent body's system position for logistics; local orbit-ring position is visual and used for placement only.

A trip is a deterministic two-segment `TravelPlan` whose origin, total/remaining segment distances, and arc direction are serialized:

1. **Radial burn:** from source radius to destination radius at the source angle. Distance is `abs(source_radius - destination_radius) * 1000` milli-units. Speed multiplier is `1/2`.
2. **Destination-lane arc:** shortest angular arc at the destination radius. Distance is `angular_diff_milli * destination_radius` milli-units. Speed uses the destination lane multiplier.

If both radii are equal, the radial segment is omitted. With the integer full-turn constant `TAU_MILLI = 6283`, the two arc lengths cannot tie exactly; the smaller arc is unique.

Lane multipliers are exact rationals:

| Lane | Multiplier |
|------|------------|
| Inner | 3/2 |
| Habitable | 1/1 |
| Outer | 7/10 |
| Fringe | 1/2 |

Payload reduces speed by up to 30%. For Cargo Ships, payload is `cargo_amount / max_cargo_capacity`; for Construction Ships, it is the sum of `build_cargo` units divided by `build_cargo_capacity`. Research Ships have zero payload capacity and use multiplier 1/1 without evaluating the division:

```text
payload_num = payload_capacity * 10 - payload_amount * 3
payload_den = payload_capacity * 10
effective_speed_milli = base_speed_milli * segment_num * payload_num
                        / (segment_den * payload_den)
```

Each tick moves `min(effective_speed_milli, segment_remaining_milli)`. All intermediate products use checked `u64`/`u128` arithmetic.

`Ship.position` is the last exact node/segment boundary, not a rounded mid-arc coordinate. At radial completion it becomes the destination lane and radius at the source angle; at arc completion it becomes the exact destination node. Clients derive smooth visual position from the TravelPlan's origin, direction, total, and remaining distance. Jobs are never reassigned at a fractional point: cancellation/awaiting-pickup expiry lets the empty ship finish its current leg, then makes it Idle at that exact endpoint. This removes any unrepresented angular remainder from authoritative state.

## Ship Jobs and Refueling

Jobs persist until completion or explicit cancellation:

| Role | Jobs |
|------|------|
| Cargo | Transport to Station/GateSite, Refuel, Idle |
| Construction | Build, Upgrade, Demolish, Refuel, Idle |
| Research | SurveyOrder, DockForResearch, Refuel, Idle |

Every empty, unreserved idle ship with `fuel < max_fuel` seeks direct refueling before ordinary role work. A refuel source needs at least one unreserved Fuel unit after subtracting `AwaitingPickup` outbound reservations plus production and research holds. Direct station refueling is local consumption and may use stock below the Cargo export-threshold floor. Choose the reachable source by route distance then Station ID. Reachability simulates the ordinary zero-payload route using cloned Fuel and Life Support remainders, the actual completed-tech set, final-segment capping, and checked arithmetic; no 10% reserve is required because a station is the destination.

Phase 2 admits a Refuel arrival to a transactional dock and records a fact. Phase 8's shared Fuel reducer first debits final movement, honors every existing/new logistics, production, and research Fuel hold/debit, then processes admitted refuel facts in Ship ID order. It publishes immutable post-phase-8 ship values and per-station unreserved budgets for phase 9. That phase uses only those facts for Fuel supply/refuel selection/route feasibility and reduces the budget as it creates Cargo Fuel reservations. A direct-refuel assignment reserves nothing, so a later transfer may be partial or zero. Same-tick delivery/production credits are unavailable until the next tick. A below-full ship already docked at a Hub with no available Fuel waits there; it never dispatches a tug to itself. Otherwise an empty, unreserved ship with no reachable source enters `AwaitingRescue`. The nearest existing Hub by route distance then Hub ID dispatches after 300 ticks and tows it directly at `base_speed_milli / 2`; tow movement consumes no ship Fuel and carries no payload or Fuel mutation. Cargo or an active reservation in AwaitingRescue is an invariant violation.

Ordinary job feasibility still includes the ship-to-source leg, loaded delivery leg, and a reserve of `max_fuel / 10` (all authored capacities divide by 10). Construction and survey/docking assignments likewise require the destination plus that reserve.

Survey targets persist as serialized orders. Assignment uses the exact ordering in GDD 7; cancellation loses only partial scan work and takes effect immediately while scanning or after the current TravelPlan leg while in transit. Research Station projects automatically create `DockForResearch` work and remain `NoResearchShip` until the selected ship actually docks.

Research reassignment is one atomic actor transaction. It validates the target first,
releases all unconsumed old-facility reservations in place, clears old facility/ship
ownership, preserves completed ticks, consumed credit, and rational remainders, then
reserves only physical target stock and derives the remaining target demand. An en-route
Research Ship finishes its current TravelPlan leg before idling. `NoResearchShip` is a
`Paused` reason, never a separate project state. Hub Haven's authored built-in console
remains Tier 1 after Hub upgrades; all Tier-2+ projects require a Research Station.

## Supply, Demand, and Reservations

Supply and demand tables are deterministic derived data rebuilt during phase 9. Supplies may be stations or salvage caches; demands may be stations, ordinary BuildOrder staging at a source Hub, a source-locked demolition evacuation cache, or the active Gate site:

- Demand exists when `current + inbound_reserved` is below the configured demand threshold. The demanded amount is the threshold target minus that total.
- Supply exists when `current - outbound_reserved` is above the export threshold. The available amount is that total minus the export floor.
- After Gate assembly begins, its demand is the exact undelivered portion of the canonical manifest at priority 100. Gate reservations count against that manifest exactly as station inbound reservations count against capacity.
- Creation of an ordinary BuildOrder first transfers matching unreserved components already in the source Hub directly into serialized staging. It then demands its exact missing component map at that Hub's priority. External delivery is capped by the outstanding manifest, not the Hub's general-buffer capacity.

For both thresholds, `target = (max * percentage) / 100` using checked integer multiplication and floor division. Matching scores use signed `i64` so a large distance penalty cannot underflow.

Idle cargo ships are evaluated in ascending ship ID order. For every compatible supply/demand pair:

```text
pickup_distance_milli = route_distance(ship_position, supply_position)
delivery_distance_milli = route_distance(supply_position, demand_position)
distance_penalty = (pickup_distance_milli + delivery_distance_milli) / 20_000
score = demand_priority * 4 + supply_priority * 3 - distance_penalty
```

The 20,000 divisor makes one penalty point equal 20 travel units, keeping 0–100 priorities meaningful at authored system scale. Selection tie-breaks are, in order: highest score, lowest raw total route distance, canonical supply key (`station:<id>` or `salvage:<id>`), canonical demand key (`station:<id>`, `build_order:<id>`, `evacuation:<id>`, or `gate_site`), then resource enum order.

The chosen amount is the minimum of ship free capacity, unreserved supply, unreserved demand capacity, and fuel-feasible capacity. Fuel-feasible capacity is the largest integer amount whose simulated two-leg plan retains the 10% reserve, found by deterministic binary search over `0..candidate_amount`. Supply and destination capacity are reserved atomically. An awaiting-pickup reservation expires at `estimated_pickup_tick + 600`; expiry releases both sides, marks the still-empty job cancelled, and lets an in-transit ship finish its current leg before becoming Idle at that exact endpoint. A loaded/in-transit reservation never expires, and cargo is never discarded. Reservations are serialized and reapplied when derived tables are rebuilt after load.

## Docks and Transfers

An arrival transaction reserves one dock for that tick. BuildOrder deliveries use the order's source-Hub dock and transfer into order staging; demolition evacuation deliveries use the recovery-Hub dock and transfer into the order's linked permanent cache. The Gate site instead has one virtual cargo-transfer berth per tick; it is neither a station nor an orbit slot. Cargo transfers atomically and the ship may receive its next job in phase 9, departing no earlier than the next tick. Persistent dock users—such as a Research Ship powering a project—continue to occupy a station dock. If no station dock/Gate berth is available, arriving ships enter `Holding` and retry in Ship ID order; holding consumes no fuel.

## Recovery and the Always-Solvable Invariant

Intentional transformations may consume raw materials, but reversible player actions never destroy invested components:

- Cancelling a build releases network reservations; staged components return to its source Hub/Hub-side salvage. An assigned Construction Ship already in transit completes its current leg, then returns any multi-component build hold to the source Hub/cache before becoming Idle.
- Demolition stops new station work and creates a permanent recovery cache at the selected Hub. Priority-100 source-locked Cargo jobs evacuate all ordinary inventory/Fuel there; already-loaded inbound deliveries finish, while awaiting inbound reservations release. After docks, buffers, outputs, and reservations clear, its Construction Ship returns the installed recipe to the recovery Hub or that cache. Scrapping is allowed only at the ship's current Hub; cargo, Fuel, and components fill compatible Hub compartments, and every overflow goes to Hub-side salvage.
- Pausing research retains completed ticks and consumed-resource credit permanently. Unused materials either remain reserved or are released in place to ordinary facility buffers; they never teleport. Resuming the same technology continues from that state.
- Production cancellation releases reserved recipe inputs; an already completed OutputBlocked batch remains recoverable output.

Gate-critical finite resources are used only by persistent research unlocks, reversible critical refining, recyclable components, or the final Gate. The authored system budget in GDD 14 covers every technology plus the minimum victory build with more than 100% headroom. Renewable belt inputs and the non-productive emergency tow prevent operational deadlocks.

## Save, Load, and Replay

The full state described in GDD 13 is serialized as JSON. Supply/demand tables, camera state, and other derived presentation data are not saved.

- Autosave every 300 committed ticks and after material player commands while Paused.
- Manual save on quit and `SaveNow`.
- One autosave slot in V1.
- Atomic temp-file write, file sync, then rename.
- A save made while Running or Advancing loads as Paused; Won remains Won.
- Schema migrations are one-way. Newer unsupported schemas fail with a clear error.
- The deterministic command log records the complete command payload, command ID, effective tick, server sequence, and outcome for replay and structural idempotency comparison.
- The next event sequence is a positive serialized lower bound and bottleneck rolling windows are serialized. The retained event ring/allocator is server-session runtime data that survives same-process NewGame/LoadAutosave; a replacement state rebases its lower bound and emits a complete replacement delta. After process restart, the empty ring's oldest-available value and allocator are seeded from the saved lower bound, while the reported latest cursor is lower-bound minus one. A cursor one below oldest available is valid; an older request receives `ResyncRequired` and continues from a fresh snapshot.

The PRNG is the project-owned xoshiro256** algorithm with four serialized `u64` state words. As the sole exception to checked gameplay arithmetic, one call uses wrapping-modulo-2^64 `u64` operations exactly as follows:

```text
result = rotl(s1 * 5, 7) * 9
t = s1 << 17
s2 ^= s0; s3 ^= s1; s1 ^= s2; s0 ^= s3
s2 ^= t; s3 = rotl(s3, 45)
return result
```

The all-zero state is invalid. Golden vectors lock this transition and avoid dependency-version drift.

## Performance Targets

At V1 ceilings—500 ships, 200 stations, and all 25 resource/component types—a release build must:

- Sustain the graphical 1 tick/sec scheduler comfortably.
- Sustain at least 100 batch ticks/sec in the reference stress fixture.
- Avoid unbounded event history or per-tick allocation proportional to total historical state.

Benchmarks report p50 and p95 tick duration on the supported macOS and Windows reference runners. Performance optimization must not weaken deterministic ordering.
