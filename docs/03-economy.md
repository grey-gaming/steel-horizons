# Economy & Resources

There is no money. Every structure, ship, and research project costs materials. The economy is a **resource flow chain** — you extract raw materials, transport them, refine them into components, and use components to build things.

## Resource Tiers

### Tier 1 — Raw Resources (Extracted from celestial bodies)

| Resource | Found On | Used For |
|----------|----------|----------|
| Metal Ore | Rocky planets, asteroids | Smelting into Metals |
| Carbon Rich Soil | Rocky planets, ice worlds | Processing into Carbon |
| Silicon Dust | Rocky planets, asteroids | Processing into Silicon |
| Volcanic Sulfur | Volcanic planets | Chemical refining |
| Water Ice | Ice worlds, some moons | Water, fuel production |
| Frozen Gases | Ice worlds, gas giants | Fuel, coolant |
| Helium-3 | Gas giants, moons | Reactor fuel |
| Rare Earth Minerals | Volcanic planets, some moons | Advanced components |
| Crystal Deposits | Asteroid belts, some moons | Electronics |

> **Deposits are hidden until surveyed.** The starting planet's deposits are known, but all other planets, moons, and belts start fogged. You must send a Research Ship to survey a body before you can see what resources it offers. Higher-tier surveys reveal more detail (surface → subsurface → hidden deposits).

### Tier 2 — Refined Resources (Produced at Factories)

| Refined Good | Requires | Used For |
|-------------|----------|----------|
| Metals | Metal Ore | All construction |
| Carbon Fiber | Carbon Rich Soil | Hulls, structural parts |
| Silicon Wafers | Silicon Dust | Electronics, solar panels |
| Chemicals | Volcanic Sulfur + Water | Fuel, life support |
| Fuel | Frozen Gases + Chemicals | Ship propulsion |
| Alloys | Metals + Rare Earth Minerals | Advanced hulls |
| Optics | Crystal Deposits | Sensors, comms |
| Reactor Rods | Helium-3 + Rare Earth Minerals | Power generation |

### Tier 3 — Components (Built at Construction Factories)

Components are complex assemblies of refined goods. They're the final inputs for building ships, stations, factories, and research.

**Bootstrap exception:** A Station Hub (Tier 1) can assemble Tier 1 components (Structural Frame, Cargo Module) at 3× slower rate (1 per 30 ticks). This enables early-game construction before a dedicated Construction Factory is built. All other components require a Construction Factory.

| Component | Requires | Used For |
|-----------|----------|----------|
| Structural Frame | Metals + Carbon Fiber | All large construction |
| Power Core | Reactor Rods + Alloys | All powered structures |
| Control System | Silicon Wafers + Optics | Ship bridges, station cores |
| Drive Assembly | Alloys + Fuel + Control System | Ship engines |
| Cargo Module | Metals + Carbon Fiber + Control System | Station storage, cargo ships |
| Research Lab | Silicon Wafers + Optics + Power Core | Research stations |
| Construction Bay | Structural Frame + Power Core + Control System | Shipyard factories |
| Gate Node | Alloys + Power Core + Optics + Reactor Rods | Space Gate component |

### Tier 4 — Final Structures (What You Actually Build)

| Final Product | Components Required | What It Does |
|---------------|--------------------|-------------|
| Cargo Ship | Structural Frame + Drive Assembly + Cargo Module | Transports goods between stations |
| Construction Ship | Structural Frame + Drive Assembly + Construction Bay | Builds stations and factories |
| Research Ship | Structural Frame + Drive Assembly + Research Lab | Surveys planets, enables research |
| Station Hub | Structural Frame + Power Core + Control System + Cargo Module | Central logistics hub |
| Mining Station | Structural Frame + Power Core + Cargo Module | Extracts raw resources |
| Refinery Factory | Structural Frame + Power Core + Control System | Refines raw → refined |
| Construction Factory | Structural Frame + Power Core + Construction Bay | Builds components |
| Research Station | Structural Frame + Power Core + Research Lab | Enables research projects |
| Space Gate (V1 goal) | 8x Gate Nodes + Structural Frame + Power Core + Control System | Opens travel to other systems |

## Economy Flow Diagram (Simplified)

```
Raw Resources
    │
    ▼
[Refinery Factories] → Refined Resources
    │
    ▼
[Construction Factories] → Components
    │
    ▼
[Shipyards / Station Builders] → Final Structures
    │
    ▼
[Research Stations] → New Technology (unlocks better versions of everything)
```

## Ship Progression

Ships have tiers too. A Tier 1 Cargo Ship carries less, moves slower, and costs fewer/basic components. As you research, you unlock higher-tier versions that are faster, carry more, and need advanced components.

The same applies to stations and factories — higher tiers have more throughput, more slots, and can handle advanced materials.

## Fuel Consumption

Ships consume Fuel during flight. Fuel is produced at Refinery Factories (Frozen Gases + Chemicals → Fuel).

**Fuel rules:**
- Consumption rate = `distance_traveled × cargo_load × 0.01` fuel units per tick
- Each ship has a fuel capacity (maxFuel) based on tier — Tier 1: 100, Tier 2: 200, Tier 3: 400, Tier 4: 800
- Ships auto-refuel at any station with Fuel in its output buffer — this happens during docking, no player action needed
- Ships always have enough reserve to reach the nearest station (minimum 10% fuel set aside) — no stranding
- A ship low on fuel prioritizes refueling over new cargo jobs
- Fuel is stored in the ship, not at stations — stations buffer Fuel for refueling only

## Power

Every station requires a **Power Core** component to operate. Unpowered stations cannot process cargo, extract resources, or run research.

**Power rules:**
- Power Core is a one-time build cost. It does not consume resources over time — install it once and the station runs indefinitely.
- Exception: the **Space Gate** requires continuous Reactor Rods or Helium-3 supply to maintain the gate connection. This is a megastructure doing active work, not a standard station.
- The Space Gate consumes 1 Reactor Rod per 100 ticks while active. If supply stops, the gate remains built but inactive (no inter-system travel).
- Higher-tier stations have larger power requirements built into their component costs (a Tier 4 Station Hub needs more/better components than a Tier 1 Hub).

## Logistics Thinking

The game is about **matching supply with demand**:
- Your mining stations extract ore but need power cores to run
- Your refinery needs ore delivered regularly to stay active
- Your construction factory needs refined goods to build components
- Your shipyard needs components to build new ships

If any link in the chain breaks, production stops. You must build enough transport capacity, storage, and redundancy to keep everything flowing. This is the core challenge.
