# Data Models — Core Entity Definitions

This document defines the fields and types for every core game entity. Technical design references these shapes.

## ResourceType

An enum identifying every resource and good in the economy:

```
enum ResourceType {
  // Tier 1 — Raw
  MetalOre, CarbonSoil, SiliconDust, VolcanicSulfur, WaterIce,
  FrozenGases, Helium3, RareEarthMinerals, CrystalDeposits,
  // Tier 2 — Refined
  Metals, CarbonFiber, SiliconWafers, Chemicals, Fuel,
  Alloys, Optics, ReactorRods,
  // Tier 3 — Components
  StructuralFrame, PowerCore, ControlSystem, DriveAssembly,
  CargoModule, ResearchLab, ConstructionBay, GateNode,
}
```

## Buffer

Each station has input and output buffers — one per resource type the station handles.

```
struct Buffer {
  resource: ResourceType
  current: int                // current amount stored
  max: int                    // storage capacity
  demandThreshold: float      // 0.0–1.0 — when input buffer falls below this %, broadcast demand
  exportThreshold: float      // 0.0–1.0 — when output buffer rises above this %, broadcast supply available
}
```

## Station

```
struct Station {
  id: string                    // unique identifier, e.g. "station_001"
  type: StationType             // Hub | Mining | Refinery | Construction | Research
  tier: int                     // 1–4
  planetId: string              // which celestial body this station orbits
  orbitRing: int                // which orbit ring (0 = closest to planet)
  slot: int                     // slot index on the orbit ring
  inputBuffers: Buffer[]        // what this station needs
  outputBuffers: Buffer[]       // what this station produces
  priority: int                 // 0 (low) – 100 (high), affects ship job selection
  powered: bool                 // false if no Power Core installed → no processing
  dockedShipIds: string[]       // ships currently docked (count = docks used)
  maxDocks: int                 // from tier table
  buildTimeRemaining: int       // ticks until construction finishes (0 = complete)
}
```

## Ship

```
struct Ship {
  id: string                    // e.g. "ship_003"
  role: ShipRole                // Cargo | Construction | Research
  tier: int                     // 1–4
  position: Position            // { laneId: string, angle: float, altitude: float }
  baseSpeed: float              // base speed from tier (units/tick)
  effectiveSpeed: float         // baseSpeed × laneMultiplier × cargoPenalty — computed each tick
  laneMultiplier: float         // set by the orbital lane the ship is in (see 12-simulation-foundations.md)
  cargoType: ResourceType | null
  cargoAmount: int              // current load
  maxCapacity: int              // from tier
  fuel: int                     // current fuel units
  maxFuel: int                  // fuel capacity
  state: ShipState              // Idle | Loading | InTransit | Unloading | Building | Surveying
  job: ShipJob | null
  targetId: string              // station or body the ship is heading to
}

// Lane multipliers are defined in 12-simulation-foundations.md (Rate Definitions section):
//   Inner: 1.5×, Habitable: 1.0×, Outer: 0.7×, Fringe: 0.5×

enum ShipState {
  Idle, Loading, InTransit, Unloading, Building, Surveying
}

enum ShipRole {
  Cargo, Construction, Research
}

struct Position {
  laneId: string    // "inner" | "habitable" | "outer" | "fringe"
  angle: float      // radians around the star
  altitude: float   // offset from lane centerline (for docking)
}
```

## ShipJob

```
struct ShipJob {
  type: JobType                 // Transport | Build | Survey
  sourceId: string              // station or body to load from / start at
  destId: string                // station or body to deliver to / build at
  cargoType: ResourceType       // what cargo to move
  cargoAmount: int              // how much to load
  progress: int                 // ticks completed so far
  totalTicks: int               // ticks needed to complete
}

enum JobType {
  Transport, Build, Survey
}
```

## ResearchProject

```
struct ResearchProject {
  stationId: string             // which Research Station runs this
  techId: string                // which technology (e.g. "advanced_refining")
  resourcesConsumed: Map<ResourceType, int>  // how much of each resource delivered so far
  resourcesRequired: Map<ResourceType, int>  // total needed (from tech cost table)
  progress: float               // 0.0–1.0, advances when all required resources are delivered
}
```

## CelestialBody

```
struct CelestialBody {
  id: string                    // e.g. "planet_volcanic_01"
  type: BodyType                // Planet | Moon | AsteroidBelt
  subtype: PlanetType           // RockyTerran | Volcanic | IceWorld | GasGiant
  laneId: string                // which orbital lane
  orbitalAngle: float           // position on the lane
  surveyed: bool                // false → fogged, resources hidden
  surveyDepth: int              // 0=none, 1=surface, 2=subsurface, 3=hidden
  resources: Map<ResourceType, int>  // available deposits (hidden until surveyed)
  orbitRingCount: int           // how many orbit rings this body supports
  slotCounts: int[]             // slots available per orbit ring
}

enum BodyType {
  Planet, Moon, AsteroidBelt
}

enum PlanetType {
  RockyTerran, Volcanic, IceWorld, GasGiant
}
```

## Logistics Tables (Runtime)

These are not persisted entities — they exist in the simulation runtime for job matching:

```
DemandEntry {
  stationId: string
  resource: ResourceType
  amount: int           // how many units demanded
  position: Position    // for distance calculations
}

SupplyEntry {
  stationId: string
  resource: ResourceType
  amount: int           // how many units available
  position: Position
}
```
