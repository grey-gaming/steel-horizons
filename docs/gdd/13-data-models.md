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
type ScenarioId = string

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

Every map whose value is a `ResourceType` quantity (`Map<ResourceType, u32>` or
`Map<ResourceType, u64>`) is sparse and canonical: every stored value is strictly
positive, a zero quantity is omitted, and no quantities serialize as `{}`. This rule
applies to authored costs/inputs/outputs and runtime inventory, reservation, delivered,
consumed, and component maps. It does not apply to maps whose values are structured
remainders or tracker records. This removes dense-versus-sparse hash ambiguity.

Authored entities keep authored display names. Generated display names are deterministic and serialized but never used for ordering: ships use `<ShipDefinition name>-<counter as zero-padded minimum-width-eight decimal>` and stations use `<StationType>-<counter as zero-padded minimum-width-eight decimal>` (for example `Builder-00000001`). This keeps the authored `Builder-1` distinct while allowing every generated counter to start at one. V1 has no Rename command.

Each `IdCounters` value is the next unused positive integer for its kind. Generated
IDs use the exact lowercase prefixes `ship_generated_`, `station_generated_`,
`build_order_generated_`, `reservation_generated_`, `salvage_generated_`, and
`survey_order_generated_`, followed by the counter in zero-padded decimal with a
minimum width of eight digits (for example `ship_generated_00000001`). Allocation and
checked counter increment are staged in the same transaction; rollback consumes
nothing, while a committed ID is never reused even if its entity is later removed.
Authored IDs are forbidden from every reserved `<kind>_generated_` prefix. Generated
display names use the same committed counter value, but identity and ordering use only
the generated ID. Each loaded counter must be greater than every surviving/logged
generated suffix of its kind; gaps from removed entities are valid and never filled.
P1-04 validates the namespace separation and P1-08 tests rollover,
overflow, rollback, deletion, and save/replay behavior.

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
  slot_index: u8                 // stable station-local slot; unique and serialized
  resource: ResourceType
  rate_remainder: RationalRemainder
  retune_ticks_remaining: u16
}
```

Mining targets are serialized by ascending `slot_index`; indices are unique and
below the station tier's target capacity, and a station cannot repeat one resource in
multiple slots. Retargeting stores the requested resource immediately, resets its
remainder, and sets the authored ten-tick countdown. Cross-station contention for one
finite body/resource deposit is reduced in Station ID then slot order (ADR-0010).

Fuel is stored in a separate station `fuel_buffer` and does not count against general cargo capacity. Every completed station has one with authored tier capacity, default demand threshold 20, and export threshold 50. Its maximum cannot be reconfigured; its thresholds can, subject to demand ≤ export.

General input buffers always serialize `export_threshold = 0`; logistics ignores that
inactive field. General output buffers always serialize `demand_threshold = 0`;
logistics ignores that inactive field. `ConfigureBuffer` preserves and validates these
zeros. Fuel is the only buffer on which both thresholds are active.

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
`production_slots` contains every recipe-running slot: its length is
`StationStats.component_slots` for a Hub, `StationStats.production_slots` for a
Refinery or Construction Station, and zero for Mining/Research. A Hub's one entry is
the slow component-assembly slot; keeping `StationStats.production_slots = 0` for Hub
therefore does not erase its separately declared component slot.
For a Hub, `ship_build_queue[0]` is the sole active-slot order and every later
entry is resource-neutral pending work. Only the first order may stage components,
receive logistics reservations, or advertise demand. Queue entries are nonterminal
ship BuildOrders in creation-sequence order; terminal orders remain in
`GameState.build_orders` history but not in the queue (ADR-0009).

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
  installed_components: Map<ResourceType, u32>
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
  Refuel { station_id: StationId }, // valid for Cargo, Construction, and Research
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
`resources_reserved` always names unconsumed units physically reserved at the current
`station_id`; reassignment releases them at the old facility and rebuilds the map only
from stock at the new facility. A project waiting for a Research Ship uses
`state = Paused` and `pause_reason = NoResearchShip` (ADR-0012).

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
  components_delivered: Map<ResourceType, u32> // exact manifest, including GateNode/ReactorRods
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
  consecutive_clear_ticks: u16   // saturates at 300 while warning_active
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
enum CommandApplicationBoundary { PausedImmediate, ScheduledTick }

type ErrorDetail = string | bool | u64 | null | string[] | u64[]

struct CommandRejection {
  code: string
  message: string
  details: Map<string, ErrorDetail>
}

enum CommandResult {
  None,
  BuildOrderCreated { order_id: BuildOrderId },
  SurveyOrderCreated { order_id: SurveyOrderId },
  ResearchProjectCreated { tech_id: TechId },
  ResearchProjectUpdated { tech_id: TechId },
  GateAssemblyStarted,
  AdvanceTicksCompleted { resulting_tick: u64 }
}

struct CommandRecord {
  id: string
  expected_tick: u64 | null      // required wire member; null opts out
  effective_tick: u64
  server_sequence: u64
  application_boundary: CommandApplicationBoundary
  command: ReplayableGameCommand // actor controls never enter deterministic replay
  outcome: CommandOutcome
  result: CommandResult | null
  rejection: CommandRejection | null
}

struct GameState {
  schema_version: u32
  content_version: string
  lifecycle: GameLifecycle
  tick: u64
  next_server_sequence: u64     // persisted lower bound; gaps from actor controls are valid
  next_event_sequence: u64      // positive persisted lower bound / next unused event sequence
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
  latest_event_sequence: u64    // trusted runtime next sequence minus one
  state: GameState
}
```

`ErrorDetail` arrays are homogeneous and nonempty; an absent list is represented by
omitting its detail-map key, never an ambiguous empty array. Floats, nested objects,
mixed arrays, and opaque JSON are forbidden in command/API/runtime error details.
`CommandResult` uses the common internally tagged `type` representation, including
`{"type":"None"}` for its unit variant.

Supply/demand tables, route overlays, throughput projections, and UI state are derived and not serialized.

`GameState.lifecycle` is restricted to Paused, Running, Advancing, or Won whenever a
state exists. Unloaded and Loading are runtime actor/status values: Unloaded has no
state, and Loading retains any prior state unchanged until atomic success/failure.
They are in the shared enum so status/events use one exhaustive vocabulary, but a save
or published `GameSnapshot` containing either is invalid.

The runtime event allocator/ring belongs to the server session and survives successful
`NewGame`/`LoadAutosave` state replacement. While a game exists, its
`next_event_sequence` is synchronized as a persisted lower bound. A same-process load
rebases an older candidate to at least the runtime next sequence before publishing;
fresh-process autoload seeds the runtime allocator from the saved lower bound. The
snapshot cursor is the trusted runtime allocator minus one, never the replaced state
field minus one; both runtime and serialized next-sequence values are at least one
(ADR-0012). A fresh process loaded at a lower bound greater than one therefore reports
a nonzero historical cursor even though its in-memory ring is empty.

`command_log` contains only the replayable construction, recovery, configuration,
research, survey, and Gate command subset from ADR-0003. Actor controls are tracked in
the runtime session receipt ledger defined by ADR-0008 and never serialized as game
timeline commands. An Accepted record has null result/rejection; Applied has a
non-null result (including the explicit `None` variant) and null rejection; Rejected
has null result and a typed rejection. Persisting the result lets idempotent retries
return an originally generated ID even after its entity is later removed. The full
command envelope comparison includes the required nullable `expected_tick`; omission
is malformed and never enters the receipt ledger.

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
  mechanic_unlocks: MechanicUnlock[]
}

enum MechanicUnlock { // internally tagged by `type`; PascalCase variants
  SurveyDepth { max_depth: u8 },
  AsteroidBeltOperations,
  LifeSupportFuelFactor { numerator: u8, denominator: u8 },
  GateSiteVisibility,
  GateAssembly
}

struct ShipStats {
  // Cargo ships: cargo_capacity > 0; build capacity/work and survey fields = 0
  // Construction ships: build capacity/work > 0; cargo capacity and survey fields = 0
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
  production_slots: u8            // 1-4 for Refinery/Construction; 0 for Hub/Mining/Research
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
  build_work: u32
  component_cost: Map<ResourceType, u32>
  required_tech: TechId
}

struct StationDefinition {
  station_type: StationType
  tier: u8
  stats: StationStats
  build_work: u32
  component_cost: Map<ResourceType, u32>
  required_tech: TechId
}

struct StartingScenario {
  id: ScenarioId
  content_version: string
  lifecycle: GameLifecycle       // Paused in V1
  tick: u64                      // 0 in V1
  next_server_sequence: u64
  next_event_sequence: u64
  id_counters: IdCounters
  rng_state: RNGState
  celestial_bodies: Map<BodyId, CelestialBody>
  stations: Map<StationId, Station>
  ships: Map<ShipId, Ship>
  completed_techs: Set<TechId>
}

struct GatePhaseDefinition {
  phase: GatePhase
  work: u32
  required_deliveries: Map<ResourceType, u32> // cumulative entry requirement
  completion_consumption: Map<ResourceType, u32>
}

struct GateDefinition {
  // Authored content definition, distinct from runtime GateBuild state
  site_position: SystemPosition
  manifest: Map<ResourceType, u32>
  required_techs: TechId[]
  required_fabricator_role: ShipRole
  minimum_fabricator_tier: u8
  logistics_priority: u8
  transfer_berths: u8
  phases: GatePhaseDefinition[]  // exactly once each, in GatePhase order
}

struct AuthoredDefaults {
  new_station_priority: u8              // 50
  general_buffer_preferred_maximum: u32 // 50
  input_demand_threshold: u8            // 100 percent
  output_export_threshold: u8           // 0 percent
  fuel_demand_threshold: u8             // 20 percent
  fuel_export_threshold: u8             // 50 percent
  mining_retune_ticks: u16              // 10
  upgrade_work_per_tier: u32            // 60
  demolition_work: u32                  // 30
  survey_depth_work: [u32; 3]           // incremental work [300, 600, 900]
  hub_shipyard_work_per_tick: u16       // 1
}

struct DefinitionsCatalog {
  content_version: string
  defaults: AuthoredDefaults
  recipes: RecipeDefinition[]
  technologies: TechDefinition[]
  ships: ShipDefinition[]
  stations: StationDefinition[]
  gate: GateDefinition
}

// Runtime aggregate and the normalized root used for content hashing.
struct ContentCatalog {
  definitions: DefinitionsCatalog
  starting_system: StartingScenario
}
```

`content/definitions.v1.json` has `DefinitionsCatalog` as its root.
`content/starting_system.v1.json` has `StartingScenario` as its root. Catalog
arrays have one canonical order which the validator enforces: recipes and technologies
by ID; ships by `(role, tier)`; stations by `(station_type, tier)`; Gate phases by
`GatePhase`. Typed maps and sets validate and iterate in their domain key order; the
ADR-0006 canonical JSON writer independently emits every JSON object in lexicographic
member-name order. Duplicate JSON object members are rejected before deserialization.

`StartingScenario` is authored initialization data, not a saved `GameState`; keeping
the two shapes separate prevents an engine-state schema migration from masquerading as
a gameplay-content change. Initialization is nevertheless complete and has no implicit
gameplay defaults. The loader constructs `GameState` by copying every matching scenario
field, setting `schema_version` to the engine's current state-schema version, and setting
`research_projects`, `survey_orders`, `build_orders`, `salvage_caches`,
`logistics_reservations`, `bottleneck_trackers`, and `command_log` to explicit empty
collections plus `gate_build` to explicit `null`. A future root-state field must add an
explicit initialization rule and test in the same schema change.

The deployment kit exists only in Hub Haven's scenario output buffers; there is no
second deployment-kit field and the loader performs no inventory injection. Scenario
`content_version` must equal the definitions catalog. The V1 lifecycle/tick/sequences,
counters, and RNG state must equal GDD 14 exactly. The construction operation is pure:
validate the scenario, construct the complete state using this mapping, then run
invariants.

The Gate manifest includes the activation Reactor Rod. Each phase record states its
cumulative delivered-entry requirement; the Activation record consumes the Reactor
Rod atomically on completion. `required_techs` contains the direct Gate-start
requirement (`system_bridge` in V1); normal prerequisite validation proves its
transitive technology chain. GDD 14 owns the exact values for the Tier-4 Construction
Fabricator, priority 100, one berth, phase order, work, and deliveries.

Research buffer capacity is deliberately absent from `AuthoredDefaults`: it is the
dynamic full outstanding inbound amount from GDD 14, not a numeric content constant.
Recipe/mining minimum buffer sizes are likewise computed from current stock,
reservations, and the configured batch/target. Startup validation rejects duplicate
IDs, cyclic technology prerequisites, unreachable recipes, invalid thresholds,
over-capacity starting buffers, non-canonical array order, or authored values that do
not conform to these shapes.

## Schema Generation Ownership

The two files under `content/` are authored data, not JSON Schema artifacts. The Tech
Lead owns deterministic JSON Schema export from the canonical Rust Serde DTOs to:

- `schemas/content/definitions.v1.schema.json`
- `schemas/content/starting_system.v1.schema.json`

P1-01 establishes the reproducible export/check command only. P1-02b implements the
complete Serde DTOs and schema exporter; P1-03 checks in the generated schemas together
with the authored data and fails if regeneration changes them. API schemas/OpenAPI are
separate P1-14 outputs, and TypeScript generation belongs to the Phase 2 design gate.

Generated schemas include descriptions from Rust doc comments, reject unknown fields
where the corresponding DTO does, and are snapshot-tested. Hand-written edits to a
generated schema are forbidden: correct this document or its canonical Rust DTO, then
regenerate. `schema_version` belongs only to serialized `GameState` shape;
`content_version` belongs to authored catalog compatibility. Each JSON Schema uses a
versioned `$id`, but that identifier is not either runtime version.

P1-04/P1-05 validate both content files against these schemas and then apply structural
and semantic rules before a game can be created. The validated `ContentCatalog` is also
the sole input to the versioned content hash defined by ADR-0006.
