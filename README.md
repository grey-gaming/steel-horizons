# Steel Horizons

**A cozy single-system space logistics game.** Build a network of autonomous ships, stations, and factories to transport materials, research technology, and construct a Space Gate to the next system.

## Status

**Design phase.** All 14 design documents are in `docs/`. No playable build exists yet. This repository contains the game design specification, not source code.

## Quick Start

Start reading in this order:

1. `docs/01-gdd.md` — Game Design Document (pitch, scope, core loop)
2. `docs/02-the-system.md` — The starting star system (planets, resources, travel)
3. `docs/03-economy.md` — Economy overview (resources, recipes, fuel, power)
4. `docs/04-tech-tree.md` — Technology tree (unlocks, costs, prerequisites)
5. `docs/05-ships-stations-factories.md` — Entity catalog (tiers, stats, specials)
6. `docs/06-onboarding.md` — First-hour player experience walkthrough
7. `docs/07-routes-and-logistics.md` — Autonomous logistics (job matching, reservations)
8. `docs/08-visual-style.md` — Visual direction (palette, icons, camera)
9. `docs/09-zoom-levels.md` — Zoom bands and camera transitions
10. `docs/10-iconography-and-textures.md` — Asset inventory and art pipeline
11. `docs/11-ui-interactions.md` — UI panels, interactions, keyboard shortcuts
12. `docs/12-simulation-foundations.md` — Tick order, rates, save/load, invariants
13. `docs/13-data-models.md` — Entity struct definitions and types
14. `docs/v2-gate-logistics.md` — Future inter-system route model (V2 concept)

## V1 Scope

Steel Horizons V1 is a **single star system** with autonomous drone logistics. The player:
- Places stations on orbit rings around planets, moons, and belts
- Builds ships at station hubs
- Chooses what to refine, build, and research
- Watches ships autonomously transport materials between stations
- Wins by constructing and activating a Space Gate

V1 is **not** a galaxy-scale game. Inter-system travel, explicit route assignment, and fleet coordination are V2 features.

## Design Principles

- **Always solvable:** Every valid player action sequence can progress. No soft-locks.
- **No failure states:** No losing, no destruction. The player can always recover by reconfiguring their network.
- **Autonomous drones:** Ships self-organize. The player designs the network, not individual routes.
- **Distributed buffers:** Materials live in station buffers, not a global pool. Logistics emerge from supply/demand broadcasts.

## Documentation Convention

- `docs/` files are numbered by reading order (01–13). `v2-gate-logistics.md` is a future concept file outside the V1 sequence.
- **Decision** labels mark authoritative rules. **Example** labels mark illustrative values that may change during balance.
- The canonical source for entity definitions is `docs/13-data-models.md`. Prose in other files should not contradict the data model.
- Technology unlocks, recipe requirements, and entity stats have single owners — see `docs/04-tech-tree.md` for the unlock graph.

## Glossary

| Term | Definition |
|------|-----------|
| **Orbit ring** | A circular orbital lane around a celestial body. Stations float on orbit rings. |
| **Slot** | A placeable position on an orbit ring. Each ring has a fixed number of slots. |
| **Dock** | A station's ship-berth capacity. Ships must dock to load/unload/research/refuel. |
| **Buffer** | Per-resource storage at a station. Each resource type has its own buffer cap. |
| **Supply broadcast** | A signal emitted when a buffer rises above its export threshold — "I have material X to give away." |
| **Demand broadcast** | A signal emitted when a buffer drops below its demand threshold — "I need material X delivered." |
| **Job** | A one-shot autonomous task a ship picks up: transport cargo X from A to B, build a station, survey a body. |
| **Tier** | Progression level (1–4 for most entities). Higher tiers unlock better rates, capacities, and recipes. |
| **Tier 0** | Starting technologies and recipes available without research. |
| **Lane** | An orbital path shared by all bodies at a given distance from the star. Ships travel along lanes. |
| **Survey depth** | How thoroughly a body has been scanned: 0=none (fogged), 1=surface (visible deposits), 2=subsurface (hidden deposits revealed), 3=full (all resources). |
| **Belt** | Asteroid belt resource deposit that never depletes but drifts slowly over time. |
| **Space Gate** | The V1 victory structure — built from Gate Nodes, powered by Reactor Rods, enabling inter-system travel. |
| **Hub** | The Station Hub — player's main base. Can assemble Tier 1 components and has built-in research capability. |
| **Construction Ship** | Ship type that places stations, builds upgrades, and constructs the Gate. |
| **Research Ship** | Ship type that surveys bodies and powers research when docked at a Research Station. |
| **Cargo Ship** | Ship type that transports materials between station buffers autonomously. |

## Product Requirements (V1)

- **Platform:** macOS (primary), Windows (secondary). Linux is not a V1 target.
- **Renderer:** PixiJS v8 (WebGL/Canvas). No 3D or mesh rendering.
- **Resolution:** Minimum 1280×720, designed for 1920×1080. Aspect ratio 16:9.
- **Offline:** Fully offline single-player. No server or internet required.
- **Distribution:** Steam (planned), itch.io (backup).
- **Session length:** 30–90 minutes per session. Onboarding ~5 hours total.

## Deferred Documentation

These documents are recommended by the design review (M-03, M-04) but deferred until implementation begins:

- **Technical architecture** — module boundaries, content-loading approach, event/command model, rendering vs simulation ownership, dependency direction, error strategy, and deterministic test harness.
- **Test and balance strategy** — unit/property tests, golden simulation scenarios, progression reachability, resource conservation, save/load equivalence, deterministic replay, performance benchmarks, UI accessibility checks, and playtest metrics.
- **Responsive layout wireframe** — full-screen HUD layout, panel stacking, z-order, focus restoration, overflow handling.
- **Audio/settings/localization** — sound cues, volume controls, graphics/accessibility settings, key rebinding, localization-safe strings.

## Open Decisions

These design questions are unresolved and need Product Owner input before implementation proceeds:

1. **Authored vs procedural starting system** — V1 currently assumes an authored system, but procedural generation ranges are documented. Decision: authored for V1, procedural for V2.
2. **Special mechanics** — Several ship/station specials (armor, perishable goods, research bonuses, multi-build, automated cargo routing, fleet coordination, quality) have no defined mechanics. Decision: defer to post-V1; implement core hull stats only.
3. **Save slot count** — V1 documents one autosave slot. Multiple slots or cloud saves are deferred.
4. **Touch/controller support** — Mouse-only for V1. Touch and controller are V2.
5. **Audio/Settings/Localization** — All deferred to post-V1. V1 scope is visual simulation only.
6. **License** — Pending selection (MIT or custom). Repository is private during design phase.

## Repository Governance

- **Status:** Design documents only. No source code yet.
- **License:** [License TBD — see Open Decisions]
- **Contributions:** [Not open for contributions at this phase.]
