---
status: Draft
owner: Product Owner
last-reviewed: 2026-08-03
---

# UI Interactions — How the Player Plays

This document describes the core interactions — what the player clicks, where panels appear, and how information flows. Not technical wireframes, but the player's mental model.

---

## Map Interaction

### Zoom

Three discrete zoom bands (1–3). The player zooms in/out with scroll wheel or pinch:

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
│ Slots: 4/8 total   │
└─────────────────────┘
```

Buildings that are locked (not yet researched) are shown grayed out with the tech name required.

### Orbit Ring Slots

Slots are per planet — each planet has a total number of station slots distributed across its orbit rings. Rocky Terran planets have 6-8 total slots across 2-3 rings. The build menu shows "Slots: X/Y total" to track remaining capacity. Higher-tier station hubs don't add more slots — slots are fixed by planet type.

---

## Building Ships

### Interaction Flow

1. Player clicks a **Station Hub** or clicks the build button on the station's panel
2. A **shipyard menu** appears showing available ship types
3. Player selects a ship type → components are reserved from the station's output buffers. If any component is missing across all station buffers, the order is queued as "Awaiting Materials" and the UI shows which components are needed and their locations.
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

1. Player clicks an available tech → resources are reserved from station buffers (not deducted all at once). Demand entries are added to the Research Station's input buffer.
2. The project enters **AwaitingMaterials** state. Ships deliver materials to the station buffer.
3. Once all required resources are reserved in the buffer, the project enters **Ready** then **Active**. Resources are consumed incrementally as progress advances.
4. Progress bar fills over time (real-time, not instant).
5. Multiple Research Stations can work on different techs simultaneously.
6. When progress reaches 100%, the tech unlocks and can be used.
7. **Cancellation**: If the player cancels (via the Cancel/Abandon button), unused reserved resources are returned to the station buffer. Resources already consumed are lost.

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

All interactions are click-based. No drag-to-place is required — the game is playable with mouse only (V1 scope).

### Error & Blocked States

Every UI action can fail. The game shows the reason and remediation:

| Action | Blocked Condition | UI Feedback |
|--------|-------------------|-------------|
| Place station | No available orbit slot | "No free slot on this orbit ring — demolish a station or pick another body" |
| Place station | No Construction Ship available | "No idle Construction Ship — build one at the Station Hub" |
| Place station | Missing components | "Components not available: [list of missing] — build more or check supply chain" |
| Place station | Planet has no slots (gas giant) | "Gas giants have no orbit slots — use moons instead" |
| Build ship | Missing components | "Components not available: [list of missing]" |
| Build ship | Shipyard slot full | "Shipyard queue is full — wait for current build to complete" |
| Build ship | Recipe locked | "Technology required: [tech name] — research this first" |
| Research tech | No Research Station docked | "A Research Ship must be docked at the Research Station to begin research" |
| Research tech | Missing resources | "Resources not available: [list of missing]" |
| Research tech | Tech already completed | "Already researched — technology is unlocked" |
| Research tech | Tech already in progress | "Already being researched — another project is active for this tech" |
| Set station priority | Priority out of range | "Priority must be 0–100" |
| Survey body | No Research Ship available | "No idle Research Ship — build one at the Station Hub" |
| Survey body | Body already surveyed | "Already surveyed — resources are visible" |
| Cancel build | Not a valid target | "No active construction at this site" |
| Cancel research | No active project | "No research in progress to cancel" |
| Demolish station | Station has docked ships | "Station has docked ships — wait for them to depart or scrap them first" |
| Demolish station | Station is Hub (last one) | "Cannot demolish the last Station Hub — you need at least one hub for your network" |
| Scrap ship | Ship is in transit | "Ship is en route — wait for it to dock before scrapping" |

### Recovery Actions

The "always solvable" promise requires that players can undo mistakes. V1 supports these recovery actions:

| Action | Trigger | Behavior |
|--------|---------|----------|
| Cancel ship building | Click shipyard order → "Cancel" | Components reserved for that order are released back to station buffers. Progress is lost (no refund of materials already consumed). |
| Cancel station placement | Click station → "Cancel Construction" | Construction Ship returns to idle. Components delivered to the site are lost (non-refundable — the promise is no *permanent* resource loss, but consumed materials are spent). The orbit slot is freed for reuse. |
| Cancel research | Research panel → "Abandon" | All resources delivered to that project are lost (no refund). The tech slot becomes available for a new project. |
| Demolish station | Click station → "Demolish" | Station is removed after a deconstruction timer (30 ticks). Components used to build it are NOT refunded (they were consumed during construction). The orbit slot is freed. |
| Reclaim ship | Click docked ship → "Scrap" | Ship is removed. Components used to build it are NOT refunded. Crew/automation is abstract — no loss. The shipyard slot is freed. |
| Change mining target | Click Mining Station → "Set Resource" | Mining output switches to a different resource type from the same body. No construction cost — the station retunes automatically over 10 ticks. |

These actions ensure no permanent deadlock: the player can always tear down, rebuild, and re-route. Only slot availability and component supply limit what can be rebuilt.

## Keyboard Shortcuts (V1)

While mouse-only is the primary input, keyboard shortcuts are provided for power users. Touch input (V2) will require long-press as right-click alternative; V1 is mouse-only.

### Accessibility Notes

- Color is never the sole identifier: route lines use animated dash patterns, resource icons have distinct shapes, and inspection panels show text labels.
- Keyboard shortcuts supplement mouse-only play: Tab cycles through stations, Enter activates the selected action, Escape closes panels.
- Hit regions are 32×32 CSS pixels minimum for all clickable elements, including small 4px route dots (invisible hit zone surrounding the dot).
- Reduced-motion setting (planned) disables pulsing route animations, zoom transitions, and scanning ring effects — all state changes also have non-animated indicators (icon change, label update, color shift).
- All UI text uses system fonts for OS-level scaling support.

| Key | Action |
|-----|--------|
|| Space | Toggle pause/resume simulation — pause freezes all simulation ticks (ship movement, station processing, construction, research, logistics) but the UI remains interactive for planning and inspection. Research and construction timers do not advance while paused. |
| Tab | Cycle through stations (open logistics panel) |
| Esc | Close current panel / deselect |
| +/- | Zoom in/out (same as scroll) |
| Arrow keys | Pan map |
| R | Open Research Tree (if a Research Station is selected) |
| B | Open Build Menu (at current cursor position) |
| F | Toggle route density overlay |

Shortcuts are documented in-game and optional — the game never requires keyboard input. Accessibility considerations: all UI text uses high-contrast colors, panel text scales with display settings, and click targets are minimum 32×32px for fat-finger safety.
