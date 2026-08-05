#![allow(missing_docs)]

//! Serialized simulation state DTOs.
//!
//! Every struct in this module is a Serde-only data-transfer object matching
//! GDD 13.  No runtime simulation logic lives here — that belongs in the
//! tick, actor, and mechanics modules (P1-09 onward).
//!
//! ## Authoritative references
//!
//! - GDD 13 §Position and Travel
//! - GDD 13 §Buffers and Production
//! - GDD 13 §Station
//! - GDD 13 §Ships and Jobs
//! - GDD 13 §Celestial Bodies and Deposits
//! - GDD 13 §Research
//! - GDD 13 §Survey Orders
//! - GDD 13 §Construction and Salvage
//! - GDD 13 §Gate
//! - GDD 13 §Logistics Reservations
//! - GDD 13 §Bottleneck Monitoring
//! - GDD 13 §Lifecycle, RNG, Commands, and Root State

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::command::ReplayableGameCommand;
use crate::id::*;
use crate::types::*;

// ─── Position and Travel ──────────────────────────────────────────────

/// A position within the system: lane, orbital radius, and angle.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct SystemPosition {
    pub lane_id: LaneId,
    pub radius_units: u32,
    /// Normalized 0..6282 milliradians (π × 2000).
    pub angle_milli: i32,
}

/// Kind of a single travel segment.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum TravelSegmentKind {
    RadialBurn,
    LaneArc,
}

/// One segment of a ship's travel plan.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct TravelSegment {
    pub kind: TravelSegmentKind,
    pub lane_id: LaneId,
    pub total_distance_milli: u64,
    pub remaining_distance_milli: u64,
    pub speed_multiplier_num: u32,
    pub speed_multiplier_den: u32,
    pub life_support_eligible: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arc_direction: Option<ArcDirection>,
}

/// A ship's full travel plan from origin to destination.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct TravelPlan {
    pub origin: SystemPosition,
    pub destination: DestinationRef,
    pub segments: Vec<TravelSegment>,
    pub active_segment: u8,
}

// ─── Buffers and Production ───────────────────────────────────────────

/// A single resource buffer with thresholds.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct Buffer {
    pub resource: ResourceType,
    pub current: u32,
    pub max: u32,
    pub demand_threshold: u8,
    pub export_threshold: u8,
}

/// State of a single production slot.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ProductionSlot {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipe_id: Option<RecipeId>,
    pub state: ProductionSlotState,
    #[serde(default)]
    pub reserved_inputs: BTreeMap<ResourceType, u32>,
    pub progress_ticks: u32,
    pub total_ticks: u32,
    #[serde(default)]
    pub completed_output: BTreeMap<ResourceType, u32>,
}

/// Production slot state machine.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum ProductionSlotState {
    Idle,
    AwaitingInputs,
    Processing,
    OutputBlocked,
}

/// A mining target attached to a station slot.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct MiningTarget {
    pub slot_index: u8,
    pub resource: ResourceType,
    pub rate_remainder: RationalRemainder,
    pub retune_ticks_remaining: u16,
}

// ─── Station ──────────────────────────────────────────────────────────

/// A station entity in the simulation.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct Station {
    pub id: StationId,
    pub display_name: String,
    pub station_type: StationType,
    pub tier: u8,
    pub body_id: BodyId,
    pub orbit_ring: u8,
    pub slot: u8,
    #[serde(default)]
    pub input_buffers: Vec<Buffer>,
    #[serde(default)]
    pub output_buffers: Vec<Buffer>,
    pub fuel_buffer: Buffer,
    pub total_cargo_capacity: u32,
    pub priority: u8,
    #[serde(default)]
    pub installed_components: BTreeMap<ResourceType, u32>,
    #[serde(default)]
    pub docked_ship_ids: Vec<ShipId>,
    #[serde(default)]
    pub holding_ship_ids: Vec<ShipId>,
    pub max_docks: u8,
    #[serde(default)]
    pub production_slots: Vec<ProductionSlot>,
    #[serde(default)]
    pub mining_targets: Vec<MiningTarget>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub built_in_research_max_tier: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_research_id: Option<TechId>,
    #[serde(default)]
    pub ship_build_queue: Vec<BuildOrderId>,
}

// ─── Ships and Jobs ───────────────────────────────────────────────────

/// A ship entity in the simulation.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct Ship {
    pub id: ShipId,
    pub display_name: String,
    pub role: ShipRole,
    pub tier: u8,
    pub position: SystemPosition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docked_at: Option<StationId>,
    pub base_speed_milli: u32,
    pub base_mass: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cargo_type: Option<ResourceType>,
    pub cargo_amount: u32,
    pub max_cargo_capacity: u32,
    #[serde(default)]
    pub build_cargo: BTreeMap<ResourceType, u32>,
    pub build_cargo_capacity: u32,
    #[serde(default)]
    pub installed_components: BTreeMap<ResourceType, u32>,
    pub fuel: u32,
    pub fuel_remainder: u64,
    pub fuel_efficiency_remainder: u8,
    pub max_fuel: u32,
    pub build_work_per_tick: u16,
    pub survey_work_per_tick: u16,
    pub max_survey_depth: u8,
    pub state: ShipState,
    pub job: ShipJob,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub travel_plan: Option<TravelPlan>,
}

// ─── Celestial Bodies and Deposits ────────────────────────────────────

/// A resource deposit on a celestial body.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ResourceDeposit {
    pub resource: ResourceType,
    pub current: u32,
    pub baseline: u32,
    pub renewable: bool,
    pub minimum_survey_depth: u8,
}

/// A celestial body (planet, moon, or asteroid belt).
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct CelestialBody {
    pub id: BodyId,
    pub name: String,
    pub body_type: BodyType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtype: Option<PlanetSubtype>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_body_id: Option<BodyId>,
    pub position: SystemPosition,
    pub survey_depth: u8,
    #[serde(default)]
    pub deposits: Vec<ResourceDeposit>,
    pub orbit_ring_count: u8,
    #[serde(default)]
    pub slot_counts: Vec<u8>,
}

// ─── Research ─────────────────────────────────────────────────────────

/// Rational remainder for accumulator-based resource consumption.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct RationalRemainder {
    pub value: u64,
    pub denominator: u64,
}

/// A research project tracking progress toward a technology.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ResearchProject {
    pub tech_id: TechId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub station_id: Option<StationId>,
    pub created_server_sequence: u64,
    pub state: ResearchState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pause_reason: Option<ResearchPauseReason>,
    #[serde(default)]
    pub resources_required: BTreeMap<ResourceType, u32>,
    #[serde(default)]
    pub resources_reserved: BTreeMap<ResourceType, u32>,
    #[serde(default)]
    pub resources_consumed: BTreeMap<ResourceType, u32>,
    #[serde(default)]
    pub consumption_remainders: BTreeMap<ResourceType, RationalRemainder>,
    pub ticks_completed: u64,
    pub total_ticks: u64,
}

// ─── Survey Orders ────────────────────────────────────────────────────

/// A survey order for a celestial body.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct SurveyOrder {
    pub id: SurveyOrderId,
    pub body_id: BodyId,
    pub target_depth: u8,
    pub priority: u8,
    pub created_server_sequence: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assigned_ship_id: Option<ShipId>,
    pub work_completed: u32,
    pub total_work: u32,
    pub state: SurveyOrderState,
}

// ─── Construction and Salvage ─────────────────────────────────────────

/// A build order (ship, station, upgrade, or demolition).
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct BuildOrder {
    pub id: BuildOrderId,
    pub created_server_sequence: u64,
    pub target: BuildTarget,
    pub source_station_id: StationId,
    #[serde(default)]
    pub components_required: BTreeMap<ResourceType, u32>,
    #[serde(default)]
    pub components_delivered: BTreeMap<ResourceType, u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub builder_ship_id: Option<ShipId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evacuation_cache_id: Option<SalvageId>,
    pub progress_work: u32,
    pub total_work: u32,
    pub state: BuildState,
}

/// A durable salvage cache holding recovered materials.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct SalvageCache {
    pub id: SalvageId,
    pub position: SystemPosition,
    #[serde(default)]
    pub inventory: BTreeMap<ResourceType, u32>,
}

// ─── Gate ─────────────────────────────────────────────────────────────

/// The Space Gate build state during assembly.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct GateBuild {
    pub site_position: SystemPosition,
    pub phase: GatePhase,
    #[serde(default)]
    pub components_delivered: BTreeMap<ResourceType, u32>,
    pub fabricator_ship_id: ShipId,
    pub progress_work: u32,
}

// ─── Logistics Reservations ───────────────────────────────────────────

/// A logistics reservation for cargo transport.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct Reservation {
    pub id: ReservationId,
    pub ship_id: ShipId,
    pub supply_source: InventorySourceRef,
    pub demand_destination: InventoryDestinationRef,
    pub resource: ResourceType,
    pub amount: u32,
    pub state: ReservationState,
    pub tick_assigned: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pickup_expiry_tick: Option<u64>,
}

// ─── Bottleneck Monitoring ────────────────────────────────────────────

/// Rolling-window bottleneck tracker for a station/resource pair.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct BottleneckTracker {
    #[serde(default)]
    pub deliveries_by_tick: Vec<u32>,
    pub cursor: u16,
    pub rolling_delivered: u64,
    pub consecutive_deficit_ticks: u16,
    pub consecutive_clear_ticks: u16,
    pub warning_active: bool,
}

// ─── Lifecycle, RNG, Commands, and Root State ─────────────────────────

/// The xoshiro256** PRNG state (all-zero is invalid).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct RNGState {
    pub words: [u64; 4],
}

/// Monotonically increasing ID counters per entity kind.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct IdCounters {
    pub ship: u64,
    pub station: u64,
    pub build_order: u64,
    pub reservation: u64,
    pub salvage: u64,
    pub survey_order: u64,
}

/// A record of a committed command in the game timeline.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct CommandRecord {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_tick: Option<u64>,
    pub effective_tick: u64,
    pub server_sequence: u64,
    pub application_boundary: CommandApplicationBoundary,
    pub command: ReplayableGameCommand,
    pub outcome: CommandOutcome,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<CommandResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rejection: Option<CommandRejection>,
}

/// The complete simulation state at a point in time.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct GameState {
    pub schema_version: u32,
    pub content_version: String,
    pub lifecycle: GameLifecycle,
    pub tick: u64,
    pub next_server_sequence: u64,
    pub next_event_sequence: u64,
    pub id_counters: IdCounters,
    #[serde(default)]
    pub celestial_bodies: BTreeMap<BodyId, CelestialBody>,
    #[serde(default)]
    pub stations: BTreeMap<StationId, Station>,
    #[serde(default)]
    pub ships: BTreeMap<ShipId, Ship>,
    #[serde(default)]
    pub research_projects: BTreeMap<TechId, ResearchProject>,
    #[serde(default)]
    pub survey_orders: BTreeMap<SurveyOrderId, SurveyOrder>,
    #[serde(default)]
    pub completed_techs: BTreeSet<TechId>,
    #[serde(default)]
    pub build_orders: BTreeMap<BuildOrderId, BuildOrder>,
    #[serde(default)]
    pub salvage_caches: BTreeMap<SalvageId, SalvageCache>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gate_build: Option<GateBuild>,
    #[serde(default)]
    pub logistics_reservations: BTreeMap<ReservationId, Reservation>,
    #[serde(default)]
    pub bottleneck_trackers: BTreeMap<StationId, BTreeMap<ResourceType, BottleneckTracker>>,
    pub rng_state: RNGState,
    #[serde(default)]
    pub command_log: Vec<CommandRecord>,
}

/// A published immutable snapshot of game state.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct GameSnapshot {
    pub protocol_version: String,
    pub latest_event_sequence: u64,
    pub state: GameState,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip<T>(val: &T)
    where
        T: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug + PartialEq,
    {
        let json = serde_json::to_string(val).unwrap();
        let back: T = serde_json::from_str(&json).unwrap();
        assert_eq!(*val, back);
    }

    #[test]
    fn system_position_round_trip() {
        let p = SystemPosition {
            lane_id: LaneId::Inner,
            radius_units: 100,
            angle_milli: 1570,
        };
        round_trip(&p);
    }

    #[test]
    fn travel_segment_kind_radial_burn() {
        round_trip(&TravelSegmentKind::RadialBurn);
    }

    #[test]
    fn travel_segment_kind_lane_arc() {
        round_trip(&TravelSegmentKind::LaneArc);
    }

    #[test]
    fn travel_segment_round_trip() {
        let s = TravelSegment {
            kind: TravelSegmentKind::LaneArc,
            lane_id: LaneId::Habitable,
            total_distance_milli: 10000,
            remaining_distance_milli: 5000,
            speed_multiplier_num: 10,
            speed_multiplier_den: 10,
            life_support_eligible: true,
            arc_direction: Some(ArcDirection::Clockwise),
        };
        round_trip(&s);
    }

    #[test]
    fn travel_plan_round_trip() {
        let p = TravelPlan {
            origin: SystemPosition {
                lane_id: LaneId::Inner,
                radius_units: 100,
                angle_milli: 0,
            },
            destination: DestinationRef::Station {
                station_id: StationId("hub_haven".into()),
            },
            segments: vec![TravelSegment {
                kind: TravelSegmentKind::RadialBurn,
                lane_id: LaneId::Inner,
                total_distance_milli: 500,
                remaining_distance_milli: 0,
                speed_multiplier_num: 10,
                speed_multiplier_den: 10,
                life_support_eligible: false,
                arc_direction: None,
            }],
            active_segment: 0,
        };
        round_trip(&p);
    }

    #[test]
    fn buffer_round_trip() {
        let b = Buffer {
            resource: ResourceType::Metals,
            current: 500,
            max: 1000,
            demand_threshold: 50,
            export_threshold: 80,
        };
        round_trip(&b);
    }

    #[test]
    fn production_slot_idle_round_trip() {
        let s = ProductionSlot {
            recipe_id: None,
            state: ProductionSlotState::Idle,
            reserved_inputs: BTreeMap::new(),
            progress_ticks: 0,
            total_ticks: 0,
            completed_output: BTreeMap::new(),
        };
        round_trip(&s);
    }

    #[test]
    fn production_slot_processing_round_trip() {
        let s = ProductionSlot {
            recipe_id: Some(RecipeId("smelt_iron".into())),
            state: ProductionSlotState::Processing,
            reserved_inputs: BTreeMap::from([(ResourceType::MetalOre, 100)]),
            progress_ticks: 30,
            total_ticks: 100,
            completed_output: BTreeMap::from([(ResourceType::Metals, 50)]),
        };
        round_trip(&s);
    }

    #[test]
    fn production_slot_state_variants() {
        round_trip(&ProductionSlotState::Idle);
        round_trip(&ProductionSlotState::AwaitingInputs);
        round_trip(&ProductionSlotState::Processing);
        round_trip(&ProductionSlotState::OutputBlocked);
    }

    #[test]
    fn mining_target_round_trip() {
        let m = MiningTarget {
            slot_index: 0,
            resource: ResourceType::MetalOre,
            rate_remainder: RationalRemainder {
                value: 50,
                denominator: 100,
            },
            retune_ticks_remaining: 10,
        };
        round_trip(&m);
    }

    #[test]
    fn station_round_trip() {
        let s = Station {
            id: StationId("hub_haven".into()),
            display_name: "Haven Hub".to_string(),
            station_type: StationType::Hub,
            tier: 1,
            body_id: BodyId("planet_haven".into()),
            orbit_ring: 0,
            slot: 0,
            input_buffers: vec![Buffer {
                resource: ResourceType::Metals,
                current: 100,
                max: 1000,
                demand_threshold: 50,
                export_threshold: 80,
            }],
            output_buffers: vec![],
            fuel_buffer: Buffer {
                resource: ResourceType::Fuel,
                current: 500,
                max: 2000,
                demand_threshold: 30,
                export_threshold: 70,
            },
            total_cargo_capacity: 10000,
            priority: 64,
            installed_components: BTreeMap::new(),
            docked_ship_ids: vec![],
            holding_ship_ids: vec![],
            max_docks: 4,
            production_slots: vec![],
            mining_targets: vec![],
            built_in_research_max_tier: Some(5),
            active_research_id: None,
            ship_build_queue: vec![],
        };
        round_trip(&s);
    }

    #[test]
    fn ship_round_trip() {
        let s = Ship {
            id: ShipId("ship_builder_1".into()),
            display_name: "Builder-1".to_string(),
            role: ShipRole::Construction,
            tier: 1,
            position: SystemPosition {
                lane_id: LaneId::Inner,
                radius_units: 100,
                angle_milli: 0,
            },
            docked_at: Some(StationId("hub_haven".into())),
            base_speed_milli: 1000,
            base_mass: 100,
            cargo_type: None,
            cargo_amount: 0,
            max_cargo_capacity: 200,
            build_cargo: BTreeMap::new(),
            build_cargo_capacity: 50,
            installed_components: BTreeMap::new(),
            fuel: 100,
            fuel_remainder: 0,
            fuel_efficiency_remainder: 0,
            max_fuel: 100,
            build_work_per_tick: 10,
            survey_work_per_tick: 10,
            max_survey_depth: 1,
            state: ShipState::Idle,
            job: ShipJob::Idle,
            travel_plan: None,
        };
        round_trip(&s);
    }

    #[test]
    fn resource_deposit_round_trip() {
        let d = ResourceDeposit {
            resource: ResourceType::MetalOre,
            current: 5000,
            baseline: 10000,
            renewable: false,
            minimum_survey_depth: 0,
        };
        round_trip(&d);
    }

    #[test]
    fn celestial_body_round_trip() {
        let c = CelestialBody {
            id: BodyId("planet_haven".into()),
            name: "Haven".to_string(),
            body_type: BodyType::Planet,
            subtype: Some(PlanetSubtype::RockyTerran),
            parent_body_id: None,
            position: SystemPosition {
                lane_id: LaneId::Inner,
                radius_units: 100,
                angle_milli: 0,
            },
            survey_depth: 0,
            deposits: vec![ResourceDeposit {
                resource: ResourceType::MetalOre,
                current: 5000,
                baseline: 10000,
                renewable: false,
                minimum_survey_depth: 0,
            }],
            orbit_ring_count: 3,
            slot_counts: vec![4, 4, 4],
        };
        round_trip(&c);
    }

    #[test]
    fn rational_remainder_round_trip() {
        round_trip(&RationalRemainder {
            value: 1,
            denominator: 100,
        });
    }

    #[test]
    fn research_project_round_trip() {
        let r = ResearchProject {
            tech_id: TechId("refined_metals".into()),
            station_id: Some(StationId("hub_haven".into())),
            created_server_sequence: 1,
            state: ResearchState::Active,
            pause_reason: None,
            resources_required: BTreeMap::from([(ResourceType::Metals, 500)]),
            resources_reserved: BTreeMap::new(),
            resources_consumed: BTreeMap::new(),
            consumption_remainders: BTreeMap::new(),
            ticks_completed: 0,
            total_ticks: 100,
        };
        round_trip(&r);
    }

    #[test]
    fn survey_order_round_trip() {
        let s = SurveyOrder {
            id: SurveyOrderId("so_1".into()),
            body_id: BodyId("planet_haven".into()),
            target_depth: 3,
            priority: 64,
            created_server_sequence: 1,
            assigned_ship_id: None,
            work_completed: 0,
            total_work: 100,
            state: SurveyOrderState::Queued,
        };
        round_trip(&s);
    }

    #[test]
    fn build_order_round_trip() {
        let b = BuildOrder {
            id: BuildOrderId("bo_1".into()),
            created_server_sequence: 1,
            target: BuildTarget::Ship {
                hub_id: StationId("hub_haven".into()),
                role: ShipRole::Construction,
                tier: 1,
            },
            source_station_id: StationId("hub_haven".into()),
            components_required: BTreeMap::new(),
            components_delivered: BTreeMap::new(),
            builder_ship_id: None,
            evacuation_cache_id: None,
            progress_work: 0,
            total_work: 100,
            state: BuildState::AwaitingMaterials,
        };
        round_trip(&b);
    }

    #[test]
    fn salvage_cache_round_trip() {
        let s = SalvageCache {
            id: SalvageId("sv_1".into()),
            position: SystemPosition {
                lane_id: LaneId::Habitable,
                radius_units: 200,
                angle_milli: 3140,
            },
            inventory: BTreeMap::new(),
        };
        round_trip(&s);
    }

    #[test]
    fn gate_build_round_trip() {
        let g = GateBuild {
            site_position: SystemPosition {
                lane_id: LaneId::Fringe,
                radius_units: 300,
                angle_milli: 0,
            },
            phase: GatePhase::SitePreparation,
            components_delivered: BTreeMap::new(),
            fabricator_ship_id: ShipId("ship_builder_1".into()),
            progress_work: 0,
        };
        round_trip(&g);
    }

    #[test]
    fn reservation_round_trip() {
        let r = Reservation {
            id: ReservationId("res_1".into()),
            ship_id: ShipId("ship_transport_1".into()),
            supply_source: InventorySourceRef::Station {
                station_id: StationId("hub_haven".into()),
            },
            demand_destination: InventoryDestinationRef::Station {
                station_id: StationId("hub_haven".into()),
            },
            resource: ResourceType::Metals,
            amount: 200,
            state: ReservationState::AwaitingPickup,
            tick_assigned: 0,
            pickup_expiry_tick: None,
        };
        round_trip(&r);
    }

    #[test]
    fn bottleneck_tracker_round_trip() {
        let b = BottleneckTracker {
            deliveries_by_tick: vec![0u32; 600],
            cursor: 0,
            rolling_delivered: 0,
            consecutive_deficit_ticks: 0,
            consecutive_clear_ticks: 0,
            warning_active: false,
        };
        round_trip(&b);
    }

    #[test]
    fn rng_state_round_trip() {
        round_trip(&RNGState {
            words: [1, 2, 3, 4],
        });
    }

    #[test]
    fn id_counters_round_trip() {
        round_trip(&IdCounters {
            ship: 0,
            station: 0,
            build_order: 0,
            reservation: 0,
            salvage: 0,
            survey_order: 0,
        });
    }

    #[test]
    fn command_record_round_trip() {
        let r = CommandRecord {
            id: "cmd_001".to_string(),
            expected_tick: Some(42),
            effective_tick: 42,
            server_sequence: 1,
            application_boundary: CommandApplicationBoundary::ScheduledTick,
            command: ReplayableGameCommand::QueueBuildShip {
                hub_id: StationId("hub_haven".into()),
                role: ShipRole::Construction,
                tier: 1,
            },
            outcome: CommandOutcome::Applied,
            result: Some(CommandResult::None),
            rejection: None,
        };
        round_trip(&r);
    }

    #[test]
    fn game_state_round_trip() {
        let gs = GameState {
            schema_version: 1,
            content_version: "1.0".to_string(),
            lifecycle: GameLifecycle::Unloaded,
            tick: 0,
            next_server_sequence: 1,
            next_event_sequence: 1,
            id_counters: IdCounters {
                ship: 0,
                station: 0,
                build_order: 0,
                reservation: 0,
                salvage: 0,
                survey_order: 0,
            },
            celestial_bodies: BTreeMap::new(),
            stations: BTreeMap::new(),
            ships: BTreeMap::new(),
            research_projects: BTreeMap::new(),
            survey_orders: BTreeMap::new(),
            completed_techs: BTreeSet::new(),
            build_orders: BTreeMap::new(),
            salvage_caches: BTreeMap::new(),
            gate_build: None,
            logistics_reservations: BTreeMap::new(),
            bottleneck_trackers: BTreeMap::new(),
            rng_state: RNGState {
                words: [1, 2, 3, 4],
            },
            command_log: Vec::new(),
        };
        round_trip(&gs);
    }

    #[test]
    fn game_snapshot_round_trip() {
        let gs = GameState {
            schema_version: 1,
            content_version: "1.0".to_string(),
            lifecycle: GameLifecycle::Unloaded,
            tick: 0,
            next_server_sequence: 1,
            next_event_sequence: 1,
            id_counters: IdCounters {
                ship: 0,
                station: 0,
                build_order: 0,
                reservation: 0,
                salvage: 0,
                survey_order: 0,
            },
            celestial_bodies: BTreeMap::new(),
            stations: BTreeMap::new(),
            ships: BTreeMap::new(),
            research_projects: BTreeMap::new(),
            survey_orders: BTreeMap::new(),
            completed_techs: BTreeSet::new(),
            build_orders: BTreeMap::new(),
            salvage_caches: BTreeMap::new(),
            gate_build: None,
            logistics_reservations: BTreeMap::new(),
            bottleneck_trackers: BTreeMap::new(),
            rng_state: RNGState {
                words: [1, 2, 3, 4],
            },
            command_log: Vec::new(),
        };
        let snap = GameSnapshot {
            protocol_version: "1.0".to_string(),
            latest_event_sequence: 0,
            state: gs,
        };
        round_trip(&snap);
    }
}
