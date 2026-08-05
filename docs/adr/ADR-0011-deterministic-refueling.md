---
status: accepted
owner: Tech Lead
date: 2026-08-04
---

# ADR-0011 — Deterministic Refueling

## Context

P1-20 (inter-body movement and Fuel debit) and P1-21 (fuel feasibility,
refueling, reservation contention, and rescue) require one exact refueling
model. The existing GDDs establish automatic refueling, exact Fuel
accumulation, transactional docks, and rescue, but do not completely define:

- which ship roles may refuel;
- how available station Fuel and reservation pressure are calculated;
- station selection and route feasibility;
- how final-leg Fuel debit and the arrival transfer interact with the fixed
  tick phases;
- partial or depleted station stock; or
- zero-distance arrivals, dock use, and rescue at a Hub.

This ADR resolves those questions. All arithmetic below is checked; an
underflow or overflow is a typed simulation error that aborts the tick
transaction.

## Decision

### 1. Eligible ships and assignment priority

Every ship role (`Cargo`, `Construction`, and `Research`) may hold
`ShipJob::Refuel { station_id }`.

During phase 9, idle ships are visited in ascending `ShipId`. For each empty,
unreserved idle ship:

1. If `fuel == max_fuel`, skip refueling and evaluate its ordinary role work.
2. If `fuel < max_fuel` and at least one reachable station has Fuel available
   for direct refueling, assign the nearest such station as described below.
   Refueling has priority over transport, construction, survey, and research
   docking work.
3. If no such station exists and the ship is already docked at a Hub, leave it
   idle at that Hub. It is reconsidered each tick and can refuel when Fuel
   becomes available; a Hub never dispatches a rescue tug to itself.
4. If no such station exists and the ship is not docked at a Hub, enter
   `AwaitingRescue` under the existing 300-tick rescue rule.

An idle ship is empty only when Cargo cargo and Construction build cargo are
both zero and it has no active logistics reservation. Research Ships always
have zero payload capacity. A loaded or reserved ship continues its existing
job; refuel assignment is never evaluated for it. Route acceptance must have
proved that job can finish with its required reserve. A Fuel underflow or a
loaded/reserved `AwaitingRescue` state is an invariant violation.

Dispatch countdown and tow movement never debit, zero, or otherwise alter the ship's
Fuel, `fuel_remainder`, or `fuel_efficiency_remainder`. Arrival at the rescue Hub
preserves all three pre-tow values and then returns the ship to normal direct-refuel
evaluation.

Defining “low Fuel” as any strict deficit from `max_fuel` makes the priority
test objective and avoids a second authored threshold.

### 2. Direct-refuel availability and export thresholds

Every station has a separate Fuel buffer. Directly fueling a ship at that
station is local consumption, not a Cargo Ship export, so
`fuel_buffer.export_threshold` does **not** reserve Fuel against direct
refueling. The percentage export floor applies only when phase 9 derives Fuel
supply for Cargo logistics:

```text
export_floor = checked(fuel_buffer.max * fuel_buffer.export_threshold) / 100
```

For refueling, derive committed protected Fuel as the checked sum of:

- logistics reservations sourced from this station and in `AwaitingPickup`;
- Fuel in every `ProductionSlot.reserved_inputs`; and
- Fuel in every non-complete `ResearchProject.resources_reserved` at this
  station.

Loaded logistics reservations are excluded because their Fuel has already left
the station. Released and Delivered reservations are excluded. Conservation
invariants prove that these protected categories are disjoint physical units and
that their sum does not exceed `fuel_buffer.current`.

```text
available_to_refuel = fuel_buffer.current - protected_fuel
station_has_refuel_fuel = available_to_refuel > 0
```

This distinction is intentional: the export floor prevents Cargo logistics
from draining a station, while locally docked or arriving ships can use its
unreserved Fuel. It also guarantees that a rescued ship can use unreserved Hub
Fuel instead of becoming trapped behind the Hub's export threshold.

A Refuel job creates no Fuel reservation. Assignment therefore guarantees only
that some unreserved Fuel exists at assignment time. New protected holds, Cargo
pickups, and earlier refuel arrivals may reduce it before arrival; the partial-
stock rules handle that deterministically.

### 3. Station selection

Consider every station for which `available_to_refuel > 0` in the committed
tick-N state and whose position is reachable by the exact feasibility
simulation in section 4. Order candidates by:

1. lowest route distance from the ship's exact current position; then
2. lexicographically lowest `StationId`.

Route distance is independent of ship speed and Fuel stock. Candidate
collection and comparison use stable ordered iteration.

### 4. Exact reachability

No 10% reserve is required for a Refuel job because the station itself is the
safe destination. Reachability does **not** assume a full transfer on arrival;
partial or zero stock is safe because the ship arrives at a station and may
wait, retry, or be rescued under the ordinary rules.

Reachability uses the same pure travel-and-Fuel simulator as actual movement,
not a separately rounded estimate. The simulator:

- builds the ordinary radial/arc `TravelPlan`;
- uses `base_mass` with zero payload;
- clones the ship's `fuel_remainder` and `fuel_efficiency_remainder`;
- snapshots the current completed-technology set and applies Life Support
  exactly when it is already complete and each segment is eligible; it never
  assumes a future unlock;
- uses the same integer effective speeds, per-tick final-segment capping,
  segment order, checked intermediates, and Fuel accumulator as phases 2 and
  8; and
- succeeds only if every simulated Fuel debit can be paid from current Fuel.

The cloned remainder values are discarded after the feasibility check. The
real ship state changes only during ordinary committed ticks. With an unchanged
technology set, simulated and actual route debits are identical. If permanent
Life Support completes after assignment, actual remaining travel can consume
less Fuel but never more, so acceptance remains safe. A zero-distance route
consumes no Fuel and is reachable.

### 5. Dock admission, final-leg debit, and transfer order

The immutable tick phase order remains authoritative. Refueling is split into
an arrival fact and an explicit Fuel reducer so the final movement charge is
never skipped or calculated from a post-transfer tank:

1. **Phase 2 — movement and docks.** Movement records the actual capped
   distance. A Refuel ship that reaches its station competes for one
   transactional dock with other arrivals, in ascending `ShipId` under the
   ordinary holding rules. An admitted arrival emits an immutable
   `RefuelArrivalFact { ship_id, station_id }`; a held ship emits none.
2. **Phase 8 — shared Fuel reducer.** First apply every ship's actual phase-2
   movement debit, updating both serialized Fuel remainders. The reducer also
   receives explicit immutable Fuel debit/hold/release facts produced by
   scheduled commands and phases 2, 3, and 7. Existing Cargo pickups,
   production/research consumption, and every committed or newly staged
   protected hold take priority over direct refueling. Then process admitted
   RefuelArrivalFacts in ascending `ShipId`, subtracting transfers from one
   phase-local station budget. Phase-2 Fuel deliveries and phase-3 Fuel output
   are credits unavailable until the next committed tick; the final inventory
   reducer still applies those credits at commit.
3. **Phase 9 — fact-driven reassignment.** The reducer publishes one immutable
   `FuelPhaseResult` containing post-debit/transfer Fuel and remainders for every
   changed ship plus each station's remaining unreserved Fuel budget. Phase 9
   must use this fact—not tick-N ship/station Fuel—for direct-refuel selection,
   Cargo Fuel supply, and route feasibility. New Cargo Fuel reservations reduce
   the phase-local station budget before the next ship is evaluated. Refuel job
   assignment itself reserves no stock, so multiple ships may still select one
   source and later receive partial/zero transfers in stable arrival order. A
   newly assigned job may not move before the next tick.

The transfer for each admitted arrival is:

```text
ship_deficit_after_movement = ship.max_fuel - ship.fuel_after_movement
transfer_amount = min(remaining_available_to_refuel,
                      ship_deficit_after_movement)

ship.fuel = ship.fuel_after_movement + transfer_amount
station.fuel_buffer.current -= transfer_amount
```

The station debit and ship credit are one atomic checked transaction. Fuel
remainders are not reset by refueling. The transfer is not cargo loading and
creates no logistics reservation.

### 6. Partial and depleted stock

The arrival-time transfer may be anywhere from zero through the complete ship
deficit. After any successful dock transaction, including a zero transfer:

- `job` becomes `Idle`;
- `state` becomes `Idle`;
- `travel_plan` becomes null;
- `docked_at` is the destination station; and
- the transaction emits the exact transferred Fuel amount, including zero.

If the ship remains below `max_fuel`, phase 9 applies section 1 again. It may
take another Refuel job, remain idle at a Hub awaiting Fuel, or enter rescue
from a non-Hub station. Zero or partial transfer is ordinary deterministic
operation, not a command error.

### 7. Dock representation and zero-distance jobs

A Refuel arrival occupies one station dock for its transaction tick. If all
docks are reserved, the ship enters `Holding`, consumes no Fuel, and retries in
ascending `ShipId` on later ticks.

After transfer, the transaction dock is released. `Ship.docked_at` records the
ship's exact station location; it does not by itself consume persistent dock
capacity. Only explicitly persistent users, such as a Research Ship in
`PoweringResearch`, occupy a dock across ticks.

A Refuel job assigned while the ship is already at the selected station uses a
zero-distance TravelPlan. Assignment occurs in phase 9 and dock admission plus
transfer occurs in phase 2/8 of the following tick; it is never an immediate
second mutation of the assignment transaction.

### 8. Job lifecycle

```text
Idle and below max Fuel
  -> phase-9 Refuel assignment (station_id serialized)
  -> empty InTransit, or zero-distance pending arrival
  -> phase-2 dock admission, or Holding
  -> phase-8 final movement debit then atomic transfer
  -> Idle at station
  -> phase-9 ordinary/refuel/rescue evaluation
```

Refuel is automatic and has no player cancellation command in V1. Save/load
preserves the serialized `ShipJob::Refuel`, TravelPlan, Fuel values, and both
remainders exactly.

## Required dependent-document amendments

The dependent GDD/TDD summaries must express the same rules:

- GDD 12's job table includes Refuel for Cargo, Construction, and Research.
- GDD 12 identifies the phase-2 arrival fact and phase-8 ordered Fuel reducer.
- GDD 7 uses direct-refuel availability rather than comparing Fuel units to a
  percentage, and distinguishes a ship already safe at a Hub from rescue.
- GDD 13 notes that `ShipJob::Refuel` is valid for every role; no state-shape
  change is required.
- Movement/Fuel tests use the exact feasibility simulator and cover remainder,
  Life Support, zero-distance, dock contention, and final-leg cases.

## Consequences

### Positive

- Every ship role has one deterministic refueling path.
- Feasibility cannot underestimate actual movement due to rounding, Life
  Support, or serialized remainders.
- Final-leg Fuel is charged before determining the ship deficit.
- Cargo reservations cannot be consumed by refueling.
- Partial stock, zero stock, same-station refueling, and Hub waiting are all
  terminating, serialized behaviors.

### Negative

- Phase 8 needs an explicit ordered reducer spanning ship Fuel and station Fuel.
- Refuel jobs deliberately do not reserve stock, so an apparently useful trip
  can transfer zero after contention.
- Players cannot force a below-full ship to skip automatic refueling in V1.

### Compliance with invariants

- Simulation state and all calculations remain integer-only.
- Every intermediate uses checked arithmetic and typed errors.
- Station and ship iteration orders are explicit.
- Fuel moves from one concrete station buffer to one concrete ship; there is no
  global material pool and no unrecorded loss.
- Feasibility uses cloned state and never mutates authoritative state.

## Related ADRs

- ADR-0002 — Deterministic Tick Simulation
- ADR-0003 — Command/Query API with WebSocket Streaming
- ADR-0006 — Canonical Content/State Hashing
- ADR-0009 — Hub Shipyard Queue

## Future executable proof

P1-20 validates exact inter-body Fuel debit, cloned-route feasibility,
remainder/Life-Support equivalence, final-leg capping, and arrival isolation.
P1-21 validates:

- every ship role taking Refuel work;
- percentage export floors applying to Cargo supply but not direct refueling;
- AwaitingPickup Fuel reservations remaining unavailable;
- multiple same-tick arrivals in `ShipId` order without overdraw or overfill;
- zero-distance, partial, and zero transfers;
- dock contention and Holding;
- no self-rescue loop at a Hub;
- `reservation_contention`; and
- `fuel_rescue_recovery`, including save/load and replay at every lifecycle
  stage.
