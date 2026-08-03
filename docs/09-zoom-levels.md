# Zoom Levels & Camera

Three discrete zoom bands, not continuous. The camera snaps between them. Each band shows different detail and scale.

---

## Band 1 — Galaxy Map (V2)

**Scale:** Multiple star systems. Gates connect them.

**Visible elements:**
- Star systems as labeled circles
- Gate lines between systems (drawn like rail lines)
- Gate signal indicators (green/red dots on lines)
- System names

**Interaction:**
- Click a system to zoom into it (Band 2)
- Click a gate line to inspect gate status
- No ship or station detail — this is strategic navigation

This band is V2 content. In V1, the view starts at Band 2.

---

## Band 2 — System Icon View

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

## Band 3 — Detailed View

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
| Galaxy → System | Click a system |
| System → Planet | Click a planet |
| System → Station | Click a station |
| Planet → System | Click empty space or press zoom-out |
| Station → System | Click empty space or press zoom-out |
| Planet ↔ Station | Click station on orbit ring / click planet from station |

---

## What Changes at Each Band

| Feature | Galaxy Map | System Icon | Detailed |
|---------|-----------|-------------|----------|
| Planets | System circles | Textured circles | Textured + orbit rings |
| Stations | Not visible | Sprite icons | Full station visuals + docks |
| Ships | Not visible | Scaled hull sprites | Detailed hulls with cargo/engine |
| Route lines | Gate lines only | Colored curved lines | Endpoints at docks |
| Fog | N/A | Covers unexplored bodies | Cleared where surveyed |
| Orbit rings | Not visible | Not visible | Visible |
| Cargo transfers | Not visible | Not visible | Animated particles/items |
