---
status: Draft
owner: Tech Lead
last-reviewed: 2026-08-03
---

# Iconography & Textures

## Texture Generation Pipeline

Planet textures are **hand-painted** using Draw Things CLI (local Stable Diffusion) and composited with ImageMagick. The pipeline:

1. **Generate base textures** with Draw Things — prompt for each planet type (rocky surface, volcanic, icy, gas bands, cratered moon)
2. **Tile and scale** with ImageMagick — ensure textures tile seamlessly for planet surfaces
3. **Composite** — layer resource deposit indicators, crater overlays, glow effects
4. **Export** as sprite sheets and individual textures

This approach lets us generate many unique planet textures cheaply while keeping a hand-painted look.

### Planet Texture Types Needed

| Type | Description | Generation Approach |
|------|-------------|---------------------|
| Rocky Terran | Earth-like, brown/green/grey surface with terrain variation | Draw Things: "rocky desert planet surface texture tileable" |
| Volcanic | Red/orange/black, lava veins, rough | Draw Things: "volcanic planet surface lava cracks tileable" |
| Ice World | White/blue/grey, smooth with crevasses | Draw Things: "ice planet surface frozen cracks tileable" |
| Gas Giant | Horizontal banded atmosphere, multiple colors | Draw Things: "gas giant atmospheric bands tileable" |
| Moon | Grey, heavily cratered | Draw Things: "cratered moon surface texture tileable" |
| Asteroid | Irregular rocky fragments | Draw Things: "asteroid rock texture tileable" |

Each texture is generated at a base resolution, then ImageMagick scales it to the required game sizes and creates a sprite sheet.

---

## Station Icons & Visuals

### System Icon View — Sprite Icons

Each station type has a **hand-painted sprite icon** that represents it at the System Icon zoom band. These are small, distinct, and readable at a glance across 500+ stations.

| Station | Sprite Concept |
|---------|----------------|
| Station Hub | Hexagonal structure with antenna. Central command look. |
| Mining Station | Diamond-shaped with drill/extractor arms. Industrial feel. |
| Refinery Factory | Square with pipe/tank elements. Chemical/processing look. |
| Construction Factory | Gear-shaped with assembly line elements. Industrial workshop. |
| Research Station | Triangle with dish/sensor array. Scientific/lab look. |
| Orbital Mining Station | Ring-shaped with collector arms. Different from surface mining. |

### Detailed View — Full Station Visual

When zoomed to Detailed View, the sprite expands into a **full station model** (could be a mesh or a detailed sprite — implementation choice). The model shows:

- **Docks** — visible docking ports where ships attach
- **Storage** — cargo containers or tanks showing fill level
- **Activity** — animated elements (spinning fans, moving conveyor arms, blinking lights)
- **Ships docked** — hulls attached at docks

The transition from sprite to full station happens at the zoom snap. The sprite is a simplified representation; the full model adds detail but matches the same silhouette and color so the player recognizes it.

---

## Ship Hull Sprites

Ships use **scaled versions of the same hull sprite** at all zoom levels. No dot-to-hull transition. At System Icon View, ships are smaller but still recognizable as ship shapes.

### Hull Designs

| Ship Type | Silhouette | Key Visual Features |
|-----------|-----------|---------------------|
| Cargo Ship | Rectangular hull, rounded front | Cargo containers on top/sides. Color band or stripe = cargo type when loaded. |
| Construction Ship | Boxier hull, flat front | Crane arm or construction boom visible on top. Landing gear or clamp details. |
| Research Ship | Sleek hull, pointed front | Sensor dish or antenna array. Smooth, scientific look. |

### Tier Differentiation

Higher-tier ships are visually:
- **Larger** (proportional to capacity)
- **More detailed** (additional panels, engines, equipment)
- **Color accent changes** (Tier 1 = basic gray, Tier 2 = blue accent, Tier 3 = gold accent, Tier 4 = white accent)

### Scaling

All zoom levels use the same sprite asset, just rendered at different sizes:
- System Icon: ~16-24px hull size
- Detailed View: ~64-96px hull size

The sprite must be painted at a high enough resolution (256px or higher) so it doesn't look blurry when scaled up at Detailed View.

---

## Cargo Iconography (Surviving Mars Inspired)

### Design Language

Cargo icons are **simple geometric shapes with minimal interior detail**. They use a consistent visual language:

- **Materials** (raw/refined resources): Outer circle + interior shape or line
- **Machines** (ships, stations): Outer square/hex + interior structure silhouette
- **Components** (intermediate goods): Outer diamond + interior compound shape

All icons are flat colored fills, no gradients. Color follows the cargo palette. At small sizes (system view), the icon is just a colored dot with a faint shape outline. At larger sizes (panels), the full icon is visible.

### Material Icons

| Resource | Icon Shape | Color |
|----------|-----------|-------|
| Metal Ore / Metals | Circle with horizontal line | Steel Gray |
| Carbon Soil / Carbon Fiber | Circle with cross-hatch | Warm Brown |
| Silicon Dust / Silicon Wafers | Circle with diamond center | Cool Blue |
| Volcanic Sulfur / Chemicals | Circle with flame shape | Bright Yellow |
| Water Ice / Fuel | Circle with droplet | Ice Cyan |
| Helium-3 / Reactor Rods | Circle with atom/nucleus | Neon Green |
| Alloys | Circle with interlocking rings | Purple |
| Optics / Sensors | Circle with eye/dot | Teal |
| Rare Earth Minerals | Circle with sparkle | Magenta |
| Gate Nodes | Circle with ring/gate symbol | White/Gold |

### Machine Icons

| Machine | Icon Shape | Color |
|---------|-----------|-------|
| Cargo Ship | Square with hull silhouette | Blue-gray |
| Construction Ship | Square with crane silhouette | Orange |
| Research Ship | Square with dish silhouette | Teal |
| Station Hub | Hexagon with antenna | White |
| Mining Station | Hexagon with drill | Yellow |
| Refinery Factory | Hexagon with pipes | Green |
| Construction Factory | Hexagon with gear | Orange |
| Research Station | Hexagon with dish | Teal |
| Space Gate | Large hexagon with ring symbol | Gold |

### Component Icons

| Component | Icon Shape | Color |
|-----------|-----------|-------|
| Structural Frame | Diamond with grid lines | Steel Gray |
| Power Core | Diamond with lightning bolt | Neon Green |
| Control System | Diamond with circuit lines | Teal |
| Drive Assembly | Diamond with thrust lines | Orange |
| Cargo Module | Diamond with container shape | Blue-gray |
| Research Lab | Diamond with beaker | Teal |
| Construction Bay | Diamond with gear | Orange |
| Gate Node | Diamond with gate symbol | Gold |

---

## Icon Sizes & Usage

| Context | Icon Size | Detail Level |
|---------|----------|-------------|
| Route line (system view) | 4px dot | Color dot only |
| Station sprite (system view) | 24-32px | Full sprite icon |
| Ship hull (system view) | 16-24px | Scaled hull sprite |
| Planet resource overlay | 12px | Material icon (circle shape) |
| Station logistics panel | 32px | Full cargo icon |
| Build menu / tech tree | 64px | Full icon with name |
| Tooltip / inspection | 48px | Full icon with label |

---

## Texture & Icon Asset List

| Asset | Quantity | Format | Resolution |
|-------|----------|--------|-----------|
| Planet textures | 6 (one per type) | Tileable PNG | 512x512 base |
| Station sprites | 6 (one per type) | PNG with alpha | 128x128 |
| Station detailed visuals | 6 | PNG/mesh | TBD |
| Ship hull sprites | 12 (3 roles × 4 tiers) | PNG with alpha | 256x256 |
| Cargo icons (materials) | 10 | PNG with alpha | 64x64 |
| Cargo icons (machines) | 9 | PNG with alpha | 64x64 | (3 ships + 5 station types + Space Gate; Orbital Mining Station shares Mining Station icon)
| Cargo icons (components) | 8 | PNG with alpha | 64x64 |
| Route line colors | 10 | Defined in code | N/A |
| Fog overlay | 1 | PNG | 512x512 tileable |
| UI panel backgrounds | ~5 | PNG | TBD |

Total: ~50 individual assets for V1. Manageable with Draw Things + ImageMagick pipeline.
