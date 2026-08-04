---
status: Approved
owner: Tech Lead
last-reviewed: 2026-08-04
---

# Data Models — Core Entity Definitions

This document owns the serialized V1 state shapes. All integer units and phase semantics are defined by [12-simulation-foundations.md](./12-simulation-foundations.md); exact content records are defined by [14-authored-content.md](./14-authored-content.md).

The notation is language-neutral. Rust uses newtypes for every ID and scaled numeric value, `BTreeMap` for serialized keyed collections, and explicitly tagged Serde enums.

## Identifiers and Resources

```text
type BodyId = string
type StationId = string
type ShipId = string
type BuildOrderId = string
type SalvageId = string
type ReservationId = string
type SurveyOrderId = string
type TechId = string
type RecipeId = string

enum LaneId { Inner, Habitable, Outer, Fringe }
enum DestinationRef {
  Body(BodyId), Station(StationId), Salvage(SalvageId), GateSite
}
enum InventoryDestinationRef {
  Station(StationId), BuildOrder(BuildOrderId),
  Evacuation(BuildOrderId), GateSite
}

enum ResourceType {
  MetalOre, CarbonSoil, SiliconDust, VolcanicSulfur, WaterIce,
  FrozenGases, Helium3, RareEarthMinerals, CrystalDeposits,
  Metals, CarbonFiber, SiliconWafers, Chemicals, Fuel,
  Alloys, Optics, ReactorRods,
  StructuralFrame, PowerCore, ControlSystem, DriveAssembly,
  CargoModule, ResearchLab, ConstructionBay, GateNode
}
```

Authored entities keep authored display names. Generated display names are deterministic and serialized but never used for ordering: ships use `<ShipDefinition name>-<ship counter>` and stations use `<StationType>-<station counter>`. V1 has no Rename command.

## Position and Travel

```text
struct SystemPosition {
  lane_id: LaneId
  radius_units: u32
  angle_milli: i32              // normalized 0..6282
}

enum ArcDirection { Clockwise, CounterClockwise }

struct TravelSegment {
  kind: RadialBurn | LaneArc
  lane_id: LaneId
  total_distance_milli: u64
  remaining_distance_milli: u64
  speed_multiplier_num: u32
  speed_multiplier_den: u32
  life_support_eligible: bool
  arc_direction: ArcDirection | null
}

struct TravelPlan {
  origin: SystemPosition
  destination: DestinationRef
  segments: TravelSegment[]
  active_segment: u8
}

enum InventorySourceRef { Station(StationId), Salvage(SalvageId) }
```

Stations derive their `SystemPosition` from their parent body. Orbit ring and slot are placement coordinates, not system-travel coordinates. While a ship is in transit, `Ship.position` is its last exact node/segment boundary; TravelPlan progress is the authoritative interpolation input. Simulation job assignment occurs only at exact endpoints.

## Buffers and Production

```text
struct Buffer {
  resource: ResourceType
  current: u32
  max: u32
  demand_threshold: u8          // percentage, 0..100
  export_threshold: u8          // percentage, 0..100
}

struct ProductionSlot {
  recipe_id: RecipeId | null
  state: Idle | AwaitingInputs | Processing | OutputBlocked
  reserved_inputs: Map<ResourceType, u32>
  progress_ticks: u32
  total_ticks: u32
  completed_output: Map<ResourceType, u32>
}

struct MiningTarget {
  resource: ResourceType
  rate_remainder: RationalRemainder
  retune_ticks_remaining: u16
}
```

Fuel is stored in a separate station `fuel_buffer` and does not count against general cargo capacity. Every completed station has one with authored tier capacity, default demand threshold 20, and export threshold 50. Its maximum cannot be reconfigured; its thresholds can, subject to demand ≤ export.

## Station

```text
enum StationType { Hub, Mining, Refinery, Construction, Research }

struct Station {
  id: StationId
  display_name: string
  station_type: StationType
  tier: u8
  body_id: BodyId
  orbit_ring: u8
  slot: u8
  input_buffers: Buffer[]
  output_buffers: Buffer[]
  fuel_buffer: Buffer
  total_cargo_capacity: u32
  priority: u8
  installed_components: Map<ResourceType, u32>
  docked_ship_ids: ShipId[]
  holding_ship_ids: ShipId[]
  max_docks: u8
  production_slots: ProductionSlot[]
  mining_targets: MiningTarget[]
  built_in_research_max_tier: u8 | null
  active_research_id: TechId | null
  ship_build_queue: BuildOrderId[]  // Hub only; ordered FIFO queue of ship build orders; first entry is active (ADR-0009)
}
```

The sum of non-fuel buffer maxima may not exceed `total_cargo_capacity`.

## Ships and Jobs

```text
enum ShipRole { Cargo, Construction, Research }

enum ShipState {
  Idle, Holding, InTransit, Building, Surveying,
  PoweringResearch, AwaitingRescue
}

struct Ship {
  id: ShipId
  display_name: string
  role: ShipRole
  tier: u8
  position: SystemPosition
  docked_at: StationId | null
  base_speed_milli: u32
  base_mass: u32
  cargo_type: ResourceType | null
  cargo_amount: u32
  max_cargo_capacity: u32
  build_cargo: Map<ResourceType, u32>
  build_cargo_capacity: u32
  fuel: u32
  fuel_remainder: u64            // invariant: < 10,000,000
  fuel_efficiency_remainder: u8  // invariant: < 5; Life Support 4/5 factor
  max_fuel: u32
  build_work_per_tick: u16
  survey_work_per_tick: u16
  max_survey_depth: u8
  state: ShipState
  job: ShipJob
  travel_plan: TravelPlan | null
}

enum ShipJob {
  Idle,
  Transport {
    reservation_id: ReservationId,
    source: InventorySourceRef,
    destination: InventoryDestinationRef,
    resource: ResourceType,
    amount: u32,
    stage: ToPickup | ToDelivery
  },
  Refuel { station_id: StationId },
  Build { order_id: BuildOrderId },
  Upgrade { order_id: BuildOrderId },
  Demolish { order_id: BuildOrderId },
  Survey { order_id: SurveyOrderId },
  DockForResearch { station_id: StationId, tech_id: TechId },
  Rescue { hub_id: StationId, dispatch_ticks_remaining: u16 }
}
```

Variant-specific jobs avoid meaningless required fields such as cargo on Survey jobs.

`cargo_type/cargo_amount/max_cargo_capacity` are nonzero only for Cargo Ships. `build_cargo/build_cargo_capacity` are used only by Construction Ships and allow one build to carry several component types. Research Ships have both capacities zero. Movement derives the role-specific payload amount as defined in GDD 12.

`Rescue` begins with `dispatch_ticks_remaining = 300`, targets the nearest existing Hub by route distance then Hub ID, and is valid only with zero payload and no active reservation. It then creates a direct tow TravelPlan at half base speed. Tow movement is flagged by the job and does not debit Fuel.

## Celestial Bodies and Deposits

```text
enum BodyType { Planet, Moon, AsteroidBelt }
enum PlanetSubtype { RockyTerran, Volcanic, IceWorld, GasGiant }

struct ResourceDeposit {
  resource: ResourceType
  current: u32
  baseline: u32
  renewable: bool
  minimum_survey_depth: u8
}

struct CelestialBody {
  id: BodyId
  name: string
  body_type: BodyType
  subtype: PlanetSubtype | null
  parent_body_id: BodyId | null
  position: SystemPosition
  survey_depth: u8             // 0 none, 1 surface, 2 subsurface, 3 full
  deposits: ResourceDeposit[]
  orbit_ring_count: u8
  slot_counts: u8[]
}
```

`surveyed` is derived as `survey_depth > 0`. Belt deposits use a concrete representative system position; their visual distribution is presentation data.

## Research

```text
enum ResearchState {
  AwaitingMaterials, Ready, Active, Paused, Complete
}

enum ResearchPauseReason { Manual, NoResearchShip, FacilityUnavailable }

struct RationalRemainder {
  value: u64
  denominator: u64
}

struct ResearchProject {
  tech_id: TechId
  station_id: StationId | null   // null only while detached; a manual pause may retain its facility
  created_server_sequence: u64
  state: ResearchState
  pause_reason: ResearchPauseReason | null
  resources_required: Map<ResourceType, u32>
  resources_reserved: Map<ResourceType, u32>
  resources_consumed: Map<ResourceType, u32>
  consumption_remainders: Map<ResourceType, RationalRemainder>
  ticks_completed: u64
  total_ticks: u64
}
```

There is at most one project record per technology. Pausing retains progress and consumed-resource credit. Completed records may be compacted into `completed_techs` after save migration while preserving replay equivalence.

## Survey Orders

```text
enum SurveyOrderState { Queued, Assigned, Complete, Cancelled }

struct SurveyOrder {
  id: SurveyOrderId
  body_id: BodyId
  target_depth: u8
  priority: u8
  created_server_sequence: u64
  assigned_ship_id: ShipId | null
  work_completed: u32
  total_work: u32
  state: SurveyOrderState
}
```

Only one incomplete order may target a body. `total_work` is the sum of the authored remaining depth increments at creation, and each cumulative depth boundary updates `CelestialBody.survey_depth` immediately. Cancelling preserves completed depth milestones and loses only work toward the next depth; an in-transit ship finishes its current TravelPlan leg before idling at the exact endpoint.

## Construction and Salvage

```text
enum BuildTarget {
  Ship { hub_id: StationId, role: ShipRole, tier: u8 },
  Station { body_id: BodyId, orbit_ring: u8, slot: u8,
            station_type: StationType, tier: u8 },
  Upgrade { station_id: StationId, target_tier: u8 },
  Demolish { station_id: StationId, recovery_hub_id: StationId }
}

enum BuildState {
  AwaitingMaterials, Ready, Traveling, Evacuating, Building,
  Complete, Cancelling, Cancelled
}

struct BuildOrder {
  id: BuildOrderId
  created_server_sequence: u64
  target: BuildTarget
  source_station_id: StationId
  components_required: Map<ResourceType, u32>
  components_delivered: Map<ResourceType, u32> // staging at source_station_id; outside general cargo
  builder_ship_id: ShipId | null
  evacuation_cache_id: SalvageId | null // Demolish only; created at recovery Hub
  progress_work: u32
  total_work: u32
  state: BuildState
}

struct SalvageCache {
  id: SalvageId
  position: SystemPosition
  inventory: Map<ResourceType, u32> // components or recovered reserved materials
}
```

For Station/Upgrade orders, `source_station_id` is the explicitly selected Hub and is the system position of the `BuildOrder` logistics destination. For Ship orders it is the building Hub. For Demolish orders it equals `recovery_hub_id`, and `evacuation_cache_id` names the permanent cache created at that Hub. `Evacuation(order_id)` accepts only inventory sourced from that order's target station and transfers it into the linked cache. When a Construction Ship loads staged components, the map moves atomically from `components_delivered` into `Ship.build_cargo`; completion moves that map into the target's installed components. Content validation proves every station/upgrade component total fits the minimum eligible Construction Ship's build hold.

Salvage caches are durable, recoverable state and never decay. Per-resource logistics reservations provide all claiming; a cache is never locked wholesale to one order.

## Gate

```text
enum GatePhase { SitePreparation, FrameAssembly, PowerIntegration, Activation }

struct GateBuild {
  site_position: SystemPosition
  phase: GatePhase
  components_delivered: Map<ResourceType, u32> // exact manifest, including GateNodes/ReactorRod
  fabricator_ship_id: ShipId
  progress_work: u32
}
```

## Logistics Reservations

```text
enum ReservationState { AwaitingPickup, Loaded, Delivered, Released }

struct Reservation {
  id: ReservationId
  ship_id: ShipId
  supply_source: InventorySourceRef
  demand_destination: InventoryDestinationRef
  resource: ResourceType
  amount: u32
  state: ReservationState
  tick_assigned: u64
  pickup_expiry_tick: u64 | null
}
```

Only AwaitingPickup reservations expire. Loaded reservations remain valid until delivery or explicit recovery handling.

## Bottleneck Monitoring

```text
struct BottleneckTracker {
  deliveries_by_tick: [u32; 600]
  cursor: u16                     // invariant: < 600
  rolling_delivered: u64
  consecutive_deficit_ticks: u16 // saturates at 300
  warning_active: bool
}
```

Only station/resource pairs with configured recipe consumption need trackers. These rolling windows are serialized so save/load does not change warning or event timing.

## Lifecycle, RNG, Commands, and Root State

```text
enum GameLifecycle { Unloaded, Loading, Paused, Running, Advancing, Won }

struct RNGState {
  words: [u64; 4]              // xoshiro256** state; all-zero is invalid
}

struct IdCounters {
  ship: u64
  station: u64
  build_order: u64
  reservation: u64
  salvage: u64
  survey_order: u64
}

enum CommandOutcome { Accepted, Applied, Rejected }

struct CommandRecord {
  id: string
  effective_tick: u64
  server_sequence: u64
  command: Command              // tagged API/domain command payload
  outcome: CommandOutcome
}

struct GameState {
  schema_version: u32
  content_version: string
  lifecycle: GameLifecycle
  tick: u64
  next_server_sequence: u64
  next_event_sequence: u64
  id_counters: IdCounters
  celestial_bodies: Map<BodyId, CelestialBody>
  stations: Map<StationId, Station>
  ships: Map<ShipId, Ship>
  research_projects: Map<TechId, ResearchProject>
  survey_orders: Map<SurveyOrderId, SurveyOrder>
  completed_techs: Set<TechId>
  build_orders: Map<BuildOrderId, BuildOrder>
  salvage_caches: Map<SalvageId, SalvageCache>
  gate_build: GateBuild | null
  logistics_reservations: Map<ReservationId, Reservation>
  bottleneck_trackers: Map<StationId, Map<ResourceType, BottleneckTracker>>
  rng_state: RNGState
  command_log: CommandRecord[]
}

struct GameSnapshot {
  protocol_version: string
  latest_event_sequence: u64
  state: GameState
}
```

Supply/demand tables, route overlays, throughput projections, and UI state are derived and not serialized.

## Content Definitions

Content is immutable during a game and loaded separately from `GameState`:

```text
struct RecipeDefinition {
  id: RecipeId
  required_tech: TechId | null
  facilities: FacilityRequirement[] // e.g. Hub T1/30 ticks or Construction T1/10 ticks
  inputs: Map<ResourceType, u32>
  outputs: Map<ResourceType, u32>
}

struct FacilityRequirement {
  station_type: StationType
  minimum_tier: u8
  cycle_ticks: u32
}

struct TechDefinition {
  id: TechId
  tier: u8
  prerequisites: TechId[]
  costs: Map<ResourceType, u32>
  duration_ticks: u64
  unlocks: string[]
}

struct ShipStats {
  // Cargo ships: cargo_capacity > 0; build_cargo_capacity = 0; survey fields = 0
  // Construction ships: build_cargo_capacity > 0; cargo_capacity = 0; survey fields = 0
  // Research ships: survey fields > 0; cargo_capacity = 0; build_cargo_capacity = 0; build_work_per_tick = 0
  cargo_capacity: u32
  build_cargo_capacity: u32
  speed_milli: u32
  max_fuel: u32
  base_mass: u32
  build_work_per_tick: u16
  survey_work_per_tick: u16
  max_survey_depth: u8
}

struct StationStats {
  docks: u8
  cargo_capacity: u32
  fuel_capacity: u32
  production_slots: u8            // 0 for Hub (uses slow assembly slot), 1-4 for Refinery/Construction
  max_targets: u8                 // Mining only; 0 otherwise
  extraction_per_target_per_10_ticks: u8  // Mining only; 0 otherwise
  research_projects: u8           // Research only; always 1; 0 otherwise
  shipyard_slots: u8              // Hub only; always 1; 0 otherwise
  component_slots: u8             // Hub only; always 1; 0 otherwise
}

struct ShipDefinition {
  role: ShipRole
  tier: u8
  name: string
  stats: ShipStats
  component_cost: Map<ResourceType, u32>
  required_tech: TechId
}

struct StationDefinition {
  station_type: StationType
  tier: u8
  stats: StationStats
  component_cost: Map<ResourceType, u32>
  required_tech: TechId
}

struct StartingScenario {
  // Packages the tick-0 canonical game into a single content record
  celestial_bodies: Map<BodyId, CelestialBody>
  stations: Map<StationId, Station>
  ships: Map<ShipId, Ship>
  completed_techs: Set<TechId>
  deployment_kit: Map<ResourceType, u32>   // loose components in Hub Haven output buffers
  schema_version: u32
  content_version: string
  lifecycle: GameLifecycle
  tick: u64
  next_server_sequence: u64
  next_event_sequence: u64
  id_counters: IdCounters
  rng_state: RNGState
}

struct GateDefinition {
  // Authored content definition for the Space Gate, distinct from runtime GateBuild state
  site_position: SystemPosition
  manifest: Map<ResourceType, u32>   // exact cargo required
  required_techs: TechId[]            // ordered prerequisite techs
  phase_work: Map<GatePhase, u32>     // work required per phase
  activation_sink: Map<ResourceType, u32> // consumed atomically on activation
}

struct AuthoredDefaults {
  // Default buffer allocation parameters used when a command creates new buffers
  general_buffer_preferred_maximum: u32     // default 50
  general_buffer_minimum_multiplier: u8     // default 1 for inputs, 1 for outputs
  demand_threshold_default: u8             // 100 for input buffers
  export_threshold_default: u8             // 0 for output buffers
  fuel_demand_threshold_default: u8        // 20
  fuel_export_threshold_default: u8        // 50
  retune_ticks: u16                         // 10
  research_preferred_capacity: u32         // full outstanding inbound rather than 50
}

The exact V1 records are catalogued in GDD 14 and mirrored byte-for-byte in versioned content JSON. Startup validation rejects duplicate IDs, cyclic technology prerequisites, unreachable recipes, invalid thresholds, over-capacity buffers, or authored values that do not conform to these shapes.

## Schema Generation Ownership

The machine-readable JSON schemas under `content/` are owned by the Tech Lead and generated from the canonical struct definitions in this document. Specifically:

- `content/definitions.v1.json` — mirrors every `*Definition` struct (Recipe, Tech, Ship, Station, Gate) and supporting types (FacilityRequirement, ShipStats, StationStats, AuthoredDefaults). Schema generation is the responsibility of the engine build process (P1-01) and validated by the content validator (P1-04/P1-05).

- `content/starting_system.v1.json` — mirrors the `StartingScenario` record with exact GDD 14 values for bodies, Hub Haven, Builder-1, starting techs, deployment kit, and root metadata. This file is the canonical input for game creation and bootstrap scenarios.

Schema generation follows these rules:

1. Every `*Definition` struct in this document has a corresponding JSON Schema definition in the generated schema.
2. Generated JSON Schema uses `"description"` annotations derived from doc comments in the canonical struct definitions.
3. The schema generator is a Rust crate in the engine workspace that reads the canonical Rust types and emits JSON Schema + TypeScript declarations. Its output is deterministic and snapshot-tested.
4. Hand-written overrides to the generated schema are not permitted. If the generated schema is incorrect, the canonical Rust type or this document's definition is the source of truth and must be corrected.
5. `content/` files are versioned alongside the engine binary. A mismatch between `schema_version` and the content schema version is a fatal startup error.

The content validator (P1-04/P1-05) is the runtime gate that confirms every `content/` file conforms to its schema and passes structural/semantic validation before any game can be created.
