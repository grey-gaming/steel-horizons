---
status: Approved
owner: Product Owner
last-reviewed: 2026-08-04
---

# Onboarding — First Five Hours

This is the canonical teaching sequence. Balance may alter idle time, but no step may require an unavailable recipe, technology, component, survey depth, or resource.

## Phase 1 — First Production Chain (Hour 0–1)

### Starting View

The system map shows Haven and Hub Haven clearly. Pyre, Boreas, Titan with Rime and Glint, and The Veil are visible as fogged silhouettes. Builder-1 is docked; no Cargo or Research Ship exists yet.

### Player Sequence

1. Queue a Tier-1 Mining Station in an empty Haven slot. Its Frame, Power Core, and Cargo Module are reserved from the deployment kit; Builder-1 constructs it.
2. Select Metal Ore as its mining target.
3. Queue a Tier-1 Refinery. Its Frame, Power Core, and Control System come from the kit.
4. Configure one refinery slot for Metal Ore → Metals. The refinery broadcasts Metal Ore demand, but no flow appears because there is no Cargo Ship.
5. Build a Courier at Hub Haven from one Frame, Drive Assembly, and Cargo Module in the kit.
6. The Courier selects the Mine-to-Refinery transport job and begins the first autonomous material flow.
7. Reconfigure the refinery as needed to produce Carbon Fiber and Silicon Wafers sequentially. A Tier-1 refinery has one production slot; parallel recipes require an upgrade or second refinery.

### Phase 1 Concepts Taught

- Build orders reserve components and dispatch Construction Ships.
- Stations advertise supply and demand.
- Cargo Ships choose jobs autonomously.
- A single production slot runs one recipe at a time.
- Haven is pre-surveyed; other deposits are hidden.

The mandatory `bootstrap_to_first_cargo` scenario test follows this exact sequence.

## Phase 2 — Research and Surveying (Hour 1–2)

1. Accumulate 20 Metals and 30 Silicon Wafers.
2. Start Sensor Systems at Hub Haven's built-in Tier-1 research console. This project requires neither Optics nor a Research Ship.
3. When Sensor Systems completes, build the kit's Scout and Research Station using separate Frames, Research Labs, and the remaining required kit components.
4. Queue a surface survey of Pyre. The Scout travels there and reveals Volcanic Sulfur and Rare Earth Minerals; Crystal deposits remain hidden until depth 2.
5. Queue a surface survey of Boreas, revealing Water Ice and Frozen Gases.
6. Start Advanced Refining at the Research Station and dock the Scout when research progress is desired. In parallel, use the Hub console for Structural Engineering followed by Cargo Handling. If the Scout leaves for another queued survey, its project pauses automatically with all progress retained.
7. Assemble replacement Frames, Power Cores, Control Systems, and Cargo Modules at the Hub, then build Mining Stations on Pyre and Boreas. The deployment kit is not expected to fund these outposts.

### Phase 2 Concepts Taught

- Tier-1 research can bootstrap at the Hub.
- Later research needs a docked Research Ship.
- Survey targets are player-queued, not automatically chained.
- Survey depth controls which deposits are visible.
- Research pauses safely when its ship leaves.

## Phase 3 — Fuel and Replaceable Components (Hour 2–3)

1. Complete Advanced Refining and upgrade the refinery to Tier 2.
2. Configure Chemicals and Fuel production using Pyre and Boreas inputs.
3. Scale Hub assembly of replacement Frames, Power Cores, Control Systems, and Cargo Modules.
4. Build a second Courier, or save the kit's third Drive for a Tier-2 Hauler.
5. Observe that ships refuel automatically and that route feasibility includes a 10% reserve.

### Phase 3 Concepts Taught

- Research opens new production chains.
- Factory tier controls parallelism and recipe availability.
- Component production replaces the finite deployment kit.
- More transport capacity relieves bottlenecks.

## Phase 4 — Advanced Materials (Hour 3–4)

1. Use the Scout to surface-survey Titan, Rime, and Glint, revealing Helium-3 on Rime and Crystal Deposits on Glint.
2. Build Mining Stations on Rime and Glint; select Helium-3/Crystal Deposits and transport at least 10 Helium-3 to the research facility.
3. Complete Fusion Power.
4. Complete Alloy Smelting using Pyre's revealed Rare Earth Minerals and upgrade a refinery to Tier 3.
5. Produce Alloys, Optics, and Reactor Rods. Each recipe still requires its own technology even though the Tier-3 refinery can host it.
6. Build a Tier-2 Hauler. Its capacity makes it a strong candidate for the long Rime route, but deterministic job scoring—not a hard role assignment—selects its work.
7. Begin Factory Automation after Structural Engineering and Cargo Handling are complete.

### Phase 4 Concepts Taught

- Some advanced recipes combine deposits from several bodies.
- Technology and facility tier are independent recipe gates.
- Better ships influence autonomous job selection through capacity, position, and fuel feasibility.

## Phase 5 — Industrial Network and The Veil (Hour 4–5)

1. Complete Factory Automation.
2. Build the kit-funded Tier-1 Construction Factory and begin producing unlocked components in 10-tick slots.
3. Complete Deep Survey, assemble a replacement Research Lab/Drive, and build a Tier-2 Surveyor.
4. Use the Surveyor to resurvey Pyre/Rime as needed for depth-2 deposits.
5. Complete Orbital Logistics.
6. Survey The Veil to depth 2 and place a Mining Station there. A Tier-2 Construction Ship alone does not bypass the technology requirement.
7. Complete Reactor Scaling and upgrade selected factories to Tier 2 for parallel production.
8. Review the visible path through Precision Manufacturing, Gate Theory, Advanced Fabrication, Gate Construction, and System Bridge.

### State After Five Hours

The player understands extraction, distributed storage, autonomous transport, recipe configuration, component recovery, survey depth, docked research, factory tiers, fuel, and technology prerequisites. The network has renewable common inputs available through The Veil and a clear, validated path to the Space Gate.

## Onboarding Validation

CI runs the sequence as deterministic commands against the authored starting fixture. It asserts after each phase that:

- Every queued recipe and build is unlocked.
- Every required component or resource is reachable.
- No inventory count becomes negative.
- Research facility requirements are satisfied.
- Save/load at each milestone produces the same final state.
- The player retains a path to victory after every permitted cancellation, demolition, and rebuild in the recovery scenario set.
