---
status: Approved
owner: Product Owner
last-reviewed: 2026-08-04
---

# Authored Content & Balance Catalog — V1

This document is the canonical source for every exact V1 content value: starting bodies, deposits, inventory, recipes, technologies, ship statistics, station statistics, and build costs. Other GDDs explain intent and may summarize these tables, but must not override them. The implementation mirrors these records in versioned JSON and validates them at startup.

## Starting System Bodies

### Haven — Rocky Terran Player Start

| Field | Value |
|-------|-------|
| ID | `planet_haven` |
| Body type / subtype | Planet / RockyTerran |
| Parent | none |
| Position | lane `habitable`, radius 1,200, angle 0 |
| Survey depth | 3 (fully revealed) |
| Orbit slots | rings `[3, 2, 1]` — 6 total |

| Deposit | Amount | Minimum depth | Renewable |
|---------|-------:|--------------:|-----------|
| MetalOre | 4,000 | 1 | No |
| CarbonSoil | 3,000 | 1 | No |
| SiliconDust | 2,500 | 1 | No |

Hub Haven occupies ring 0, slot 0.

### Pyre — Volcanic Planet

| Field | Value |
|-------|-------|
| ID | `planet_pyre` |
| Body type / subtype | Planet / Volcanic |
| Parent | none |
| Position | lane `inner`, radius 600, angle 3,140 |
| Survey depth | 0 |
| Orbit slots | rings `[2, 1]` — 3 total |

| Deposit | Amount | Minimum depth | Renewable |
|---------|-------:|--------------:|-----------|
| VolcanicSulfur | 1,800 | 1 | No |
| RareEarthMinerals | 1,200 | 1 | No |
| CrystalDeposits | 500 | 2 | No |

### Boreas — Ice World

| Field | Value |
|-------|-------|
| ID | `planet_boreas` |
| Body type / subtype | Planet / IceWorld |
| Parent | none |
| Position | lane `outer`, radius 2,800, angle 4,710 |
| Survey depth | 0 |
| Orbit slots | rings `[2, 1]` — 3 total |

| Deposit | Amount | Minimum depth | Renewable |
|---------|-------:|--------------:|-----------|
| WaterIce | 2,500 | 1 | No |
| FrozenGases | 1,500 | 1 | No |
| CarbonSoil | 800 | 2 | No |

### Titan — Gas Giant

| Field | Value |
|-------|-------|
| ID | `planet_titan` |
| Body type / subtype | Planet / GasGiant |
| Parent | none |
| Position | lane `outer`, radius 3,200, angle 1,570 |
| Survey depth | 0 |
| Orbit slots | none; use moons |
| Deposits | none |

### Rime — Moon of Titan

| Field | Value |
|-------|-------|
| ID | `moon_rime` |
| Body type / subtype | Moon / IceWorld |
| Parent | `planet_titan` |
| Position | lane `outer`, radius 3,250, angle 1,870 |
| Survey depth | 0 |
| Orbit slots | ring `[2]` — 2 total |

| Deposit | Amount | Minimum depth | Renewable |
|---------|-------:|--------------:|-----------|
| Helium3 | 1,000 | 1 | No |
| RareEarthMinerals | 400 | 2 | No |

### Glint — Moon of Titan

| Field | Value |
|-------|-------|
| ID | `moon_glint` |
| Body type / subtype | Moon / RockyTerran |
| Parent | `planet_titan` |
| Position | lane `outer`, radius 3,300, angle 5,670 |
| Survey depth | 0 |
| Orbit slots | ring `[1]` — 1 total |

| Deposit | Amount | Minimum depth | Renewable |
|---------|-------:|--------------:|-----------|
| CrystalDeposits | 500 | 1 | No |
| MetalOre | 600 | 1 | No |

### The Veil — Asteroid Belt

| Field | Value |
|-------|-------|
| ID | `belt_veil` |
| Body type / subtype | AsteroidBelt / null |
| Parent | none |
| Representative position | lane `inner`, radius 900, angle 0 |
| Survey depth | 0 |
| Orbit slots | rings `[2, 2]` — 4 total |

Renewable belt values represent density, not a consumable quantity. Every 1,000 ticks, resources are visited in ResourceType order. For integer `range = baseline / 10`, `delta = (next_rng_u64 % (2 * range + 1)) - range`; current density adds delta and clamps to 70–130% of baseline. Mining output is multiplied by `current / baseline` using an exact rational accumulator.

| Deposit | Baseline | Minimum depth | Renewable |
|---------|---------:|--------------:|-----------|
| MetalOre | 2,000 | 1 | Yes |
| CarbonSoil | 1,200 | 1 | Yes |
| SiliconDust | 800 | 1 | Yes |
| CrystalDeposits | 300 | 2 | Yes |
| WaterIce | 800 | 2 | Yes |
| FrozenGases | 600 | 2 | Yes |
| VolcanicSulfur | 400 | 2 | Yes |

The seven bodies provide **19 station slots**: 6 + 3 + 3 + 0 + 2 + 1 + 4.

The starting JSON maps each body table directly to the complete GDD 13 record:
`orbit_ring_count` is the length of `slot_counts`; finite deposits serialize
`current = baseline = Amount`; renewable deposits serialize
`current = baseline = Baseline`; and deposit arrays use ResourceType order. Names are
the heading names, absent parents/subtypes are explicit `null`, and empty deposits or
slot arrays are explicit `[]`.

## Starting State

### Hub Haven

| Field | Value |
|-------|-------|
| ID | `hub_haven` |
| Display name | Hub Haven |
| Type / tier | Hub / 1 |
| Placement | `planet_haven`, ring 0, slot 0 |
| Docks | 2 |
| General cargo capacity | 200 |
| Fuel compartment | 200 Fuel, full |
| Priority | 50 |
| Installed components | 1 StructuralFrame, 1 PowerCore, 1 ControlSystem, 1 CargoModule |
| built_in_research_max_tier | 1 (unique authored Hub Haven console) |

Hub Haven's installed hull components are part of the station and are not loose inventory. The following **deployment kit** is loose cargo in output buffers:

| Component | Quantity | Planned bootstrap use |
|-----------|---------:|-----------------------|
| StructuralFrame | 6 | Mine, refinery, cargo ship, research ship, research station, construction factory |
| DriveAssembly | 3 | First cargo ship, research ship, second/Tier-2 cargo ship |
| ResearchLab | 2 | Research ship and research station |
| ConstructionBay | 1 | First construction factory |
| PowerCore | 4 | Mine, refinery, research station, construction factory |
| ControlSystem | 1 | Refinery |
| CargoModule | 2 | Mine and first cargo ship |

The kit deliberately bypasses technology locks for these pre-assembled units. Replacement production still requires the technology in the canonical recipe table.

Each deployment-kit component is an output buffer whose `current = max` equals its quantity in the table, `demand_threshold = 0`, and `export_threshold = 0`, so build orders can allocate it immediately. The seven component maxima total 19 of the Hub's 200 general capacity, leaving 181 for bootstrap refining/research buffers. Later component assembly expands/reconfigures output maxima explicitly. Input buffers are created/reallocated by valid production/research configuration; every buffer-max change must keep the total at or below 200. An input maximum cannot fall below `current + inbound_reserved`; an output maximum cannot fall below `current`, which already includes outbound-reserved units. Fuel is the separate `resource = Fuel`, `current = max = 200`, `demand_threshold = 20`, `export_threshold = 50` compartment.

Every other Hub field is explicit in `starting_system.v1.json`: `input_buffers = []`;
`docked_ship_ids = ["ship_builder_1"]`; `holding_ship_ids = []`;
`max_docks = 2`; one idle slow component-assembly `ProductionSlot` with null recipe,
empty maps, and zero progress/work fields; `mining_targets = []`;
`active_research_id = null`; and `ship_build_queue = []`. The output buffers use
ResourceType order. No other implicit station default is permitted.

The Hub has a built-in Tier-1 research console. It can run one Tier-1 technology at a time without a Research Ship. Every project assigned to a Research Station requires a docked Research Ship, and Tier-2+ projects can run only there.

### Starting Ship

| Field | Value |
|-------|-------|
| ID / name | `ship_builder_1` / Builder-1 |
| Role / tier | Construction / 1 |
| Location | docked at `hub_haven` |
| Fuel | 100 / 100 |
| State | Idle |
| Installed components | 1 StructuralFrame, 1 DriveAssembly, 1 ConstructionBay |

No Cargo or Research Ship exists at tick 0.

The complete serialized ship copies the Construction-T1 definition: position is
Haven's exact system position; `docked_at = hub_haven`; base speed 3,000; base mass
15; Cargo fields are null/zero; `build_cargo = {}` with capacity 20; Fuel and Life
Support remainders are zero; build work is 1; survey work/depth are zero; `job = Idle`;
and `travel_plan = null`. Hub Haven's dock list and the ship's `docked_at` field must
agree.

### Starting Technologies

- `basic_construction`
- `basic_refining`
- `basic_power`
- `basic_control`

### Starting Root Metadata

| Field | Value |
|-------|-------|
| scenario ID | `starting_system` |
| schema_version | 1 |
| content_version | `v1` |
| lifecycle | Paused |
| tick | 0 |
| next_server_sequence | 1 |
| next_event_sequence | 1 |
| generated ID counters | ship/station/build_order/reservation/salvage/survey_order all start at 1 |
| gate_build | null |
| RNG words | `[0x243f6a8885a308d3, 0x13198a2e03707344, 0xa4093822299f31d0, 0x082efa98ec4e6c89]` |

`schema_version` is the engine-owned state-schema value used when constructing
`GameState`; it is not duplicated in `StartingScenario` or included as authored
content. `gate_build = null` is the explicit initialization rule from GDD 13. The
remaining rows are serialized starting-scenario values. The initializer never injects
inventory beyond the serialized Hub buffers.

## Automatic Buffer Defaults

Configuration commands allocate buffers deterministically so a newly configured station can run without a separate setup command:

1. For every non-Fuel recipe input, `SetProductionRecipe` computes a minimum of `max(current + inbound_reserved, one_batch_input)`; for every non-Fuel output it uses `max(current, one_batch_output)`. Fuel instead validates space/stock against the fixed compartment. The command first attempts a preferred maximum of `max(minimum, 50)` for every general buffer it must create/expand. If all preferred maxima do not fit, it tries the complete minimum set. If that also does not fit, the command is rejected atomically with the additional capacity required. Existing maxima are never reduced automatically.
2. Newly created recipe input buffers use `demand_threshold = 100`; newly created outputs use `export_threshold = 0`. Existing player thresholds are preserved.
3. `SetMiningTarget` follows the same preferred maximum of 50 and a minimum of `max(current, 1)` for its output buffer. Retuning never deletes the prior target's buffer or inventory.
4. Research uses its full outstanding inbound amount rather than the 50-unit preference, as defined in GDD 4. Fuel requirements reserve the fixed Fuel compartment and do not allocate general buffers.

All candidate resource keys are processed in ResourceType order, but preferred-versus-minimum is chosen for the complete set rather than partially allocating preferred buffers. Fuel always uses the separate Fuel compartment: `make_fuel`, `assemble_drive`, and Life Support research access it there rather than through general buffers.

The exact `AuthoredDefaults` record is: new-station priority 50, preferred
general-buffer maximum 50, input demand 100, output export 0, Fuel demand 20,
Fuel export 50, mining retune 10 ticks, upgrade work 60 per tier crossed,
demolition work 30, incremental survey depth work `[300, 600, 900]`, and Hub
shipyard work 1 per tick. General input
buffers serialize inactive `export_threshold = 0`; general outputs serialize inactive
`demand_threshold = 0`.

## Canonical Refining Recipes

Every refinery recipe has a 10-tick cycle. One production slot reserves the listed integer inputs at cycle start and transfers outputs at completion.

| Recipe ID | Tech | Minimum refinery | Inputs | Outputs |
|-----------|------|------------------|--------|---------|
| `smelt_metals` | Basic Refining | T1 | 2 MetalOre | 1 Metals |
| `process_carbon` | Basic Refining | T1 | 2 CarbonSoil | 1 CarbonFiber |
| `cut_wafers` | Basic Refining | T1 | 2 SiliconDust | 1 SiliconWafers |
| `make_chemicals` | Advanced Refining | T2 | 1 VolcanicSulfur + 1 WaterIce | 1 Chemicals |
| `make_fuel` | Advanced Refining | T2 | 2 FrozenGases + 1 Chemicals | 2 Fuel |
| `smelt_alloys` | Alloy Smelting | T3 | 2 Metals + 1 RareEarthMinerals | 1 Alloys |
| `make_optics` | Sensor Systems | T3 | 1 CrystalDeposits + 1 SiliconWafers | 1 Optics |
| `make_reactor_rods` | Fusion Power | T3 | 2 Helium3 + 1 RareEarthMinerals | 1 ReactorRods |
| `reclaim_alloys` | Alloy Smelting | T3 | 1 Alloys | 2 Metals + 1 RareEarthMinerals |
| `reclaim_optics` | Sensor Systems | T3 | 1 Optics | 1 CrystalDeposits + 1 SiliconWafers |
| `reclaim_reactor_rods` | Fusion Power | T3 | 1 ReactorRods | 2 Helium3 + 1 RareEarthMinerals |

The three reclamation recipes are exact inverses. They prevent a player from soft-locking the Gate by overproducing the wrong critical refined good; time and transport are the only reclamation costs.

## Canonical Component Recipes

The Station Hub assembles any unlocked non-Gate component at one unit per 30 ticks. A Construction Factory uses one 10-tick slot per unit. Gate Nodes require a Tier-4 Construction Factory.

Accordingly, every non-Gate assembly record has exactly
`[{ station_type: Hub, minimum_tier: 1, cycle_ticks: 30 },
{ station_type: Construction, minimum_tier: 1, cycle_ticks: 10 }]` in that order.
`assemble_gate_node` has only
`{ station_type: Construction, minimum_tier: 4, cycle_ticks: 10 }`.

| Recipe ID | Tech | Inputs | Output |
|-----------|------|--------|--------|
| `assemble_frame` | Structural Engineering | 4 Metals + 2 CarbonFiber | 1 StructuralFrame |
| `assemble_power_core` | Basic Power | 4 Metals + 2 SiliconWafers | 1 PowerCore |
| `assemble_control` | Basic Control | 2 SiliconWafers + 1 Metals | 1 ControlSystem |
| `assemble_drive` | Factory Automation | 3 Metals + 2 Fuel + 1 ControlSystem | 1 DriveAssembly |
| `assemble_cargo_module` | Cargo Handling | 3 Metals + 2 CarbonFiber + 1 ControlSystem | 1 CargoModule |
| `assemble_research_lab` | Sensor Systems | 3 SiliconWafers + 1 Optics + 1 PowerCore | 1 ResearchLab |
| `assemble_construction_bay` | Factory Automation | 1 StructuralFrame + 1 PowerCore + 2 ControlSystem | 1 ConstructionBay |
| `assemble_gate_node` | Gate Construction | 4 Alloys + 1 PowerCore + 2 Optics + 1 ReactorRods | 1 GateNode |

Every component recipe has one exact inverse whose ID replaces the `assemble_`
prefix with `disassemble_`: `disassemble_frame`, `disassemble_power_core`,
`disassemble_control`, `disassemble_drive`, `disassemble_cargo_module`,
`disassemble_research_lab`, `disassemble_construction_bay`, and
`disassemble_gate_node`. Each inverse uses the same required technology, facility
requirements, tiers, and cycle lengths as its assembly recipe, consumes that recipe's
one output unit, and returns its complete input map. Thus the component catalog has
exactly eight assembly and eight disassembly records. This includes surplus Gate Nodes
at a Tier-4 Construction Factory.

## Canonical Technology Definitions

Costs and durations below are authoritative gameplay values, not examples.

| Technology ID | Tier | Prerequisites | Cost | Ticks | Principal unlocks |
|---------------|-----:|---------------|------|------:|-------------------|
| `basic_construction` | 0 | none | none | 0 | Cargo/Construction T1, Hub/Mining T1 |
| `basic_refining` | 0 | none | none | 0 | Refinery T1 and basic recipes |
| `basic_power` | 0 | none | none | 0 | PowerCore recipe |
| `basic_control` | 0 | none | none | 0 | ControlSystem recipe |
| `advanced_refining` | 1 | Basic Refining | 40 Metals + 20 CarbonFiber + 20 SiliconWafers | 600 | Refinery T2; Chemicals; Fuel |
| `structural_engineering` | 1 | Basic Construction | 30 Metals + 20 CarbonFiber | 400 | StructuralFrame; Cargo/Construction T2 |
| `sensor_systems` | 1 | Basic Control | 20 Metals + 30 SiliconWafers | 500 | Research Ship/Station T1; ResearchLab; Optics; survey depth 1 |
| `fusion_power` | 1 | Basic Power | 30 Metals + 20 CarbonFiber + 10 Helium3 | 700 | ReactorRods; Hub T2 |
| `cargo_handling` | 1 | Basic Construction | 30 Metals + 20 CarbonFiber | 400 | CargoModule |
| `alloy_smelting` | 2 | Advanced Refining | 40 Metals + 20 RareEarthMinerals + 20 Chemicals | 1,200 | Alloys; Refinery T3 |
| `factory_automation` | 2 | Structural Engineering + Cargo Handling | 40 Metals + 30 SiliconWafers + 10 ControlSystem | 900 | Construction Factory T1; DriveAssembly; ConstructionBay |
| `deep_survey` | 2 | Sensor Systems + Alloy Smelting | 30 SiliconWafers + 20 Metals | 900 | Research Ship/Station T2; survey depth 2 |
| `reactor_scaling` | 2 | Fusion Power + Alloy Smelting | 30 Alloys + 10 ReactorRods + 5 PowerCore | 1,200 | Construction Factory T2; Grid Power prerequisite |
| `orbital_logistics` | 2 | Cargo Handling + Deep Survey | 30 Alloys + 20 Optics + 5 ControlSystem | 800 | Belt mining; Mining T2 |
| `life_support` | 2 | Advanced Refining | 30 Chemicals + 20 Fuel + 10 SiliconWafers | 900 | 20% fuel reduction for outer/fringe arcs and radial burns touching those lanes; Heavy Transport prerequisite |
| `precision_manufacturing` | 3 | Factory Automation + Alloy Smelting | 60 Alloys + 30 SiliconWafers + 20 Optics + 10 PowerCore | 2,400 | Construction Factory T3; Mining T3; Engineer |
| `heavy_transport` | 3 | Orbital Logistics + Life Support + Factory Automation | 50 Alloys + 20 CargoModule + 10 DriveAssembly | 1,800 | Cargo T3; Hub T3 |
| `exploration_suite` | 3 | Deep Survey + Reactor Scaling | 40 SiliconWafers + 30 Optics + 10 PowerCore | 1,800 | Research Ship/Station T3; survey depth 3 |
| `grid_power` | 3 | Reactor Scaling | 50 Alloys + 30 ReactorRods + 10 ControlSystem | 2,400 | High-capacity grid infrastructure; Gate Construction prerequisite |
| `gate_theory` | 3 | Precision Manufacturing + Exploration Suite | 60 Alloys + 40 Optics + 20 ReactorRods | 2,400 | Gate site and theory |
| `advanced_fabrication` | 4 | Precision Manufacturing + Heavy Transport | 80 Alloys + 40 Optics + 20 ControlSystem + 10 PowerCore | 3,600 | All T4 ships/stations; Fabricator |
| `gate_construction` | 4 | Gate Theory + Advanced Fabrication + Grid Power | 100 Alloys + 60 ReactorRods + 20 PowerCore | 4,800 | GateNode recipe |
| `system_bridge` | 4 | Gate Construction | 50 Alloys + 30 ReactorRods + 10 PowerCore | 1,200 | Space Gate assembly and activation |

Tier names are progression labels; prerequisites, not a generic “complete N prior techs” rule, determine availability.

Recipe, ship, and station availability is derived and validated from each definition's
typed `required_tech`; it is not duplicated in `mechanic_unlocks`. The only independent
mechanic records are: `sensor_systems -> SurveyDepth { max_depth: 1 }`,
`deep_survey -> SurveyDepth { max_depth: 2 }`,
`exploration_suite -> SurveyDepth { max_depth: 3 }`,
`orbital_logistics -> AsteroidBeltOperations`,
`life_support -> LifeSupportFuelFactor { numerator: 4, denominator: 5 }`,
`gate_theory -> GateSiteVisibility`, and `system_bridge -> GateAssembly`. Every other
technology has an empty `mechanic_unlocks` array. The validator rejects duplicate or
contradictory mechanic records.

## Canonical Ship Definitions

### Cargo Ships

| Tier | Name | Tech | Capacity | Speed milli | Fuel | Base mass | Component cost |
|-----:|------|------|---------:|------------:|-----:|----------:|----------------|
| 1 | Courier | Basic Construction | 50 | 3,000 | 100 | 10 | 1 Frame + 1 Drive + 1 CargoModule |
| 2 | Hauler | Structural Engineering | 120 | 3,500 | 200 | 18 | 1 Frame + 1 Drive + 2 CargoModule |
| 3 | Bulk Carrier | Heavy Transport | 300 | 3,000 | 400 | 30 | 2 Frame + 2 Drive + 3 CargoModule |
| 4 | Fast Freighter | Advanced Fabrication | 100 | 6,000 | 800 | 20 | 1 Frame + 2 Drive + 1 CargoModule |

### Construction Ships

| Tier | Name | Tech | Build cargo | Speed milli | Fuel | Base mass | Build work/tick | Component cost |
|-----:|------|------|------------:|------------:|-----:|----------:|----------------:|----------------|
| 1 | Builder | Basic Construction | 20 | 3,000 | 100 | 15 | 1 | 1 Frame + 1 Drive + 1 ConstructionBay |
| 2 | Constructor | Structural Engineering | 40 | 3,500 | 200 | 25 | 2 | 1 Frame + 1 Drive + 1 ConstructionBay |
| 3 | Engineer | Precision Manufacturing | 80 | 4,000 | 400 | 40 | 3 | 2 Frame + 2 Drive + 2 ConstructionBay |
| 4 | Fabricator | Advanced Fabrication | 160 | 5,000 | 800 | 50 | 5 | 2 Frame + 2 Drive + 2 ConstructionBay |

### Research Ships

| Tier | Name | Tech | Speed milli | Fuel | Base mass | Survey work/tick | Max depth | Component cost |
|-----:|------|------|------------:|-----:|----------:|-----------------:|----------:|----------------|
| 1 | Scout | Sensor Systems | 3,500 | 100 | 8 | 1 | 1 | 1 Frame + 1 Drive + 1 ResearchLab |
| 2 | Surveyor | Deep Survey | 4,000 | 200 | 12 | 2 | 2 | 1 Frame + 1 Drive + 1 ResearchLab |
| 3 | Explorer | Exploration Suite | 4,500 | 400 | 18 | 3 | 3 | 2 Frame + 2 Drive + 1 ResearchLab |
| 4 | Pioneer | Advanced Fabrication | 5,500 | 800 | 24 | 5 | 3 | 2 Frame + 2 Drive + 2 ResearchLab |

Survey work requirements are 300 for depth 1, an additional 600 for depth 2, and an additional 900 for depth 3.

Every ship definition also serializes `build_work` by tier as
`[30, 60, 120, 240]`; the value is the same for all three roles at a tier.
In `ShipStats`, `build_work_per_tick` is zero for Cargo and Research Ships and only
Construction Ships use the table's positive build-work value. All other role-
inapplicable numeric stats are likewise explicit zeroes under GDD 13.

Research Ships have zero payload capacity. Cargo Ships use their cargo capacity and Construction Ships use their build-hold capacity in the common payload speed formula. For zero-capacity Research Ships, the multiplier is exactly 1/1 and the division is not evaluated.

## Canonical Station Definitions

### Capacity and Throughput

| Type | T1 | T2 | T3 | T4 |
|------|----|----|----|----|
| Hub docks / cargo | 2 / 200 | 4 / 500 | 6 / 1,000 | 8 / 2,000 |
| Hub Fuel compartment | 200 | 500 | 1,000 | 2,000 |
| Hub shipyard / component slots | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 |
| Non-Hub docks | 1 | 2 | 3 | 4 |
| Non-Hub Fuel compartment | 100 | 200 | 400 | 800 |
| Mining targets / units per active target per 10 ticks | 1 / 1 | 2 / 2 | 3 / 3 | 4 / 5 |
| Mining cargo capacity | 100 | 200 | 400 | 800 |
| Refinery slots / cargo | 1 / 200 | 2 / 400 | 3 / 800 | 4 / 1,600 |
| Construction slots / cargo | 1 / 200 | 2 / 400 | 3 / 800 | 4 / 1,600 |
| Research projects / cargo | 1 / 200 | 1 / 400 | 1 / 800 | 1 / 1,600 |

Every non-Hub station has one dock at T1, two at T2, three at T3, and four at T4. Research ships powering projects occupy docks.

Every newly completed station's Fuel compartment starts at 0 with demand threshold 20% and export threshold 50%; Hub Haven is the authored exception that starts at 200/200. Upgrading raises the maximum without creating Fuel. Fuel demand must remain less than or equal to its export threshold.

Every `StationStats.production_slots` value is zero for Hub, Mining, and Research;
Refinery and Construction use tiers 1/2/3/4 respectively. Zero cells in the cost table
below are explanatory only: all ResourceType quantity maps are sparse, omit zero-valued
keys, and serialize no required quantity as zero.

A newly completed station copies its definition's type/tier/stats and exact moved
component map. It begins with priority 50, empty input/output buffers, an empty Fuel
compartment at the authored maximum, empty holding/mining/shipyard collections, null
active research, and null built-in research (Hub Haven is the only console exception).
It owns the exact number of idle zeroed recipe slots defined by GDD 13. The completing
Construction Ship has an empty build hold and becomes Idle at/docked to the new
station; the station's sorted docked-ship list contains it. No inventory, recipe,
research capability, or Fuel is synthesized.

### Tier Unlocks

| Type | T1 | T2 | T3 | T4 |
|------|----|----|----|----|
| Hub | Basic Construction | Fusion Power | Heavy Transport | Advanced Fabrication |
| Mining | Basic Construction | Orbital Logistics | Precision Manufacturing | Advanced Fabrication |
| Refinery | Basic Refining | Advanced Refining | Alloy Smelting | Advanced Fabrication |
| Construction | Factory Automation | Reactor Scaling | Precision Manufacturing | Advanced Fabrication |
| Research | Sensor Systems | Deep Survey | Exploration Suite | Advanced Fabrication |

Research facility tier does not gate technology tier: any Research Station with a docked Research Ship can run an available Tier-2+ project. Higher Research Station tiers add buffer/dock capacity and resilience, avoiding a circular “research the upgrade before the facility can research it” rule.

### Full Build Costs

Upgrade cost is the target row minus the installed current-tier row. No component count may become negative.

| Type / Tier | StructuralFrame | PowerCore | ControlSystem | CargoModule | ResearchLab | ConstructionBay |
|-------------|----------------:|----------:|--------------:|------------:|------------:|----------------:|
| Hub T1 | 1 | 1 | 1 | 1 | 0 | 0 |
| Hub T2 | 2 | 2 | 2 | 2 | 0 | 0 |
| Hub T3 | 3 | 3 | 3 | 4 | 0 | 0 |
| Hub T4 | 4 | 4 | 4 | 6 | 0 | 0 |
| Mining T1/T2/T3/T4 | 1/2/3/4 | 1/2/3/4 | 0 | 1/2/3/4 | 0 | 0 |
| Refinery T1/T2/T3/T4 | 1/2/3/4 | 1/2/3/4 | 1/2/3/4 | 0 | 0 | 0 |
| Construction T1/T2/T3/T4 | 1/2/3/4 | 1/2/3/4 | 0 | 0 | 0 | 1/2/3/4 |
| Research T1/T2/T3/T4 | 1/2/3/4 | 1/2/3/4 | 0 | 0 | 1/2/3/4 | 0 |

Station build work is 60/120/240/480 for tiers 1–4. Ship build work is 30/60/120/240. Upgrade work is `60 * (target_tier - current_tier)` and demolition work is 30. A Construction Ship's work-per-tick stat applies to station, upgrade, demolition, and Gate work; Hub shipyard work advances at one work per tick.

Every station definition serializes `build_work` by tier as
`[60, 120, 240, 480]` for every station type. The remaining work constants come from
the exact `AuthoredDefaults` record above rather than hidden engine literals.

On upgrade completion, the delivered component delta moves into
`installed_components`; tier, stats, dock limit, general capacity, and Fuel maximum
become the target definition atomically without creating inventory or Fuel. Existing
buffers, recipe progress/output, mining targets/remainders, research, queues,
reservations, dock/holding order, priority, and Hub built-in-research value persist.
New recipe-slot indices are appended as idle zeroed slots; increased mining target
capacity creates no target until configured. The completing Construction Ship clears
its build hold and becomes Idle at/docked to the upgraded station. Tier skipping uses
the same rule once against the selected target definition.

## Space Gate Definition

The Gate site is at lane `fringe`, radius 4,000, angle 0. `BeginGateAssembly { fabricator_ship_id }` requires `system_bridge` complete and an idle Tier-4 Fabricator, then creates the unique `GateBuild`, assigns that ship, and advertises the exact undelivered manifest at logistics priority 100. The site has one virtual cargo-transfer berth per tick and no general storage or orbit slot.

Assembly prerequisites are `system_bridge` complete and the selected Tier-4 Fabricator. The cargo manifest is exactly:

- 8 `GateNode`
- 1 StructuralFrame
- 1 PowerCore
- 1 ControlSystem
- 1 `ReactorRods` for activation

| Phase | Work | Entry requirement |
|-------|-----:|-------------------|
| Site Preparation | 300 | Fabricator present |
| Frame Assembly | 600 | 8 `GateNode` + 1 `StructuralFrame` delivered |
| Power Integration | 300 | PowerCore + ControlSystem delivered |
| Activation | 120 | 1 delivered `ReactorRods`; consumed atomically on completion |

Activation consumes one `ReactorRods` unit and immediately transitions the lifecycle to Won. Continuing post-victory play is outside V1.

## Solvability Budget

The non-recyclable worst-case budget for completing **all** technologies plus the final Gate is bounded by:

| Critical raw input | Authored finite amount | Validated maximum spend | Headroom |
|--------------------|-----------------------:|------------------------:|---------:|
| RareEarthMinerals | 1,600 | 725 | 121% |
| Helium3 | 1,000 | 350 | 186% |
| CrystalDeposits | 1,000 finite + renewable belt | 400 | at least 150% |

The calculated minimum non-recyclable path uses 721 RareEarthMinerals and 328 Helium3; the validated maxima round those upward to 725 and 350 as an implementation guard. Crystal's 400 guard exceeds the current 166-unit research/Gate Optics path before considering the renewable belt.

Build components are fully recoverable and therefore excluded from non-recyclable spend except for the completed Gate. Common fuel-chain inputs also exist as renewable belt deposits. A content validation test expands every recipe, sums every technology cost and minimum victory build, verifies the table above, and proves the authored bootstrap command sequence reaches its first Cargo Ship.
