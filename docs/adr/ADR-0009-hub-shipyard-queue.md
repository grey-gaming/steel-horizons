---
status: accepted
owner: Tech Lead
date: 2026-08-04
---

# ADR-0009: Hub Shipyard Queue Semantics

## Context

Phase 1 requires ship construction at Station Hub shipyards. GDD 5 §Station Hub
establishes that "every tier has one shipyard queue and one slow component-assembly
slot" and "ship construction, one ship order at a time." GDD 13 defines
`StationStats.shipyard_slots: u8` as always 1 for Hub and 0 otherwise. GDD 13 also
defines `BuildTarget::Ship { hub_id, role, tier }` and `BuildOrder` with its
`components_required`, `components_delivered`, `builder_ship_id`, `progress_work`,
`total_work`, and `state` fields. ADR-0003 defines the `QueueBuildShip` command.
GDD 5 §Build Work assigns ship build work values (30/60/120/240 for T1/T2/T3/T4)
and states "Hub shipyards contribute one ship-build work per tick." GDD 7 §Docks
and Holding specifies that "a ship under construction occupies the Hub shipyard
slot, not an ordinary dock until completion."

Six unresolved specification questions remain:

1. **Queue structure.** With `shipyard_slots = 1` and "one ship order at a time,"
   does the shipyard accept a second `QueueBuildShip` while one is active, or does
   it reject with an error? If it accepts, how are pending orders ordered?

2. **Queue representation in state.** The Hub Station struct (GDD 13) has no field
   linking it to its active or pending ship BuildOrders. How does the Hub know
   which BuildOrder is currently under construction and which are queued behind it?

3. **Component staging for ship builds.** GDD 5 §Building and Upgrading describes
   station construction staging at the source Hub. For ship builds at the Hub, the
   Hub is both the builder and the source. How do components move into staging? Do
   Cargo Ships deliver remote components for ship builds?

4. **Build progress and completion.** Hub contributes 1 work per tick. How does
   this advance `BuildOrder.progress_work`? What happens when `progress_work >=
   total_work`? Where does the completed ship appear?

5. **Cancellation semantics.** What happens to staged components and progress work
   when a ship build is cancelled? Does cancellation of a pending (non-active)
   order differ from cancellation of the active order?

6. **Serialized fields beyond BuildOrder.** Which additional fields must the Hub
   Station carry to represent its shipyard queue, and what invariants apply?

## Decision

### 1. The shipyard queue is a FIFO ordered queue with exactly one active build

The Hub shipyard accepts `QueueBuildShip` commands while an active build is in
progress. The new order is appended to a **pending queue** and does not begin
staging or construction until the active build completes. This means:

- At any time, at most one BuildOrder with `BuildTarget::Ship` targeting this Hub
  is in state `Building` (the active build).
- Zero or more BuildOrders with `BuildTarget::Ship` targeting this Hub are in
  state `AwaitingMaterials` (pending, not yet started).
- The queue order is **creation `server_sequence`** (the order in which the
  commands were accepted), matching the ordering principle used by construction
  ship selection in GDD 7 §Construction and Survey Jobs.

Rejection occurs only when the Hub's shipyard is physically incapable of accepting
another order—specifically, when the component cost cannot fit in the Hub's
available general cargo capacity after the active order's staging reservation is
subtracted, or when the required technology is not unlocked. These are domain
validation rejections, not a queue-full rejection.

**Rationale.** A multi-order queue lets the player chain multiple ship builds
without micro-managing each command after completion. The single active build
preserves "one ship order at a time." FIFO ordering by server_sequence is the
simplest deterministic model and matches every other ordered queue in the
simulation (survey orders, construction ship selection).

### 2. Hub Station carries an ordered ship_build_queue field

The Hub Station struct gains:

```text
struct Station {
  // ... existing fields ...
  ship_build_queue: BuildOrderId[]  // ordered queue; first = active build (if any)
  // ... no new active_ship_build_id — the first non-terminal entry IS the active build
}
```

**Invariants:**

- `ship_build_queue` is non-empty only for Hub stations.
- `ship_build_queue` has at most one entry with BuildState `Building` or
  `Traveling` or `Evacuating` — that entry is the active build.
- All other entries have BuildState `AwaitingMaterials` or `Cancelled` (a
  cancelled pending entry is removed from the queue on next tick maintenance).
- The maximum length of `ship_build_queue` is bounded only by authored Hub cargo
  capacity and component availability; there is no separate hard queue-depth limit.
- `ship_build_queue` entries reference valid `BuildOrderId` values in
  `GameState.build_orders`.

**Rationale.** A single ordered list of IDs is simpler than separate
`active_ship_build_id` and `pending_ship_build_queue` fields. The active build is
the first non-terminal entry. This avoids maintaining two separate invariants.
BuildOrder state already captures the build lifecycle (AwaitingMaterials →
Ready → Building → Complete/Cancelled). The queue is only the ordering mechanism.

### 3. Component staging follows the same rules as station builds

Ship construction uses the identical staging model from GDD 5 §Building and
Upgrading:

1. On `QueueBuildShip`, a new `BuildOrder` with `BuildTarget::Ship` is created.
   Its `source_station_id` is the target Hub.
2. At creation, the Hub's unreserved matching component inventory moves directly
   into `components_delivered` (staging), exactly as GDD 12 §Supply Demand and
   Reservations describes for ordinary BuildOrders: "matching unreserved
   components already at its source Hub move directly into staging without a
   Cargo Ship."
3. The remaining unfulfilled `components_required` map is exposed as logistics
   demand at the Hub's priority, using the Hub's dock for delivery into staging.
4. Cargo Ships deliver remote components to the Hub's dock; each delivery
   increments `components_delivered`.
5. The order transitions to `Ready` state when `components_delivered ==
   components_required`.

There is **no Construction Ship involvement** for ship builds. The Hub itself
provides the build work.

**Rationale.** Reusing the existing BuildOrder staging model avoids a special
case for ship construction. The Hub's own inventory supplies local components,
and Cargo Ships supply the rest—the same logistics pattern used everywhere else.
No Construction Ship is needed because the Hub's shipyard is a fixed facility,
not a mobile builder.

### 4. Build progress and completion

Each tick, when the Hub has an active (Building) ship BuildOrder:

1. The Hub contributes 1 work to `BuildOrder.progress_work`.
2. When `progress_work >= total_work`:
   a. A new `Ship` entity is created with the authored `ShipDefinition` stats
      for the target `role` and `tier`. Its `id` is generated from
      `IdCounters.ship`.
   b. The ship's `installed_components` is set to the authored component cost
      map for that ship definition. These components are consumed from the
      build order staging (they become installed, not loose).
   c. The ship is placed at the Hub's system position, `docked_at: hub_id`,
      state `Idle`, job `Idle`, and fuel set to `max_fuel` (full tank).
   d. The BuildOrder enters `Complete` state.
   e. The Hub's `ship_build_queue` advances: the completed entry is removed.
      If a pending entry exists, it becomes the new active build and begins
      component staging on the next tick's construction phase.

The new ship is immediately available for job assignment in the next logistics
phase. It does not need a separate "launch" animation—it appears docked and idle
at the Hub.

**Rationale.** Placing the completed ship docked and idle at the Hub means it
can participate in logistics immediately. Full-fuel initialization avoids a
refuel bottleneck for freshly built ships. Using authored component cost as
installed components means scrapping recovers the correct component set.

### 5. Cancellation semantics

Cancelling a ship BuildOrder follows GDD 5 §Cancellation Demolition and Scrapping:

**Active build (state = Building or Ready or AwaitingMaterials with components
staged):**

- All staged components in `components_delivered` return to the Hub's general
  cargo inventory (or Hub-side salvage if cargo capacity is insufficient).
- The BuildOrder enters `Cancelled` state.
- Progress work (`progress_work`) is **lost** — it represents irreversible
  fabrication labor, not recoverable materials.
- The order is removed from `ship_build_queue`. The next pending entry (if any)
  becomes active and begins staging on the next construction phase.

**Pending build (state = AwaitingMaterials, no components staged because it
hasn't reached the active slot yet):**

- No components have been staged, so nothing to return.
- The BuildOrder enters `Cancelled` state.
- The order is removed from `ship_build_queue`. No effect on the active build.

Cancellation of a pending order is always safe because no resources have been
committed. Cancellation of the active order returns staged components to the Hub
inventory, which may exceed buffer capacity—the overflow goes to a permanent
salvage cache at that Hub (same rule as station build cancellation).

**Rationale.** Distinguishing active vs pending cancellation avoids unnecessary
component return processing for orders that never staged anything. Losing
progress work on the active build is consistent with the irreversible nature of
fabrication work (GDD 5 §Cancellation: "staged components return" but work is
not recovered). This matches station build cancellation semantics.

### 6. Queue maintenance and invariants

**Tick maintenance.** During the construction phase (phase 5 of the tick
transaction), the Hub performs these operations in order:

1. If the active build is `Complete` or `Cancelled`, remove it from
   `ship_build_queue`.
2. If `ship_build_queue` is non-empty and the first entry has state
   `AwaitingMaterials`, check if `components_delivered >= components_required`:
   - If yes, transition to `Ready` and then immediately to `Building` (the Hub
     needs no travel—it is already at the site). The Hub starts contributing
     1 work per tick on the next construction phase.
3. If the active build is `Building`, add 1 to `progress_work`. Check for
   completion (see §4 above).
4. Remove any `Cancelled` entries from the queue (they are cleaned up
   regardless of position).

**Idempotency and replay.** The `ship_build_queue` is serialized in `GameState`
and included in the replay-mode hash. On load, the queue is restored exactly.
Pending commands that were accepted but not yet reflected in the queue are
rebuilt from `command_log` per ADR-0008.

**Serialization.** The `ship_build_queue` field is serialized as an ordered array
of `BuildOrderId` strings. It appears in the Hub `Station` struct and therefore
in `GameState.stations`. No separate top-level collection is needed.

## Consequences

### Positive

- A clear FIFO queue model lets players chain ship builds without manual
  re-queueing after each completion.
- Reusing BuildOrder staging for components avoids a separate ship-build
  material-tracking system.
- No Construction Ship involvement for ship builds keeps the model simple—the
  Hub provides build work directly.
- Cancellation semantics distinguish active vs pending, avoiding unnecessary
  work for orders that never started.
- The single `ship_build_queue: BuildOrderId[]` field is minimal: no separate
  active/pending split, no new state machine.

### Negative

- The queue is serialized and grows over a session. At V1 scale (tens or
  hundreds of ships over hours) this is negligible.
- Pending orders reserve no components until they reach the active slot. A
  player could queue many orders that later cannot be fulfilled due to component
  shortages, but this is a player-planning concern, not a simulation invariant.
- The Hub's component staging for the active build reduces available general
  cargo capacity until completion or cancellation. A large active build (T4
  Fast Freighter) may crowd out other Hub storage. This is an authored-balance
  concern handled by GDD 14's capacity values.

### Mitigations

- Queue length is self-limiting: each pending order exposes demand, consuming
  logistics bandwidth. The player naturally limits queue depth to available
  components.
- Component staging returns to Hub inventory on cancellation, preserving the
  always-solvable invariant.
- Full-fuel initialization on completion prevents stranded newly-built ships.

## Related ADRs

- ADR-0003 — Command/Query API with WebSocket Streaming (defines
  `QueueBuildShip`, command envelope, server_sequence ordering)
- ADR-0006 — Canonical Content/State Hashing (replay-mode hash includes
  ship_build_queue)
- ADR-0007 — Save Envelope Format, Content Hash Placement, and Migration
  Fixtures (serialization of ship_build_queue in save files)
- ADR-0008 — Accepted-Command Persistence (pending-command rebuild on load
  restores ship_build_queue from command_log)
