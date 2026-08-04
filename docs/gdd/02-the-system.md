---
status: Approved
owner: Product Owner
last-reviewed: 2026-08-04
---

# The System — Worlds & Geography

## The Star

The system is centered on a single stable star. All planets orbit at various distances. The star itself is not directly interacted with — it provides light and defines the orbital lanes that ships navigate.

## Orbital Lanes

Space is divided into **orbital lanes** — concentric rings around the star. Each lane has different travel characteristics:

| Lane | Position | Travel Speed | What lives here |
|------|----------|-------------|-----------------|
| Inner | Closest to star | Fast (short orbits) | Small rocky planets |
| Habitable | Middle zone | Moderate | Larger planets with moons |
| Outer | Far from star | Slow (long orbits) | Gas giants, many moons |
| Fringe | Edge of system | Very slow | Fixed Space Gate site |

Ships travel with a deterministic two-segment plan: a half-speed radial burn to the destination radius, followed by the shortest arc in the destination lane. The destination lane's rational multiplier controls arc speed. Bodies are fixed gameplay nodes in V1; orbital motion is cosmetic and cannot change logistics outcomes. GDD 12 owns the exact formula.

## Planets

Each planet has:
- **Surface conditions** (rocky, icy, volcanic, gas, etc.)
- **Resource profile** (which raw resources are present)
- **Gravity/atmosphere flavor** (presentation only in V1; it does not alter travel or throughput)
- **Station slots** — total slots across all orbit rings (see Orbit Rings below)

### Example Planet Types

| Planet Type | Description | Resources | Total Slots (all rings) | Orbit Rings |
|-------------|-------------|-----------|------------------------|-------------|
| Rocky Terran | Earth-like, stable surface | Minerals, Metals, Carbon | 6-8 | 2-3 rings, 2-4 slots each |
| Volcanic | High heat, unstable | Rare Metals, Sulfur, Energy minerals | 2-4 | 1-2 rings, 1-3 slots each |
| Ice World | Frozen surface | Water, Frozen Gases, Carbon | 2-4 | 1-2 rings, 1-3 slots each |
| Gas Giant | No surface, atmospheric mining | Hydrogen, Helium, Noble Gases | 0 (use moons) | Moons only |
| Asteroid Cluster | Not a planet — many small objects | All basic resources scattered | 4-6 (orbital stations only) | 2 rings, 2-3 slots each |

## Moons

Moons orbit planets. They are smaller than planets and offer:
- Smaller resource deposits (often 1-2 resource types)
- 1-2 station slots at most (single orbit ring)
- Distinct low-gravity visual flavor; V1 has no landing/launch modifier

Moons are good for specialized extraction outposts.

## Asteroid Belts

Belt regions contain many small objects. The ordinary `Mining` StationType is rendered as an orbital collector when placed there; there is no separate Orbital Mining Station entity. Belts provide:
- Bulk raw resources (Metal Ore, Carbon Soil, Silicon Dust, Water Ice, Frozen Gases, and Volcanic Sulfur)
- A renewable Crystal Deposits field revealed at survey depth 2
- Orbital-collector presentation; setup is gated only by survey depth, a free slot, and Orbital Logistics
- **Resource shifting** — every 1,000 ticks, each renewable belt density changes by a deterministic seeded amount within ±10% of baseline and is clamped to 70–130%. Mining output scales by current density divided by baseline.

**Deposit semantics:** Planet and moon deposits are finite and decrement by whole extracted units. Belt deposits are renewable density fields and never decrement. Each deposit also has a minimum survey depth. The authored system budgets finite critical resources for every technology plus the Gate, while reversible builds return their components; common construction and fuel inputs remain renewable.

> **Design note:** Belt shifting is a minor background mechanic. The clamp prevents unbounded random walk, and mining stations adapt automatically. The project-owned PRNG makes shifts replayable across save/load and platforms.

## Travel & Logistics (V1 — Autonomous Drone Model)

Ships are autonomous drones. The player does not assign routes — ships read the network state and self-organize. Each station has an input buffer (what it needs) and an output buffer (what it produces). Ships fly from stations with surplus output to stations with demand for that material.

Travel time depends on radial distance, destination-lane arc distance, ship speed, lane multiplier, and cargo load. Fuel feasibility additionally includes the ship's position-to-supplier leg and base mass.

The player sees **flow lines** on the map — animated colored lines that appear automatically between supply and demand stations. Ship icons move along these flows. The player influences logistics by:
- Building more ships (more throughput)
- Upgrading ships (faster, larger capacity)
- Setting station priorities (which stations get served first)
- Placing stations strategically (shorter trips = more throughput)

> Explicit route assignment (Origin → Destination → Cargo → Frequency) is **V2 Gate Logistics** for inter-system travel. V1 uses the autonomous drone model only.

## Key Map Locations
The starting system map should have:

- 1 rocky terran planet in the habitable lane (player start)
- 1 volcanic planet in the inner lane
- 1 ice world in the outer lane
- 1 gas giant with moons in the outer lane
- 1 asteroid belt between inner and habitable lanes

Planets have a number of orbit rings and slots per ring, determining how many stations can be placed around each body.

**Note:** The ranges below are for procedural generation (future). For V1, the starting system is **authored** with exact body IDs, radii/angles, slots, deposits, survey depth, names, and starting inventory. See [14-authored-content.md](./14-authored-content.md) for the canonical content catalog.

| Body | Type | Total Slots |
|------|------|------------|
| Rocky Terran (starting) | Planet | 6–8 |
| Volcanic Planet | Planet | 2–4 |
| Ice World | Planet | 2–4 |
| Gas Giant | Planet | 0 (moons only) |
| Moon A (gas giant) | Moon | 1–2 |
| Moon B (gas giant) | Moon | 1–2 |
| Asteroid Belt | Belt | 4–6 |

The authored V1 system has exactly **19 station slots**. Slot counts are per body—once its slots are full, new stations must be placed elsewhere or an existing station must be upgraded/demolished.

This gives a variety of resources and travel distances to learn logistics with.
