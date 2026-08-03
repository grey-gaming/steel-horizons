# Simulation Foundations — Tick Rate & Game Loop

## Tick Rate

The game runs on a **discrete tick simulation**. One tick equals one second of game time. All simulation logic advances by one tick per real-time second.

Real-time and game-time are 1:1. There is no time acceleration or pause (V1 scope).

## What Happens Each Tick

In order, every tick:

1. **Ship movement** — each ship advances along its current path by `speed × lane_multiplier` distance units. If it reaches its destination, it transitions to the arrival state.
2. **Station processing** — each powered factory/refinery consumes input materials and produces output materials according to its throughput rate. Output is added to the station's output buffer.
3. **Mining extraction** — each powered mining station adds resources to its output buffer according to its extraction rate.
4. **Construction progress** — active construction (ships at shipyard, stations at build site, Gate assembly) advances by 1 tick toward completion.
5. **Survey progress** — Research Ships currently surveying a body advance toward completion.
6. **Research progress** — each Research Station with sufficient materials advances its active project by 1 tick.
7. **Fuel consumption** — ships in transit consume fuel based on distance traveled this tick.
8. **Logistics re-evaluation** — ships that completed a job this tick pick a new job. Ships that were idle check for available work. (See job model below.)
9. **Bottleneck detection** — the throughput monitor updates its moving window. (See 07-routes-and-logistics.md for the detection algorithm.)

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
2. If there's an **unsurveyed body** and a Research Ship is idle, it takes the nearest unsurveyed body.
3. For Cargo Ships: find the closest station pair (A→B) where A has surplus output of resource X and B has demand for X, and the distance-weighted value is highest. The ship takes a transport job for that cargo.
4. If no work is found, the ship remains idle.

### Job Completion & Re-Evaluation

A ship re-evaluates only when:
- Its current job completes (cargo delivered, survey done, build done)
- The player changes a station's priority (triggers idle ships to re-scan, but does not interrupt ships already in transit)
- A new build order or ship order is placed (idle Construction Ships re-scan)

Ships **in transit are never interrupted**. Once a ship loads cargo and departs, it commits to that delivery.

## Logistics Event Model

Station buffers drive logistics **through demand/supply broadcasts**, not a global poll:

- **Demand broadcast**: When a station's input buffer for resource X drops below its threshold, it broadcasts a demand: "I need X." This adds an entry to a global demand table.
- **Supply broadcast**: When a station's output buffer for resource X rises above its export threshold, it broadcasts a supply: "I have X available." This adds an entry to a global supply table.
- **Matching**: Idle cargo ships consult the supply/demand tables to find the best match. They do not poll every station.

Demand/supply entries are removed when the buffer crosses back below/above the threshold.

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

Each component requires its recipe cost (defined in 03-economy.md) as input. Inputs are consumed smoothly per tick. If any input is insufficient, that production slot stalls.

### Station Hub Basic Assembly (Bootstrapping)

A Station Hub (Tier 1) can assemble **Tier 1 components only** at a rate of 1 component per 30 ticks (3× slower than a Construction Factory). This allows early-game bootstrap before a Construction Factory is built. The Hub requires the refined goods as input, drawn from its output buffer. Only the following components can be assembled at a Hub:

- Structural Frame (requires Metals + Carbon Fiber)
- Cargo Module (requires Metals + Carbon Fiber + Control System)

All other components require a dedicated Construction Factory.

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

Every entity defined in 13-data-models.md is serialized:

- All `CelestialBody` instances (survey state, resources)
- All `Station` instances (buffers, tier, priority, build progress)
- All `Ship` instances (position, cargo, fuel, job, state)
- All active `ResearchProject` instances (progress, resources consumed)
- The global supply/demand tables (can be rebuilt on load from station buffer states — these are runtime ephemeral)
- Tick counter (elapsed game time)
- Gate build progress (if any)

### Save Frequency

Auto-save every 300 ticks (5 minutes real-time). Manual save on quit. One save slot (V1 scope — no multiple save files).

### What Is Not Saved

- Visual-only state (camera position, zoom band, UI panel open/close state) — these are transient and reset to defaults on load.

## Performance Assumptions

- Max ~500 ships, ~200 stations, ~20 resource types
- Job assignment is O(supply_entries × demand_entries) per idle ship — cheap because tables are sparse
- Ship movement is O(ships) — linear, no spatial index needed at V1 scale
- Station processing is O(stations × resource_types) — linear
