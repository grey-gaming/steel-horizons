# Ships, Stations & Factories

## Ships

Ships are the backbone of your logistics network. They fly assigned routes autonomously. You never pilot them — you design routes and choose which ship class to assign.

### Cargo Ships

Purpose: Move materials from place to place.

| Tier | Name | Capacity | Speed | Cost (Components) | Special |
|------|------|----------|-------|-------------------|---------|
| 1 | Courier | 50 units | Medium | 1 Frame + 1 Drive + 1 Cargo Module | None |
| 2 | Hauler | 120 units | Medium | 1 Frame + 1 Drive + 2 Cargo Modules | Better fuel efficiency |
| 3 | Bulk Carrier | 300 units | Slow | 2 Frames + 2 Drives + 3 Cargo Modules | Can carry any cargo, armored |
| 4 | Fast Freighter | 100 units | Fast | 1 Frame + 2 Drives + 1 Cargo Module | Rush deliveries, perishable goods |

Higher-tier cargo ships can carry more, travel faster, or handle special conditions (long distances, high-gravity planets).

### Construction Ships

Purpose: Build stations, factories, and the Space Gate. They carry construction crews and components to a build site.

| Tier | Name | Build Speed | Cost | Special |
|------|------|------------|------|---------|
| 1 | Builder | Slow | 1 Frame + 1 Drive + 1 Construction Bay | Basic structures only |
| 2 | Constructor | Medium | 1 Frame + 1 Drive + 1 Construction Bay | Can build on any terrain |
| 3 | Engineer | Fast | 2 Frames + 2 Drives + 2 Construction Bays | Builds multiple structures at once |
| 4 | Fabricator | Very Fast | 2 Frames + 2 Drives + 2 Construction Bays | Can build in hazardous zones (asteroid belts, volcanic) |

Construction ships travel to a site, deploy, and build. The player chooses what to build and where — the ship handles the construction.

### Research Ships

Purpose: Survey celestial bodies for resources and unlock research data.

| Tier | Name | Survey Speed | Cost | Special |
|------|------|-------------|------|---------|
| 1 | Scout | Slow | 1 Frame + 1 Drive + 1 Research Lab | Basic surface survey |
| 2 | Surveyor | Medium | 1 Frame + 1 Drive + 1 Research Lab | Deep subsurface scan |
| 3 | Explorer | Fast | 2 Frames + 2 Drives + 1 Research Lab | Full system mapping, hidden deposits |
| 4 | Pioneer | Very Fast | 2 Frames + 2 Drives + 2 Research Labs | Can analyze samples for research bonuses |

Research ships don't just enable research — they also **clear fog** and reveal resource deposits on planets, moons, and belts. The system starts partially fogged — you must survey a body before you can see what it offers. Higher-tier surveys reveal more detail (surface → subsurface → hidden deposits).

## Stations

Stations are the fixed points in your logistics network. Every station needs a **Power Core** to operate. Unpowered stations can't process cargo.

### Station Hub

The central station of your network. Every system starts with one.

| Tier | Name | Docks | Storage | Special |
|------|------|-------|---------|---------|
| 1 | Waypoint | 2 | 200 units | Basic hub |
| 2 | Exchange | 4 | 500 units | Can transfer cargo between ships |
| 3 | Terminal | 6 | 1,000 units | Automated cargo routing |
| 4 | Nexus | 8 | 2,000 units | Can coordinate fleet routes |

Docks are how many ships can be at the station simultaneously. Storage is how much cargo the station can hold before it needs to be moved out.

### Mining Station

Extracts raw resources from a celestial body. Must be placed on or in orbit of a planet, moon, or asteroid belt.

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
| 2 | Processor | + Volcanic Sulfur, Water Ice | + Chemicals, Fuel | 2 parallel cycles |
| 3 | Refinery | + Frozen Gases, Rare Earth Minerals | + Alloys, Optics, Reactor Rods | 3 parallel cycles |
| 4 | Advanced Refinery | + Crystal Deposits, Helium-3 | + All refined goods at max quality | 4 parallel cycles, quality bonus |

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
1. Research **Gate Theory** → **Gate Construction** → **System Bridge**
2. Build **8 Gate Nodes** at a Construction Factory (Tier 4)
3. Transport the Gate Nodes to the construction site
4. Build a **Structural Frame** and **Power Core** and **Control System** at the site
5. A **Tier 4 Construction Ship** (Fabricator) assembles the Gate over time
6. Once built, the Gate must be powered — this requires a constant supply of Reactor Rods or Helium-3 fuel

When the Gate activates, the system display changes — a destination route appears to the next star. V1 is complete.
