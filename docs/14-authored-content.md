---
status: Draft
owner: Tech Lead
last-reviewed: 2026-08-04
---

# Authored Content Catalog — Starting System

This document defines the exact authored data for the V1 starting system. It
replaces the placeholder ranges in [02-the-system.md](./02-the-system.md) with
concrete values for every celestial body, station, ship, and inventory item.
Technical design uses these exact numbers.

---

## Starting System Bodies

### 1. Rocky Terran — "Haven" (Player Start)

| Field | Value |
|-------|-------|
| id | `planet_haven` |
| type | Planet |
| subtype | RockyTerran |
| laneId | `habitable` |
| orbitalAngle | 0 rad (top of system view) |
| orbitalRadius | 1200 |
| surveyed | true (pre-surveyed — player sees resources immediately) |
| surveyDepth | 3 (fully revealed) |
| orbitRingCount | 3 |
| slotCounts | [3, 2, 1] → total 6 slots |

**Deposits:**

| Resource | Amount |
|----------|--------|
| MetalOre | 2,000 |
| CarbonSoil | 1,500 |
| SiliconDust | 1,200 |

**Starting station placement:** Station Hub occupies slot 0 on orbit ring 0
(innermost ring, closest to planet).

---

### 2. Volcanic Planet — "Pyre"

| Field | Value |
|-------|-------|
| id | `planet_pyre` |
| type | Planet |
| subtype | Volcanic |
| laneId | `inner` |
| orbitalAngle | 3.14 rad (opposite Haven) |
| orbitalRadius | 600 |
| surveyed | false |
| surveyDepth | 0 |
| orbitRingCount | 2 |
| slotCounts | [2, 1] → total 3 slots |

**Deposits (hidden until surveyed):**

| Resource | Amount |
|----------|--------|
| VolcanicSulfur | 800 |
| RareEarthMinerals | 400 |
| CrystalDeposits | 300 |

---

### 3. Ice World — "Boreas"

| Field | Value |
|-------|-------|
| id | `planet_boreas` |
| type | Planet |
| subtype | IceWorld |
| laneId | `outer` |
| orbitalAngle | 4.71 rad (three-quarters position) |
| orbitalRadius | 2,800 |
| surveyed | false |
| surveyDepth | 0 |
| orbitRingCount | 2 |
| slotCounts | [2, 1] → total 3 slots |

**Deposits (hidden until surveyed):**

| Resource | Amount |
|----------|--------|
| WaterIce | 1,000 |
| FrozenGases | 600 |
| CarbonSoil | 400 |

---

### 4. Gas Giant — "Titan"

| Field | Value |
|-------|-------|
| id | `planet_titan` |
| type | Planet |
| subtype | GasGiant |
| laneId | `outer` |
| orbitalAngle | 1.57 rad (quarter position) |
| orbitalRadius | 3,200 |
| surveyed | false |
| surveyDepth | 0 |
| orbitRingCount | 0 (gas giants have no direct slots) |
| slotCounts | [] |

**Deposits:** None — resources are on moons only.

---

### 5. Moon A (of Titan) — "Rime"

| Field | Value |
|-------|-------|
| id | `moon_rime` |
| type | Moon |
| subtype | RockyTerran (icy surface) |
| laneId | `outer` |
| orbitalAngle | same as Titan + 0.3 rad |
| orbitalRadius | 3,250 (slightly beyond Titan) |
| surveyed | false |
| surveyDepth | 0 |
| orbitRingCount | 1 |
| slotCounts | [2] |

**Deposits (hidden until surveyed):**

| Resource | Amount |
|----------|--------|
| Helium3 | 500 |
| RareEarthMinerals | 200 |

---

### 6. Moon B (of Titan) — "Glint"

| Field | Value |
|-------|-------|
| id | `moon_glint` |
| type | Moon |
| subtype | RockyTerran (small, cratered) |
| laneId | `outer` |
| orbitalAngle | same as Titan + 4.1 rad |
| orbitalRadius | 3,300 (slightly beyond Rime) |
| surveyed | false |
| surveyDepth | 0 |
| orbitRingCount | 1 |
| slotCounts | [1] |

**Deposits (hidden until surveyed):**

| Resource | Amount |
|----------|--------|
| CrystalDeposits | 250 |
| MetalOre | 300 |

---

### 7. Asteroid Belt — "The Veil"

| Field | Value |
|-------|-------|
| id | `belt_veil` |
| type | AsteroidBelt |
| subtype | N/A |
| laneId | `inner` (between inner and habitable lanes) |
| orbitalAngle | distributed across lane (abstract — belt is a region) |
| orbitalRadius | 900 |
| surveyed | false |
| surveyDepth | 0 |
| orbitRingCount | 2 |
| slotCounts | [2, 2] → total 4 slots |

**Deposits (hidden until surveyed; never deplete, use shifting mechanic):**

| Resource | Baseline Amount | Shift Range |
|----------|----------------|-------------|
| MetalOre | 2,000 | ±200 per shift |
| CarbonSoil | 1,200 | ±120 per shift |
| SiliconDust | 800 | ±80 per shift |
| CrystalDeposits | 150 | ±15 per shift |

---

## Starting Inventory

### Station Hub — "Hub Haven"

| Field | Value |
|-------|-------|
| id | `hub_haven` |
| type | StationHub |
| tier | 1 |
| planetId | `planet_haven` |
| orbitRing | 0 |
| slot | 0 |
| powered | true |
| maxDocks | 2 |

**Input buffers:** (empty at start — no demand until configured)

**Output buffers:**

| Resource | Current | Max |
|----------|---------|-----|
| Fuel | 200 | 200 (reserved compartment — separate from cargo storage; see Fuel rules in 03-economy.md) |

**Starter Kit components (in Hub storage — available immediately):**

| Component | Quantity |
|-----------|----------|
| StructuralFrame | 1 (already installed in Hub hull; counts as built-in, not a separate storage item) |
| DriveAssembly | 1 |
| ResearchLab | 1 |
| ConstructionBay | 1 |
| PowerCore | 2 |
| ControlSystem | 1 |
| CargoModule | 2 |

**Starter Kit usage notes:**
- The installed StructuralFrame is not a loose component — it is part of the
  Hub's construction and cannot be used elsewhere.
- One PowerCore is consumed when the first Construction Factory is built.
- One PowerCore is consumed when the first Research Station is built.
- One CargoModule is consumed when the first Cargo Ship is built.
- One CargoModule is consumed when installed in the Hub's storage (the Hub's
  total storage capacity includes this module).

### Starting Ships

**Construction Ship — "Builder-1"**

| Field | Value |
|-------|-------|
| id | `ship_builder_1` |
| role | Construction |
| tier | 1 |
| position | docked at `hub_haven` |
| baseSpeed | 3.0 units/tick |
| fuel | 100 (full) |
| maxFuel | 100 |
| state | Idle |

**Cargo Ship — "Courier-1"** (not yet built — see onboarding; player builds
this in Phase 1 after placing the Mining Station and Refinery. No pre-spawned
cargo ship at game start.)

### Tech Research Status

**Already researched (Tier 0):**

- BasicConstruction
- BasicRefining
- BasicPower
- BasicControl

---

## System Map Summary

```
★ Star (center)
│
├── Inner Lane
│   └── Pyre (volcanic) — fogged
│
├── Habitable Lane
│   └── Haven (rocky terran) — pre-surveyed, Hub Haven visible
│
├── [Asteroid Belt — The Veil] (between inner & habitable)
│   └── fogged
│
├── Outer Lane
│   ├── Boreas (ice world) — fogged
│   └── Titan (gas giant) — fogged
│       ├── Rime (moon) — fogged
│       └── Glint (moon) — fogged
│
└── Fringe Lane
    └── (empty — Space Gate site appears here later)
```

**Total station slots across all bodies: 18** (6 Haven + 3 Pyre + 3 Boreas + 2
Rime + 1 Glint + 4 Veil). This gives the player comfortable room for early
expansion and forces strategic choices when scaling into mid-game.

---

## Deposit Guarantee — Gate-Victory Feasibility

The starting system guarantees sufficient resources for V1 completion:

| Gate-Critical Resource | Total Available | Required for Gate |
|------------------------|-----------------|-------------------|
| Alloys (produced from RareEarthMinerals + Metals) | ~1,200 potential | ~800 (Gate construction + research) |
| ReactorRods (produced from Helium3 + RareEarthMinerals) | ~500 Helium3 + ~400 RareEarth = ~300 Rods potential | ~200 |
| PowerCores (produced from ReactorRods + Alloys) | ~200 potential (using gate-criticals as inputs) | ~90 |
| Optics (produced from CrystalDeposits) | ~550 Crystal potential | ~200 |

No soft-lock is possible through depletion alone. The player can always reach
the Gate by scaling extraction and transport.

---

*This catalog is the source of truth for the authored starting system.
Technical design should reference these exact values for map generation,
starting state, and balance testing.*
