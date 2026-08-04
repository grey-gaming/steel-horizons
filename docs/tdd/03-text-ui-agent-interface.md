---
status: Approved
owner: Tech Lead
date: 2026-08-04
---

# Text UI / Agent Interface

## Overview

The text UI is a CLI/TUI client that renders game state as text by consuming the simulation engine's API. It serves two purposes:

1. **Human-readable view** — the developer (you) watches the simulation progress in the terminal
2. **AI agent play-testing interface** — the agent (me) connects to the API to send commands and observe state, with the text UI as a visual feedback channel

The text UI is a **separate Python console entry point** (`steel-horizons-text-ui`) that connects to the API via HTTP/WS. It is not part of the simulation engine.

## Architecture

```
┌─────────────────┐     HTTP/WS      ┌──────────────────┐
│  Text UI Client │◄────────────────►│  Simulation API  │
│     (Python)      │ discovery + token │  (Rust binary)   │
└─────────────────┘                  └──────────────────┘
```

**Decision:** Phase 1 uses Python with `httpx`, `websockets`, and generated protocol models. Phase 2 uses PixiJS v8 in Tauri; it does not replace the Phase 1 TUI with another native UI stack.

## Rendering Modes

The text UI supports multiple rendering modes selected by CLI flag or an interactive key:

### Mode 1: System Map (Default)

A top-down ASCII view of the star system showing celestial bodies, orbital lanes, stations, and ships.

```
★ Star System — Tick 0420 [Paused]
═══════════════════════════════════════════════

 Inner Lane ────── [Pyre] ● Mining-1 (Sulfur: 45/100)
                  [The Veil] ● Belt Mine (Silicon: 30/60)

 Habitable ─── [★ Starting Planet]
    ┌─ Hub-1 ───────────────────────┐
    │ Ships: Cargo-1 (idle)         │
    │        Fabricator-1 (Gate)    │
    │ Metals: ████████░░ 80/100     │
    │ Fuel:   ████░░░░░░ 40/100     │
    └────────────────────────────────┘
    └── Mine-1 (Metal Ore: 50/100)

 Outer Lane ───── [Boreas] ● Mine-2 (Frozen Gases: 20/50)
                  [Rime] ● Mine-3 (Helium-3: 20/50)

 Fringe ── Gate Site (Site Preparation 150/300 work)

═══════════════════════════════════════════════
Research: Advanced Refining (60%)
Gate: 1/8 Nodes delivered — Site Preparation
```

### Mode 2: Logistics Table

Tabular view of all supply/demand flows, sorted by priority or throughput.

```
Logistics ── Tick 0420
═══════════════════════════════════════════════
SUPPLY:
│ Mine-1     │ Metals    │ 80/100 │ ⬆ │ ⏺ P50 │
│ Mine-2     │ Helium3   │ 20/50  │ ⬆ │ ⏺ P50 │
│ Refinery-A │ Fuel      │ 60/100 │ ⬆ │ ⏺ P50 │
──
DEMAND:
│ Hub-1      │ Metals    │ 20/100 │ ⬇ │ ⏺ P80 │ ★
│ Gate Site  │ Gate Nodes│ 0/8    │ ⬇ │ ⏺ P100 │ ★★★
──
SHIPS:
│ Cargo-1    │ Metals → Hub-1   │ ETA: 12 ticks │
│ Cargo-2    │ Helium3 → Hub-1  │ ETA: 45 ticks │
│ Fabricator-1│ Gate Site Prep   │ 150/300 work  │
```

### Mode 3: Entity Detail

Single-entity view with full stats, buffers, and logs.

### Mode 4: Event Log

Time-ordered log of events (builds completed, ships arrived, bottlenecks detected).

## Agent Interaction Model

The AI agent connects to the API and interacts in one of two modes:

### Turn-Based Mode (Default)

1. Agent sends planning commands via `POST /api/v1/command` while Paused.
2. Agent sends `AdvanceTicks {count}` and waits for the committed acknowledgement.
3. Agent fetches the resulting snapshot/events.
4. Text UI renders the committed state and the agent decides its next command.

This is deterministic and contains no wall-clock sleeps.

### Streaming Mode (Optional)

1. Agent connects via `WS /api/v1/stream`
2. Agent sends commands as WS messages
3. Text UI renders continuously as ticks advance
4. Agent observes the stream and may pause or submit new planning commands; active ship jobs remain non-interruptible unless their owning order is cancelled

The agent can switch between modes per session.

## CLI Interface

The text UI binary accepts:

```
steel-horizons-text-ui [flags]

Flags:
  --connection <path>      connection.json path (default: OS app-data path)
  --api-host <host>        Explicit development override
  --api-port <port>        Explicit development override
  --token <token>          Explicit development override
  --mode <mode>            Rendering mode: map | logistics | detail | log (default: map)
  --refresh <interval>     Refresh interval in ticks (default: 1, 0 = manual only)
  --no-color               Disable ANSI color
  --help                   Show help
```

When the text UI starts, it:
1. Reads `connection.json` and authenticates to `GET /api/v1/status`
2. Fetches full state via `GET /api/v1/state`
3. Renders initial view
4. Subscribes to `WS /api/v1/stream` for tick-by-tick updates
5. Re-renders on each tick event (or on command response)

## Agent Script Template

A Python script for agent play-testing looks like:

```python
import httpx, websockets, json, asyncio
from steel_horizons_client import discover

connection = discover()
API = f"http://{connection.host}:{connection.port}/api/v1"
HEADERS = {"Authorization": f"Bearer {connection.token}"}

def get_state():
    r = httpx.get(f"{API}/state", headers=HEADERS)
    r.raise_for_status()
    return r.json()

def send_command(cmd):
    r = httpx.post(f"{API}/command", headers=HEADERS, json=cmd)
    r.raise_for_status()
    return r.json()

async def stream_events():
    uri = f"ws://{connection.host}:{connection.port}/api/v1/stream"
    async with websockets.connect(uri, additional_headers=HEADERS) as ws:
        async for msg in ws:
            event = json.loads(msg)
            print(f"[Tick {event['tick']}] {event['type']}")
```

## Related ADRs

- ADR-0003 (Command/Query API with WebSocket Streaming)
