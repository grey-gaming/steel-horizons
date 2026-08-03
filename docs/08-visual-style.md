# Visual Style — Chosen Direction

## The Map

Top-down 2D view of the star system. The star sits at the center, rendered massive and luminous — it's the anchor of every view. Orbital lanes are drawn as concentric rings around the star (Inner, Habitable, Outer, Fringe).

Planets are circles positioned on their orbital lane. Each planet has a **glow effect** that scales with distance from the star — inner planets glow brightly, outer planets are dimmer and cooler-toned. This immediately communicates distance and orbital position.

The map is the main interface. Zooming, panning, and selecting are the core interactions.

## Zoom Levels

Continuous zoom from **system view** to **station view**:

| Zoom Band | What You See |
|-----------|-------------|
| **System view** (farthest out) | Full system — star, orbital lanes, planets as labeled circles, route lines, ship icons, fog over unexplored regions |
| **Planet view** (medium) | A single planet fills more of the view. Orbit rings become visible. Stations appear as icons on rings. Ships become small hull shapes. Route endpoints visible at docks. |
| **Station view** (closest) | A single station fills the view. Ship hulls detailed. Cargo loading/unloading visible. Logistics panel overlays. |

Between these bands, detail fades in progressively — no hard cutoffs. Route lines become thicker and gain arrow animation as you zoom in. Ship icons transition from dots to hull shapes.

## Fog & Exploration

Unexplored regions of the system are covered by **fog**. Planets and asteroid belts are visible as silhouettes, but their resource contents are hidden. To reveal them, you must send a **Research Ship** to survey.

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
| System view | Small colored dots or arrows — visible as movement on route lines |
| Planet view | Hull shapes — distinguishable by class (boxy cargo, crane construction, dish research) |
| Station view | Detailed hull — cargo containers visible, engine glow, docking animation |

Ship class is communicated by silhouette at planet zoom:

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
| Ship rendering | Dots → hulls → detailed as zoom increases |
| Cargo colors | Distinct palette, readable at system zoom |
| Zoom | Continuous, no hard bands — detail fades in progressively |
