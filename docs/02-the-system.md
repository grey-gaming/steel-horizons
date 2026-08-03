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
| Fringe | Edge of system | Very slow | Asteroid belts, derelicts |

Ships travel along orbital lanes. Moving between lanes requires a "burn" — slower but needed to reach different bodies. A ship's class determines how fast it can move in each lane.

## Planets

Each planet has:
- **Surface conditions** (rocky, icy, volcanic, gas, etc.)
- **Resource profile** (which raw resources are present)
- **Gravity** (affects how easy it is to land and launch)
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
- Faster to land/launch from (low gravity)

Moons are good for specialized extraction outposts.

## Asteroid Belts

Belt regions contain many small objects. Instead of landing, you build **orbital mining stations** that drift with the belt. Belts provide:
- Bulk common resources (Metals, Carbon, Silicon)
- Occasional rare finds
- No gravity concerns — easy to set up
- **Resource shifting** — every 1,000 ticks (~17 min real-time), each belt resource deposit fluctuates by ±10% of its base amount (random drift). This means a belt that initially has 1,000 units of Metal Ore might have 1,050 after one shift and 980 after the next. The drift is slow and small — it never depletes a resource entirely, but over long play sessions the belt's resource mix changes subtly. Mining stations in the belt automatically adapt: their output rate adjusts proportionally to the remaining deposit fraction.

> **Design note:** Belt shifting is a minor background mechanic. It prevents belts from feeling static and gives a slight advantage to players who monitor and adapt. Most players won't notice day-to-day changes, but over a 10-hour session the shift totals ~±30% cumulative drift. No player action is required to benefit — the mining stations handle it automatically.

## Travel & Logistics (V1 — Autonomous Drone Model)

Ships are autonomous drones. The player does not assign routes — ships read the network state and self-organize. Each station has an input buffer (what it needs) and an output buffer (what it produces). Ships fly from stations with surplus output to stations with demand for that material.

Travel time depends on:
- Distance between bodies (orbital lane positions)
- Lane speed of the ship class
- Cargo weight (heavier loads are slower)

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
- 1 gas giant with 2 moons in the outer lane
- 1 asteroid belt in the fringe

**Total system station capacity** (V1 starting system):

| Body | Type | Total Slots |
|------|------|------------|
| Rocky Terran (starting) | Planet | 6–8 |
| Volcanic Planet | Planet | 2–4 |
| Ice World | Planet | 2–4 |
| Gas Giant | Planet | 0 (moons only) |
| Moon A (gas giant) | Moon | 1–2 |
| Moon B (gas giant) | Moon | 1–2 |
| Asteroid Belt | Belt | 4–6 |

**Estimated total: ~16–26 station slots across the entire system.** This gives the player a clear upper bound for capacity planning. Slot counts are per-planet — once a planet's slots are full, new stations must be placed on another body.

This gives a variety of resources and travel distances to learn logistics with.
