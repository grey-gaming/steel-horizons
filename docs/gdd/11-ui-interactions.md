---
status: Approved
owner: Product Owner
last-reviewed: 2026-08-04
---

# UI Interactions — How the Player Plays

This document describes the core interactions — what the player clicks, where panels appear, and how information flows. Not technical wireframes, but the player's mental model.

---

## Map Interaction

### Zoom

Three discrete zoom bands (1–3). The player zooms in/out with the scroll wheel or keyboard shortcuts. Pinch is deferred with touch support:

| Action | Result |
|--------|--------|
| Scroll up | Zoom in: System View (Band 1) → Planet View (Band 2) → Station View (Band 3) |
| Scroll down | Zoom out: Station View (Band 3) → Planet View (Band 2) → System View (Band 1) |
| Zoom out at System View | Already at widest zoom — no further zoom-out. Galaxy Map (V2) will add a Band 0 outside the V1 sequence. |

Zoom is centered on the cursor position — if you scroll while hovering over a planet, it zooms into that planet.

### Pan

Click empty space and drag to pan the map. Panning is unrestricted — you can move anywhere in the system. The star stays at center but the view can scroll to show outer planets.

### Click Behavior

Click behavior depends on the current zoom band:

| Current Band | Click Target | Result |
|-------------|-------------|--------|
| Any | Planet | Zoom to Band 2 (Planet View framed on that planet) |
| Any | Station | Zoom to Band 3 (Station View framed on that station) |
| Any | Ship | Open ship inspection panel (side panel, doesn't zoom) |
| Any | Route line | Open route inspection panel (shows throughput, cargo type) |
| Band 2 or 3 | Empty space | Return to Band 1 (System View) |
| Band 1 | Empty space | Deselect everything, close panels; drag to pan |
| Any | Right-click empty | Context menu: "Build New Station" options |
| Band 1 or 2 | Right-click planet | Context menu: "Survey", "Build Mining Station" |

---

## Placing Stations

### Station Placement Flow

1. Player zooms to **Planet View** (click on planet)
2. Orbit rings become visible around the planet
3. Available station slots appear as ghost outlines on orbit rings
4. Player clicks an empty slot — a **build menu** appears
5. Build menu shows station types available (based on researched tech)
6. Player selects a station type and source Hub (shown only when multiple Hubs exist) → a build order is queued; missing components become demand and an eligible Construction Ship is assigned when available
7. Once a Construction Ship arrives, construction animation plays — the ghost outline fills in as building progresses
8. When complete, the station sprite appears on the orbit ring

### Build Menu

A radial menu or panel at the click position showing:

```
BUILD STATION — Planet: [Name]
┌─────────────────────┐
│ 🏠 Station Hub      │  [Cost: Frame + Core + Control + Cargo]
│ ⛏ Mining Station    │  [Cost: Frame + Core + Cargo]
│ 🏭 Refinery Factory │  [Cost: Frame + Core + Control]
│ ⚙ Construction Fac  │  [Cost: Frame + Core + Bay]
│ 🔬 Research Station │  [Cost: Frame + Core + Lab]
│                     │
│ Slots: 1/6 used    │
└─────────────────────┘
```

Buildings that are locked (not yet researched) are shown grayed out with the tech name required.

Build, upgrade, and demolition dialogs use an explicit source/recovery Hub selector when multiple Hubs exist; with one Hub it is selected implicitly. The resulting command always contains the resolved Hub ID.

### Orbit Ring Slots

Slots are authored per body and distributed across its orbit rings; the exact counts are in GDD 14. The build menu shows "Slots: X/Y used" to track capacity. Higher-tier station hubs do not add slots.

---

## Building Ships

### Shipyard Flow

1. Player clicks a **Station Hub** or clicks the build button on the station's panel
2. A **shipyard menu** appears showing available ship types
3. Player selects a ship type → available components are reserved and missing components become demand at that Hub. The order remains Awaiting Materials until its complete canonical cost is delivered.
4. Construction progress bar appears at the station
5. When complete, the new ship hull appears docked at the station
6. The ship automatically begins finding work (drone logistics)

### Shipyard Menu

```
SHIPYARD — Station Hub: [Name]
┌─────────────────────────────┐
│ ⛴ Cargo Ship (Tier 1)      │  [Cost: Frame + Drive + Cargo Module]
│   Capacity: 50  Speed: Med  │
│                             │
│ ⛴ Hauler (Tier 2)          │  [Cost: Frame + Drive + 2x Cargo Module]
│   Capacity: 120  Speed: Med │  ← Requires: Structural Engineering
│                             │
│ 🏗 Construction Ship (T1)   │  [Cost: Frame + Drive + Bay]
│ 🚀 Research Ship (T1)      │  [Cost: Frame + Drive + Lab]
└─────────────────────────────┘
```

Locked ships grayed out with their research requirement shown.

---

## Logistics Panel (Station Configuration)

Every station has a **logistics panel** that appears when clicked. This is the main interaction for tuning the network.

### Panel Layout

```
┌──────────────────────────────────────┐
│ STATION: Refinery-2          [⚙]    │
│ Planet: Rocky Terran — Orbit Ring 2 │
│──────────────────────────────────────│
│                                      │
│ INPUTS           DEMAND              │
│ ────────────────────────────────     │
│ [◉] Metal Ore    200 / 400 ████░░   │
│   Threshold: [===|=========]  50%    │
│                                      │
│ [◇] Volcanic Sulfur  80 / 200 ██░░  │
│   Threshold: [===|=========]  50%    │
│                                      │
│ OUTPUTS           SUPPLY             │
│ ────────────────────────────────     │
│ [═] Metals     300 / 300 ██████     │
│   Export: [===|=========]  Auto      │
│                                      │
│ [△] Chemicals  40 / 200   ██░░░     │
│   Export: [===|=========]  Auto      │
│                                      │
│──────────────────────────────────────│
│ PRIORITY: [Normal] ──────────────── │
│ SHIPS ASSIGNED: 2                    │
│ └─ [⛴] Courier-3 — en route (Metals)│
│ └─ [⛴] Hauler-1 — loading (Ore)     │
└──────────────────────────────────────┘
```

### What the Player Adjusts

- **Demand threshold** — how low before the station broadcasts a demand (e.g., "request ore when stock drops below 50%")
- **Export threshold** — how full before the station broadcasts available supply
- **Priority** — makes ships favor this station over others (higher priority = served first)
- **Inspect ships** — see which ships are currently assigned to this station

The player tunes these thresholds to prevent bottlenecks and ensure steady production.

---

## Inspecting Ships

Click a ship hull to open its inspection panel:

```
SHIP: Courier-3
┌──────────────────────────────┐
│ Type: Cargo Ship (Tier 1)    │
│ Status: In Transit            │
│                              │
│ CARGO: Metal Ore              │
│ Loaded: 50/50 [████████]     │
│                              │
│ CURRENT JOB:                  │
│ ◀ Mining-1 ───── Refinery-2  │
│   (Round trip: 45s)          │
│                              │
│ NEXT ACTION:                  │
│ Arriving Refinery-2 in 12s   │
└──────────────────────────────┘
```

---

## Inspecting Routes

Click a route line to open its inspection panel:

```
FLOW: Mining-1 → Refinery-2
┌──────────────────────────────────┐
│ Cargo: Metal Ore [Steel Gray]    │
│ Throughput: 120 units/min        │
│ Ships on route: 2                │
│ └─ Courier-3 (50/50, arriving)   │
│ └─ Hauler-1 (80/120, loading)    │
│                                  │
│ ⚠ Bottleneck detected:           │
│   Delivery to Refinery-2 is      │
│   below recipe consumption.      │
└──────────────────────────────────┘
```

Flow lines show a **bottleneck indicator** when trailing delivery to a destination/resource remains below configured recipe consumption for the canonical detection window. This is the game's primary feedback loop.

---

## Research UI

### Opening Research

1. Click Hub Haven for Tier-1 research or a **Research Station** for any supported tier
2. Its panel opens with the **tech tree** tab
3. The full tech tree is visible — all tiers, all unlocks
4. Locked techs are grayed out with their prerequisites shown
5. Completed techs are highlighted green
6. Available to research techs are highlighted blue

### Starting Research

```
RESEARCH STATION: Research-3
┌──────────────────────────────────────────┐
│ TECH TREE                                │
│                                          │
│ ┌─── Basic Techs (Completed)             │
│ │ ✓ Basic Construction                   │
│ │ ✓ Basic Refining                       │
│ │ ✓ Basic Power                          │
│ │ ✓ Basic Control                        │
│ │                                        │
│ ┌─── Tier 1                              │
│ │ 🔵 Advanced Refining [40M+20CF+20W]   │ ← Active
│ │   ◯ Structural Engineering             │
│ │   ◯ Sensor Systems                     │
│ │   ...                                  │
│ │                                        │
│ ┌─── Tier 2 (Explicit prerequisites)     │
│ │   ◯ Alloy Smelting                     │
│ │   ◯ Factory Automation                 │
│ │   ...                                  │
│ └──────────────────────────────────────────┘
│                                          │
│ ACTIVE PROJECT: Advanced Refining        │
│ Progress: █████░░░░░░ 45%                │
│ Resources consumed: 18M / 40M            │
│ [Pause Project]                          │
└──────────────────────────────────────────┘
```

### Research Flow

1. Player clicks an available tech → demand entries are added to the facility's input buffers and existing stock is reserved for the project.
2. The project enters **AwaitingMaterials** state. Ships deliver materials to the station buffer.
3. Once all required resources are reserved in the buffer, the project enters **Ready** then **Active**. Resources are consumed incrementally as progress advances.
4. Progress bar fills over time (real-time, not instant).
5. Multiple Research Stations can work on different techs simultaneously.
6. When progress reaches 100%, the tech unlocks and can be used.
7. **Pause**: The player may pause a project. Completed ticks, consumed-resource credit, and delivered materials remain attached to that technology; resuming continues from the same state. Research cannot be abandoned in V1.

---

## Building the Space Gate

### End-Game Flow

1. Research **Gate Theory** → **Advanced Fabrication**, complete **Grid Power**, then **Gate Construction** → **System Bridge**
2. Build **8 Gate Nodes** at a Tier-4 Construction Factory
3. A **Gate Construction Site** appears on the map at the fringe lane
4. Player clicks the site → "Begin Gate Assembly" button appears
5. The player selects an idle Tier-4 Fabricator when beginning assembly
6. Gate Nodes are transported to the site by cargo ships
7. Assembly progress bar fills as materials arrive
8. When complete, the Gate activates, the lifecycle enters Won, and an aperture animation plays. Intersystem routes are post-V1.

### Gate Inspection Panel

```
SPACE GATE
┌──────────────────────────────────────┐
│ STATUS: BUILDING                     │
│                                      │
│ COMPONENTS REQUIRED:                 │
│ Gate Nodes: ████████░░ 6/8           │
│ Structural Frame: ✓ Delivered        │
│ Power Core: ✓ Delivered              │
│ Control System: ✓ Delivered          │
│                                      │
│ ASSIGNED CONSTRUCTION:               │
│ Fabricator-1 (Assembly in progress)  │
│                                      │
│ SUPPLY CHAIN:                        │
│ ◀ Construction-4 → Gate (Node, 45s) │
│ ◀ Hub-5 → Gate (Frame, 120s)        │
└──────────────────────────────────────┘
```

---

## Summary of Core Interactions

| Interaction | Trigger | Result |
|------------|---------|--------|
| Zoom in | Scroll up on target | Snaps to next zoom band centered on cursor |
| Zoom out | Scroll down | Snaps to wider zoom band |
| Pan | Drag empty space | Moves map view |
| Place station | Click orbit ring slot → build menu → select type | BuildOrder queues; staging and Construction Ship assignment follow |
| Build ship | Click station hub → shipyard menu → select type | Components consumed, ship built |
| Configure station | Click station → logistics panel | Adjust demand/supply thresholds |
| Inspect ship | Click ship hull | Ship panel with cargo and route |
| Inspect flow | Click flow line | Flow panel with trailing throughput and current jobs |
| Research | Click research station → tech tree → select project | Project queues; it waits for materials/ship when necessary |
| Survey | Click planet → "Survey" action | SurveyOrder queues; an eligible Research Ship is assigned when available |
| Build Gate | Click gate site → assemble | Gate construction begins |

All interactions are click-based. No drag-to-place is required — the game is playable with mouse only (V1 scope).

### Error & Blocked States

Every UI action has an explicit blocked, queued, or error state with remediation:

| Action | Blocked Condition | UI Feedback |
|--------|-------------------|-------------|
| Place station | No available orbit slot | "No free slot on this orbit ring — demolish a station or pick another body" |
| Place station | No Construction Ship available | Order queues in Ready/Traveling wait: "Awaiting an eligible Construction Ship" |
| Place station | Missing components | Order queues in Awaiting Materials and lists/broadcasts each missing component |
| Place station | Planet has no slots (gas giant) | "Gas giants have no orbit slots — use moons instead" |
| Place station | Body not surface-surveyed | "Survey this body to depth 1 before building" |
| Place station in The Veil | Orbital Logistics incomplete | "Technology required: Orbital Logistics" |
| Build ship | Missing components | Order queues in Awaiting Materials and lists/broadcasts each missing component |
| Build ship | Shipyard slot full | "Shipyard queue is full — wait for current build to complete" |
| Build ship | Recipe locked | "Technology required: [tech name] — research this first" |
| Research tech at a Research Station | No Research Ship docked | Project queues with NoResearchShip: "Awaiting an eligible Research Ship; research starts after it docks" |
| Research tech | Missing resources | Project enters Awaiting Materials and lists/broadcasts the missing amounts |
| Research tech | Facility buffer maxima cannot fit required inbound material | "Needs [N] more cargo capacity — shrink empty buffers or choose another Research Station" |
| Research tech | Tech already completed | "Already researched — technology is unlocked" |
| Research tech | Tech already in progress | "Already being researched — another project is active for this tech" |
| Set station priority | Priority out of range | "Priority must be 0–100" |
| Change production recipe | Slot is OutputBlocked | "Collect or make space for the completed output before changing this slot" |
| Survey body | No eligible Research Ship available | Order queues: "Awaiting a Research Ship capable of target depth [N]" |
| Survey body | Target depth is already complete | "Depth [N] already surveyed — choose a deeper available target" |
| Cancel build | Not a valid target | "No active construction at this site" |
| Pause research | No active project | "No research project is active" |
| Demolish station | Station has docked ships | "Station has docked ships — wait for them to depart or scrap them first" |
| Demolish station | Inventory/reservations remain | "Evacuating station — move or release listed inventory before demolition" |
| Demolish station | Station is Hub (last one) | "Cannot demolish the last Station Hub — you need at least one hub for your network" |
| Scrap ship | Ship is in transit or docked outside a Hub | "Return this ship to a Station Hub before scrapping" |

### Recovery Actions

The "always solvable" promise requires that players can undo mistakes. V1 supports these recovery actions:

| Action | Trigger | Behavior |
|--------|---------|----------|
| Cancel ship building | Click shipyard order → "Cancel" | Inbound reservations release and staged components return to their source buffers/Hub-side salvage; completed work time is lost. |
| Cancel station placement | Click build site → "Cancel Construction" | Inbound reservations release and staged components return immediately; a loaded Construction Ship first returns its build hold to the source Hub/cache, then becomes idle. The slot is freed immediately because no station exists yet. |
| Pause research | Research panel → "Pause" | Progress and consumed credit persist. The player chooses whether unused delivered materials stay reserved or are released in place to the facility's ordinary buffers. |
| Cancel survey | Survey order panel → "Cancel" | Completed depths persist and partial next-depth work is lost. A scanning ship idles immediately; an in-transit ship safely finishes its current leg, then idles without scanning. |
| Demolish station | Click station → "Demolish" | Work stops, research detaches, and priority evacuation jobs move all inventory/Fuel to a permanent cache at the selected Hub. After the station clears and 30 base work completes, installed components return to that Hub/cache and the slot is freed. |
| Reclaim ship | Click ship docked at a Hub → "Scrap" | Cargo, Fuel, and the full component recipe fill compatible Hub compartments, with overflow in Hub-side salvage; the occupied dock is freed. |
| Change mining target | Click Mining Station → "Set Resource" | Mining output switches to a different resource type from the same body. No construction cost — the station retunes automatically over 10 ticks. |

These actions ensure no permanent deadlock: the player can tear down, rebuild, and re-route without losing invested components. Salvage caches never decay and remain valid logistics sources.

## Keyboard Shortcuts (V1)

While mouse-only is the primary input, keyboard shortcuts are provided for power users. Touch input (V2) will require long-press as right-click alternative; V1 is mouse-only.

### Accessibility Notes

- Color is never the sole identifier: route lines use animated dash patterns, resource icons have distinct shapes, and inspection panels show text labels.
- Keyboard shortcuts supplement mouse-only play: Tab cycles through stations, Enter activates the selected action, Escape closes panels.
- Hit regions are 32×32 CSS pixels minimum for all clickable elements, including small 4px route dots (invisible hit zone surrounding the dot).
- The V1 reduced-motion setting disables pulsing route animations and scanning ring effects; all state changes also have non-animated indicators (icon change, label update, color shift).
- A V1 UI-scale setting provides 100%, 125%, 150%, and 200%; all UI text uses system fonts and reflows without clipping at those sizes.

| Key | Action |
|-----|--------|
| Space | Toggle pause/resume simulation — pause freezes all simulation ticks (ship movement, station processing, construction, research, logistics) but the UI remains interactive for planning and inspection. Research and construction timers do not advance while paused. |
| Tab | Cycle through stations (open logistics panel) |
| Esc | Close current panel / deselect |
| +/- | Zoom in/out (same as scroll) |
| Arrow keys | Pan map |
| R | Open Research Tree (if a Research Station is selected) |
| B | Open Build Menu (at current cursor position) |
| F | Toggle route density overlay |

Shortcuts are documented in-game and optional—the game never requires keyboard input. All UI text uses high-contrast colors, panel text scales with the V1 UI-scale setting, and click targets are at least 32×32 CSS pixels.
