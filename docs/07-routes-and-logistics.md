# Routes & Logistics — The Core Interaction

## Logistics Model

Ships are autonomous drones — you design the station network, ships self-organize supply and demand. This is the only logistics model in V1. Gate logistics (inter-system travel) is V2 content.

### How It Works

Ships act like **autonomous drones**. You don't assign routes to individual ships. Instead, you build the network and ships self-organize to move materials where they're needed.

**Key principles:**

1. **Stations are nodes** — each station has an **input buffer** (what it needs) and an **output buffer** (what it produces)
2. **Ships are drones** — they fly between stations autonomously, carrying materials from stations with surplus to stations with demand
3. **No explicit routes** — you set station priorities, not ship paths
4. **Ships pick the best job** — they evaluate which pending move is shortest/most valuable and do it

### Priority System — Numeric Definition

Each station has a `priority` field (0–100). This influences which supply/demand pair an idle cargo ship selects.

**Job selection algorithm** (simplified):

1. Collect all supply entries (stations with surplus above exportThreshold) and demand entries (stations with demand below demandThreshold).
2. For each supply–demand pair (S→D) where `supply.resource == demand.resource`, compute a **score**:
   `score = demand.priority × 2 + supply.priority × 1.5 - distanceCost × 0.5`
   where `distanceCost = angularDistance × orbitalRadius`.
3. The ship picks the highest-scoring pair that it can reach within its fuel reserve.
4. If multiple pairs have equal score, pick the closest one.

**Effect of priority values:**
- Priority 100 (max): the station's supply/demand is weighted heavily — ships favor it strongly.
- Priority 50 (default): neutral — ships evaluate based on distance and availability.
- Priority 0 (min): ships only serve this station if no higher-priority work exists.

Priority does not interrupt ships already in transit — it only affects idle ship job selection. Changing priority triggers a re-scan for currently idle ships only.

### Station Configuration

Every station and factory has a **logistics panel** with two lists:...

```
[Station: Refinery Alpha]
├── INPUT BUFFER
│   ├── Metal Ore: Requesting 200
│   └── Volcanic Sulfur: Requesting 100
├── OUTPUT BUFFER
│   ├── Metals: Available 150
│   └── Chemicals: Available 60
└── PRIORITY: Normal
```

- **Input buffer**: What this station needs to function. When input falls below a threshold, it broadcasts a demand.
- **Output buffer**: What this station produces. When output exceeds a threshold, it broadcasts a supply.
- **Priority**: You can raise/lower a station's priority to make ships favor or deprioritize it.

### Fog & Surveying

Before a celestial body is surveyed, its resource deposits are hidden. No station can be placed, no demand or supply is broadcast. Surveying a body with a Research Ship clears the fog and reveals its resource profile, making those resources available to the logistics network.

### Cargo Transfer Mechanics

When a ship arrives at a station:

- **Cargo transfer is instant (gameplay)** — the ship arrives, cargo moves from ship buffer to station buffer (or vice versa), and the ship departs immediately. There is no gameplay-relevant loading/unloading delay.
- **Loading/Unloading visual animation** — the ship state machine includes `Loading` and `Unloading` states, but these are purely cosmetic. They control animation playback (cargo containers moving, dock arms extending) for visual feedback. The actual cargo transfer completes in zero game ticks regardless of animation duration.
- **Animation duration** — a cosmetic loading animation may play for 1–3 seconds of real-time, but the ship is considered "arrived and unloaded" for gameplay purposes the instant it docks. The next job assignment happens immediately; the animation is purely eye candy.
- The station must have a **free dock** to receive the ship. Dock count (from Station Hub tier) limits how many ships can be at a station simultaneously.
- If all docks are occupied, arriving ships **queue in a holding pattern** — they orbit the station until a dock frees up. Queued ships do not consume fuel.
- A ship carries **one cargo type per trip**. Mixed cargo loads are not supported in V1.
- When a station's output buffer is full, production pauses automatically (backpressure). Ships cannot load cargo from a station with no available output.
- When a station's input buffer is full and more cargo arrives, the ship queues at the dock until the station processes enough input to free buffer space.

### Ship Behavior

A ship at idle does this:

1. Look for the closest station with an **available output** that another station is **demanding**
2. Fly there, load cargo
3. Fly to the demanding station, unload
4. Repeat

Ships don't need to be told which route to fly — they read the network state and act.

**You control logistics by:**
- Building more ships (more throughput)
- Upgrading ships (faster, larger capacity)
- Setting station priorities (which stations get served first)
- Placing stations strategically (shorter trips = more throughput)

### Bottleneck Detection

The simulation monitors throughput between station pairs and detects bottlenecks automatically.

**Detection algorithm:**
- For each station pair (A→B) where A has persistent surplus of resource X and B has persistent demand:
  - Measure throughput over the last 600 ticks (10 minutes real-time): `units_delivered / 600`
  - Compare to A's production rate of X
  - If throughput < production rate AND the gap persists for 300+ ticks, flag as bottleneck

**Visual indicators:**
- The route/flow line between A and B pulses red
- The route inspection panel shows a warning: "Bottleneck detected: supply exceeds route capacity"
- Suggested fix text: "Add more ships" or "Upgrade ships on this route" or "Increase station priority"

**Advisory only:**
- Bottlenecks are purely informational. No gameplay penalty for ignoring them.
- The player can inspect the warning and decide whether to add ships, upgrade, or restructure the network.
- Multiple bottlenecks can exist simultaneously — each is reported independently on its route line.

### Visualizing the Network

The map shows:

- **Flow arrows** — animated lines from supply stations to demand stations, thickness proportional to material volume
- **Ship icons** — moving along their current path with cargo indicator
- **Hot/cold station colors** — stations with full output buffers glow green (supply), empty inputs glow red (demand), balanced is blue
- **Route density overlay** — a heatmap showing where ships travel most, helping the player see bottlenecks

### Why This Model

OpenTTD-style explicit routes work when you have a few vehicles and fixed cargo types. But in a space system with many ships, many stations, and many resources, manual routing becomes tedious. The drone model means:

- Player focuses on **network design** (where to put stations, what to produce)
- Ships handle the **execution** (finding the best route dynamically)
- The challenge becomes **capacity planning** — do you have enough ships? Are stations placed efficiently?

<!-- V2 Gate Logistics moved to v2-gate-logistics.md -->

---

## UI Panels Summary

### Logistics Panel (System View)

The primary interaction for managing a station's supply/demand:

```
┌──────────────────────────────────────┐
│ STATION: Refinery Alpha              │
│ Planet: Rocky Terran (orbit)         │
│                                      │
│ INPUTS:           DEMAND:            │
│ Metal Ore          200 / 400 ████░░ │
│ Volcanic Sulfur    80 / 200  ██░░░░ │
│                                      │
│ OUTPUTS:           SUPPLY:           │
│ Metals             300 / 300 ██████ │
│ Chemicals          40 / 200   ██░░░ │
│                                      │
│ PRIORITY: [Normal] ──────────────── │
│                                      │
│ SHIPS ASSIGNED: 2                    │
│ └─ Courier-3 (en route, Metals)      │
│ └─ Hauler-1 (loading, Metal Ore)     │
└──────────────────────────────────────┘
```

### Route Density Overlay

A toggleable overlay on the system map showing all active material flows:

```
┌─ SYSTEM MAP ────────────────────────┐
│   ★ Star                              │
│                                      │
│   [Starting Planet]                  │
│     ─── (thick flow) ──▶ Refinery    │
│     ◀── (thin flow) ──── Mining B    │
│                                      │
│   [Volcanic Planet]                  │
│     ── (med flow) ──▶ Refinery       │
│                                      │
│   Legend:                            │
│     ████ = heavy traffic             │
│     ██   = medium traffic            │
│     ░    = light traffic             │
└──────────────────────────────────────┘
```

---

## Summary

The two modes give Steel Horizons a **logistics puzzle at every scale**:

- **Within a system** (V1): Design the station network and let drones handle the rest. The puzzle is placement, capacity, and priority.
- **Between systems** (V2): Design the gate network like a railway. The puzzle is signal layout, directionality, and throughput planning.

Both modes are visible from the same map — system view shows drone flows, and zooming out to the inter-system view shows gate lines. The player seamlessly moves between the two scales.
