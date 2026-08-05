---
status: Approved
owner: Product Owner
last-reviewed: 2026-08-04
---

# Routes & Logistics — Core Interaction

V1 uses one logistics model: stations expose distributed supply and demand, while autonomous ships select one-shot jobs. The player never assigns recurring routes to individual ships.

## Player Controls

The player influences transport through:

- Station placement
- Production and mining configuration
- Per-buffer demand and export thresholds
- Station priority from 0–100
- Ship count, tier, and current location
- Survey and construction queues

Priority affects future idle-ship selection and never interrupts an in-transit job.

## Derived Supply and Demand

The simulation rebuilds ephemeral tables once per tick from committed buffers, ordinary BuildOrder staging manifests, demolition evacuation manifests, permanent salvage caches, the active Gate manifest, and persistent reservations. When an order is created, matching unreserved components already at its source Hub move directly into staging without a Cargo Ship. Salvage advertises all unclaimed inventory at priority 50 with an export floor of zero. A BuildOrder advertises its exact remaining components at the priority of its source Hub and receives external supply through that Hub's dock. After `BeginGateAssembly`, the Gate site advertises the still-undelivered canonical manifest at priority 100; neither manifest ever requests more than the exact required amount.

Demolition evacuation pairs are source-locked: the supply must be the target station and the destination is the order's recovery cache at the selected Hub. They include input/output buffers, Fuel, completed outputs, and released inputs, use priority 100, and are assigned before ordinary compatible pairs using the same ship/distance tie-breaks. Awaiting inbound reservations are released when demolition queues; already-loaded arrivals finish and become new evacuation supply. New production, extraction, research, and ordinary inbound demand remain disabled while Evacuating.

- An input buffer demands material when `current + inbound_reserved` is below its demand-threshold target.
- An output buffer supplies material when `current - outbound_reserved` is above its export-threshold floor.
- A station Fuel compartment participates in both rules using its demand and export thresholds; it remains outside general cargo capacity.
- The amount is the exact distance to the applicable threshold, capped by free capacity or available inventory.
- A buffer can never advertise reserved units to a second ship.

“Broadcast” is the player-facing metaphor for these derived entries; there is no persisted event table.

## Deterministic Cargo Matching

Idle Cargo Ships are considered in ascending ship ID order. For each compatible supply/demand pair:

```text
pickup_distance_milli = route_distance(ship_position, supply_position)
delivery_distance_milli = route_distance(supply_position, demand.position)
distance_penalty = (pickup_distance_milli + delivery_distance_milli) / 20_000
score = demand.priority * 4 + supply.priority * 3 - distance_penalty
```

`route_distance` uses the radial-burn plus destination-arc model in GDD 12. One distance-penalty point equals 20 travel units, so station priority remains meaningful at authored scale. The calculation includes the ship's trip to the supplier; a remote idle ship no longer treats a distant source/destination pair as “closest.”

Selection order is:

1. Highest score
2. Lowest raw `pickup_distance_milli + delivery_distance_milli`
3. Lexicographically lowest canonical supply key (`station:<id>` or `salvage:<id>`)
4. Lexicographically lowest canonical demand key (`station:<id>`, `build_order:<id>`, `evacuation:<id>`, or `gate_site`)
5. Resource enum order

The ship rejects pairs it cannot complete with its current Fuel plus 10% reserve. Every empty, unreserved idle role below `max_fuel` first evaluates the exact direct-refuel policy below.

## Reservation Lifecycle

On selection, the simulation atomically reserves both source inventory and destination capacity. The cargo amount is the minimum of ship free capacity, available supply, requested demand, destination capacity, and fuel-feasible capacity.

| Reservation state | Behavior |
|-------------------|----------|
| AwaitingPickup | Source inventory and destination capacity are reserved. Expiry is estimated pickup arrival + 600 ticks. |
| Loaded | Inventory has moved to the ship; reservation no longer expires. |
| Delivered | Destination capacity is consumed and the reservation closes. |
| Released | Awaiting inventory/capacity is returned after cancellation, invalidation, or expiry. |

Reservations are serialized. After load, derived tables are rebuilt and active reservation amounts are subtracted before any new matching.

When an AwaitingPickup reservation expires, both sides release and the empty Transport job is marked cancelled. A ship already in transit finishes its current leg without loading, then becomes Idle at that exact endpoint; a ship already at an endpoint becomes Idle immediately. Loaded cargo is never discarded or expired.

## Cargo Job Sequence

1. Travel empty to the source.
2. Reserve one available source dock for the arrival tick; salvage-cache pickups require no dock.
3. Atomically load the reserved amount.
4. Travel loaded to the destination.
5. Reserve the destination Station dock, source-Hub dock for BuildOrder staging/evacuation-cache delivery, or the Gate site's single transfer berth for the arrival tick.
6. Atomically unload and complete the reservation.
7. Become eligible for another job during that tick's logistics phase and depart no earlier than the next tick.

A ship carries one resource type per trip. Cargo transfer has no multi-tick gameplay duration; animations may continue cosmetically after the transaction.

## Docks and Holding

Docks are transactional capacity as well as visible berths:

- Cargo arrival occupies one dock for its transaction tick.
- The Gate site has one virtual cargo-transfer berth. It is not a station or orbit slot; arrivals beyond one transaction in a tick hold and retry in Ship ID order.
- A Research Ship powering a project occupies a dock persistently.
- A ship under construction occupies the Hub shipyard slot, not an ordinary dock until completion.
- If all docks are reserved, an arriving ship enters Holding and consumes no Fuel.
- An input buffer cannot become overfull because destination capacity was reserved at job assignment.

## Construction and Survey Jobs

Construction Ships select only Ready Station/Upgrade/Demolish BuildOrders, ordered by creation server sequence, estimated travel cost, then BuildOrder ID. Ship BuildOrders remain in their Hub's single shipyard queue and never receive a Construction Ship.

Research Ships first serve survey orders sorted by descending queue priority, creation server sequence, then SurveyOrder ID. For each order, choose the eligible idle ship with lowest travel cost, then Ship ID. If none is idle, use the same tie-break over ships powering research and pause the selected ship's project with `NoResearchShip`. A ship never continues to an unqueued body. Next, sort Research Station projects needing ships by descending station priority, project creation server sequence, then Station ID; for each, choose the nearest eligible idle ship, then Ship ID. A completed/cancelled survey leaves its ship eligible for research assignment in the next logistics phase.

## Fuel Safety

Route feasibility uses both legs and the ship's base mass plus cargo. If Fuel production fails:

1. For each empty, unreserved idle ship below `max_fuel`, compute each station's direct-refuel stock from the phase-8 result: post-debit Fuel minus `AwaitingPickup` logistics reservations and production/research holds. The percentage export floor applies to Cargo supply, not local ship refueling. Phase-9 Cargo Fuel reservations reduce this fact budget in assignment order; direct-refuel jobs do not reserve it.
2. Select a station with positive direct-refuel stock that the ship can reach by the exact ordinary Fuel simulator using its cloned remainders and completed Life Support state; tie-break by route distance, then Station ID. Refuel work precedes ordinary role work.
3. If no source is reachable and the ship is already docked at a Hub, it waits and retries without self-rescue. Otherwise it enters AwaitingRescue for exactly 300 ticks.
4. The nearest existing Hub (route distance, then Hub ID) sends its solar tug, which tows the ship directly home at half base speed without ship Fuel. On docking its pre-tow Fuel and both remainders are unchanged, and it retries direct refueling.

The tug cannot carry cargo or do productive work. A Refuel arrival competes for a transactional dock. Phase 8 charges its final movement before transferring unreserved station Fuel, and same-tick arrivals allocate partial/zero stock in Ship ID order. Route acceptance guarantees loaded/reserved ships have enough Fuel to finish, so entering AwaitingRescue with cargo or an active reservation is an invariant violation.

## Buffer Configuration

Each station panel exposes:

```text
Station: Refinery-2
  Inputs
    Metal Ore  20/100  demand below 50%
  Outputs
    Metals     70/100  export above 60%
  Priority: 50
```

Threshold changes take effect in the next logistics phase. Invalid configurations—threshold outside 0–100, an input maximum below `current + inbound_reserved`, an output maximum below `current` (outbound reservations are already part of current), summed maxima above total cargo capacity, changing an authored Fuel-compartment maximum, Fuel demand above Fuel export threshold, or an unsupported resource—are rejected atomically. Fuel thresholds remain configurable. Reconfiguration never deletes inventory.

## Bottleneck Detection

For each destination/resource that remains below its demand target:

- Track delivered units over the last 600 ticks.
- Compare delivery throughput with configured recipe consumption.
- If delivery remains below consumption for 300 consecutive ticks, emit a bottleneck warning.

Warnings are per destination/resource and advisory only. Suggested remedies include more ships, higher capacity, nearer supply, increased priority, or additional production. A warning clears after 300 ticks without the deficit.

The 600 delivery buckets, cursor, rolling total, consecutive-deficit and consecutive-clear counts, and warning flag are serialized. Save/load therefore preserves warning/clear timing and the canonical deterministic event payload; the server-session event envelope sequence is intentionally outside gameplay equivalence.

## Visualization

- Curved flow lines visualize active reservations and recent deliveries, not permanent routes.
- Color and icon/dash pattern identify the resource.
- Thickness represents trailing 600-tick throughput.
- A density overlay aggregates actual completed travel segments.
- Clicking a line opens a flow inspection panel listing the dynamic jobs currently contributing to it.

V2 inter-system Gate logistics remains a separate concept in [v2-gate-logistics.md](./v2-gate-logistics.md).
