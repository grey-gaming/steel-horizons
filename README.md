# Steel Horizons

**A cozy single-system space logistics game.** Build autonomous ships, stations, and factories; research technology; and construct a Space Gate.

## Status

**V1 design approved; Phase 1 implementation ready.** The repository currently contains specifications rather than source code. Phase 1 is the Rust deterministic simulation engine, authenticated local HTTP/WS API, Python text UI, and agent play-tests. Phase 2 is the PixiJS v8/Tauri graphical client.

## Reading Order

### Game Design

1. [`docs/gdd/01-gdd.md`](docs/gdd/01-gdd.md) — pitch, scope, loop, difficulty
2. [`docs/gdd/02-the-system.md`](docs/gdd/02-the-system.md) — geography, deposits, travel
3. [`docs/gdd/03-economy.md`](docs/gdd/03-economy.md) — economy and recovery rules
4. [`docs/gdd/04-tech-tree.md`](docs/gdd/04-tech-tree.md) — unlock graph and research
5. [`docs/gdd/05-ships-stations-factories.md`](docs/gdd/05-ships-stations-factories.md) — entity behavior
6. [`docs/gdd/06-onboarding.md`](docs/gdd/06-onboarding.md) — validated first-five-hour sequence
7. [`docs/gdd/07-routes-and-logistics.md`](docs/gdd/07-routes-and-logistics.md) — autonomous matching and reservations
8. [`docs/gdd/08-visual-style.md`](docs/gdd/08-visual-style.md) — visual direction
9. [`docs/gdd/09-zoom-levels.md`](docs/gdd/09-zoom-levels.md) — camera bands
10. [`docs/gdd/10-iconography-and-textures.md`](docs/gdd/10-iconography-and-textures.md) — asset system
11. [`docs/gdd/11-ui-interactions.md`](docs/gdd/11-ui-interactions.md) — interactions and recovery UI
12. [`docs/gdd/12-simulation-foundations.md`](docs/gdd/12-simulation-foundations.md) — authoritative tick/math/travel rules
13. [`docs/gdd/13-data-models.md`](docs/gdd/13-data-models.md) — authoritative state schema
14. [`docs/gdd/14-authored-content.md`](docs/gdd/14-authored-content.md) — authoritative exact content/balance

[`docs/gdd/v2-gate-logistics.md`](docs/gdd/v2-gate-logistics.md) is a non-implementation V2 concept.

### Architecture and Implementation

- [`docs/adr/`](docs/adr/) — accepted architectural decisions
- [`docs/tdd/`](docs/tdd/) — approved Phase 1 implementation design
- [`docs/IMPLEMENTATION_PLAN.md`](docs/IMPLEMENTATION_PLAN.md) — ordered autonomous implementation backlog and cumulative verification gates
- [`AGENTS.md`](AGENTS.md) — repository-wide per-turn instructions for implementation agents

## V1 Scope

V1 is one authored star system. The player:

- Places stations in fixed orbit-ring slots
- Configures mining, refining, construction, and research
- Builds ships at Station Hubs
- Influences autonomous logistics with placement, thresholds, capacity, and priority
- Surveys bodies to progressive depths
- Wins by building and activating the Space Gate

Inter-system travel, recurring manual routes, combat, destruction, perishable cargo, quality systems, multi-build ships, and fleet coordination are post-V1.

## Design Invariants

- **Always solvable:** the validated starting state and every reversible recovery action retain a path to victory.
- **No accidental material loss:** cancelling, demolition, and scrapping return complete invested components; salvage never decays.
- **Persistent research value:** pausing research retains progress and consumed-resource credit.
- **Autonomous drones:** ships select deterministic one-shot jobs; players design the network.
- **Distributed storage:** no global material pool.
- **Deterministic simulation:** integer-only state, stable command order, exact save/load/replay equivalence.

## Authority and Change Rules

When documents overlap, authority is:

1. Accepted ADRs for architectural decisions
2. GDD 12 for simulation semantics and formulas
3. GDD 13 for serialized state shapes
4. GDD 14 for exact content, costs, statistics, and starting values
5. Other approved GDDs for player-facing behavior and presentation
6. TDDs for implementation structure, which must implement the above contracts

Changing an authoritative rule requires updating every dependent summary and its executable validation/test in the same change. “Example” values are non-authoritative only when explicitly labelled; GDD 14 contains no example balance values.

## Product Requirements

- **Platforms:** macOS primary, Windows secondary
- **Renderer:** PixiJS v8, packaged with Tauri in Phase 2; 2D sprites only
- **Resolution:** minimum 1280×720, designed for 1920×1080 at 16:9
- **Offline:** fully offline single-player; the engine's authenticated server is loopback-only
- **Distribution:** Steam planned, itch.io backup
- **Session length:** 30–90 minutes; onboarding approximately five hours across sessions
- **Input:** mouse and keyboard in V1; touch/controller post-V1
- **Save:** one local autosave slot in V1

## Minimal V1 Settings and Accessibility

V1 includes reduced motion, UI scale, color-independent resource identification, keyboard navigation, and documented shortcuts. Audio, full localization, controller input, cloud saves, and advanced graphics settings are post-V1.

## Implementation Entry Criteria

The approved documents require CI gates for content validation, bootstrap reachability, all-tech reachability, full Gate victory, recovery, conservation, deterministic replay, save/load equivalence, API transport conformance, and supported-platform state hashes. Implementation is ready to begin with those tests driving behavior.

## Repository Governance

- **License:** pending before public distribution; private internal development may proceed. See [`LICENSE`](LICENSE).
- **Contributions:** not open during initial implementation.
