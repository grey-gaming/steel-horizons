//! Serialized content definition DTOs.
//!
//! Every struct in this module is a Serde-only data-transfer object matching
//! GDD 13 §Content Definitions and GDD 14.  No runtime simulation logic
//! lives here — that belongs in the loader and validator modules (P1-03
//! onward).
//!
//! ## Authoritative references
//!
//! - GDD 13 §Content Definitions
//! - GDD 13 §Schema Generation Ownership
//! - GDD 14 §Starting System Bodies, §Starting State, §Canonical Refining
//!   Recipes, §Canonical Component Recipes, §Canonical Technology
//!   Definitions, §Canonical Ship Definitions, §Canonical Station
//!   Definitions, §Space Gate Definition, §Solvability Budget

#![allow(missing_docs)]

#[allow(unused_imports)]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::id::*;
use crate::state::*;
use crate::types::*;

// ─── Content definition types ─────────────────────────────────────────

/// Facility requirement for a recipe (station type, minimum tier, cycle length).
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct FacilityRequirement {
    pub station_type: StationType,
    pub minimum_tier: u8,
    pub cycle_ticks: u32,
}

/// A recipe definition (refining, assembly, or disassembly).
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct RecipeDefinition {
    pub id: RecipeId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_tech: Option<TechId>,
    pub facilities: Vec<FacilityRequirement>,
    #[serde(default)]
    pub inputs: BTreeMap<ResourceType, u32>,
    #[serde(default)]
    pub outputs: BTreeMap<ResourceType, u32>,
}

/// A technology definition (tier, prerequisites, cost, duration, unlocks).
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct TechDefinition {
    pub id: TechId,
    pub tier: u8,
    #[serde(default)]
    pub prerequisites: Vec<TechId>,
    #[serde(default)]
    pub costs: BTreeMap<ResourceType, u32>,
    pub duration_ticks: u64,
    #[serde(default)]
    pub mechanic_unlocks: Vec<MechanicUnlock>,
}

/// Mechanic unlock granted by a technology.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum MechanicUnlock {
    SurveyDepth { max_depth: u8 },
    AsteroidBeltOperations,
    LifeSupportFuelFactor { numerator: u8, denominator: u8 },
    GateSiteVisibility,
    GateAssembly,
}

/// Ship statistics (role-dependent — see GDD 13 §Content Definitions).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ShipStats {
    pub cargo_capacity: u32,
    pub build_cargo_capacity: u32,
    pub speed_milli: u32,
    pub max_fuel: u32,
    pub base_mass: u32,
    pub build_work_per_tick: u16,
    pub survey_work_per_tick: u16,
    pub max_survey_depth: u8,
}

/// Station statistics (type-dependent — see GDD 13 §Content Definitions).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct StationStats {
    pub docks: u8,
    pub cargo_capacity: u32,
    pub fuel_capacity: u32,
    pub production_slots: u8,
    pub max_targets: u8,
    pub extraction_per_target_per_10_ticks: u8,
    pub research_projects: u8,
    pub shipyard_slots: u8,
    pub component_slots: u8,
}

/// A ship definition in the content catalog.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ShipDefinition {
    pub role: ShipRole,
    pub tier: u8,
    pub name: String,
    pub stats: ShipStats,
    pub build_work: u32,
    #[serde(default)]
    pub component_cost: BTreeMap<ResourceType, u32>,
    pub required_tech: TechId,
}

/// A station definition in the content catalog.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct StationDefinition {
    pub station_type: StationType,
    pub tier: u8,
    pub stats: StationStats,
    pub build_work: u32,
    #[serde(default)]
    pub component_cost: BTreeMap<ResourceType, u32>,
    pub required_tech: TechId,
}

/// Authored default constants for new stations, buffers, work, etc.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct AuthoredDefaults {
    pub new_station_priority: u8,
    pub general_buffer_preferred_maximum: u32,
    pub input_demand_threshold: u8,
    pub output_export_threshold: u8,
    pub fuel_demand_threshold: u8,
    pub fuel_export_threshold: u8,
    pub mining_retune_ticks: u16,
    pub upgrade_work_per_tier: u32,
    pub demolition_work: u32,
    pub survey_depth_work: [u32; 3],
    pub hub_shipyard_work_per_tick: u16,
}

/// One phase of the Space Gate assembly process.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct GatePhaseDefinition {
    pub phase: GatePhase,
    pub work: u32,
    #[serde(default)]
    pub required_deliveries: BTreeMap<ResourceType, u32>,
    #[serde(default)]
    pub completion_consumption: BTreeMap<ResourceType, u32>,
}

/// The authored Space Gate definition.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct GateDefinition {
    pub site_position: SystemPosition,
    #[serde(default)]
    pub manifest: BTreeMap<ResourceType, u32>,
    pub required_techs: Vec<TechId>,
    pub required_fabricator_role: ShipRole,
    pub minimum_fabricator_tier: u8,
    pub logistics_priority: u8,
    pub transfer_berths: u8,
    pub phases: Vec<GatePhaseDefinition>,
}

// ─── Root content containers ──────────────────────────────────────────

/// The full definitions catalog (recipes, technologies, ships, stations, gate).
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct DefinitionsCatalog {
    pub content_version: String,
    pub defaults: AuthoredDefaults,
    #[serde(default)]
    pub recipes: Vec<RecipeDefinition>,
    #[serde(default)]
    pub technologies: Vec<TechDefinition>,
    #[serde(default)]
    pub ships: Vec<ShipDefinition>,
    #[serde(default)]
    pub stations: Vec<StationDefinition>,
    pub gate: GateDefinition,
}

/// The starting scenario (initial game state before any player action).
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct StartingScenario {
    pub id: ScenarioId,
    pub content_version: String,
    pub lifecycle: GameLifecycle,
    pub tick: u64,
    pub next_server_sequence: u64,
    pub next_event_sequence: u64,
    pub id_counters: IdCounters,
    pub rng_state: RNGState,
    #[serde(default)]
    pub celestial_bodies: BTreeMap<BodyId, CelestialBody>,
    #[serde(default)]
    pub stations: BTreeMap<StationId, Station>,
    #[serde(default)]
    pub ships: BTreeMap<ShipId, Ship>,
    #[serde(default)]
    pub completed_techs: BTreeSet<TechId>,
}

/// The runtime aggregate of content definitions and starting state.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ContentCatalog {
    pub definitions: DefinitionsCatalog,
    pub starting_system: StartingScenario,
}
