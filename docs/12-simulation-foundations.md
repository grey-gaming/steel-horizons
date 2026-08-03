---
status: Draft
owner: Tech Lead
last-reviewed: 2026-08-03
---

# Simulation Foundations — Tick Rate & Game Loop

## Tick Rate

The game runs on a **discrete tick simulation**. One tick equals one second of game time. All simulation logic advances by one tick per real-time second.

Real-time and game-time are 1:1. There is no time acceleration in V1 scope. Pause (Space key) freezes the simulation tick loop — all ship movement, station processing, construction, research, and logistics evaluation stop. The UI remains interactive for planning: the player can inspect entities, open panels, and queue build orders. Autosave does not occur while paused.

## Numeric Representation

All simulation uses **integer arithmetic** with explicit remainder accumulators. Fractional rates (e.g., 0.1 units/tick) are not represented as floating-point amounts — instead each production/buffer tracks a remainder field that accumulates fractional progress until it reaches a whole unit.

**Mechanism:**

- Every production rate is stored as a rational value. At simulation start, each production source is assigned a **rate numerator** (units per cycle) and a **cycle length** (10 ticks by default). The per-tick accumulator adds `rate × 1000` each tick.
- When the accumulator ≥ 1000, exactly 1 integer unit is added to the buffer and 1000 is subtracted from the accumulator. The remainder (0–999) persists across ticks and is serialized.
- Fuel consumption follows the same pattern: ships consume fuel at a fractional rate per tick; the fuel remainder accumulates and a whole unit is deducted when the remainder crosses 1000.

**Serialization:**

- Remainders are serialized as `remainder: int` (0–999, scaled by 1000). This ensures save/load preserves fractional progress with no drift.
- A field `remainder: int` on every Buffer (see [13-data-models.md](./13-data-models.md)) tracks fractional production that hasn't reached 1.0 yet.
- Ships carry `fuelRemainder: int` for fractional fuel consumption.

**Rounding boundary:**

- All fractional rates are defined as rational numbers with a denominator of 10 (cycles of 10 ticks). Rate × 1000 always produces a multiple of 100, so the accumulator never exceeds 9999 between ticks in a single rate group — conservation is exact.

**Conservation invariant:**

- Within each rate group (a set of production sources that share a resource flow), the sum of all remainder fields across all input/output buffers is guaranteed to be < 1000 per tick step. No material is created or destroyed by rounding. Over the lifetime of the simulation, total production matches total consumption plus buffer deltas exactly.

**Why integer arithmetic:**

- Floating-point accumulators cause silent drift over long simulations, produce non-deterministic results across CPU architectures, and cannot be serialized losslessly without a fixed-point scheme. The 1000-scaled integer remainder approach is deterministic, serializable, and exact.

## What Happens Each Tick

**Within-tick visibility:** Each tick phase reads from the state at the start of the tick and writes to a pending buffer that is committed at the end of the tick. A phase cannot see the outputs of a later phase in the same tick — cargo arriving during movement cannot be processed by the receiving station until the next tick's logistics phase. State is immutable during a phase's reads; all writes are deferred until commit.

**Missed-time handling:** If the application frame is stalled (backgrounded, dropped frame), the tick accumulator catches up by processing missed ticks at 1 tick per frame. A maximum of 10 ticks per frame is allowed — beyond that, the game slows down rather than trying to catch up. Offline progress is not simulated (V1 scope: pausing while backgrounded is equivalent to freezing; no background simulation).

In order, every tick:

1. **Ship movement** — each ship advances along its current path by `speed × lane_multiplier` distance units. If it reaches its destination, it transitions to the arrival state.
2. **Station processing** — each powered factory/refinery consumes input materials and produces output materials according to its throughput rate. Output is added to the station's output buffer.
3. **Mining extraction** — each powered mining station adds resources to its output buffer according to its extraction rate.
4. **Construction progress** — active construction (ships at shipyard, stations at build site, Gate assembly) advances by 1 tick toward completion.
5. **Survey progress** — Research Ships currently surveying a body advance toward completion.
6. **Research progress** — each Research Station with sufficient materials advances its active project by 1 tick.
7. **Fuel consumption** — ships in transit consume fuel based on distance traveled this tick.
8. **Logistics re-evaluation** — ships that completed a job this tick pick a new job. Ships that were idle check for available work. (See job model below.)
9. **Bottleneck detection** — the throughput monitor updates its moving window. (See [07-routes-and-logistics.md](./07-routes-and-logistics.md) for the detection algorithm.)

## Ship Job Model

Ships do **not** re-evaluate their task every tick. Each ship holds a **job** — a persistent task it works on until completion:

| Job Type | Description | Completion Condition |
|----------|-------------|---------------------|
| **Transport** | Fly to source station, load cargo, fly to dest station, unload | Cargo delivered |
| **Survey** | Fly to celestial body, survey it for N ticks | Survey timer expires |
| **Build** | Fly to build site, construct structure for N ticks | Build timer expires |
| **Idle** | No job — ship waits at a station | N/A (picks a job when idle) |

### Job Assignment (Idle Ships)

An idle ship scans the network for available work. The evaluation order:

1. If there's a **pending build** (player placed a new station/factory/ship) and a Construction Ship is idle nearby, it takes the build job.
2. If there's a **queued survey target** (player clicked a body and selected 'Survey') and a Research Ship is idle, it takes the nearest queued unsurveyed body. If no survey queue exists, the ship remains idle at dock.
3. For Cargo Ships: find the closest station pair (A→B) where A has surplus output of resource X and B has demand for X, and the distance-weighted value is highest. The ship takes a transport job for that cargo.
4. If no work is found, the ship remains idle.

### Job Completion & Re-Evaluation

A ship re-evaluates only when:
- Its current job completes (cargo delivered, survey done, build done)
- The player changes a station's priority (triggers idle ships to re-scan, but does not interrupt ships already in transit)
- A new build order or ship order is placed (idle Construction Ships re-scan)

Ships **in transit are never interrupted**. Once a ship loads cargo and departs, it commits to that delivery.

### Reservation Model

Job assignment uses a **reservation model** to prevent double-booking. When an idle cargo ship selects a supply–demand pair, it atomically reserves the cargo amount against both the supply entry and the destination's input buffer capacity. Reservations are tracked in the logistics runtime tables (see `Reservation` struct in 13-data-models.md).

**Reservation rules:**

- **Atomic reservation**: When a ship picks a transport job, it reserves `cargoAmount` from the chosen supply entry's `amount` field and `cargoAmount` of capacity from the destination station's input buffer. Reserved amounts are subtracted from the available totals visible to other ships.
- **Reservation release**: A reservation is released when the ship completes delivery (cargo transferred, reservation consumed), when the ship is destroyed, or when the ship remains idle for more than 600 ticks (10 minutes) without starting the job. On release, the reserved amount is added back to the supply/demand entry's available total.
- **Exclusion**: A supply or demand entry with `reserved >= amount` is treated as fully claimed and excluded from other ships' job selection.
- **Idle timeout**: If a ship selects a job but does not depart within 10 ticks (e.g., docks are busy), the reservation is temporarily held. After 600 idle ticks, it is released and the ship reverts to idle.

### Logistics Event Model

Station buffers drive logistics **through demand/supply broadcasts**, not a global poll:

- **Demand broadcast**: When a station's input buffer for resource X drops below its threshold, it broadcasts a demand: "I need X." This adds an entry to a global demand table.
- **Supply broadcast**: When a station's output buffer for resource X rises above its export threshold, it broadcasts a supply: "I have X available." This adds an entry to a global supply table.
- **Matching**: Idle cargo ships consult the supply/demand tables to find the best match. They do not poll every station.

Demand broadcasts are removed when the buffer rises back above its demandThreshold (replenished by incoming supply). Supply broadcasts are removed when the buffer drops back below its exportThreshold (depleted by outgoing deliveries). This prevents one-tick oscillation: a broadcast persists until the opposite crossing is confirmed.

## Rate Definitions — Throughput & Cycles

All production, extraction, and construction uses **cycles** as the base unit. One cycle = 10 ticks (10 seconds real-time).

### Mining Station Extraction Rates

| Tier | Output per Cycle | Output per Tick | Storage Buffer |
|------|-----------------|----------------|----------------|
| 1    | 1 unit/cycle    | 0.1 units/tick | 100            |
| 2    | 2 units/cycle   | 0.2 units/tick | 200            |
| 3    | 3 units/cycle   | 0.3 units/tick | 400            |
| 4    | 5 units/cycle   | 0.5 units/tick | 800            |

Output is added to the station's output buffer each tick at the per-tick rate (smooth accumulation, not burst at cycle end).

### Refinery Factory Throughput

| Tier | Cycles | Input per Cycle | Output per Cycle | Throughput per Tick |
|------|--------|----------------|------------------|-------------------|
| 1    | 1 at a time | 1 unit input → 1 unit output | 0.1 units/tick |
| 2    | 2 parallel | 1 unit input each → 1 unit output each | 0.2 units/tick |
| 3    | 3 parallel | 1 unit input each → 1 unit output each | 0.3 units/tick |
| 4    | 4 parallel | 1 unit input each → 1 unit output each, quality bonus | 0.4 units/tick |

Refineries consume input and produce output smoothly per tick, not in bursts. Input must be available in the station's input buffer; if insufficient, that cycle's production stalls.

### Construction Factory Throughput

| Tier | Components per Cycle | Throughput per Tick |
|------|---------------------|-------------------|
| 1    | 1 component/cycle   | 0.1 components/tick |
| 2    | 2 parallel          | 0.2 components/tick |
| 3    | 3 parallel          | 0.3 components/tick |
| 4    | 4 parallel, bulk    | 0.4 components/tick |

Each component requires its recipe cost (defined in [03-economy.md](./03-economy.md)) as input. Inputs are consumed smoothly per tick. If any input is insufficient, that production slot stalls.

### Station Hub Basic Assembly (Bootstrapping)

A Station Hub (Tier 1) can assemble all **Tier 1 components** at a rate of 1 component per 30 ticks (3× slower than a Construction Factory). This allows early-game bootstrap before a Construction Factory is built. The Hub requires the refined goods as input, drawn from its output buffer. The Tier 1 components that can be assembled at a Hub are:

- Structural Frame (requires Metals + Carbon Fiber)
- Cargo Module (requires Metals + Carbon Fiber + Control System)
- Power Core (requires Reactor Rods + Alloys)
- Control System (requires Silicon Wafers + Optics)
- Drive Assembly (requires Alloys + Fuel + Control System)
- Research Lab (requires Silicon Wafers + Optics + Power Core)
- Construction Bay (requires Structural Frame + Power Core + Control System)

All other components (Gate Node and above) require a dedicated Construction Factory.

### Lane Multipliers for Ship Speed

Each orbital lane has a speed multiplier applied to a ship's base speed:

| Lane | Multiplier | Characteristic |
|------|-----------|----------------|
| Inner | 1.5× | Fast — short orbits, small planets |
| Habitable | 1.0× | Baseline — moderate distance |
| Outer | 0.7× | Slow — long orbits |
| Fringe | 0.5× | Very slow — edge of system |

Ship effective speed = `baseSpeed × laneMultiplier × (1 - cargoLoad / maxCapacity × 0.3)` (cargo penalty: a full ship moves 30% slower).

Distance traveled per tick = effectiveSpeed. Travel time between two bodies = `angularDistance × orbitalRadius / effectiveSpeed`.

## Save / Load & Persistence

The game state must be fully serializable to JSON for save/load. Each tick the simulation operates on a mutable state object; at save time, the entire state tree is serialized. At load time, it is deserialized and the simulation resumes from tick N+1.

### Serializable State

Every entity defined in [13-data-models.md](./13-data-models.md) is serialized:

- All `CelestialBody` instances (survey state, resources)
- All `Station` instances (buffers, tier, priority, build progress)
- All `Ship` instances (position, cargo, fuel, job, state)
- All active `ResearchProject` instances (progress, resources consumed)
- All active `BuildOrder` instances (components required/delivered, builder assignment, progress, state)
- All `Reservation` instances (active cargo reservations — serialized for deterministic replay)
- `RNGState` (seed and call count — ensures belt drift and any future randomness is deterministic across save/load)
- Tick counter (elapsed game time)
- Gate build progress (if any)
- Schema version (`schemaVersion` field in GameState — enables future migration)

### Save Frequency

Auto-save every 300 ticks (5 minutes real-time). Manual save on quit. One save slot (V1 scope — no multiple save files). Saves are written atomically (write to temp file, rename to final path) to prevent corruption from crash during write.

### Migration

- Schema version is checked on load. If save version < current version, a migration function runs before the simulation resumes.
- Migration functions are one-way (no downgrade support in V1).
- If save version > current version, the game refuses to load and shows "Save from a newer version — please update the game."

### What Is Not Saved

- Supply/demand runtime tables (these are rebuilt on load from station buffer states — they are runtime ephemeral)
- Visual-only state (camera position, zoom band, UI panel open/close state) — these are transient and reset to defaults on load.

## Performance Assumptions

These are **V1 ceilings** — the simulation must handle these numbers without frame drops. Actual starting system capacity is ~16–26 slots (see [02-the-system.md](./02-the-system.md)). The ceilings account for station upgrades, multiple mining outposts, and future system expansion.

- Max ~500 ships, ~200 stations, ~20 resource types
- Job assignment is O(supply_entries × demand_entries) per idle ship — cheap because tables are sparse
- Ship movement is O(ships) — linear, no spatial index needed at V1 scale
- Station processing is O(stations × resource_types) — linear
