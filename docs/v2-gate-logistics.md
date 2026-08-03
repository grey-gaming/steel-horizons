# V2 — Gate Logistics (Future Content)

This document captures the V2 gate logistics design that was stripped from V1 docs. Kept here for future reference when V2 development begins.

## Gate Network Model

Gates are fixed infrastructure connecting star systems. A gate has a destination (another gate in another system). Ships traveling between systems must use a gate — they can't fly interstellar directly.

**Gates act like OpenTTD rail tracks:**
- Gates have **tracks** (a gate-to-gate connection)
- Tracks have **signals** to prevent multiple ships using the same gate simultaneously
- Tracks have **directionality** — some gates are one-way, some bidirectional
- **Path occupancy** — a gate track is "occupied" while a ship is in transit between systems

## Gate Panel

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

## Signal Types

| Signal | Behavior | Use Case |
|--------|----------|----------|
| **Simple** | Green if gate is idle, red if occupied | Low-traffic gates |
| **Block** | Green if gate AND destination gate buffer are free | High-traffic gates, prevents destination congestion |
| **Priority** | Ships can reserve a slot, others wait | High-value cargo needs guaranteed passage |

## Directionality

| Mode | Behavior |
|------|----------|
| **One-way out** | Ships can only depart through this gate. Cannot arrive. |
| **One-way in** | Ships can only arrive through this gate. Cannot depart. |
| **Bidirectional** | Ships can both depart and arrive. Requires signals to prevent head-on collisions. |

## Path Occupancy

While a ship is using a gate (in transit between systems), the gate track is **occupied**. No other ship can use that specific gate-to-gate connection until the ship arrives at the destination system.

This creates a **gate network** where you must plan capacity:
- Build multiple gates between busy systems
- Use one-way pairs (Gate A → B for outbound, Gate C ← B for inbound)
- Signal priority for time-sensitive cargo

## Visualizing Gate Networks

The inter-system map shows:
- **Gate lines** — drawn between systems like rail lines on OpenTTD
- **Signal indicators** — colored dots along the line (green/red/yellow)
- **Ship markers** — icons moving along gate lines during transit
- **Congestion alerts** — blinking when gates are queued or blocked

## Summary (V2 Complete)

The two modes give Steel Horizons a **logistics puzzle at every scale**:
- **Within a system** (V1): Design the station network and let drones handle the rest.
- **Between systems** (V2): Design the gate network like a railway. The puzzle is signal layout, directionality, and throughput planning.
