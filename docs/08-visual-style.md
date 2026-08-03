# Visual Style — Chosen Direction

## The Map

Top-down 2D view of the star system. The star sits at the center, rendered massive and luminous — it's the anchor of every view. Orbital lanes are drawn as concentric rings around the star (Inner, Habitable, Outer, Fringe).

Planets are circles positioned on their orbital lane. Each planet has a **glow effect** that scales with distance from the star — inner planets glow brightly, outer planets are dimmer and cooler-toned. This immediately communicates distance and orbital position.

The map is the main interface. Zooming, panning, and selecting are the core interactions.

## Zoom Levels

Three discrete zoom bands. The camera snaps between them, not smooth-scrolls:

| Zoom Band | What You See |
|-----------|-------------|
| **System Icon View** (farthest out) | Full system — star, orbital lanes, planets as textured circles, route lines, ship hulls at small scale, fog over unexplored regions. Ships are visible as recognizable hull shapes (not dots). |
| **Planet View** (medium) | Framed on a single planet — orbit rings become visible, stations appear as sprite icons on rings, ship hulls larger with visible silhouette, route endpoints visible at docks. |
| **Station View** (closest) | Framed on a single station — detailed hulls, cargo loading/unloading visible, logistics panel overlays the view. |

Transition is instant (snap). See `09-zoom-levels.md` for the full transition rules and per-band visibility tables.

## Fog & Exploration

Unexplored regions of the system are covered by **fog**. Planets and asteroid belts are visible as silhouettes, but their resource contents are hidden. To reveal them, build a **Research Ship** — it autonomously seeks out unexplored bodies and surveys them.

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

| Resource | Color |
|----------|-------|
| Metal Ore / Metals | Steel Gray |
| Carbon Soil / Carbon Fiber | Warm Brown |
| Silicon Dust / Silicon Wafers | Cool Blue |
| Volcanic Sulfur / Chemicals | Bright Yellow |
| Water Ice / Fuel | Ice Cyan |
| Helium-3 / Reactor Rods | Neon Green |
| Alloys | Purple |
| Optics / Sensors | Teal |
| Rare Earth Minerals | Magenta |
| Gate Nodes | White / Gold |

This palette is distinct enough to read at a glance even at system zoom. Route lines use these colors so you immediately see what cargo is flowing where.

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
| Zoom | Three discrete bands — System Icon, Planet View, Station View. Camera snaps between them |
