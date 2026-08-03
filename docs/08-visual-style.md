---
status: Draft
owner: Tech Lead
last-reviewed: 2026-08-03
---

# Visual Style & Direction

## The Map

Top-down 2D view of the star system. The star sits at the center, rendered massive and luminous — it's the anchor of every view. Orbital lanes are drawn as concentric rings around the star (Inner, Habitable, Outer, Fringe).

Planets are circles positioned on their orbital lane. Each planet has a **glow effect** that scales with distance from the star — inner planets glow brightly, outer planets are dimmer and cooler-toned. This immediately communicates distance and orbital position.

The map is the main interface. Zooming, panning, and selecting are the core interactions.

## Zoom Levels

Three discrete zoom bands (1–3). The camera snaps between them, not smooth-scrolls:

| Zoom Band | What You See |
|-----------|-------------|
| **Band 1 — System View** (farthest out) | Full system — star, orbital lanes, planets as textured circles, route lines, ship hulls at small scale, fog over unexplored regions. Ships are visible as recognizable hull shapes (not dots). |
| **Band 2 — Planet View** (medium) | Framed on a single planet — orbit rings become visible, stations appear as sprite icons on rings, ship hulls larger with visible silhouette, route endpoints visible at docks. |
| **Band 3 — Station View** (closest) | Framed on a single station — detailed hulls, cargo loading/unloading visible, logistics panel overlays the view. |

Transition is instant (snap). See [09-zoom-levels.md](./09-zoom-levels.md) for the full transition rules and per-band visibility tables.

## Fog & Exploration

Unexplored regions of the system are covered by **fog**. Planets and asteroid belts are visible as silhouettes, but their resource contents are hidden. To reveal them, build a **Research Ship** — the player clicks a body and selects 'Survey', then the ship autonomously flies there and scans. Once dispatched, the ship travels, enters a scanning orbit, and reveals resources after scan time completes.

- Surveyed planets show their resource profile (icons on the planet circle)
- Surveyed belts show deposit density (scattered dots in the belt region)
- Fog recedes around surveyed areas permanently
- Higher-tier research ships survey faster and reveal more detail (surface → subsurface → hidden deposits)

This creates a natural early loop: build a Research Ship, survey the system, then decide where to place mining stations.

## Route Lines

Routes are drawn as **curved lines** that follow orbital paths between stations. Each route line has:

- **Color** — determined by cargo type (see palette below)
- **Thickness** — scales with throughput (how much material is flowing)
- **Animation** — a moving dash pattern showing flow direction

At system view, routes are thin colored lines. At planet view, they thicken and show animated flow. A route density overlay (toggleable) shows a heatmap of all active flows.

## Cargo Color Palette

| Resource | Color | Icon Pattern | Label (inspection panel) |
|----------|-------|-------------|---------------------------|
| Metal Ore | Steel Gray | Coarse triangle | "Metal Ore" |
| Metals | Steel Gray | Smooth ingot | "Metals" |
| Carbon Soil | Warm Brown | Leaf silhouette | "Carbon Soil" |
| Carbon Fiber | Warm Brown | Fiber strand | "Carbon Fiber" |
| Silicon Dust | Cool Blue | Scattered dots | "Silicon Dust" |
| Silicon Wafers | Cool Blue | Solid circle | "Silicon Wafers" |
| Volcanic Sulfur | Bright Yellow | Rising gas plume | "Volcanic Sulfur" |
| Chemicals | Bright Yellow | Vial bottle | "Chemicals" |
| Water Ice | Ice Cyan | Hexagonal snowflake | "Water Ice" |
| Fuel | Ice Cyan | Flame shape | "Fuel" |
| Helium-3 | Neon Green | Three-ring atom | "Helium-3" |
| Reactor Rods | Neon Green | Cylinder bars | "Reactor Rods" |
| Alloys | Purple | Gear cog | "Alloys" |
| Optics | Teal | Concentric lens | "Optics" |
| Rare Earth Minerals | Magenta | Jagged crystal | "Rare Earth Minerals" |
| Crystal Deposits | Pink | Faceted diamond | "Crystal Deposits" |
| Gate Nodes | White / Gold | Six-sided hexagon | "Gate Node" |

Colors are shared between raw/refined pairs (e.g., Metal Ore and Metals both use Steel Gray) because they are visually adjacent in the production chain. Each resource is uniquely identified by a combination of color + icon pattern, and the inspection panel shows the full text label. Route lines use color + animated dash pattern for identification. Non-color cues (icon shape, line dash style, text labels) ensure accessibility for color-blind players.

## Planets

Planets are rendered as circles with:

- **Radius** proportional to planet size
- **Surface texture** — subtle patterns (rocky, icy, volcanic, gaseous) visible at planet zoom
- **Glow** — scales with distance from star (inner = bright warm, outer = dim cool)
- **Resource icons** — shown on the planet circle once surveyed (small colored dots/symbols)
- **Orbit rings** — concentric circles around the planet where stations are placed

## Stations & Orbit Rings

Stations do not sit on planet surfaces. They float on **orbit rings** — concentric circles around a planet. Each orbit ring has station slots. Ships fly between orbit rings and between planets.

Station types are distinguished by shape and icon:

| Station | Shape | Icon |
|---------|-------|------|
| Station Hub | Hexagon | House icon |
| Mining Station | Diamond | Pick icon |
| Refinery Factory | Square with pipes | Chemical flask |
| Construction Factory | Gear shape | Gear icon |
| Research Station | Triangle with dish | Telescope icon |

At system view, stations are small shapes. At station view, they expand to show docks, storage indicators, and active cargo transfers.

## Ships

| Zoom Level | Ship Appearance |
|------------|----------------|
| System view | Scaled hull sprites — recognizable by silhouette (boxy cargo, crane construction, dish research), ~16-24px size |
| Planet view | Hull shapes larger — cargo containers visible, crane arm visible, sensor dish visible |
| Station view | Detailed hull — cargo containers visible, engine glow, docking animation |

Ship class is communicated by silhouette at all zoom levels:

- **Cargo**: rectangular hull with cargo containers
- **Construction**: boxy with crane/arm appendages
- **Research**: streamlined with sensor dish

## Fog & Darkness

The system has a **deep space background** — dark but not black, with subtle nebula coloration. Unexplored areas have a denser fog layer that hides resource data but not celestial bodies themselves (you can see a planet is there, but not what it contains). Surveyed areas are clear and bright.

Outer regions of the system are inherently darker (less star light), making the fringe lane feel distant and mysterious — appropriate for the Space Gate location.

## Summary of Visual Decisions

| Element | Decision |
|---------|----------|
| Perspective | Top-down 2D |
| Star | Center, massive, luminous |
| Orbital lanes | Concentric rings |
| Routes | Curved, color-coded by cargo, thickness = throughput, animated flow |
| Planet glow | Proportional to distance from star |
| Fog | Covers unexplored areas — scanning reveals resources |
| Station placement | Orbit rings around planets |
| Ship rendering | Scaled hull sprites at all zoom levels — smaller at system view, larger at detailed view |
| Cargo colors | Distinct palette, readable at system zoom |
| Zoom | Three discrete bands — Band 1 (System View), Band 2 (Planet View), Band 3 (Station View). Camera snaps between them |
