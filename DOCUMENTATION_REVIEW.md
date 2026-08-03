# Steel Horizons Documentation Review

**Review date:** 2026-08-03
**Scope:** All 14 Markdown files under `docs/` (2,067 lines), plus repository-level documentation coverage
**Repository state reviewed:** Documentation-only repository; no source code, build metadata, tests, README, or contributor files were present

## Executive summary

The documentation communicates a strong, coherent game premise: a cozy, single-system logistics game built around autonomous ships, distributed station buffers, progressive industry, and a Space Gate victory goal. The documents are unusually concrete for an early design set: they include resource chains, a tech tree, onboarding beats, UI sketches, tick ordering, sample formulas, entity shapes, and an asset inventory.

However, the set is **not yet implementation-ready**. The main problem is not lack of ideas; it is that the same rule is often specified differently in several places. The opening progression, tech unlocks, refinery tiers, research flow, survey control, camera states, pause behavior, and persistence contract all contain contradictions. Several simulation rules also cannot be represented by the proposed data model. Most seriously, the documented bootstrap path cannot produce all of the components it needs unless a substantial starting inventory or additional bootstrap recipes are assumed but not specified.

The highest-value next step is to establish one canonical, preferably machine-readable, content specification for resources, recipes, buildings, ships, technology unlocks, costs, rates, and the starting state. The narrative documents should reference that source instead of restating the same data. A small automated reachability test should then prove that a fresh game can reach the Space Gate without relying on undocumented inventory or impossible unlocks.

### Finding counts

| Severity | Count | Meaning |
|---|---:|---|
| Critical | 10 | Blocks a coherent implementation or can make the documented game progression impossible |
| High | 24 | Major ambiguity or inconsistency likely to produce incompatible implementations or poor player experience |
| Medium | 12 | Important missing coverage, maintainability concern, or incomplete specification |
| Low | 4 | Editorial or organizational improvement |
| **Total** | **50** | Categorized findings; cross-cutting issues may share the same root cause |

## What is already working well

- `docs/01-gdd.md` gives the game a clear identity, core loop, player role, V1 boundary, and no-failure philosophy.
- The autonomous-drone logistics model is repeated consistently as the intended V1 direction, even though its detailed algorithm needs work.
- `docs/06-onboarding.md` is player-centered: it describes what the player sees, does, learns, and achieves rather than only listing systems.
- `docs/11-ui-interactions.md` provides useful concrete panel sketches and interaction triggers.
- `docs/12-simulation-foundations.md` establishes a tick order and persistent-job model, both good foundations for deterministic simulation work.
- `docs/13-data-models.md` makes the design testable by attempting explicit entity shapes instead of leaving every concept in prose.
- The future inter-system concept has its own file, which is the right organizational direction even though V2 references still leak into V1 documents.
- The visual documents consistently favor readability, silhouettes, route direction, and map-first interaction.

## Critical findings

### C-01 — The opening economy has an unresolved bootstrap cycle

**Evidence**

- The player starts with one basic station and one Construction Ship (`docs/01-gdd.md:36`; `docs/06-onboarding.md:21`).
- Most ships and structures require a Drive Assembly, Power Core, Control System, Research Lab, or Construction Bay (`docs/03-economy.md:45-64`).
- The first Mining Station needs a Power Core and Cargo Module, and the first Refinery needs a Power Core and Control System (`docs/03-economy.md:60-63`); the starting Hub cannot make those complete component sets.
- A Tier 1 Hub may assemble only Structural Frames and Cargo Modules, while every other component requires a Construction Factory (`docs/03-economy.md:40`; `docs/05-ships-stations-factories.md:65`; `docs/12-simulation-foundations.md:99-106`).
- A Construction Factory itself requires a Construction Bay (`docs/03-economy.md:63`), but Construction Bays are only produced by a Tier 4 Construction Factory (`docs/05-ships-stations-factories.md:104-109`).
- Drive Assembly and the Construction Factory are unlocked together by Factory Automation (`docs/04-tech-tree.md:55`), yet the first Cargo and Research Ships already require Drive Assemblies (`docs/06-onboarding.md:28,49`).
- Research requires a Research Station, but Sensor Systems—the technology that unlocks the first Research Station—must itself be researched (`docs/04-tech-tree.md:16-18,46`). No earlier research facility is defined.
- The onboarding resolves this only by saying the components are available or are built manually at the Hub (`docs/06-onboarding.md:28,49`), which conflicts with the Hub's documented recipe limit.

**Impact:** A literal implementation can soft-lock before the first Cargo Ship, Research Ship, or Construction Factory. This directly violates the promise that there is always a solution.

**Recommendation:** Define an exact starting inventory and a formally reachable bootstrap chain. Options include starting with a stock of advanced components, allowing the Hub to craft a complete set of inefficient bootstrap components, giving the starter ship a limited fabrication ability, or moving the relevant recipes/unlocks earlier. Add an automated dependency-graph test proving that every required unlock and the Space Gate are reachable from the starting state.

### C-02 — Technology unlock ownership contradicts itself

**Evidence**

- Basic Power already unlocks Power Cores (`docs/04-tech-tree.md:35`), while the onboarding says Fusion Power unlocks them (`docs/06-onboarding.md:102,122`). The tech tree says Fusion Power unlocks Reactor Rods and a Tier 2 Hub instead (`docs/04-tech-tree.md:47`).
- Structural Frames and Cargo Modules are used in the opening and are described as Hub-craftable (`docs/03-economy.md:40`; `docs/06-onboarding.md:28`), but they are later unlocked by Structural Engineering and Cargo Handling (`docs/04-tech-tree.md:45,48`).
- Advanced Refining says it unlocks Alloys (`docs/04-tech-tree.md:44`), while Alloy Smelting separately says it unlocks Alloys (`docs/04-tech-tree.md:54`).
- Fusion Power says a Tier 2 Hub has “more slots” without saying whether these are docks, storage/processing slots, or planetary placement slots (`docs/04-tech-tree.md:47`). If it means station placement slots, it contradicts the rule that those are fixed by the planet and never added by a Hub upgrade (`docs/11-ui-interactions.md:71-73`); if not, the terminology is still unsafe.

**Impact:** The tech tree cannot be used as an authoritative unlock graph, and UI lock states, build validation, and progression tests will disagree.

**Recommendation:** Create a single unlock matrix with one owner per recipe, entity tier, ability, and upgrade. Separate “known recipe,” “facility capable of producing it,” and “materials currently available,” which are currently conflated.

### C-03 — The five-hour onboarding path is not executable under the documented rules

**Evidence**

- Phase 2 builds a Research Station and starts research before building the required docked Research Ship (`docs/06-onboarding.md:48-50`; dock rule at `docs/04-tech-tree.md:18`).
- Sensor Systems unlocks the Research Ship and Research Station (`docs/04-tech-tree.md:46`), but the onboarding uses both before researching Sensor Systems and even lists Sensor Systems as a future choice in Phase 5 (`docs/06-onboarding.md:128`).
- The only Scout surveys multiple bodies continuously (`docs/06-onboarding.md:50-52,78`), while research must stop whenever that ship is away from the Research Station (`docs/05-ships-stations-factories.md:44-51`). The onboarding nevertheless expects Advanced Refining and Structural Engineering to complete during that period (`docs/06-onboarding.md:73,97`).
- In Phase 4, the player chooses Fusion Power **or** Factory Automation (`docs/06-onboarding.md:102`). Phase 5 assumes both a completed Fusion Power project and the ability to build a Construction Factory (`docs/06-onboarding.md:122,125`). Factory Automation also requires Cargo Handling, which the path never researches (`docs/04-tech-tree.md:55`).
- Phase 5 builds an Orbital Mining Station based on “Orbital Logistics tech or a Tier 2 Construction Ship” (`docs/06-onboarding.md:124`), but the tech tree makes Orbital Logistics the unlock and documents no Constructor bypass (`docs/04-tech-tree.md:58`).
- Advanced Refining requires Carbon Fiber and Silicon Wafers, while Phase 1 explicitly establishes only Ore → Metals and a Tier 1 mine extracts one resource at a time (`docs/06-onboarding.md:25-29,48`; `docs/05-ships-stations-factories.md:76-81`). Extra mining/refining steps or starter stock are missing.

**Impact:** This document cannot serve as a tutorial script, balance target, acceptance test, or reliable first-session plan.

**Recommendation:** Rewrite onboarding from a validated “golden path” generated from the canonical unlock graph. For every step, record prerequisites, starting/produced inventory, facility, elapsed-time budget, ship availability, and recovery behavior. Simulate the path before treating the hour ranges as targets.

### C-04 — Recipes, refinery tiers, and production unlocks disagree

**Evidence**

- Fuel requires Frozen Gases plus Chemicals (`docs/03-economy.md:31,93`), but the Tier 2 refinery claims to make Fuel without accepting Frozen Gases; Frozen Gases are not accepted until Tier 3 (`docs/05-ships-stations-factories.md:93-98`). The onboarding expects Tier 2 Fuel production (`docs/06-onboarding.md:73-77`).
- Alloys and Reactor Rods are Tier 3 refinery outputs (`docs/05-ships-stations-factories.md:97`), but the onboarding produces them after only describing a Tier 2 upgrade (`docs/06-onboarding.md:74,101`).
- Optics require Crystal Deposits (`docs/03-economy.md:33`), while the refinery table lists Optics at Tier 3 but does not accept Crystal Deposits until Tier 4 (`docs/05-ships-stations-factories.md:97-98`).
- Reactor Rods similarly appear as a Tier 3 output even though their Helium-3 input is not accepted until Tier 4 (`docs/03-economy.md:34`; `docs/05-ships-stations-factories.md:97-98`).
- Chemicals use “Water” in one recipe (`docs/03-economy.md:30`) but only Water Ice exists in `ResourceType` (`docs/13-data-models.md:12-16`) and in the refinery input table (`docs/05-ships-stations-factories.md:96`).
- The onboarding says a newly built Construction Factory produces Power Cores, but a Tier 1 Workshop produces only Frames and Cargo Modules; Power Cores start at Tier 2 (`docs/06-onboarding.md:125`; `docs/05-ships-stations-factories.md:104-109`).
- Recipe quantities and yields are generally omitted. The refinery table compresses processing to “1 unit input → 1 unit output” without explaining whether a multi-input recipe consumes one unit of every listed ingredient, and its header/data rows are malformed (`docs/12-simulation-foundations.md:79-86`). It therefore does not provide authoritative stoichiometry for the multi-input recipes in `docs/03-economy.md:25-51`.

**Impact:** Production code, balance data, UI recipe displays, and onboarding cannot all implement the current documentation.

**Recommendation:** Replace the repeated prose/tables with one canonical recipe catalog containing exact input quantities, output quantities, duration, required facility type/tier, required technology, power/fuel behavior, and by-products. Generate or validate the human-readable tables from it.

### C-05 — Research has three incompatible resource-consumption contracts

**Evidence**

- Research resources are supposed to become station-buffer demand and arrive via ships (`docs/04-tech-tree.md:7-14`).
- Progress is said to begin only when **all** required resources are present, while those same resources are then consumed incrementally (`docs/04-tech-tree.md:13,91-95`). Without defining “remaining requirement,” the project can cease satisfying “all present” immediately after its first consumption tick.
- The UI instead says resources are deducted from “network storage” when the player clicks the technology (`docs/11-ui-interactions.md:245-251`).
- `ResearchProject.resourcesConsumed` is described as both consumption progress and “how much ... delivered so far” (`docs/13-data-models.md:117-123`). The project has no duration, tick progress, status, required docked-ship ID, or cancellation state.

**Impact:** Research demand, reservation, cancellation loss, progress, UI feedback, and saves cannot be implemented consistently.

**Recommendation:** Define an explicit state machine such as `AwaitingMaterials → Ready → Active → PausedNoShip/PausedMaterials → Complete/Cancelled`, with remaining material demand, reservation rules, atomic or incremental consumption semantics, ship assignment, and cancellation effects. Make the UI use the same contract.

### C-06 — Fractional simulation rates cannot be represented by integer state

**Evidence**

- Extraction, refining, and construction add or consume `0.1–0.5` units per tick and research consumes fractional cost per tick (`docs/12-simulation-foundations.md:68-97`; `docs/04-tech-tree.md:91-95`).
- Buffers, cargo, fuel, deposits, supply/demand amounts, and most material accounting are integers (`docs/13-data-models.md:28-34,68-72,117-123,137,155-168`).

**Impact:** Implementations must silently round, lose material, produce at different rates, or invent hidden accumulators that are not saved. Save/load could change outcomes.

**Recommendation:** Choose one numeric contract: atomic production at cycle boundaries, fixed-point integers with a documented scale, or explicit fractional/remainder accumulators that are serialized. Define rounding at every boundary and add conservation-of-material invariants.

### C-07 — Logistics matching has no reservation or deterministic assignment model

**Evidence**

- Idle ships independently inspect supply/demand pairs and take the best job (`docs/07-routes-and-logistics.md:22-29`; `docs/12-simulation-foundations.md:34-41`).
- Supply and demand entries contain only station, resource, amount, and position (`docs/13-data-models.md:151-168`); there is no reserved amount, job ID, version, priority, or capacity already committed.
- The score uses demand and supply priorities, but runtime entries do not snapshot them; an implementation could look up the referenced Station, though lookup/update semantics are not stated (`docs/07-routes-and-logistics.md:20-29`; `docs/13-data-models.md:155-168`).
- Job selection has three competing descriptions: highest priority/distance score, the closest pair with highest distance-weighted value, and the closest available source (`docs/07-routes-and-logistics.md:22-29,74-83`; `docs/12-simulation-foundations.md:34-41`). Score terms also mix unnormalized priority and distance units.
- No stable iteration or tie-breaking order exists after “pick the closest,” and no atomic step prevents several idle ships from claiming the same units in one tick.

**Impact:** Cargo can be double-booked, jobs can arrive to empty/full buffers, behavior can vary by container iteration order, and replays/save-loads can diverge.

**Recommendation:** Specify a single scheduler pass with stable ordering and atomic reservations. Include ship-to-source travel, source-to-destination travel, cargo capacity, available fuel, reserved supply, reserved destination capacity, priority normalization, partial-load policy, cancellation, and reservation release. Persist active reservations or define how they are reconstructed safely.

### C-08 — Construction and upgrade material flow is not defined end to end

**Evidence**

- Construction Ships are said to carry components to build sites (`docs/05-ships-stations-factories.md:20-31`).
- Station placement immediately dispatches a Construction Ship, with no material sourcing/delivery step (`docs/11-ui-interactions.md:41-50`). Ship building consumes components immediately from unspecified “storage” (`docs/11-ui-interactions.md:79-86`).
- Upgrades create demand, await delivery, then dispatch a Construction Ship (`docs/05-ships-stations-factories.md:143-150`), while Gate construction uses cargo deliveries to a distinct site (`docs/11-ui-interactions.md:255-266`).
- `ShipJob` has mandatory cargo fields for every job but no build-order ID, required-material map, phase, assigned builder, reservation, or cancellation state (`docs/13-data-models.md:96-111`). No build-site or shipyard-order entity exists.

**Impact:** It is unclear when costs are reserved/consumed, who transports them, where they wait, what happens if supply disappears, and how saves restore an in-progress build.

**Recommendation:** Define one construction-order model for ships, stations, upgrades, and the Gate. Include placement validity, material demand/reservation, delivery destination, builder assignment, progress, pause conditions, cancel/refund policy, completion, and serialized state.

### C-09 — The proposed data model cannot hold the game state the other documents require

**Missing or incomplete state includes:** root `GameState`; current/completed technology; recipe and production-slot selection; starting/global settings; construction and shipyard queues; survey orders; dock/holding queues; research-ship assignment; logistics reservations; station upgrades; Gate/site/phases/activation fuel; route throughput history; tutorial progress; RNG state; schema version; and victory state.

Additional type issues:

- `StationType` is referenced but never defined (`docs/13-data-models.md:40-54`).
- Stations have a `planetId` even when attached to moons or belts, and no station world position is defined (`docs/13-data-models.md:44-46`).
- Moons have no parent-body relationship; all bodies have a stellar lane and angle (`docs/13-data-models.md:126-149`).
- `AsteroidBelt` is a `BodyType`, but the required `PlanetType subtype` cannot represent an asteroid belt (`docs/13-data-models.md:129-148`).
- Research/Build jobs require transport-only cargo fields instead of type-specific payloads (`docs/13-data-models.md:96-111`). Idle itself is representable through nullable `Ship.job` plus `ShipState.Idle`, but the prose should make that distinction explicit (`docs/13-data-models.md:73-83`).

**Impact:** “Fully serializable” state (`docs/12-simulation-foundations.md:123-137`) is not achievable using the documented entities.

**Recommendation:** Design the aggregate state from required use cases and save/load round trips, not entity nouns alone. Add discriminated job/order types, content IDs, runtime/persisted boundaries, invariants, optional/null rules, stable enum/ID representations, JSON encoding for maps, integer widths, canonical ordering, and schema versioning.

### C-10 — The documented Space Gate dependency chain is unreachable

The Gate needs eight Gate Nodes from a Tier 4 Construction Factory and assembly by a Tier 4 Fabricator (`docs/05-ships-stations-factories.md:117-123`). The tech tree never unlocks either prerequisite: Precision Manufacturing unlocks only a Tier 3 Construction Factory (`docs/04-tech-tree.md:65`), Alloy Smelting mentions Tier 3 ships (`docs/04-tech-tree.md:54`), Gate Construction unlocks only Gate Nodes (`docs/04-tech-tree.md:75`), and System Bridge unlocks the Gate (`docs/04-tech-tree.md:76`). No technology grants the Tier 4 factory or Fabricator. This remains a blocker even after the opening bootstrap is repaired.

**Recommendation:** Add explicit Tier 4 facility/Fabricator unlocks and their complete costs to the Gate path, or reduce the Gate's facility/ship requirements. Validate the entire Gate dependency graph from a legal mid-game state, including Gate Node production, transport, assembly, activation fuel, and victory.

## High-priority findings

### H-01 — Survey control is both manual and fully autonomous

The UI dispatches a survey from a planet action (`docs/11-ui-interactions.md:34-35,305`), while the visual, onboarding, and simulation documents have idle Research Ships automatically choose the nearest unsurveyed body and continue onward (`docs/08-visual-style.md:23-32`; `docs/06-onboarding.md:50-52`; `docs/12-simulation-foundations.md:36-41`). Survey depth is also controlled independently by both technology and ship tier, with no rule for how the two gates combine (`docs/04-tech-tree.md:46,56,67`; `docs/05-ships-stations-factories.md:37-42,53`). Hidden deposits can be revealed, but even the Tier 4 Mining Station explicitly adds only subsurface extraction (`docs/04-tech-tree.md:67`; `docs/05-ships-stations-factories.md:76-83`). The onboarding discovers Helium-3 on moons while surveying the gas giant, then surveys those moons again (`docs/06-onboarding.md:78,98`). This changes player agency, job priority, research-ship availability, and reveal/extraction state. A clean contract would be: the player selects or queues a target; ship capability and researched sensor depth determine the scan result; then the ship autonomously travels and scans. Optional auto-survey can be an explicit toggle.

### H-02 — The camera has incompatible band counts and navigation rules

- `docs/08-visual-style.md:11-21` defines three V1 states: System, Planet, and Station. `docs/11-ui-interactions.md:9-19` also says there are three bands, but its table describes System, Detailed, and a future Galaxy map; its click rules separately distinguish Planet and Station (`docs/11-ui-interactions.md:25-35`).
- `docs/09-zoom-levels.md:7-10` says Band 2 is an unused former Galaxy band and Planet/Station are both Band 3 sub-modes.
- The summary table then labels Detailed View as Band 2 (`docs/09-zoom-levels.md:88-98`).
- Clicking empty space pans in `docs/09-zoom-levels.md:29-35`, returns to System in `docs/09-zoom-levels.md:74-84`, but only deselects/closes panels while dragging pans in `docs/11-ui-interactions.md:21-35`.

Choose one camera state machine. The least disruptive interpretation is System = Band 1, Planet = Band 2, Station = Band 3, with the future Galaxy view outside the V1 sequence (for example Band 0). Define scroll thresholds, focus preservation, back behavior, click-versus-drag handling, and transition accessibility.

### H-03 — Pause is simultaneously excluded and assigned a shortcut

The simulation explicitly has no pause or time acceleration in V1 (`docs/12-simulation-foundations.md:3-7`), while Space toggles pause/resume (`docs/11-ui-interactions.md:310-317`). Decide the product rule, then document what happens to autosave, animations, menus, background/window blur, and queued inputs while paused.

### H-04 — The orbital travel formula cannot represent the described geography

Cross-lane travel requires a slower “burn” (`docs/02-the-system.md:7-18`), but travel time is only `angularDistance × orbitalRadius / effectiveSpeed` (`docs/12-simulation-foundations.md:108-121`). `orbitalRadius` is absent from `CelestialBody`, and a ship has only one current lane multiplier (`docs/13-data-models.md:57-93,126-149`). The model also lacks radial distance, transfer segments, moon-relative orbits, moving-body interception, and the lane used during a transfer. Define whether this is an abstract graph or orbital motion, then specify path segments, distance units, body motion, and arrival calculations accordingly.

### H-05 — Fuel rules are incomplete and can produce nonsensical results

Fuel use is `distance_traveled × cargo_load × 0.01` (`docs/03-economy.md:91-101`), which means empty Cargo Ships—and apparently Construction and Research Ships—consume zero fuel. Ships consume Fuel before Advanced Refining can unlock its production, yet no starting fuel quantity or pre-fuel exemption is defined (`docs/03-economy.md:91-101`; `docs/04-tech-tree.md:44`). “Always enough reserve to reach the nearest station” is guaranteed without defining how it is enforced or what happens when no station has Fuel. Job selection mentions reachability within reserve but not the empty pickup leg plus loaded delivery plus refuel diversion (`docs/07-routes-and-logistics.md:22-29`). Fuel efficiency is a ship special with no value (`docs/05-ships-stations-factories.md:11-18`) and no field beyond capacity (`docs/13-data-models.md:68-72`). Add base mass use, load factor, per-tier efficiency, route feasibility, starting fuel, refuel reservation, emergency behavior, and exact gate Helium-3 equivalence.

### H-06 — Instant transfer, cosmetic states, and dock occupancy conflict

Cargo transfers in zero ticks; the ship may receive its next job immediately, while a 1–3 second Loading/Unloading animation continues (`docs/07-routes-and-logistics.md:61-72`). It is unclear whether the ship still occupies a dock, can visually appear in two places, or can depart through another queued ship. The text alternates between a holding pattern outside the dock and waiting “at the dock” for input capacity. Specify authoritative gameplay position, dock acquisition/release, source/destination capacity reservations, animation cancellation, and queue fairness. Add dock counts for every station type, not only Hubs.

### H-07 — Bottleneck detection compares the wrong quantities in common network shapes

The algorithm compares throughput for one A→B pair with all production at A (`docs/07-routes-and-logistics.md:91-109`). It will falsely flag A→B when A legitimately splits output across several destinations, and it ignores B's consumption rate, reserved/in-transit cargo, dock congestion, and output backpressure. “Increase station priority” is not necessarily a route-capacity fix. The monitor computes `units_delivered / 600` (units per tick/second), while the UI displays units/min without documenting conversion or rounding. The UI example reports 120 units/min (`docs/11-ui-interactions.md:179-198`), above the sustained 30 units/min production of a single Tier 4 mine (`docs/12-simulation-foundations.md:66-75`); that could be a short burst draining inventory, but no common measurement window is stated. Define bottlenecks from unmet demand/accumulating queues over a window and make example numbers use the same documented window and conversion.

### H-08 — Ship, station, and technology “specials” introduce mechanics that do not exist elsewhere

Examples include armor and perishable goods (`docs/05-ships-stations-factories.md:15-16`), terrain/hazard restrictions and multi-build (`docs/05-ships-stations-factories.md:27-29`), research bonuses (`docs/05-ships-stations-factories.md:42`), quality (`docs/05-ships-stations-factories.md:98,108`; `docs/12-simulation-foundations.md:84`), life support (`docs/04-tech-tree.md:59,119`), Grid Power, and station-to-station power sharing. These lack rules, state, UI, or balance. Autonomous routing is defined globally, but the distinct Tier 3/4 Hub benefit promised by “automated cargo routing” and “coordinate fleet routes” is not (`docs/05-ships-stations-factories.md:63-68`). Either remove unsupported specials from V1 or give each an explicit mechanic and acceptance criteria. “Life Support is required” is especially inconsistent because no later tech lists it as a prerequisite.

### H-09 — Core numeric catalogs are incomplete

Only three technologies have example costs (`docs/04-tech-tree.md:97-105`); ship speeds and survey/build speeds are qualitative (`docs/05-ships-stations-factories.md:11-42`); tiered station/factory full build costs are missing; survey durations are missing; gravity effects are missing; upgrade deltas depend on unavailable full target-tier costs (`docs/05-ships-stations-factories.md:129-160`); buffer defaults and per-resource allocations are missing; and no Construction Ship build-speed multipliers connect to station times. Marking costs as “Example” prevents the tables from being normative. Complete the catalog before balancing or implementation.

### H-10 — Recovery and lifecycle actions needed by the no-failure design are absent

The player is told they can always recover by redesigning routes or adding capacity (`docs/01-gdd.md:46-58`), but there are no defined actions for cancelling construction, moving/demolishing/reconfiguring a station, changing a mining target, decommissioning or recalling a ship, reclaiming components, cancelling a transport reservation, or recovering from zero fuel/zero build capacity. Research cancellation exists only as an “Abandon” label and a no-refund statement (`docs/04-tech-tree.md:20-25`; `docs/11-ui-interactions.md:238-242`). The absolute “no permanent resource loss” phrase also conflicts semantically with intentional Fuel, Gate-fuel, and research consumption; narrow it to no accidental/destructive loss if that is the intended promise. Document safe recovery paths and ensure no valid player action sequence can permanently remove the ability to progress.

### H-11 — Distributed inventory is repeatedly treated as global storage

The logistics model is explicitly station-buffer based (`docs/07-routes-and-logistics.md:7-17`), but ship building consumes from unspecified “storage” (`docs/11-ui-interactions.md:81-85`) and research deducts from “network storage” (`docs/11-ui-interactions.md:245-251`). Define whether orders require local inventory, reserve materials anywhere and request delivery, or spend from a magical global pool. The answer affects player strategy, UI availability, cancellation, and the construction/research state machines.

### H-12 — UI error, empty, blocked, and confirmation states are largely missing

The happy paths are illustrated well, but there are no specified messages/actions for insufficient or reserved components, no suitable Construction Ship, no free slot/dock, unsurveyed or invalid sites, locked recipes, unreachable fuel, full input/output, a stalled Research Station, duplicate research, a lost selection after zoom, full build queues, or save failure. Tech visibility also disagrees: the onboarding treats seeing the full tree as a Phase 5 milestone (`docs/06-onboarding.md:128-129`), while the Research UI always shows it (`docs/11-ui-interactions.md:204-211`). The UI mockup says Tier 2 globally requires two Tier 1 technologies, whereas several Tier 2 nodes have one named prerequisite (`docs/11-ui-interactions.md:226-235`; `docs/04-tech-tree.md:50-59`). Define availability versus enabled state, visibility versus discoverability, reason text, remediation links, destructive confirmations, and whether orders may be placed before materials exist.

### H-13 — Input and accessibility requirements are internally incomplete

The design claims mouse-only play, also mentions pinch, right-click, hover-style detail, keyboard shortcuts, and “fat-finger safety” (`docs/11-ui-interactions.md:9-35,308-325`). Touch has no alternative for right-click, and keyboard navigation does not define focus, activation, context menus, or map-object selection. Tab is assigned to cycling stations without explaining how normal focus traversal works. A 32×32 target is not tied to physical/CSS pixels or a target platform, while important visible objects can be only 4–24px (`docs/10-iconography-and-textures.md:142-152`); larger invisible hit regions and overlap disambiguation are not specified. Red/green state, colored routes, pulse/blink alerts, and animated dashes lack non-color cues and reduced-motion behavior (`docs/07-routes-and-logistics.md:101-118`; `docs/v2-gate-logistics.md:59-65`). Specify supported input devices, complete keyboard/controller/touch mappings, focus order, semantic labels, hit regions, contrast targets, text scaling bounds, color-independent state, and reduced motion.

### H-14 — Cargo color and icon rules do not uniquely identify resources

Many distinct cargo types deliberately share a color and icon: Metal Ore/Metals, Carbon Soil/Carbon Fiber, Water Ice/Fuel, and Helium-3/Reactor Rods (`docs/08-visual-style.md:44-59`; `docs/10-iconography-and-textures.md:98-111`). Yet route color is supposed to identify cargo at a glance. Frozen Gases and Crystal Deposits do not have clear entries, “Sensors” is not a `ResourceType`, and Gate Node appears both as a material and a component. Define a unique combination of hue, shape, pattern, and label for each of the 25 resource/component enum values, with exact color tokens and contrast tests.

### H-15 — The visual asset specification is not production-ready

The chosen 2D direction still leaves the detailed station as “PNG/mesh” and TBD (`docs/09-zoom-levels.md:61-70`; `docs/10-iconography-and-textures.md:44-53,156-169`). The inventory describes roughly 62 file assets (72 row items if the 10 code-defined route colors are counted), rather than “~50”; the machine-icon table contains nine entries while the inventory says eight, and an Orbital Mining Station icon would make ten (`docs/10-iconography-and-textures.md:113-125,156-171`). Selection/focus, locked, unpowered, construction, warning, survey-progress, route-arrow, animation, Gate-phase, and reduced-motion variants are also absent. The generation pipeline lacks model/version, seeds, complete prompts, negative prompts, licenses, source-file policy, naming, sprite-sheet layout, color space, alpha handling, compression, filtering, mip/LOD strategy, and engine import settings. A single 256px sprite scaled to 16px is unlikely to preserve deliberately painted detail without authored small-size variants. Decide the renderer and deliver an art bible plus a generated/reconciled asset manifest before producing the full set.

### H-16 — Documented scale targets contradict each other and lack budgets

The V1 system supports only about 16–26 station slots (`docs/02-the-system.md:83-95`), the simulation assumes about 200 stations and 500 ships (`docs/12-simulation-foundations.md:147-152`), and the art document discusses readability across 500+ stations (`docs/10-iconography-and-textures.md:31-34`). Job matching is labeled cheap despite worst-case pair matching per idle ship, with no benchmark or maximum entries per station. Choose separate V1 and future stress targets, then state frame/tick budgets, representative save sizes, route-line limits, UI aggregation/decluttering, and benchmark scenarios.

### H-17 — Save/load requirements omit reliability, compatibility, and deterministic state

The save list simultaneously includes the supply/demand tables and calls them rebuildable/ephemeral (`docs/12-simulation-foundations.md:127-137`; `docs/13-data-models.md:151-153`). It omits RNG state for belt drift, active reservations, queues, throughput windows, content/schema version, and migration. “Manual save on quit” and one autosave slot provide no atomic-write, backup, corruption, crash, quota/disk failure, or version-compatibility behavior (`docs/12-simulation-foundations.md:139-145`). Define the persisted/runtime boundary, deterministic reconstruction, atomic replacement plus backup, validation, migrations, and failure UI. Decide whether camera/UI state should really reset, as that affects player continuity.

### H-18 — Gate completion and V2 logistics are not coherent yet

V1 victory is variously completing, powering, operating, or activating the Gate (`docs/01-gdd.md:42-44`; `docs/03-economy.md:103-110`; `docs/04-tech-tree.md:75-78`; `docs/05-ships-stations-factories.md:113-125`). The UI does not show the continuous activation fuel requirement (`docs/11-ui-interactions.md:268-287`). In V2, another file promises explicit Origin/Destination/Cargo/Frequency routes (`docs/02-the-system.md:72`), but `docs/v2-gate-logistics.md` defines only connections, signals, direction, and occupancy. It does not define gate construction/topology, remote-end creation, travel time, connection versus gate capacity, queue policy, route integration, state models, or conflicting direction settings. Rename “Summary (V2 Complete)” (`docs/v2-gate-logistics.md:67`) and treat the file as a concept note until those decisions are made.

### H-19 — Surface, orbital, gravity, and hazard concepts do not form one placement model

Planets have gravity and surface conditions affecting landing/launch (`docs/02-the-system.md:20-26`), Mining Stations may be “on or in orbit” (`docs/05-ships-stations-factories.md:72-83`), and Construction Ships gain terrain/hazard permissions (`docs/05-ships-stations-factories.md:24-29`). In contrast, the visual/UI model places all stations in orbital ring slots (`docs/08-visual-style.md:61-85`; `docs/11-ui-interactions.md:39-73`). Gas giants advertise atmospheric resources but have zero slots (`docs/02-the-system.md:30-36`). Choose an all-orbital abstraction or specify surface sites, landing legs, gravity cost, hazard checks, gas-giant platforms, and how orbital facilities extract surface resources.

### H-20 — Fixed-tick execution and within-tick visibility are underspecified

One tick is tied directly to one real-time second (`docs/12-simulation-foundations.md:3-7`), but the documents do not define a monotonic-clock accumulator, catch-up after a frame stall, application backgrounding, clock changes, maximum catch-up work, or offline progress. Entity iteration and write visibility within each phase are also unspecified (`docs/12-simulation-foundations.md:9-21`): for example, whether cargo arriving during movement can be processed that tick and whether mined output can be assigned immediately. Movement may apply the lane multiplier twice because the tick step says `speed × lane_multiplier`, while `effectiveSpeed` already includes it (`docs/12-simulation-foundations.md:13,119`; `docs/13-data-models.md:65-67`). Define stable iteration, snapshot/intent/commit boundaries, derived versus stored speed, missed-time policy, and safe post-commit save points.

### H-21 — Supply/demand threshold opening and closing rules are incomplete and possibly reversed

Demand opens below its threshold and supply opens above its threshold (`docs/12-simulation-foundations.md:54-58`), but the following sentence says demand/supply entries are removed when crossing back “below/above” (`docs/12-simulation-foundations.md:60`), which is reversed if the nouns retain their order. Equality, amount updates, initial entry creation, threshold edits, reserved quantities, and oscillation at the boundary are not defined. State exact comparison operators and use hysteresis or separate open/close thresholds to prevent one-tick chatter.

### H-22 — Per-resource buffers conflict with station-wide storage capacity

Each `Buffer` has its own `max` (`docs/13-data-models.md:23-34`), while Hub storage is presented as a single total capacity (`docs/05-ships-stations-factories.md:63-70`). Giving every handled resource a full-size buffer could multiply actual capacity by the number of resource types. The Hub assembly rule also draws inputs from its output buffer (`docs/12-simulation-foundations.md:99-106`), blurring producer/consumer semantics. Define whether storage is shared, partitioned, or per-resource; how players allocate it; whether fuel has a reserved compartment; and invariants for current, reserved, in-transit, and available capacity.

### H-23 — Production, upgrade, orbital-mining, and tutorial interfaces are missing

The onboarding selects refinery products and builds components (`docs/06-onboarding.md:25-29,73-77,120-129`), but `docs/11-ui-interactions.md` has no recipe selector, production-slot/queue UI, Hub assembly UI, Construction Factory order UI, mining-target control, or station-upgrade flow. Orbital Mining is required and has art, but is absent from the build menu and has no Belt placement flow (`docs/06-onboarding.md:123-125`; `docs/10-iconography-and-textures.md:35-43`; `docs/11-ui-interactions.md:52-69`). Add these production and placement flows. If the onboarding document is intended to drive a scripted tutorial rather than merely pacing/playtest goals, separately define prompt triggers, objective UI, success/fallback conditions, hints, skip/replay, and save/restore behavior; analytics checkpoints are optional product instrumentation, not an implied current requirement.

### H-24 — Deposit semantics are ambiguous and leave the no-failure promise unverified

Difficulty is described as finite **throughput** and the game promises there is always a solution (`docs/01-gdd.md:46-58`). Celestial resources are stored as integer “available deposits” (`docs/13-data-models.md:126-140`), but mining adds output without decrementing a deposit (`docs/12-simulation-foundations.md:13-20,66-75`). Belt deposits explicitly never deplete, randomly move by ±10% of base every 1,000 ticks, and alter output by a “remaining deposit fraction” (`docs/02-the-system.md:47-55`). The documents do not actually require ordinary deposits to deplete, but they also do not say whether these values represent reserves, richness, or throughput caps. Different implementations could therefore produce materially different economies, and an exhaustible/random interpretation has not been proven compatible with the solvability promise. Define depletion or non-depletion, regeneration, drift bounds/distribution/seed, survey visibility, and minimum Gate-critical availability; validate generated worlds if content is variable.

## Medium-priority findings

### M-01 — The starting system is partly example, partly requirement

Planet-type tables are labeled examples and use slot ranges (`docs/02-the-system.md:28-36`), while the “starting system map should have” a fixed body list but still uses ranged capacity (`docs/02-the-system.md:74-95`). The ranges are not internally tight: 2–3 rings with 2–4 slots permit 4–12 slots, not the stated 6–8, and 1–2 rings with 1–3 slots permit 1–6, not 2–4. The onboarding assumes particular resource reveals, including Crystal Deposits on the volcanic world even though the economy locates them only in belts and some moons (`docs/06-onboarding.md:11-19,51-54,78,98,123`; `docs/03-economy.md:19`). State whether V1 uses one authored system, seeded procedural generation, or both. If authored, provide exact body IDs, radii/angles, slots, deposits, survey depth, names, and starting inventory. If procedural, provide valid slot combinations, generation constraints, and solvability rules.

### M-02 — Product requirements and target platforms are missing

There is no definitive platform, renderer/engine, minimum resolution, aspect ratio, performance class, offline/online expectation, distribution format, audience, session-length target beyond onboarding, or localization target. These decisions materially affect touch/right-click support, save behavior, asset formats, UI density, and performance assumptions. Add a short product requirements document and label unmade decisions explicitly.

### M-03 — Technical architecture is missing

The tick loop and structs are useful, but there is no module boundary, content-loading approach, event/command model, rendering-versus-simulation ownership, UI state model, dependency direction, error strategy, or deterministic test harness. Add a concise architecture document and ADRs for numeric representation, scheduling, pathing, persistence, and content data.

### M-04 — There is no verification or balance plan

No documentation defines unit/property tests, golden simulation scenarios, progression reachability, resource conservation, save/load equivalence, deterministic replay, performance benchmarks, UI accessibility checks, or playtest metrics. Add a test strategy and a balance model. At minimum, model the first five hours and Gate completion using the same canonical data the game will load.

### M-05 — Responsive layout and overall information architecture are unspecified

Individual panels are sketched, but there is no full-screen wireframe showing HUD, objective/tutorial area, notifications, game time/pause, save/settings, global alerts, panel stacking, tooltip placement, or behavior at narrow/wide aspect ratios. Define layout regions, modal versus non-modal panels, z-order, focus restoration, selection persistence, and overflow/virtualization for large station/resource lists.

### M-06 — Audio, settings, localization, and narrative presentation are absent

No file covers sound/music cues, volume controls, graphics/accessibility settings, key rebinding, localization-safe strings/layout, naming, intro/victory presentation, or post-victory continuation. These may be intentionally deferred, but should appear in V1 scope/non-goals so omission is deliberate rather than accidental.

### M-07 — The repository has no README or documentation index

A new reader has no root explanation of what Steel Horizons is, project status, canonical document order, V1/V2 boundary, or how to contribute. Add `README.md` with a concise pitch, current phase, documentation map, source-of-truth rules, and links to key documents. The numeric filenames imply an order, but `v2-gate-logistics.md` sits outside it and cross-references are mostly plain text rather than links.

### M-08 — Canonical ownership and change governance are missing

Files have no owner, status (concept/draft/approved), last-reviewed date, decision history, or statement of which file wins when facts conflict. Add lightweight front matter or a docs index with status/owner, and use ADRs or a decision log for foundational choices. Add a changelog or release/version marker for the design baseline.

### M-09 — Terminology needs a controlled glossary and data dictionary

Examples include Carbon Rich Soil/Carbon Soil, Water/Water Ice, Metal Ore/Metals/minerals, Asteroid Belt/Asteroid Cluster, route/flow/job, station/factory/structure, slot/dock, storage/buffer/network storage, tier across technologies/resources/entities, and “surface” mining even though stations are orbital. Add a glossary with canonical player-facing names, internal IDs, units, and distinctions.

### M-10 — Licensing and contribution policy are absent

There is no repository license, contribution guide, code of conduct, or asset provenance policy. This matters before external sharing, reuse, or contribution and is especially important for locally generated AI-assisted art (`docs/10-iconography-and-textures.md:3-25`) and named design influences (`docs/01-gdd.md:60-65`). Record model/output licensing, human edits, source prompts/seeds, third-party fonts/icons, and attribution requirements.

### M-11 — Cross-links and Markdown quality need automation

References such as `07-routes-and-logistics.md` and `13-data-models.md` are plain text rather than relative links. `docs/02-the-system.md:26` says “see Orbit Rings below,” but that section is not in that file. The refinery throughput table declares five columns and supplies four in every data row (`docs/12-simulation-foundations.md:79-84`), causing shifted/empty cells. Add Markdown linting, link checking, terminology checks, and a small validator for table/catalog consistency.

### M-12 — Normative requirements are mixed with examples and unresolved choices

Several implementation-critical values are labeled “Example,” “usually,” “may,” “could,” “Auto,” approximate, or TBD. Examples include planet layouts, research costs, Gate placement, station model format, UI control type, animation duration, and asset counts. Introduce explicit labels such as **Decision**, **Requirement**, **Example**, **Open question**, and **V2**, and maintain an open-decisions list with owners.

## Low-priority findings

### L-01 — Some documents repeat the same prose without adding authority

Autonomous logistics, fog, zoom, route visuals, and the Gate goal are restated across many files. Repetition helps local readability but creates the drift seen in this audit. Keep a short summary locally and link to one normative section for detailed rules.

### L-02 — A few labels and examples reduce clarity

Examples include `Construction Fac`, single-letter `M` for Metals, “Ships Assigned” for ships merely serving an autonomous one-shot job, and “round trip” for a ship whose next job is not guaranteed to return (`docs/11-ui-interactions.md:52-67,108-175,213-243`). The Gate mockup says 6/8 nodes but visually fills eight blocks (`docs/11-ui-interactions.md:270-287`), and “Slots: 4/8” does not say used or remaining. Use full resource names or a documented unit legend, correct progress examples, and distinguish assigned, reserved, en route, and recently served.

### L-03 — Historical V2 residue remains in V1 files

HTML comments and skipped band numbers describe content that was moved (`docs/07-routes-and-logistics.md:128`; `docs/09-zoom-levels.md:7-10`), while Galaxy/V2 behavior still appears in V1 interaction and summary sections (`docs/11-ui-interactions.md:17`; `docs/07-routes-and-logistics.md:183-190`). Remove migration notes from normative docs or move them to history/ADRs.

### L-04 — Documents lack basic maintenance metadata

Adding status, owner, last-reviewed date, intended audience, and normative/illustrative classification would make future reviews faster and reduce accidental reliance on stale sketches.

## Document-by-document assessment

| Document | Assessment | Most important next change |
|---|---|---|
| `docs/01-gdd.md` | Strong pitch, loop, scope, and tone; “always solvable” is not backed by recovery/resource guarantees. | Add product pillars/non-goals and formal solvability/soft-lock requirements. |
| `docs/02-the-system.md` | Useful authored-system outline; geometry, gravity, resource names, deposits, and belt randomness are underdefined. | Decide authored versus procedural system and specify exact travel/deposit model. |
| `docs/03-economy.md` | Clear tiered flow; recipes lack quantities and conflict with facility tiers and bootstrap. | Make a canonical recipe/build-cost catalog and starting inventory. |
| `docs/04-tech-tree.md` | Readable graph and durations; several unlocks conflict or are unreachable, and most costs are absent. | Rebuild from a validated dependency graph with one unlock owner. |
| `docs/05-ships-stations-factories.md` | Helpful catalogs and build times; many specials have no mechanics, full costs are missing, and Research Station details are absent. | Complete numeric entity definitions and remove/defer unsupported specials. |
| `docs/06-onboarding.md` | Excellent player-learning structure; current sequence violates prerequisites and docked-ship constraints. | Derive a timed golden path from canonical data and simulate it. |
| `docs/07-routes-and-logistics.md` | Communicates the intended player mental model; scheduler, reservation, docking, and bottleneck rules are unsafe or incomplete. | Specify deterministic job/reservation state transitions with worked examples. |
| `docs/08-visual-style.md` | Cohesive top-down direction; palette claims are not supported by exact tokens or accessibility checks. | Add design tokens, non-color encodings, and reference compositions. |
| `docs/09-zoom-levels.md` | Visibility tables are useful; band numbering and empty-click behavior contradict this and other documents. | Replace with one explicit camera state/transition table. |
| `docs/10-iconography-and-textures.md` | Good initial asset inventory; counts/identities and production pipeline are incomplete. | Choose renderer/format and create a reproducible, licensed asset manifest. |
| `docs/11-ui-interactions.md` | Strong happy-path mockups; research/build semantics conflict with simulation and failure/accessibility states are missing. | Add command/state/error matrices and reconcile with simulation contracts. |
| `docs/12-simulation-foundations.md` | Good tick-order starting point; numeric representation, distance, determinism, scheduler, and persistence are incomplete. | Write invariants and deterministic algorithms against the revised data model. |
| `docs/13-data-models.md` | Useful attempt at explicit types; cannot represent much of the required aggregate state. | Redesign around commands/orders/state machines and save round-trip tests. |
| `docs/v2-gate-logistics.md` | Appropriate place for future concepts; it is a sketch, not a completed V2 design, and contradicts the promised route model. | Mark as concept status and defer detail until the V1 Gate contract is stable. |

## Missing documentation and artifacts

Recommended additions, in priority order:

1. **Root README and documentation index** — project status, scope, reading order, canonical sources, and V1/V2 boundary.
2. **Canonical content catalog** — machine-readable resources, recipes, techs, unlocks, entities, tier stats, build/upgrade costs, durations, starting inventory, and authored system data.
3. **Progression reachability specification** — dependency graph, golden onboarding path, minimum guaranteed deposits, soft-lock analysis, and automated reachability rules.
4. **Simulation contract** — units, fixed-point/atomic math, tick ordering including transfers, deterministic scheduling/reservations, travel/path model, invariants, and performance budgets.
5. **Complete persisted-state schema** — aggregate `GameState`, discriminated orders/jobs, runtime/persisted boundary, RNG, versions, migrations, validation, atomic saves, and backups.
6. **UX state and accessibility specification** — overall layout, camera state machine, command availability, blocked/error/cancel states, keyboard/controller/touch behavior, focus, contrast, reduced motion, and scaling.
7. **Balance model** — executable spreadsheet or simulation covering the first five hours, representative mid-game networks, Gate completion, fuel, docks, and bottleneck thresholds.
8. **Test strategy** — unit/property tests, deterministic replay, conservation checks, progression tests, save/load equivalence, performance scenarios, accessibility checks, and playtest metrics.
9. **Art bible and asset manifest** — exact tokens, fonts, rendering rules, LOD sizes, naming/import settings, prompts/models/seeds, licenses, attribution, and source/generated asset handling.
10. **Architecture decisions and roadmap** — engine/platform decision, system boundaries, content pipeline, milestones, definition of V1 done, and explicit non-goals.
11. **Project governance files** — `LICENSE`, `CONTRIBUTING.md`, change/decision log, and ownership/status metadata.

## Recommended remediation sequence

### Phase 1 — Resolve product and progression decisions

1. Freeze the V1 feature boundary and move all Galaxy/inter-system operation details to clearly marked future material.
2. Decide the starting state, bootstrap recipes, survey control, pause behavior, camera states, deposit behavior, and precise Gate victory trigger.
3. Build the canonical tech/recipe/entity graph and make the Space Gate mechanically reachable.
4. Rewrite the onboarding from that verified graph.

**Exit criterion:** A scripted fresh-game path can legally reach every onboarding milestone and the Gate using defined resources, facilities, ships, and elapsed time.

### Phase 2 — Make the simulation contract executable

1. Choose units and numeric representation.
2. Specify deterministic order/reservation, travel, fuel, dock, production, research, construction, and bottleneck state machines.
3. Redesign `GameState` and save/load around those rules.
4. Add invariants and automated simulation tests.

**Exit criterion:** Two implementations using only the specification produce the same state for the same content, seed, commands, and ticks, including after save/load.

### Phase 3 — Align UX and visual documentation

1. Map every player command to simulation preconditions, state changes, failures, cancellation, and feedback.
2. Finalize the camera and overall screen layout.
3. Establish accessible visual tokens and unique cargo encodings.
4. Lock the renderer and asset pipeline before bulk asset production.

**Exit criterion:** Every V1 action has happy, blocked, error, and recovery behavior; every state is operable with each supported input method and is not communicated by color/motion alone.

### Phase 4 — Add repository governance and continuous checks

1. Add the README/index, status/owner metadata, license, contribution guide, and ADR/change log.
2. Add Markdown lint/link checks and custom validators for resource names, recipes, unlock references, table dimensions, and duplicate ownership.
3. Treat balance data and docs as outputs of the same canonical source where practical.

**Exit criterion:** Documentation changes cannot introduce broken links, malformed tables, unknown content IDs, duplicate unlock ownership, or an unreachable V1 progression without automated checks failing.

## Suggested definition of “implementation-ready documentation”

- Every resource, recipe, entity, technology, tier, cost, rate, and unit has one canonical definition.
- The exact starting state and V1 system content are defined.
- Every technology and the Space Gate are reachable in an automated dependency test.
- The first-five-hours scenario is simulated and matches its time/resource claims.
- Fractional/atomic resource math and rounding are explicit and conserve materials.
- Logistics assignments reserve supply, capacity, docks, and fuel deterministically.
- Construction, research, surveying, upgrades, and shipbuilding have serialized state machines.
- Save/load round trips preserve authoritative behavior and support version migration/failure recovery.
- Every UI command has preconditions, success, blocked, cancel, and error behavior.
- Camera, input, accessibility, responsive layout, and reduced-motion behavior are specified.
- Performance targets are separated into V1 and future scale and have benchmark scenarios.
- V1 and V2 material is clearly labeled, with no future mechanic presented as current behavior.
- Repository navigation, ownership, status, licensing, and documentation checks are in place.

## Overall conclusion

Steel Horizons has a solid design foundation and a notably clear player fantasy. The documents already contain enough specificity to expose real design risks, which is valuable. The immediate priority should be reconciliation, not adding more prose: establish canonical content data, prove the opening and Gate progression, and define the missing state machines and numeric invariants. Once those foundations are authoritative, the existing onboarding, UI, visual, and simulation documents can be revised into a cohesive implementation specification with much less duplication and drift.
