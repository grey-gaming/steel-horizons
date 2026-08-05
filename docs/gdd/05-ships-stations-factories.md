---
status: Approved
owner: Product Owner
last-reviewed: 2026-08-04
---

# Ships, Stations & Factories

This document defines entity behavior and progression. Exact statistics, costs, rates, and work requirements are owned by [14-authored-content.md](./14-authored-content.md).

## Ships

Ships are autonomous drones. The player creates work through station configuration, build orders, and survey targets; ships select and execute eligible jobs deterministically.

V1 has deterministic, non-editable display names. Generated ships use `<authored tier name>-<generated ship counter>` and generated stations use `<station type>-<generated station counter>`; authored Hub Haven and Builder-1 keep their scenario names. Renaming is post-V1 and names never affect ordering.

### Cargo Ships

| Tier | Name | Role |
|-----:|------|------|
| 1 | Courier | Entry-level transport |
| 2 | Hauler | Larger and more fuel-efficient long-haul transport |
| 3 | Bulk Carrier | Maximum bulk capacity |
| 4 | Fast Freighter | High-speed priority delivery |

A Cargo Ship carries one resource type per trip. Capacity, speed, base mass, and fuel capacity all participate in route feasibility. “Armour,” perishable goods, manual route assignment, and quality mechanics are post-V1 and have no V1 fields or UI.

### Construction Ships

| Tier | Name | Role |
|-----:|------|------|
| 1 | Builder | Basic station construction |
| 2 | Constructor | Two build work per tick |
| 3 | Engineer | Three build work per tick |
| 4 | Fabricator | Five build work per tick; required for the Gate |

Construction Ships accept Build, Upgrade, and Demolish jobs. Cargo Ships first deliver all required component types into the BuildOrder staging area at its source Hub. The assigned Construction Ship loads that map into its dedicated build hold, travels to the site, and contributes its authored build work each tick. A ship works on one order at a time; multi-build mechanics are post-V1.

Orbital Logistics—not a ship-tier workaround—is required to place Mining Stations in The Veil.

### Research Ships

| Tier | Name | Maximum survey depth |
|-----:|------|---------------------:|
| 1 | Scout | 1 — surface |
| 2 | Surveyor | 2 — subsurface |
| 3 | Explorer | 3 — full |
| 4 | Pioneer | 3 — faster full survey |

Research Ships perform Survey jobs and power any project assigned to a Research Station. A ship cannot survey and power research simultaneously. A station project automatically requests the nearest eligible idle Research Ship; if none is idle, it waits without losing value. This creates the intended choice between a second ship and intermittent station research without making early research circular: Hub Haven's built-in console handles Tier-1 technology without a ship.

Survey jobs are player-queued and persist as `SurveyOrder` records until assigned, completed, or cancelled. A Research Ship does not automatically leave for another body unless another queued target exists. Each completed depth milestone updates the body immediately. Cancelling an assigned survey preserves completed depths and loses only work accumulated toward the next depth. A ship already scanning becomes Idle immediately at the body; a ship en route completes its current travel leg without scanning, then becomes Idle at that exact endpoint.

## Stations

Stations occupy fixed slots on orbit rings. Every station has installed components, buffers, one or more docks, a priority, and role-specific state. A Power Core is required for operation.

### Station Hub

The Hub is the logistics and shipbuilding center. Every tier has one shipyard queue and one slow component-assembly slot; Hub upgrades increase docks/storage rather than parallel build slots. The shipyard queue is FIFO with exactly one active order. Only the first order stages local components, accepts Cargo reservations, and advertises its missing manifest; later orders remain resource-neutral until promotion. Completed ships start with zero Fuel and zero Fuel remainders, then use ordinary conservation-safe refueling from a real station compartment (ADR-0009). It provides:

- General cargo storage plus a separate Fuel compartment
- Ship construction, one ship order at a time
- Slow assembly of unlocked non-Gate components
- Tier-1 research through Hub Haven's built-in console
- A slow solar rescue tug for empty, unreserved stranded ships

Higher tiers increase docks and storage. Automated route assignment and fleet coordination remain simulation-wide V1 behavior; they are not future Hub “specials.”

### Mining Station

Mining Stations extract authored deposits from their parent body. Tier determines simultaneous targets and base throughput. Changing a target takes 10 ticks and does not consume components. A target is legal only when its deposit is visible at the body's current survey depth. `SetMiningTarget` creates a default output buffer under GDD 14's non-destructive allocation rule or rejects if the station cannot fit its one-unit minimum.

Planet and moon deposits decrement by whole extracted units. Belt density never depletes and modifies output rate around its baseline.

### Refinery

Refineries reserve complete recipe inputs, process for ten ticks, and transfer complete outputs. Tiers provide one through four parallel slots. Recipe technology and minimum refinery tier are both required.

### Construction Factory

Construction Factories assemble any unlocked non-Gate component. Tier controls one through four parallel slots. Gate Nodes additionally require Gate Construction and Tier 4.

They can also run the exact inverse disassembly recipe for any component available at that tier. Hub Haven can disassemble unlocked non-Gate components at its slower assembly rate.

### Research Station

A Research Station runs one project at a time and always requires a docked Research Ship. Tier-2+ technologies can run only at a Research Station; Tier-1 may instead use Hub Haven's built-in console. Station tier controls buffer/dock capacity, not the number of concurrent projects. Multiple stations enable parallel technologies.

## Building and Upgrading

### New Structures

1. The player chooses an eligible empty slot and station tier.
   If more than one Hub exists, the player also chooses the source Hub used for staging and Construction Ship dispatch.
2. A BuildOrder moves locally available unreserved Hub components into staging and exposes only the remaining map as demand at its source Hub; ordinary per-ship logistics reservations claim remote supply.
3. Cargo Ships deliver components through the source Hub dock into BuildOrder staging. When the full manifest is staged, the order becomes Ready.
4. An idle Construction Ship that can hold the complete manifest takes the Ready order, loads it atomically, and travels to the site.
5. Build work advances until the authored requirement is met.
6. The completed station uses GDD 14's exact empty/default constructor. The moved
   build hold becomes its installed components, and the Construction Ship becomes
   Idle at/docked to that station with an empty hold; no Fuel or inventory is created.

No station is placed instantly by an API command.

A body requires survey depth 1 before ordinary station placement. Mining targets additionally require the individual deposit's authored minimum survey depth. Gas giants have no slots, and The Veil requires Orbital Logistics.

### Upgrades

Upgrade cost equals the canonical full cost of the target tier minus the installed component counts. The station continues operating at its current tier while work proceeds. On completion, tier, capacity, slots, and docks change atomically.

`QueueUpgrade` includes an explicit source Hub. Its component delta is staged there like a station build, loaded into the assigned Construction Ship, and carried to the target station.

Tier skipping is allowed if the target technology is unlocked and the complete positive delta is supplied.

### Cancellation, Demolition, and Scrapping

- Cancelling releases network reservations. Staged components return to the source Hub/Hub-side salvage; an assigned Construction Ship returns any loaded build hold before becoming idle.
- `QueueDemolishStation` creates a demolition BuildOrder whose recovery Hub is also its dispatch/source Hub plus an empty permanent recovery cache at that Hub. The target stops mining/production, pauses/detaches research, releases cancellable reservations into existing buffers, and exposes source-locked evacuation jobs for all ordinary inventory and Fuel. Loaded inbound cargo may finish delivery and is then evacuated. Once docks, buffers, outputs, and reservations are clear, 30 base work begins and the Construction Ship returns installed components to the Hub or the same cache.
- Cancelling a demolition before completion re-enables the station. Already evacuated material remains recoverable in the Hub-side cache; an empty cache is removed.
- Scrapping is allowed only while docked at a Hub. Cargo, Fuel, and the ship's component recipe fill compatible Hub compartments first; all overflow goes to a permanent cache at that Hub.
- Returned components go to the selected Hub or a permanent salvage cache at that Hub when storage is unavailable. `ScrapShip` has no separate destination: its recovery Hub is the Hub where the ship is docked.
- The final Hub cannot be demolished.

These rules preserve the always-solvable invariant.

## Build Work

| Build | Required work |
|-------|--------------:|
| Ship T1/T2/T3/T4 | 30 / 60 / 120 / 240 |
| Station T1/T2/T3/T4 | 60 / 120 / 240 / 480 |
| Upgrade | 60 × tiers advanced |
| Demolition | 30 |

Hub shipyards contribute one ship-build work per tick. Construction Ships contribute their authored work per tick to stations, upgrades, demolition, and Gate phases. Pause freezes work.

## Space Gate

The Gate is a unique fringe-lane megastructure, not an orbit-slot station.

1. Complete Gate Theory, Advanced Fabrication, Grid Power, Gate Construction, and System Bridge.
2. Produce eight Gate Nodes in a Tier-4 Construction Factory.
3. Select an idle Tier-4 Fabricator with `BeginGateAssembly`; it travels to the fixed site and exposes the exact Gate manifest as logistics demand.
4. Deliver the Nodes, Structural Frame, Power Core, Control System, and activation Reactor Rod to the site's single transfer berth.
5. Complete Site Preparation, Frame Assembly, Power Integration, and Activation.
6. Consume the delivered Reactor Rod for activation and transition immediately to Won.

The exact site and work values are defined in GDD 14.
