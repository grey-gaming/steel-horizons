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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn round_trip<T>(val: &T)
    where
        T: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug + PartialEq,
    {
        let json = serde_json::to_string(val).unwrap();
        let back: T = serde_json::from_str(&json).unwrap();
        assert_eq!(*val, back);
    }

    /// Load a JSON file from the content directory.
    fn load_json<T: serde::de::DeserializeOwned>(path: &str) -> T {
        let content_dir = std::env::var("CARGO_MANIFEST_DIR")
            .map(|p| PathBuf::from(p).parent().unwrap().join("content"))
            .unwrap_or_else(|_| PathBuf::from("content"));
        let full_path = content_dir.join(path);
        let data = std::fs::read_to_string(&full_path)
            .unwrap_or_else(|e| panic!("Cannot read {}: {}", full_path.display(), e));
        serde_json::from_str(&data)
            .unwrap_or_else(|e| panic!("Cannot parse {}: {}", full_path.display(), e))
    }

    /// Verify definitions.v1.json parses and round-trips.
    #[test]
    fn definitions_v1_round_trip() {
        let defs: DefinitionsCatalog = load_json("definitions.v1.json");
        round_trip(&defs);
    }

    /// Verify starting_system.v1.json parses and round-trips.
    #[test]
    fn starting_system_v1_round_trip() {
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        round_trip(&scenario);
    }

    /// Verify ContentCatalog round-trip from both files.
    #[test]
    fn content_catalog_round_trip() {
        let defs: DefinitionsCatalog = load_json("definitions.v1.json");
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let catalog = ContentCatalog {
            definitions: defs,
            starting_system: scenario,
        };
        round_trip(&catalog);
    }

    /// Exact record counts per GDD 14.
    #[test]
    fn definitions_record_counts() {
        let defs: DefinitionsCatalog = load_json("definitions.v1.json");

        // 27 recipes: 11 refining (8 forward + 3 inverse) + 8 assembly + 8 disassembly
        assert_eq!(defs.recipes.len(), 27, "expected 27 recipes");

        // 23 technologies
        assert_eq!(defs.technologies.len(), 23, "expected 23 technologies");

        // 12 ship definitions: 4 cargo + 4 construction + 4 research
        assert_eq!(defs.ships.len(), 12, "expected 12 ship definitions");

        // 20 station definitions: 5 types × 4 tiers
        assert_eq!(defs.stations.len(), 20, "expected 20 station definitions");
    }

    /// Verify recipe IDs are unique.
    #[test]
    fn recipe_ids_unique() {
        let defs: DefinitionsCatalog = load_json("definitions.v1.json");
        let mut seen = std::collections::BTreeSet::new();
        for r in &defs.recipes {
            assert!(seen.insert(&r.id), "duplicate recipe id: {}", r.id.0);
        }
    }

    /// Verify technology IDs are unique and DAG prerequisites reference existing IDs.
    #[test]
    fn tech_ids_unique_and_prereqs_exist() {
        let defs: DefinitionsCatalog = load_json("definitions.v1.json");
        let mut seen = std::collections::BTreeSet::new();
        for t in &defs.technologies {
            assert!(seen.insert(&t.id), "duplicate tech id: {}", t.id.0);
        }
        // Verify all prerequisites reference known techs
        for t in &defs.technologies {
            for prereq in &t.prerequisites {
                assert!(
                    seen.contains(prereq),
                    "tech {} references unknown prerequisite {}",
                    t.id.0,
                    prereq.0
                );
            }
        }
    }

    /// Verify ship definitions have unique name per role+tier.
    #[test]
    fn ship_definitions_unique() {
        let defs: DefinitionsCatalog = load_json("definitions.v1.json");
        let mut seen = std::collections::BTreeSet::new();
        for s in &defs.ships {
            assert!(
                seen.insert((s.role, s.tier)),
                "duplicate ship role+tier: {:?} / {}",
                s.role,
                s.tier
            );
        }
    }

    /// Verify station definitions have unique type+tier.
    #[test]
    fn station_definitions_unique() {
        let defs: DefinitionsCatalog = load_json("definitions.v1.json");
        let mut seen = std::collections::BTreeSet::new();
        for s in &defs.stations {
            assert!(
                seen.insert((s.station_type, s.tier)),
                "duplicate station type+tier: {:?} / {}",
                s.station_type,
                s.tier
            );
        }
    }

    /// Verify celestial body IDs are unique and count matches GDD 14 (7 bodies).
    #[test]
    fn starting_bodies_count() {
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        assert_eq!(
            scenario.celestial_bodies.len(),
            7,
            "expected 7 celestial bodies"
        );
    }

    /// Verify total station slots across all bodies (19).
    #[test]
    fn total_station_slots() {
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let mut total_slots: u32 = 0;
        for body in scenario.celestial_bodies.values() {
            for count in &body.slot_counts {
                total_slots += *count as u32;
            }
        }
        assert_eq!(total_slots, 19, "expected 19 total station slots");
    }

    /// Verify Hub Haven has the correct authored output buffers.
    #[test]
    fn hub_haven_output_buffers() {
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let hub = scenario
            .stations
            .get(&StationId("hub_haven".into()))
            .expect("hub_haven should exist");
        assert_eq!(hub.station_type, StationType::Hub);
        assert_eq!(hub.tier, 1);
        assert_eq!(hub.input_buffers.len(), 0);
        assert_eq!(hub.output_buffers.len(), 7, "expected 7 output buffers");

        // Verify output buffer resources in ResourceType order
        let resources: Vec<ResourceType> = hub.output_buffers.iter().map(|b| b.resource).collect();
        assert_eq!(resources[0], ResourceType::StructuralFrame);
        assert_eq!(resources[1], ResourceType::PowerCore);
        assert_eq!(resources[2], ResourceType::ControlSystem);
        assert_eq!(resources[3], ResourceType::DriveAssembly);
        assert_eq!(resources[4], ResourceType::CargoModule);
        assert_eq!(resources[5], ResourceType::ResearchLab);
        assert_eq!(resources[6], ResourceType::ConstructionBay);

        // Verify output buffer quantities
        for b in &hub.output_buffers {
            assert_eq!(
                b.current, b.max,
                "current should equal max for output buffers"
            );
        }
    }

    /// Verify the starting ship Builder-1.
    #[test]
    fn starting_ship_builder() {
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let ship = scenario
            .ships
            .get(&ShipId("ship_builder_1".into()))
            .expect("ship_builder_1 should exist");
        assert_eq!(ship.role, ShipRole::Construction);
        assert_eq!(ship.tier, 1);
        assert_eq!(ship.display_name, "Builder-1");
        assert_eq!(ship.docked_at.as_ref().unwrap().0.as_str(), "hub_haven");
        assert_eq!(ship.fuel, 100);
        assert_eq!(ship.max_fuel, 100);
        assert_eq!(ship.state, ShipState::Idle);
        assert!(matches!(ship.job, ShipJob::Idle));
    }

    /// Verify starting techs are the four tier-0 techs.
    #[test]
    fn starting_techs() {
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        assert_eq!(scenario.completed_techs.len(), 4);
        assert!(scenario
            .completed_techs
            .contains(&TechId("basic_construction".into())));
        assert!(scenario
            .completed_techs
            .contains(&TechId("basic_refining".into())));
        assert!(scenario
            .completed_techs
            .contains(&TechId("basic_power".into())));
        assert!(scenario
            .completed_techs
            .contains(&TechId("basic_control".into())));
    }

    /// Verify starting counters and metadata.
    #[test]
    fn starting_metadata() {
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        assert_eq!(scenario.id.0.as_str(), "starting_system");
        assert_eq!(scenario.content_version, "v1");
        assert_eq!(scenario.lifecycle, GameLifecycle::Paused);
        assert_eq!(scenario.tick, 0);
        assert_eq!(scenario.next_server_sequence, 1);
        assert_eq!(scenario.next_event_sequence, 1);
        assert_eq!(scenario.id_counters.ship, 1);
        assert_eq!(scenario.id_counters.station, 1);
        assert_eq!(scenario.id_counters.build_order, 1);
        assert_eq!(scenario.id_counters.reservation, 1);
        assert_eq!(scenario.id_counters.salvage, 1);
        assert_eq!(scenario.id_counters.survey_order, 1);
    }

    /// Verify the Gate definition has all required fields.
    #[test]
    fn gate_definition() {
        let defs: DefinitionsCatalog = load_json("definitions.v1.json");
        let gate = &defs.gate;
        assert_eq!(gate.site_position.lane_id, LaneId::Fringe);
        assert_eq!(gate.site_position.radius_units, 4000);
        assert_eq!(gate.site_position.angle_milli, 0);
        assert_eq!(gate.minimum_fabricator_tier, 4);
        assert_eq!(gate.logistics_priority, 100);
        assert_eq!(gate.transfer_berths, 1);
        assert_eq!(gate.phases.len(), 4);
        assert!(gate
            .required_techs
            .contains(&TechId("system_bridge".into())));

        // Verify manifest contains expected items
        assert_eq!(*gate.manifest.get(&ResourceType::GateNode).unwrap(), 8);
        assert_eq!(
            *gate.manifest.get(&ResourceType::StructuralFrame).unwrap(),
            1
        );
        assert_eq!(*gate.manifest.get(&ResourceType::PowerCore).unwrap(), 1);
        assert_eq!(*gate.manifest.get(&ResourceType::ControlSystem).unwrap(), 1);
        assert_eq!(*gate.manifest.get(&ResourceType::ReactorRods).unwrap(), 1);
    }

    /// Verify that every recipe has a valid required_tech or is None.
    #[test]
    fn recipe_techs_valid() {
        let defs: DefinitionsCatalog = load_json("definitions.v1.json");
        let tech_ids: std::collections::BTreeSet<&TechId> =
            defs.technologies.iter().map(|t| &t.id).collect();
        for r in &defs.recipes {
            if let Some(ref tech) = r.required_tech {
                assert!(
                    tech_ids.contains(tech),
                    "recipe {} requires unknown tech {}",
                    r.id.0,
                    tech.0
                );
            }
        }
    }
}
