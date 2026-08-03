# Technology Tree

## Research Mechanics

### Resource Delivery

Research materials are delivered by the drone logistics network automatically:

- When the player starts a research project, the required resources are added to the Research Station's **input buffer** as demand entries.
- Ships in the network detect this demand and deliver the required materials like any other cargo — no manual routing needed.
- If the required materials exist anywhere in the network (at station output buffers), ships will transport them to the Research Station.
- If materials are not yet available (not yet produced or not yet extracted), research stalls. The panel shows a "Materials pending" indicator listing what's missing.
- Research progress advances only when all required resources are present in the station's input buffer. Resources are consumed incrementally as progress advances, not all upfront.
- Multiple Research Stations can run different projects simultaneously — each station has its own input buffer and demand list.

Research is done at **Research Stations**. Each station can run one research project at a time. A project costs resources — you feed it raw, refined, or component materials over time until the research completes.

**Research Stations need a Research Ship docked to operate.** The ship provides the computation and lab equipment. If no Research Ship is docked, research stalls even if materials are present. The ship can leave to survey and must return for research to resume.

Key rules:
- Research consumes resources (they're not returned if you cancel)
- Multiple research stations can work on different projects simultaneously
- Some techs require a previous tech to be completed (the tree)
- Higher-tier research costs more and takes longer
- You can see what each tech unlocks before starting it

## The Tech Tree

### Tier 0 — Starting Tech (Already known)

| Technology | What It Unlocks |
|-----------|----------------|
| Basic Construction | Tier 1 Cargo Ship, Tier 1 Construction Ship, Station Hub, Mining Station |
| Basic Refining | Refinery Factory (can refine ore → metals, soil → carbon, dust → silicon) |
| Basic Power | Power Core component (uses Reactor Rods) |
| Basic Control | Control System component (uses Silicon Wafers + Optics) |

The player starts with these already researched — they can build their first station and mining operation immediately.

### Tier 1 — Early Expansion

| Technology | Requires | Unlocks |
|-----------|----------|---------|
| Advanced Refining | Basic Refining | Refinery upgrade — can produce Alloys, Chemicals, Fuel |
| Structural Engineering | Basic Construction | Structural Frame component; Tier 2 Cargo Ship, Tier 2 Construction Ship |
| Sensor Systems | Basic Control | Optics component; Research Ship, Research Station. Survey reveals surface resources on planets and moons. |
| Fusion Power | Basic Power | Reactor Rod component; Tier 2 Station Hub (more slots) |
| Cargo Handling | Basic Construction | Cargo Module component; Station Storage upgrade |

### Tier 2 — Industrial Age

| Technology | Requires | Unlocks |
|-----------|----------|---------|
| Alloy Smelting | Advanced Refining | Alloys (stronger hulls); Tier 3 ships, faster/larger |
| Factory Automation | Structural Engineering + Cargo Handling | Construction Factory (builds components); Drive Assembly |
| Deep Survey | Sensor Systems | Reveals subsurface deposits (more resources than surface scan alone). Clears fog on entire planet. |
| Reactor Scaling | Fusion Power | Larger Power Cores; Tier 2 factories (more throughput) |
| Orbital Logistics | Cargo Handling + Sensor Systems | Orbital mining stations in belts; Tier 2 Mining Station |
| Life Support | Advanced Refining | Enables long-haul routes to outer system |

### Tier 3 — Advanced Construction

| Technology | Requires | Unlocks |
|-----------|----------|---------|
| Precision Manufacturing | Factory Automation + Alloy Smelting | Tier 3 Construction Factory; advanced components |
| Heavy Transport | Orbital Logistics | Tier 3 Cargo Ship (bulk carrier); Station Hub Tier 3 |
| Exploration Suite | Deep Survey + Reactor Scaling | Tier 3 Research Ship (faster survey, more data). Reveals hidden deposits (rare resources not visible on surface or subsurface). |
| Grid Power | Reactor Scaling | Station-to-station power sharing; larger station networks |
| Gate Theory | Precision Manufacturing + Exploration Suite | Space Gate research — prerequisites for the final goal |

### Tier 4 — The Space Gate (V1 Goal)

| Technology | Requires | Unlocks |
|-----------|----------|---------|
| Gate Construction | Gate Theory + Heavy Transport + Grid Power | **Gate Node** component (requires Alloys + Power Core + Optics + Reactor Rods) |
| System Bridge | Gate Construction | The **Space Gate** itself — built from 8 Gate Nodes plus a Structural Frame, Power Core, and Control System |

Once the Space Gate is built and powered, V1 is complete. The gate opens a route to another star system — that's V2.

## Research Duration

Research progresses at 1 tick per tick (real-time). Each technology has a duration in ticks that determines how long it takes to complete once all resources are delivered.

| Tier | Duration Range | Example |
|------|---------------|---------|
| Tier 1 (Early Expansion) | 600–900 ticks (10–15 min) | Advanced Refining: 600 ticks |
| Tier 2 (Industrial Age) | 900–1,800 ticks (15–30 min) | Factory Automation: 1,200 ticks |
| Tier 3 (Advanced Construction) | 1,800–3,600 ticks (30–60 min) | Precision Manufacturing: 2,400 ticks |
| Tier 4 (Space Gate) | 3,600–7,200 ticks (60–120 min) | Gate Construction: 4,800 ticks |

### Research Progress Model

Each tick, a Research Station with sufficient materials increments progress by 1. Progress is stored as `ticksCompleted / totalTicks`. Resources are consumed incrementally at a rate of `totalRequired / totalTicks` per tick — they are not consumed all upfront.

If materials run out mid-research, progress stalls but is not lost. When materials are replenished, progress resumes.

## Research Costs (Example — how much of what)

| Technology | Example Cost | Duration |
|-----------|-------------|---------|
| Advanced Refining | 200 Metals + 100 Carbon Fiber + 50 Silicon Wafers | 600 ticks |
| Fusion Power | 150 Metals + 80 Carbon Fiber + 30 Helium-3 | 700 ticks |
| Precision Manufacturing | 500 Alloys + 300 Silicon Wafers + 200 Optics + 100 Power Cores | 2,400 ticks |

Costs scale with tier. Higher-tier research requires refined goods and components, not just raw materials.

## Technology Progression Strategy

The player's natural path:
1. Set up basic mining and refining on the starting planet
2. Research early expansion techs to reach other bodies
3. Build a second mining outpost on a different planet (volcanic for sulfur, ice for water)
4. Research industrial techs to unlock factories and better ships
5. Build a construction factory to produce components in bulk
6. Research advanced techs to reach the outer system and asteroid belt
7. Build a research station and push for Gate Theory
8. Mass-produce Gate Nodes and construct the Space Gate

Research is never wasted — even "dead end" techs like Life Support are required to reach certain planets or run certain factories efficiently.
