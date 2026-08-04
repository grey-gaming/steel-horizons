---
status: Approved
owner: Tech Lead
last-reviewed: 2026-08-04
---

# Zoom Levels & Camera

Three discrete zoom bands (1–3), not continuous. The camera snaps between them. Each band shows different detail and scale. Galaxy Map (V2) is outside the V1 band sequence.

---

## Band 1 — System View

**Scale:** Single star system. All planets, moons, belts visible.

**Visible elements:**

| Element | Appearance |
|---------|-----------|
| Star | Large, centered, luminous glow |
| Orbital lanes | Concentric rings (Inner, Habitable, Outer, Fringe) |
| Planets | Circles with surface texture pattern. Glow proportional to distance from star. Resource icons visible once surveyed. |
| Moons | Small circles orbiting their planet |
| Asteroid belts | Scattered dots in a region |
| Stations | **Sprite icons** — a small authored 2D image representing the station type. Each station has its own distinct sprite. |
| Ships | **Scaled hull sprites** — visible as recognizable ship shapes, not dots. Smaller at this band but still identifiable as a cargo/construction/research hull. |
| Route lines | Curved colored lines following orbital paths. Color = cargo type, thickness = throughput. Animated dash shows flow direction. |
| Fog | Covers unexplored bodies. Celestial silhouettes visible but contents hidden. |

**Interaction:**
- Click a planet → zoom into Band 2 (Planet View)
- Click a station → zoom into Band 3 (Station View)
- Click a ship → inspect its cargo and route
- Click a route line → inspect throughput
- Click empty space → pan the view within System View
- Right-click → context menu for building actions

---

## Band 2 — Planet View

Framed on a single planet. The planet fills a portion of the view. Orbit rings around it become visible.

**Visible elements (additional to Band 1):**
- **Orbit rings** — concentric circles around the planet showing station slots
- **Stations as sprite icons** — station icons visible on orbit rings (same sprites as Band 1)
- **Ship hulls larger** — the scaled hull sprite is larger now:
  - Cargo containers visible on cargo ships
  - Crane arm visible on construction ships
  - Sensor dish visible on research ships
  - Engine glow at the rear
- **Route endpoints** — route lines terminate at specific docks
- **Planet surface detail** — texture pattern more visible, resource deposit icons on the surface

**Interaction:**
- Click a station → zoom into Band 3 (Station View)
- Click a ship → inspect its cargo and route
- Click a route line → inspect throughput
- Click empty space → returns to Band 1 (System View)
- Right-click → context menu for building actions

---

## Band 3 — Station View

Framed on a single station. The station is the focal point.

**Visible elements (additional to Band 2):**
- Detailed 2D station sprite assembled from PixiJS sprite layers; V1 has no mesh or 3D rendering
- Docks with ship docking animations
- Cargo loading/unloading (items moving along conveyors or drones)
- Station status indicators (activity, throughput, storage fill)
- The logistics panel overlays the view (UI panel, not 3D)

**Interaction:**
- Click planet → returns to Band 2 (Planet View)
- Click empty space → returns to Band 1 (System View)
- Click a ship docked at the station → inspect its cargo
- Right-click → context menu for station actions

---

## Transition Rules

Between bands, the transition is instant (snap), not animated:

| Transition | Trigger |
|-----------|---------|
| System → Planet | Click a planet |
| System → Station | Click a station |
| Planet → System | Click empty space or press zoom-out |
| Station → Planet | Press zoom-out or click the framed planet |
| Station → System | Click empty space |
| Planet ↔ Station | Click station on orbit ring / click planet from station |

---

## What Changes at Each Band

| Feature | Band 1 (System View) | Band 2 (Planet View) | Band 3 (Station View) |
|---------|----------------------|----------------------|-----------------------|
| Planets | Textured circles | Textured + orbit rings | Textured + orbit rings |
| Stations | Sprite icons | Sprite icons on orbit rings | Full station visuals + docks |
| Ships | Scaled hull sprites | Larger hulls, cargo/engine visible | Detailed hulls, docking animations |
| Route lines | Colored curved lines | Endpoints at docks | Endpoints at docks |
| Fog | Covers unexplored bodies | Cleared where surveyed | Cleared where surveyed |
| Orbit rings | Not visible | Visible | Visible |
| Cargo transfers | Not visible | Not visible | Animated particles/items |
