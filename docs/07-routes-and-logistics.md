# Routes & Logistics — The Core Interaction

## Two Modes of Logistics

Steel Horizons has two fundamentally different logistics systems:

| Mode | Scope | Model | Inspiration |
|------|-------|-------|-------------|
| **System Logistics** | Within one star system | Autonomous drones | Factorio logistics bots |
| **Gate Logistics** | Between star systems (V2) | Track-and-signal network | OpenTTD rail system |

---

## Mode 1 — System Logistics (V1)

> **This is the chosen model for V1.** Ships are autonomous drones — you design the station network, ships self-organize supply and demand.

### How It Works

Ships act like **autonomous drones**. You don't assign routes to individual ships. Instead, you build the network and ships self-organize to move materials where they're needed.

**Key principles:**

1. **Stations are nodes** — each station has an **input buffer** (what it needs) and an **output buffer** (what it produces)
2. **Ships are drones** — they fly between stations autonomously, carrying materials from stations with surplus to stations with demand
3. **No explicit routes** — you set station priorities, not ship paths
4. **Ships pick the best job** — they evaluate which pending move is shortest/most valuable and do it

### Station Configuration

Every station and factory has a **logistics panel** with two lists:

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

---

## Mode 2 — Gate Logistics (V2 Concept)

### How It Works

Gates are the **fixed infrastructure** connecting star systems. A gate has a destination (another gate in another system). Ships traveling between systems must use a gate — they can't fly interstellar directly.

**Gates act like OpenTTD rail tracks:**
- Gates have **tracks** (a gate-to-gate connection)
- Tracks have **signals** to prevent multiple ships using the same gate simultaneously
- Tracks have **directionality** — some gates are one-way, some bidirectional
- **Path occupancy** — a gate track is "occupied" while a ship is in transit between systems

### Gate Panel

Clicking a gate shows:

```
[Gate: Alpha-Gate]
├── CONNECTIONS
│   ├── → Beta-Gate [SIGNAL: Green | DIR: One-way out]
│   └── ↔ Gamma-Gate [SIGNAL: Red | DIR: Bidirectional]
├── OCCUPANCY
│   ├── Cargo Ship "Hauler-3" → Beta-Gate (in transit, ETA 45s)
│   └── Cargo Ship "Freighter-1" ← Gamma-Gate (in transit, ETA 120s)
├── QUEUE
│   └── (no waiting ships)
└── SETTINGS
    ├── Direction: [One-way out] [One-way in] [Bidirectional]
    └── Signal type: [Simple] [Block] [Priority]
```

### Signal Types

| Signal | Behavior | Use Case |
|--------|----------|----------|
| **Simple** | Green if gate is idle, red if occupied | Low-traffic gates |
| **Block** | Green if gate AND destination gate buffer are free | High-traffic gates, prevents destination congestion |
| **Priority** | Ships can reserve a slot, others wait | High-value cargo needs guaranteed passage |

### Directionality

| Mode | Behavior |
|------|----------|
| **One-way out** | Ships can only depart through this gate. Cannot arrive. |
| **One-way in** | Ships can only arrive through this gate. Cannot depart. |
| **Bidirectional** | Ships can both depart and arrive. Requires signals to prevent head-on collisions. |

### Path Occupancy

While a ship is using a gate (in transit between systems), the gate track is **occupied**. No other ship can use that specific gate-to-gate connection until the ship arrives at the destination system.

This creates a **gate network** where you must plan capacity:
- Build multiple gates between busy systems
- Use one-way pairs (Gate A → B for outbound, Gate C ← B for inbound)
- Signal priority for time-sensitive cargo

### Visualizing Gate Networks

The inter-system map shows:

- **Gate lines** — drawn between systems like rail lines on OpenTTD
- **Signal indicators** — colored dots along the line (green/red/yellow)
- **Ship markers** — icons moving along gate lines during transit
- **Congestion alerts** — blinking when gates are queued or blocked

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

### Gate Panel (Inter-System View)

```
┌──────────────────────────────────────┐
│ GATE: Alpha Gate                     │
│ System: Sol                        │
│                                      │
│ CONNECTIONS                          │
│ ───── Beta Gate  [SIG: ●] [→]       │
│ ───── Gamma Gate [SIG: ●] [↔]       │
│                                      │
│ TRANSIT TRAFFIC:                     │
│ ──▶ Beta Gate: Hauler-3 (45s)       │
│ ◀── Gamma Gate: Freighter-1 (120s)  │
│                                      │
│ QUEUE: (empty)                       │
│                                      │
│ DIRECTION: [One-way out] ────────── │
│ SIGNAL: [Block] ──────────────────  │
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
