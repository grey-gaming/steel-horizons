# Zoom Levels & Camera

Three discrete zoom bands, not continuous. The camera snaps between them. Each band shows different detail and scale.

---

<!-- V2 Galaxy Map moved to v2-gate-logistics.md -->

Three discrete zoom bands, numbered 1–3. Band 2 was previously reserved for the Galaxy Map (V2); in V1 it is unused and the camera snaps directly from Band 1 (System) to Band 3 (Detailed).

## Band 1 — System Icon View

**Scale:** Single star system. All planets, moons, belts visible.

**Visible elements:**

| Element | Appearance |
|---------|-----------|
| Star | Large, centered, luminous glow |
| Orbital lanes | Concentric rings (Inner, Habitable, Outer, Fringe) |
| Planets | Circles with surface texture pattern. Glow proportional to distance from star. Resource icons visible once surveyed. |
| Moons | Small circles orbiting their planet |
| Asteroid belts | Scattered dots in a region |
| Stations | **Sprite icons** — a small hand-painted image representing the station type. Each station has its own distinct sprite. |
| Ships | **Scaled hull sprites** — visible as recognizable ship shapes, not dots. Smaller at this band but still identifiable as a cargo/construction/research hull. |
| Route lines | Curved colored lines following orbital paths. Color = cargo type, thickness = throughput. Animated dash shows flow direction. |
| Fog | Covers unexplored bodies. Celestial silhouettes visible but contents hidden. |

**Interaction:**
- Click a planet → zoom into it (Band 3, planet view)
- Click a station → zoom into it (Band 3, station view)
- Click a ship → inspect its cargo and route
- Click a route line → inspect throughput
- Click empty space → pan the view
- Right-click → context menu for building actions

---

## Band 3 — Detailed View (Planet & Station)

Two sub-modes: **Planet View** and **Station View**. Both are at the same zoom level but framed differently.

### Planet View

Framed on a single planet. The planet fills a portion of the view. Orbit rings around it become visible.

**New visible elements:**
- **Orbit rings** — concentric circles around the planet showing station slots
- **Stations as full visuals** — the station sprite expands into a detailed structure with:
  - Docks (visible as docking ports with ships attached)
  - Storage indicators (cargo containers or bars showing fill level)
  - Active cargo transfers (small animated particles or icons moving between dock and station)
- **Ship hulls detailed** — the scaled hull sprite is larger now:
  - Cargo containers visible on cargo ships
  - Crane arm visible on construction ships
  - Sensor dish visible on research ships
  - Engine glow at the rear
- **Route endpoints** — route lines terminate at specific docks
- **Planet surface detail** — texture pattern more visible, resource deposit icons on the surface

### Station View

Framed on a single station. The station is the focal point.

**New visible elements:**
- Station model in full detail (could be a mesh/model or a detailed sprite — TBD during implementation)
- Docks with ship docking animations
- Cargo loading/unloading (items moving along conveyors or drones)
- Station status indicators (power, throughput, storage fill)
- The logistics panel overlays the view (UI panel, not 3D)

---

## Transition Rules

Between bands, the transition is instant (snap), not animated:

| Transition | Trigger |
|-----------|---------|
| System → Planet | Click a planet |
| System → Station | Click a station |
| Planet → System | Click empty space or press zoom-out |
| Station → System | Click empty space or press zoom-out |
| Planet ↔ Station | Click station on orbit ring / click planet from station |

---

## What Changes at Each Band

| Feature | Band 1 (System View) | Band 2 (Detailed View) |
|---------|----------------------|------------------------|
| Planets | Textured circles | Textured + orbit rings |
| Stations | Sprite icons | Full station visuals + docks |
| Ships | Scaled hull sprites | Detailed hulls with cargo/engine |
| Route lines | Colored curved lines | Endpoints at docks |
| Fog | Covers unexplored bodies | Cleared where surveyed |
| Orbit rings | Not visible | Visible |
| Cargo transfers | Not visible | Animated particles/items |
