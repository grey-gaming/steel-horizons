---
status: Draft
owner: Tech Lead
last-reviewed: 2026-08-03
---

# Ships, Stations & Factories

## Ships

Ships are the backbone of your logistics network. They are autonomous drones — you never assign routes. Build them and they self-organize to move materials where they're needed.

### Cargo Ships

Purpose: Move materials from place to place.

| Tier | Name | Capacity | Speed | Cost (Components) | Special |
|------|------|----------|-------|-------------------|---------|
| 1 | Courier | 50 units | Medium | 1 Frame + 1 Drive + 1 Cargo Module | None |
| 2 | Hauler | 120 units | Medium | 1 Frame + 1 Drive + 2 Cargo Modules | Better fuel efficiency |
|| 3 | Bulk Carrier | 300 units | Slow | 2 Frames + 2 Drives + 3 Cargo Modules | Can carry any cargo, armored (mechanics TBD) |
| 4 | Fast Freighter | 100 units | Fast | 1 Frame + 2 Drives + 1 Cargo Module | Rush deliveries, perishable goods (mechanics TBD) |

Higher-tier cargo ships can carry more, travel faster, or handle special conditions (long distances, high-gravity planets).

### Construction Ships

Purpose: Build stations, factories, and the Space Gate. They carry construction crews and components to a build site.

| Tier | Name | Build Speed | Cost | Special |
|------|------|------------|------|---------|
| 1 | Builder | Slow | 1 Frame + 1 Drive + 1 Construction Bay | Basic structures only |
| 2 | Constructor | Medium | 1 Frame + 1 Drive + 1 Construction Bay | Can build on any terrain |
| 3 | Engineer | Fast | 2 Frames + 2 Drives + 2 Construction Bays | Builds multiple structures at once (mechanics TBD) |
| 4 | Fabricator | Very Fast | 2 Frames + 2 Drives + 2 Construction Bays | Can build in hazardous zones (asteroid belts, volcanic) |

Construction ships travel to a site, deploy, and build. The player chooses what to build and where — the ship handles the construction.

### Research Ships

Purpose: Survey celestial bodies for resources and unlock research data.

| Tier | Name | Survey Speed | Cost | Special |
|------|------|-------------|------|---------|
| 1 | Scout | Slow | 1 Frame + 1 Drive + 1 Research Lab | Basic surface survey |
| 2 | Surveyor | Medium | 1 Frame + 1 Drive + 1 Research Lab | Deep subsurface scan |
| 3 | Explorer | Fast | 2 Frames + 2 Drives + 1 Research Lab | Full system mapping, hidden deposits |
| 4 | Pioneer | Very Fast | 2 Frames + 2 Drives + 2 Research Labs | Can analyze samples for research bonuses (mechanics TBD) |

Research ships have two roles: **surveying** (clearing fog on celestial bodies) and **powering research** at Research Stations.

**Research requires a docked Research Ship** — the station can't process research independently. The ship provides the computation and lab equipment. When a Research Ship is docked at a Research Station, research progresses using materials delivered by the drone network. When no Research Ship is docked, research stalls (even if materials are present).

**Design note — dual-role tension:** A single Research Ship cannot survey AND power research simultaneously. If the only Research Ship leaves to survey a new body, research stalls until it returns. The player has two options:
1. Build multiple Research Ships — one surveys while others power research stations.
2. Accept intermittent research progress — survey in bursts, then return to dock for research periods.
This is a deliberate strategic choice: investing in a second Research Ship costs components but enables parallel research. The onboarding (06) and tech tree (04) assume the player will eventually need a second Research Ship as the game progresses.

Research ships also **clear fog** and reveal resource deposits on planets, moons, and belts. The system starts partially fogged — you must survey a body before you can see what it offers. Higher-tier surveys reveal more detail (surface → subsurface → hidden deposits).

## Stations

Stations are the fixed points in your logistics network. Every station needs a **Power Core** to operate. Unpowered stations can't process cargo.

### Station Hub

The central station of your network. Every system starts with one.

| Tier | Name | Docks | Storage | Special |
|------|------|-------|---------|---------|
| 1 | Waypoint | 2 | 200 units | Basic hub. Can assemble all Tier 1 components at 1/3 rate (1 per 30 ticks) — bootstrapping before a Construction Factory is built. |
| 2 | Exchange | 4 | 500 units | Can transfer cargo between ships |
| 3 | Terminal | 6 | 1,000 units | Automated cargo routing (mechanics TBD — routing is autonomous drone-based, not station-controlled) |
| 4 | Nexus | 8 | 2,000 units | Can coordinate fleet routes (mechanics TBD) |

Docks are how many ships can be at the station simultaneously. Storage is the total cargo capacity the station can hold across all resources. Each resource gets a per-buffer `max` that caps its share of the total. The sum of all per-buffer `max` values must not exceed the station's total capacity. If no cap is configured for a resource, it defaults to `totalCapacity / numResourceTypes`. Fuel has a reserved compartment separate from cargo storage (see Fuel rules in 03-economy.md).

### Mining Station

Extracts raw resources from a celestial body. Must be placed in orbit of a planet, moon, or asteroid belt. Stations do not land on surfaces — they float on orbit rings. Surface conditions (gravity, terrain) affect landing/launching only for the Construction Ship delivery animation; gameplay-relevant placement is orbital-only.

| Tier | Name | Output Rate | Storage | Special |
|------|------|------------|---------|---------|
| 1 | Excavator | Slow (1 unit/cycle) | 100 | One resource type |
| 2 | Drill | Medium (2 units/cycle) | 200 | Two resource types |
| 3 | Extractor | Fast (3 units/cycle) | 400 | All surface resources |
| 4 | Harvester | Very Fast (5 units/cycle) | 800 | Can extract subsurface resources |

Higher-tier mining stations extract more resources per cycle and can tap multiple resource types from the same body.

## Factories

Factories convert materials from one tier to the next.

### Refinery Factory

Converts raw resources into refined goods.

| Tier | Name | Input Types | Output Types | Throughput |
|------|------|------------|-------------|-----------|
| 1 | Smelter | Metal Ore, Carbon Soil, Silicon Dust | Metals, Carbon Fiber, Silicon Wafers | 1 cycle at a time |
| 2 | Processor | + Volcanic Sulfur, Water Ice, Frozen Gases | + Chemicals, Fuel | 2 parallel cycles |
| 3 | Refinery | + Rare Earth Minerals, Crystal Deposits, Helium-3 | + Alloys, Optics, Reactor Rods | 3 parallel cycles |
| 4 | Advanced Refinery | + All raw resources (all Tier 1 inputs accepted) | + All refined goods at max quality | 4 parallel cycles, quality bonus (mechanics TBD) |

### Construction Factory

Converts refined goods into components.

| Tier | Name | Components Produced | Throughput | Special |
|------|------|-------------------|-----------|---------|
| 1 | Workshop | Structural Frame, Cargo Module | 1 component at a time | Manual assembly |
| 2 | Assembly Line | + Power Core, Control System | 2 parallel | Automated assembly |
| 3 | Manufacturing Plant | + Drive Assembly, Research Lab | 3 parallel | Quality control (better components) |
| 4 | Fabrication Yard | + Construction Bay, Gate Node | 4 parallel, bulk orders | Can build components in batches |

Construction factories are where you produce the components needed for ships, stations, and the Space Gate.

## Building the Space Gate

The Space Gate is the V1 victory condition. It's not built in a station — it's a megastructure built at a specific location (usually at the edge of the system, in the fringe lane).

Requirements:
1. Research **Gate Theory** → **Advanced Fabrication** (unlocks Tier 4 Construction Factory and Tier 4 Fabricator) → **Gate Construction** → **System Bridge**
2. Build **8 Gate Nodes** at a Tier 4 Construction Factory (Fabrication Yard)
3. Transport the Gate Nodes to the construction site
4. Build a **Structural Frame** and **Power Core** and **Control System** at the site
5. A **Tier 4 Fabricator** assembles the Gate over time
6. Once built, the Gate must be powered — this requires a constant supply of Reactor Rods or Helium-3 fuel

When the Gate activates, the system display changes — a destination route appears to the next star. V1 is complete.

---

## Upgrading Stations & Factories

Stations and factories can be upgraded to a higher tier. This is **not** building a new station — it retrofits the existing structure.

### Upgrade Cost

Upgrade cost = **full cost of the target tier** minus the original build cost of the current tier. Components already invested are reused; only the delta needs to be delivered.

Example: Tier 1 Station Hub → Tier 2:
- Tier 2 Hub costs: Frame + Power Core + Control System + Cargo Module (×2)
- Tier 1 Hub already has: Frame + Power Core + Control System + Cargo Module (×1)
- Delta cost: 1 additional Cargo Module
- The delta components must be delivered to the station by cargo ships before upgrading begins.

### Upgrade Process

1. Player selects a station and chooses "Upgrade to Tier N" from its logistics panel.
2. The station broadcasts a demand for the delta components (just like any other input demand).
3. Once delta components are delivered to the station's input buffer, a Construction Ship must travel to the station and perform the upgrade.
4. Upgrade build time = `(targetTier - currentTier) × 60s` — upgrading 1 tier takes 60s, 2 tiers takes 120s, etc.
5. During the upgrade, the station continues to operate at its **current** tier — it does not pause production.
6. When the timer expires, the station's `tier` field updates, its throughput/rates increase, and its visual appearance changes.

### Tier Skipping

Upgrading from Tier 1 directly to Tier 3 costs the delta from Tier 1 to Tier 3 (Tier 3 cost − Tier 1 cost). The station operates at Tier 1 rates until the upgrade completes, then jumps to Tier 3. There is no intermediate state.

### Building a New Station vs. Upgrading

Players choose between:
- **Upgrade**: cheaper (reuses existing components), faster (no site selection), but limited by the planet's existing slot count.
- **New build**: full cost, but adds a new slot on a planet if one is free. Sometimes the only option if you need a second station of the same type.

---

## Construction Times

All construction is timed (not instant). The player sees progress bars for active builds.

### Ship Build Times

Ships are built at a Station Hub's shipyard. Build time is in real-time seconds (1:1 with game ticks):

| Tier | Build Time | Notes |
|------|-----------|-------|
| 1 | 30s | Basic hull, quick assembly |
| 2 | 60s | Larger hull, more complex systems |
| 3 | 120s | Advanced hull, heavy components |
| 4 | 240s | Flagship-grade assembly |

Multiple ships can be built simultaneously at different Station Hubs, but each hub can only build one ship at a time (limited by dock usage during construction).

### Station Build Times

Stations are built by a Construction Ship at the orbit ring site. The ship travels to the site first (travel time varies), then builds:

| Station Tier | Build Time | Notes |
|-------------|-----------|-------|
| 1 | 60s | Basic structure, one dock |
| 2 | 120s | Reinforced structure, more docks |
| 3 | 240s | Advanced facilities, multiple systems |
| 4 | 480s | Complex hub with full infrastructure |

Multiple Construction Ships can work on different stations simultaneously.

### Space Gate Build Phases

The Gate is a megastructure with sequential build phases:

| Phase | Duration | Requirement |
|-------|---------|-------------|
| Site Preparation | 300s | Construction Ship arrives, lays foundation |
| Frame Assembly | 600s | 8 Gate Nodes delivered + Structural Frame onsite |
| Power Integration | 300s | Power Core + Control System delivered |
| Activation | 120s | Final calibration — Gate becomes active |

Total: ~1,320s (22 minutes real-time) plus transport time for materials. The player can see progress per phase in the Gate inspection panel.
