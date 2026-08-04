---
status: Approved
owner: Product Owner
last-reviewed: 2026-08-04
---

# Technology Tree

This document owns the unlock graph and research behavior. Exact costs and durations are canonical in [14-authored-content.md](./14-authored-content.md).

## Research State Machine

| State | Meaning |
|-------|---------|
| **AwaitingMaterials** | The station broadcasts demand for missing project resources. |
| **Ready** | All required resources are delivered and reserved. |
| **Active** | Progress advances and reserved resources are consumed deterministically. |
| **Paused** | Progress, consumed credit, and delivered materials are retained; a reason distinguishes Manual, NoResearchShip, and FacilityUnavailable. |
| **Complete** | The technology is permanently unlocked. |

Starting or resuming a project reserves matching stock already in the facility, then creates/expands per-resource input buffers by the remaining inbound amount and adds demand for `required - reserved - consumed`. It uses unallocated cargo capacity but never shrinks another buffer automatically. If the facility cannot fit the inbound maxima, the command is rejected with the exact additional capacity required; the player can shrink empty buffers or choose another facility. Once all materials are reserved, the project becomes Ready and begins on the next tick if its research facility requirements are met.

Tier-1 technologies can run at Hub Haven's built-in research console without a Research Ship. Any project assigned to a Research Station requires a docked Research Ship, and Tier-2 through Tier-4 technologies can run only at a Research Station. Each facility runs one project at a time; multiple Research Stations may run different technologies concurrently. A technology may have only one project record globally.

A Research Station project creates automatic `DockForResearch` work. During logistics assignment, stations needing a ship are considered by descending station priority, project creation sequence, then Station ID; the nearest eligible idle Research Ship is chosen, breaking ties by Ship ID. A ship en route counts as the project's assignment, but progress begins only after it docks. If no ship is available, the project remains paused with `NoResearchShip` and is reconsidered each tick.

Pausing never destroys research value. Consumed materials stay credited to that technology, and resumption continues at the prior tick. The pause action may keep unused materials reserved or release them in place to the facility's ordinary buffers so normal logistics can use them. `NoResearchShip` projects resume automatically when an eligible ship docks; manually paused projects require `QueueResearch`/Resume.

## Tier 0 — Starting Technology

| Technology | Unlocks |
|-----------|---------|
| Basic Construction | Cargo/Construction Ship T1, Hub T1, Mining T1 |
| Basic Refining | Refinery T1 and the three basic refining recipes |
| Basic Power | Power Core recipe |
| Basic Control | Control System recipe |

The deployment kit contains pre-assembled exceptions for recipes not yet unlocked.

## Tier 1 — Early Expansion

| Technology | Prerequisite | Unlocks |
|-----------|--------------|---------|
| Advanced Refining | Basic Refining | Refinery T2, Chemicals, Fuel |
| Structural Engineering | Basic Construction | Structural Frames; Cargo/Construction T2 |
| Sensor Systems | Basic Control | Research Ship/Station T1, Research Labs, Optics, survey depth 1 |
| Fusion Power | Basic Power | Reactor Rods, Hub T2 |
| Cargo Handling | Basic Construction | Cargo Modules and storage upgrades |

Sensor Systems has no Optics or Power Core cost; it is deliberately researchable from Haven's basic refined goods. It unlocks the Optics recipe immediately, while that recipe becomes runnable once Alloy Smelting provides a Tier-3 refinery; the deployment kit covers the first Research Lab pair until then.

## Tier 2 — Industrial Age

| Technology | Prerequisites | Unlocks |
|-----------|---------------|---------|
| Alloy Smelting | Advanced Refining | Alloys, Refinery T3 |
| Factory Automation | Structural Engineering + Cargo Handling | Construction Factory T1, Drive Assemblies, Construction Bays |
| Deep Survey | Sensor Systems + Alloy Smelting | Research Ship/Station T2, survey depth 2 |
| Reactor Scaling | Fusion Power + Alloy Smelting | Construction Factory T2, Grid Power prerequisite |
| Orbital Logistics | Cargo Handling + Deep Survey | Belt mining, Mining T2 |
| Life Support | Advanced Refining | 20% fuel reduction on outer/fringe arcs and radial burns touching those lanes; Heavy Transport prerequisite |

Life Support is an efficiency upgrade, not a travel gate. All bodies remain reachable before it, so exploration cannot be soft-locked.

## Tier 3 — Advanced Construction

| Technology | Prerequisites | Unlocks |
|-----------|---------------|---------|
| Precision Manufacturing | Factory Automation + Alloy Smelting | Construction Factory T3, Mining T3, Engineer |
| Heavy Transport | Orbital Logistics + Life Support + Factory Automation | Cargo T3, Hub T3 |
| Exploration Suite | Deep Survey + Reactor Scaling | Research Ship/Station T3, survey depth 3 |
| Grid Power | Reactor Scaling | High-capacity grid infrastructure; Gate Construction prerequisite |
| Gate Theory | Precision Manufacturing + Exploration Suite | Gate site and theory |

## Tier 4 — Space Gate

| Technology | Prerequisites | Unlocks |
|-----------|---------------|---------|
| Advanced Fabrication | Precision Manufacturing + Heavy Transport | All T4 ships/stations, including Fabricator |
| Gate Construction | Gate Theory + Advanced Fabrication + Grid Power | Gate Node recipe |
| System Bridge | Gate Construction | Space Gate assembly and activation |

Completing System Bridge, constructing the Gate, and supplying its activation Reactor Rod completes V1.

## Availability Rules

- A technology is available only when every listed prerequisite is Complete.
- Tier labels never substitute for explicit prerequisites; there is no “complete any two Tier-1 technologies” rule.
- The full tree and all costs remain visible from the start.
- Starting a project does not require materials to exist yet; unavailable materials produce an AwaitingMaterials state with clear demand.
- Research cannot consume material from ordinary station buffers unless it is reserved for that project.
- Research progress is deterministic and save/load equivalent.

## Natural Progression

1. Refine Haven's Metal Ore, Carbon Soil, and Silicon Dust.
2. Research Sensor Systems at the Hub and build the pre-kitted Research Ship and Station.
3. Survey Pyre and Boreas; complete Advanced Refining.
4. Complete Structural Engineering and Cargo Handling to replace basic components.
5. Survey Rime; complete Fusion Power and Alloy Smelting.
6. Build a Construction Factory through Factory Automation.
7. Complete deeper surveying and industrial upgrades.
8. Reach Gate Theory, Advanced Fabrication, Gate Construction, and System Bridge.
9. Build and activate the Space Gate.

Every prerequisite path is checked for cycles and recipe reachability by the content validator.
