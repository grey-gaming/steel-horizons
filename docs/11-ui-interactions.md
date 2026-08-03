# UI Interactions — How the Player Plays

This document describes the core interactions — what the player clicks, where panels appear, and how information flows. Not technical wireframes, but the player's mental model.

---

## Map Interaction

### Zoom

Three discrete zoom bands. The player zooms in/out with scroll wheel or pinch:

| Action | Result |
|--------|--------|
| Scroll up | Zoom in: System Icon → Detailed View |
| Scroll down | Zoom out: Detailed View → System Icon View |
| At Galaxy Map (V2) | Scroll further out enters Galaxy Map |

Zoom is centered on the cursor position — if you scroll while hovering over a planet, it zooms into that planet.

### Pan

Click empty space and drag to pan the map. Panning is unrestricted — you can move anywhere in the system. The star stays at center but the view can scroll to show outer planets.

### Click Behavior

| Click Target | Result |
|-------------|--------|
| Planet | Zoom to Planet View (Detailed View framed on that planet) |
| Station | Zoom to Station View (Detailed View framed on that station) |
| Ship | Open ship inspection panel (side panel, doesn't zoom) |
| Route line | Open route inspection panel (shows throughput, cargo type) |
| Empty space | Deselect everything, close panels |
| Right-click empty | Context menu: "Build New Station" options |
| Right-click planet | Context menu: "Survey", "Build Mining Station" |

---

## Placing Stations

### Interaction Flow

1. Player zooms to **Planet View** (click on planet)
2. Orbit rings become visible around the planet
3. Available station slots appear as ghost outlines on orbit rings
4. Player clicks an empty slot — a **build menu** appears
5. Build menu shows station types available (based on researched tech)
6. Player selects a station type → Construction Ship is dispatched
7. Construction animation plays — ghost outline fills in as building progresses
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
│ Available Slots: 3/6│
└─────────────────────┘
```

Buildings that are locked (not yet researched) are shown grayed out with the tech name required.

### Orbit Ring Slots

Each orbit ring has a limited number of slots. Slots are shared between all station types. The player sees how many slots remain. Higher-tier station hubs add more slots.

---

## Building Ships

### Interaction Flow

1. Player clicks a **Station Hub** or clicks the build button on the station's panel
2. A **shipyard menu** appears showing available ship types
3. Player selects a ship type → components are consumed from storage
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
│ STATION: Refinery Alpha      [⚙]    │
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
│ ROUTE:                        │
│ ◀ Mining Alpha ── Refinery B │
│   (Round trip: 45s)          │
│                              │
│ NEXT ACTION:                  │
│ Arriving Refinery B in 12s   │
└──────────────────────────────┘
```

---

## Inspecting Routes

Click a route line to open its inspection panel:

```
ROUTE: Mining Alpha → Refinery Beta
┌──────────────────────────────────┐
│ Cargo: Metal Ore [Steel Gray]    │
│ Throughput: 120 units/min        │
│ Ships on route: 2                │
│ └─ Courier-3 (50/50, arriving)   │
│ └─ Hauler-1 (80/120, loading)    │
│                                  │
│ ⚠ Bottleneck detected:           │
│   Mining output exceeds route    │
│   capacity. Add more ships.      │
└──────────────────────────────────┘
```

Route lines also show a **bottleneck indicator** — if supply persistently exceeds route capacity, the line pulses red. This is the game's primary feedback loop.

---

## Research UI

### Opening Research

1. Click a **Research Station** on the map
2. Its panel opens with the **tech tree** tab
3. The full tech tree is visible — all tiers, all unlocks
4. Locked techs are grayed out with their prerequisites shown
5. Completed techs are highlighted green
6. Available to research techs are highlighted blue

### Starting Research

```
RESEARCH STATION: Lab Alpha
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
│ │ 🔵 Advanced Refining [Cost: 200M+...] │ ← Active
│ │   ◯ Structural Engineering             │
│ │   ◯ Sensor Systems                     │
│ │   ...                                  │
│ │                                        │
│ ┌─── Tier 2 (Requires 2 Tier 1 techs)    │
│ │   ◯ Alloy Smelting                     │
│ │   ◯ Factory Automation                 │
│ │   ...                                  │
│ └──────────────────────────────────────────┘
│                                          │
│ ACTIVE PROJECT: Advanced Refining        │
│ Progress: █████░░░░░░ 45%                │
│ Resources consumed: 90M / 200M           │
│ Cancel [Abandon]                         │
└──────────────────────────────────────────┘
```

### Research Flow

1. Player clicks an available tech → resources are deducted from network storage
2. The tech starts progressing at the Research Station
3. Progress bar fills over time (real-time, not instant)
4. Multiple Research Stations can work on different techs simultaneously
5. When progress reaches 100%, the tech unlocks and can be used

---

## Building the Space Gate

### End-Game Flow

1. Research **Gate Theory** → **Gate Construction** → **System Bridge**
2. Build **8 Gate Nodes** at a Construction Factory (Tier 4 required)
3. A **Gate Construction Site** appears on the map at the fringe lane
4. Player clicks the site → "Begin Gate Assembly" button appears
5. A Tier 4 Construction Ship (Fabricator) must be assigned
6. Gate Nodes are transported to the site by cargo ships
7. Assembly progress bar fills as materials arrive
8. When complete, the Gate activates — animation plays, destination route appears

### Gate Inspection Panel

```
SPACE GATE: Sol Gate
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
│ ◀ Factory Alpha → Gate (Node, 45s)  │
│ ◀ Hub Gamma → Gate (Frame, 120s)    │
└──────────────────────────────────────┘
```

---

## Summary of Core Interactions

| Interaction | Trigger | Result |
|------------|---------|--------|
| Zoom in | Scroll up on target | Snaps to next zoom band centered on cursor |
| Zoom out | Scroll down | Snaps to wider zoom band |
| Pan | Drag empty space | Moves map view |
| Place station | Click orbit ring slot → build menu → select type | Construction dispatched |
| Build ship | Click station hub → shipyard menu → select type | Components consumed, ship built |
| Configure station | Click station → logistics panel | Adjust demand/supply thresholds |
| Inspect ship | Click ship hull | Ship panel with cargo and route |
| Inspect route | Click route line | Route panel with throughput |
| Research | Click research station → tech tree → select project | Research starts |
| Survey | Click planet → "Survey" action | Research ship dispatched |
| Build Gate | Click gate site → assemble | Gate construction begins |

All interactions are click-based. No drag-to-place, no keyboard shortcuts required. The game is playable with mouse only.
