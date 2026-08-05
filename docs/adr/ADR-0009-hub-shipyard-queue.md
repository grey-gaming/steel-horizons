---
status: accepted
owner: Tech Lead
date: 2026-08-04
---

# ADR-0009: Hub Shipyard Queue Semantics

## Context

Every Station Hub has one authored shipyard work slot. Ship construction uses the
ordinary `BuildOrder` component-staging model, but it never uses a Construction
Ship: the target Hub supplies one work per tick.

The serialized model needs to distinguish the order occupying that work slot from
orders merely waiting behind it, define when component demand begins, recover
every material during cancellation, and avoid creating Fuel on completion.

## Decision

### 1. Capacity is one active slot plus an ordered pending queue

`StationStats.shipyard_slots` is active-work capacity. In V1 it must be exactly 1
for a Hub and 0 for every other station. A Hub therefore has:

- zero or one **active-slot order**: the first ID in `ship_build_queue`; and
- zero or more **pending orders**: every later ID.

V1 has no separate gameplay queue-depth cap. A valid `QueueBuildShip` may append a
pending order regardless of current component stock or Hub general-cargo free
space. API body/rate limits protect the process, but they do not change simulation
acceptance or create a platform-dependent queue limit.

Queue acceptance validates the Hub/type, target role and tier, unlocked
technology, and authored definition. It does not reject for material shortage:
that shortage is represented by the active order's `AwaitingMaterials` state.
BuildOrder staging is outside general cargo, so general cargo capacity is not a
queue-capacity test.

The queue is FIFO by `BuildOrder.created_server_sequence`, then BuildOrder ID as
an invariant-only diagnostic tie-break. Since command sequences are unique, the
tie-break is never reached in a valid state.

### 2. Serialized queue representation and invariants

The Hub `Station` carries:

```text
ship_build_queue: BuildOrderId[]
```

The array contains only nonterminal `BuildTarget::Ship` orders for that Hub. It is
included transitively in canonical state/save/replay serialization.

The complete invariants are:

- Non-Hub stations have an empty queue.
- Each ID resolves to exactly one `GameState.build_orders` record whose target Hub
  and `source_station_id` equal this Hub.
- Every nonterminal ship BuildOrder appears exactly once in exactly one Hub queue.
- `Complete` and `Cancelled` orders remain available in the top-level history but
  never remain in a Hub queue and have empty staging. A cancelled ship order has
  zero progress; a complete order retains its exact completed work total.
- IDs are in strictly increasing `created_server_sequence` order.
- The first order may be `AwaitingMaterials`, `Ready`, `Building`, or
  `Cancelling`; it is the only order occupying the active slot.
- Every later order is `AwaitingMaterials`, has zero progress, an empty
  `components_delivered` map, no builder, and no logistics reservation targeting
  it.
- Ship orders never enter `Traveling` or `Evacuating` and never have a
  `builder_ship_id`.
- For every component,
  `components_delivered[resource] <= components_required[resource]`; unlisted
  values are zero. An active `AwaitingMaterials` order has at least one strictly
  missing unit; `Ready` and `Building` require exact per-resource equality.
- `AwaitingMaterials` and `Ready` have zero progress; `Building` has
  `progress_work < total_work`; `Complete` has exact total work.
- A `Cancelling` order has zero progress and no `AwaitingPickup` reservation; it
  may temporarily contain recovery staging and retain `Loaded` reservations.
- Only an active `AwaitingMaterials` order advertises component demand. Pending,
  `Ready`, `Building`, and `Cancelling` orders advertise none.

Malformed saves violating any invariant are rejected rather than repaired or
reordered.

### 3. Creation and activation

Applying `QueueBuildShip` creates a BuildOrder with:

- `target = Ship { hub_id, role, tier }`;
- `source_station_id = hub_id`;
- the authored component cost and work total;
- empty `components_delivered`;
- zero progress, null builder, and `AwaitingMaterials` state; and
- the command's `server_sequence` as `created_server_sequence`.

The ID is appended to `ship_build_queue`.

If it is the first entry, activation occurs in that same command transaction:
matching unreserved component units move from Hub buffers into staging in
`ResourceType` order, capped exactly by the missing manifest. Staging does not
consume general-cargo capacity. If the manifest becomes exact, the order enters
`Ready`; otherwise only its missing map is exposed as BuildOrder demand during a
later logistics rebuild.

If another entry is already active, the new order remains a resource-neutral
pending record. It stages nothing, creates no inbound reservation, and advertises
no demand until it reaches the first position.

When completion or cancellation removes the first ID, the next ID occupies the
active slot immediately by queue position. Because removal commits at the tick
boundary, the next order is first seen as active on the following construction
phase. Every construction phase idempotently offers the active
`AwaitingMaterials` order unreserved local components before testing readiness;
there is no un-serialized "newly promoted" flag. Missing demand becomes visible
when a later logistics phase reads the committed staging result.

### 4. Delivery and readiness

Normal Cargo reservations may target only the active order's exact missing map.
BuildOrder staging, not Hub buffer capacity, reserves destination capacity. A
delivery is capped by the still-missing per-resource amount, preventing
overdelivery.

When `components_delivered` equals `components_required` for every key, the active
order enters `Ready`. Equality is per resource; map ordering or a language-level
map comparison is never used as a quantity relation.

### 5. Work and completion timing

During construction phase 5, the Hub examines the committed active order:

1. `AwaitingMaterials`: no work.
2. `Ready`: transition to `Building`; contribute no work on this transition tick.
3. `Building`: add exactly one checked work unit.
4. `Cancelling`: run the recovery maintenance in §6; contribute no work.

If checked progress reaches the exact authored total, completion is one atomic
transaction:

1. Validate the staged map exactly equals the authored component cost.
2. Move the entire map from `components_delivered` into the new Ship's
   `installed_components`, leaving the BuildOrder staging map empty. This is a
   move, not a copy.
3. Generate the Ship ID from the serialized ship counter and construct the exact
   authored role/tier stats.
4. Set Cargo/build holds empty with their role-appropriate null/zero fields; set
   `fuel = 0`, both Fuel remainders to zero, `state = Idle`, `job = Idle`,
   `travel_plan = null`, `docked_at = hub_id`, and position to the Hub's body
   position. Insert the ID in the Hub's canonically sorted docked-location list;
   this marker is not a persistent dock reservation.
5. Mark the BuildOrder `Complete` and remove its ID from `ship_build_queue` in the
   same transaction.

Ship construction never creates Fuel. On a later tick the ordinary deterministic
refueling rules may perform a zero-distance transfer from the Hub Fuel compartment
before the ship takes productive work. That transfer debits the Hub exactly and
therefore remains conservation-safe.

The entity creation commits at the end of the tick. Under the immutable phase
contract it is not visible to phase 9 of its creation tick and first becomes
eligible for refueling/job assignment in phase 9 of the next tick. An idle
`docked_at` marker is not a persistent dock reservation, so completion does not
require an ordinary arrival dock; the order occupied the shipyard slot until the
commit.

### 6. Cancellation and in-flight recovery

Cancellation loses only fabrication work. Components, reservations, and Cargo
payload are always recovered.

#### Pending order

A pending order has no staging or reservations by invariant. Cancellation marks
it `Cancelled` and removes its ID atomically. It never affects the active order.

#### Active order with no loaded inbound reservation

The cancellation transaction:

1. Releases every `AwaitingPickup` logistics reservation targeting the order and
   returns its source claim unchanged.
2. Moves all staged components back into compatible Hub buffers up to their
   maxima, in `ResourceType` order.
3. Places any overflow into at most one newly generated permanent salvage cache
   at the Hub position for that return transaction.
4. Clears `components_delivered`, resets `progress_work` to zero, marks the order
   `Cancelled`, and removes it from the queue.

The next pending order activates at the next construction phase as defined in
§3.

#### Active order with loaded inbound Cargo

Loaded reservations never expire and loaded Cargo is never discarded. If any
`Loaded` reservation targets the active order, the cancellation command:

1. Releases only `AwaitingPickup` reservations.
2. Returns currently staged components to Hub buffers/salvage and clears staging.
3. Resets `progress_work` to zero, sets the BuildOrder to `Cancelling`, with zero
   further demand or work, and leaves it first in the queue.
4. Leaves each loaded transport and reservation intact. The Cargo Ship completes
   its already-fuel-feasible delivery to the BuildOrder at the source Hub.

An arrival into a `Cancelling` BuildOrder is allowed and moves the loaded units
into its staging map solely as recovery transit. On the next construction phase,
the Hub moves those units from staging into compatible buffers/salvage using the
same deterministic return rule. When no `Loaded` reservation targets the order
and staging is empty, maintenance marks it `Cancelled` and removes it atomically.

This may hold the shipyard slot while remote loaded Cargo returns, but it prevents
a later build from becoming active while material belonging to the cancelled
manifest is still in flight. Awaiting-pickup Cargo jobs whose reservations were
released finish their current empty travel leg and become Idle under the ordinary
reservation-expiry/cancellation rule.

Each overflow-return transaction creates at most one salvage cache, using the
serialized salvage counter. Caches never decay and are normal logistics supply;
their IDs, positions, and maps make recovery deterministic across save/load.

### 7. Queue maintenance order

At phase 5, Hubs are processed by ascending Hub ID. For each Hub:

1. Validate the queue invariants against the committed snapshot.
2. If the first order is `AwaitingMaterials`, idempotently move any matching
   unreserved local components into its still-missing staging map.
3. Apply the state-specific work/recovery rule from §§5–6.
4. Stage terminal-order removal and Ship/salvage creation atomically.

There is no separate cleanup pass that leaves terminal IDs serialized for another
tick. All arithmetic and counter increments are checked; failure rolls the entire
tick transaction back.

## Required executable proofs

P1-15 and later logistics increments must cover:

- One active order plus multiple pending orders retain strict FIFO order through
  save/load and replay.
- Pending orders stage no stock, create no reservation, and advertise no demand.
- Promotion/local staging is idempotent, never duplicates stock, and exposes only
  the active order's missing map.
- General cargo capacity does not reject or constrain BuildOrder staging.
- Readiness uses exact per-resource equality and delivery never exceeds the
  manifest.
- Ready/Building timing contributes exactly the authored work and completion is
  visible only after commit.
- Completion clears staging, installs exactly the component recipe, and creates a
  zero-Fuel ship.
- Building then scrapping a ship cannot create Fuel or duplicate components.
- Cancellation at pending, AwaitingMaterials, Ready, Building, and Cancelling
  states preserves the whole component multiset.
- AwaitingPickup cancellation releases both reservation sides; loaded inbound
  Cargo finishes and is recovered without loss or double counting.
- Buffer overflow creates permanent, logistics-recoverable salvage.
- Every state change above retains canonical-hash, save/load, command replay, and
  whole-economy conservation equivalence.

## Consequences

- Players may plan multiple ship builds while one physical shipyard slot remains
  the only active work source.
- Pending orders cannot reserve the entire economy or create phantom demand.
- A blocked first order intentionally blocks later FIFO entries.
- Newly completed ships require real station Fuel before productive work.
- Cancellation may temporarily hold the active slot for already-loaded inbound
  Cargo, trading immediate queue progress for strict material conservation.

## Related ADRs

- ADR-0003 — Command/Query API with WebSocket Streaming
- ADR-0006 — Canonical Content/State Hashing
- ADR-0007 — Save Envelope
- ADR-0008 — Accepted-Command Persistence
- ADR-0011 — Deterministic Refueling
