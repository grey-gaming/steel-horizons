---
status: Active
owner: Tech Lead
date: 2026-08-04
---

# ADR-0011 — Deterministic Refueling

## Context

P1-20 (inter-body movement and Fuel debit) and P1-21 (fuel feasibility, refueling,
reservation contention, and rescue) require a fully specified deterministic refueling
model. The existing GDDs state high-level rules but leave the following open questions:

- Which ship roles may take a Refuel job?
- How does a ship select which station to refuel from?
- What defines "reachable" for refueling purposes?
- When does a ship enter refueling vs ordinary work?
- How much fuel transfers and when?
- How does partial stock at the station affect the transfer?
- Does a Refuel job occupy a dock?

This ADR resolves these questions and is required before P1-20/P1-21.

## Decision

### 1. Eligible ship roles

All idle ships of any role (Cargo, Construction, Research) may receive a Refuel job
when fuel is below `max_fuel` and a reachable station with available fuel exists. The
`ShipJob::Refuel { station_id }` variant defined in GDD 13 is role-agnostic — there is
no structural restriction to Cargo-only in the serialized shape. The existing GDD 12
job table (Cargo: Transport/Refuel/Idle; Construction: Build/Upgrade/Demolish/Idle;
Research: SurveyOrder/DockForResearch/Idle) is updated to include Refuel for all roles.

**Rationale.** Construction and Research ships also consume fuel during travel (GDD 12
fuel accumulator uses base_mass + payload for all roles). Without a refueling mechanism
they could strand after a long build or survey assignment. Making Refuel available to
all roles is the minimal change — no new job variants, no special-cased auto-refuel
logic for non-Cargo ships.

### 2. Refuel trigger priority

An idle ship with `fuel < max_fuel` and a reachable station with available fuel
**always takes a Refuel job before ordinary work**. It does not attempt to match
transport/build/survey/docking jobs while eligible for refueling. This implements the
GDD 7 rule "A low-fuel ship seeks Fuel before ordinary work" as a hard priority — the
ship tops up before taking on any cargo or travel assignment.

If no reachable station with available fuel exists:
- An empty, unreserved ship (fuel or no fuel) enters `AwaitingRescue` per the
  existing GDD 12/7 rescue rules.
- A loaded or reserved ship with no reachable fuel source is an invariant violation:
  job feasibility (including the 10 % reserve) was checked at assignment time, so a
  loaded ship must have had enough fuel for its delivery leg.

**Rationale.** Checking feasibility first and then falling through to refueling would
let a low-fuel ship accept a job it could barely complete, arriving at the destination
with near-zero fuel and no nearby station — creating a near-rescue scenario on every
trip. Refueling-first keeps tanks topped and eliminates the edge case.

### 3. Fuel-station selection and tie-breaks

From the set of stations where `fuel_buffer.current > fuel_buffer.export_threshold`
(fuel available for supply, per GDD 13 buffer semantics), select stations reachable
by the ship with its current fuel (one-way empty trip — see §5).

Order by:
1. Lowest route distance (empty ship, current position → station position).
2. Lowest station ID (lexicographic `StationId`).

If no station in the set is reachable, the ship has no reachable fuel source.

**Rationale.** Route distance is the natural "nearest" metric. Station ID is a
deterministic tie-breaker consistent with the cargo-matching tie-break pattern
(GDD 7 §Deterministic Cargo Matching). Using `export_threshold` rather than
`current > 0` prevents selecting a station whose fuel is reserved for outbound
evacuation or logistics.

### 4. "Reachable" definition

A station is reachable if the ship can travel from its current position to the station
empty (no cargo, no build hold, zero payload) using the GDD 12 fuel accumulator with
current fuel alone. No 10 % reserve is required because refueling is the destination —
the ship will have full fuel after the transfer.

Compute:

```
travel_plan = TravelPlan { origin: ship.position, destination: station_position }
fuel_needed = fuel_consumption(travel_plan, empty_ship_mass, no_life_support)
reachable = (ship.fuel >= fuel_needed)
```

`fuel_consumption` uses the GDD 12 fuel accumulator formula with `actual_distance_moved_milli`
derived from the travel plan's radial and arc segments, `mass_units = base_mass`, and
`payload_amount = 0`. Life Support discount is not applied during reachability
calculation (the ship may not have Life Support complete).

**Rationale.** Using the actual fuel formula for reachability ensures the ship never
accepts a refuel job it cannot reach. Excluding the 10 % reserve (which exists for
return-trip safety on cargo jobs) is safe because the ship reaches the station, fills
its tank, and can then depart with full fuel. Excluding Life Support discount gives a
conservative estimate — the ship may have more fuel than computed, making more stations
reachable, but never fewer.

### 5. Transfer quantity and timing

When a Refuel-journey ship arrives at the destination station (arrival tick, phase 2),
the fuel transfer occurs atomically as part of the arrival transaction:

```
transfer_amount = min(
    station.fuel_buffer.current - station.fuel_buffer.outbound_reserved,
    ship.max_fuel - ship.fuel
)
ship.fuel += transfer_amount
station.fuel_buffer.current -= transfer_amount
```

The transfer does **not** use cargo loading/unloading mechanics. Fuel is a separate
storage location (GDD 3 §Distributed Storage). No cargo reservation is created for a
Refuel job.

After the transfer:
- The ship's `job` transitions to `Idle`.
- The ship is docked at the station (`docked_at = station_id`).
- The ship is eligible for new job assignment in the same tick's logistics phase
  (phase 9), departing no earlier than the next tick.

**Rationale.** Atomic tick-boundary transfer matches the cargo arrival pattern and
avoids multi-tick fuel accounting. Setting the ship Idle and docked lets it immediately
participate in logistics, consistent with how a newly built ship starts (ADR-0009).

### 6. Partial stock behavior

If `station.fuel_buffer.current - station.fuel_buffer.outbound_reserved` is less than
`ship.max_fuel - ship.fuel`, the ship receives only the available amount and becomes
Idle with partial fuel. It may trigger another Refuel job on a subsequent tick if
fuel remains below `max_fuel` and a reachable station exists.

This is not an error or warning condition — partial refueling is ordinary operation
when a station's fuel stock is low.

**Rationale.** Forcing a full tank would require the ship to wait until the station has
enough fuel, adding complexity and potential deadlock. Partial refueling lets the ship
take what exists and continue. The logistics system will naturally route Fuel to
stations with demand.

### 7. Dock usage

A Refuel-job arrival occupies one station dock for its transaction tick, exactly like
a cargo arrival (GDD 12 §Docks and Transfers). If no dock is available, the ship
enters `Holding` and retries in Ship ID order on subsequent ticks. Holding consumes
no fuel.

After the fuel transfer, the ship releases the dock. If the ship remains docked at the
station (it is now Idle and docked), it occupies a dock only if it would normally
occupy one as an idle docked ship — it does not hold the dock persistently.

**Rationale.** Consistent with the cargo-arrival dock model. A Refuel job is a
ship-initiated station visit, not fundamentally different from a cargo delivery.

### 8. Job lifecycle

```
Idle (fuel < max_fuel, reachable station exists)
  │
  ▼
RefuelAssigned (station_id recorded in job)
  │  logistics phase 9
  ▼
InTransit (travel empty to station, no cargo reservation)
  │
  ▼
Arrival (phase 2)
  ├── dock reserved (or Holding)
  ├── fuel transferred (atomic)
  └── job → Idle, docked_at = station_id
```

No cargo reservation is involved. The Refuel job is not cancellable by the player in
V1 — it completes automatically.

### 9. Updates to authoritative documents

#### GDD 12 §Ship Jobs and Refueling

The job table changes from:

```
| Role         | Jobs                                                     |
|--------------|-----------------------------------------------------------|
| Cargo        | Transport to Station/GateSite, Refuel, Idle              |
| Construction | Build, Upgrade, Demolish, Idle                            |
| Research     | SurveyOrder, DockForResearch, Idle                        |
```

To:

```
| Role         | Jobs                                                     |
|--------------|-----------------------------------------------------------|
| Cargo        | Transport to Station/GateSite, Refuel, Idle              |
| Construction | Build, Upgrade, Demolish, Refuel, Idle                    |
| Research     | SurveyOrder, DockForResearch, Refuel, Idle                |
```

The refueling paragraph is amended to read:

> All idle ships with fuel below `max_fuel` take a Refuel job before ordinary work,
> using the nearest reachable station with available fuel. Construction and survey/docking
> assignment feasibility checks (destination plus 10 % reserve) remain unchanged — a
> ship that has just refueled has `fuel = max_fuel` and passes the check normally.

#### GDD 13 §Ships and Jobs

No structural change needed. `ShipJob::Refuel { station_id }` already accepts any role.
The field comment may note: "Available to all ship roles."

#### GDD 7 §Fuel Safety

The fuel safety steps are clarified to:

1. Any idle ship with `fuel < max_fuel` evaluates reachable stations with available
   fuel (ADR-0011 §3/§4). The nearest reachable station is selected, and a Refuel job
   is assigned.
2. If no reachable station with available fuel exists and the ship is empty with no
   active reservation, it enters `AwaitingRescue` for exactly 300 ticks.
3. If no reachable station with available fuel exists and the ship has cargo or an
   active reservation, the simulation asserts an invariant failure — the job
   feasibility check at assignment should have prevented this state.

## Consequences

### Positive

- All ship roles have a deterministic refueling path.
- Refueling-first priority keeps tanks topped and eliminates near-rescue edge cases.
- Reachability uses the actual fuel formula, guaranteeing the ship never attempts an
  unreachable refuel station.
- Partial stock is handled gracefully without deadlock.
- Dock usage is consistent with cargo arrival mechanics.
- The existing `ShipJob::Refuel` shape needs no structural change.

### Negative

- The GDD 12 job table must be updated (adding Refuel to Construction and Research).
- A Refuel job cannot be cancelled by the player — if a player wants to skip refueling
  and send a ship out with partial fuel, there is no mechanism in V1. This is
  consistent with the "automatic" refueling principle.

### Risks and mitigations

- **Rescue bypass.** If a ship has fuel < max_fuel but no reachable station, and it's
  also empty/unreserved, it enters AwaitingRescue. The 300-tick dispatch wait plus tow
  is slower than ordinary refueling — this is by design: the rescue tug is a recovery
  mechanism, not a logistics shortcut.

### Compliance with non-negotiable invariants

- Integer-only deterministic simulation: all fuel computations use the GDD 12 checked
  integer accumulator; no floating point.
- Stable iteration: station selection uses deterministic route-distance comparison and
  station-ID tie-breaks. Ship evaluation order is ascending Ship ID.
- No UUIDs or wall-clock values: all identifiers are serialized counters.
- Distributed storage: fuel transfers debit one station's fuel compartment and credit
  the ship's fuel field; no global material pool.

## Related ADRs

- ADR-0002 — Deterministic Tick Simulation (fuel accumulator formula)
- ADR-0009 — Hub Shipyard Queue (full-fuel initialization for new ships)
- ADR-0010 — Mining Boundary Behavior (extraction/belt-drift order)

## Future executable proof

P1-20 validates multi-body delivery with fuel debit and arrival isolation.
P1-21 validates:
- `reservation_contention` — fuel-feasible cargo binary search, refuel job assignment,
  dock contention
- `fuel_rescue_recovery` — empty ship with no reachable fuel source enters
  AwaitingRescue, tug dispatch, tow, refuel at Hub
- No double reservation or overfill
- Loaded jobs never expire or rescue
