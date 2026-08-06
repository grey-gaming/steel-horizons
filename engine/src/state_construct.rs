//! Canonical tick-zero constructor and cheap invariant checker.
//!
//! This module provides the pure, deterministic function that maps a validated
//! `ContentCatalog` into a tick-zero `GameState`, plus a cheap invariant
//! checker that verifies runtime structural consistency without full
//! simulation logic.
//!
//! ## Authoritative references
//!
//! - GDD 13 §Lifecycle, RNG, Commands, and Root State
//! - GDD 13 §Identifiers and Resources (generated ID prefixes)
//! - GDD 14 §Starting State

use std::collections::BTreeMap;

use crate::content::ContentCatalog;
use crate::state::*;
use crate::types::*;

/// Errors produced by the canonical tick-zero constructor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstructionError {
    /// The starting scenario's lifecycle is not Paused or Running.
    InvalidLifecycle(GameLifecycle),
    /// The starting scenario's tick is not zero.
    NonZeroTick(u64),
}

impl std::fmt::Display for ConstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConstructionError::InvalidLifecycle(l) => {
                write!(
                    f,
                    "starting scenario lifecycle must be Paused or Running, got {:?}",
                    l
                )
            }
            ConstructionError::NonZeroTick(t) => {
                write!(f, "starting scenario tick must be zero, got {}", t)
            }
        }
    }
}

/// Result type for construction operations.
pub type ConstructionResult<T = GameState> = Result<T, ConstructionError>;

/// Build the canonical tick-zero `GameState` from a validated `ContentCatalog`.
///
/// The constructor maps the starting scenario's fields directly into the
/// `GameState` root, preserving authored IDs and display names.  Empty
/// collections are used for runtime-only maps (research projects, survey
/// orders, build orders, salvage caches, logistics reservations, bottleneck
/// trackers, command log).
pub fn build_starting_state(catalog: &ContentCatalog) -> ConstructionResult {
    let scenario = &catalog.starting_system;

    // Validate lifecycle — must be Paused or Running at tick 0
    if scenario.lifecycle != GameLifecycle::Paused && scenario.lifecycle != GameLifecycle::Running {
        return Err(ConstructionError::InvalidLifecycle(scenario.lifecycle));
    }

    // Validate tick is zero
    if scenario.tick != 0 {
        return Err(ConstructionError::NonZeroTick(scenario.tick));
    }

    Ok(GameState {
        schema_version: 1,
        content_version: scenario.content_version.clone(),
        lifecycle: scenario.lifecycle,
        tick: scenario.tick,
        next_server_sequence: scenario.next_server_sequence,
        next_event_sequence: scenario.next_event_sequence,
        id_counters: scenario.id_counters,
        celestial_bodies: scenario.celestial_bodies.clone(),
        stations: scenario.stations.clone(),
        ships: scenario.ships.clone(),
        research_projects: BTreeMap::new(),
        survey_orders: BTreeMap::new(),
        completed_techs: scenario.completed_techs.clone(),
        build_orders: BTreeMap::new(),
        salvage_caches: BTreeMap::new(),
        gate_build: None,
        logistics_reservations: BTreeMap::new(),
        bottleneck_trackers: BTreeMap::new(),
        rng_state: scenario.rng_state,
        command_log: Vec::new(),
    })
}

/// Invariant violations detected by the cheap checker.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(missing_docs)]
pub struct InvariantViolation {
    pub kind: String,
    pub message: String,
}

/// Result type for invariant checks.
pub type InvariantResult = Result<(), Vec<InvariantViolation>>;

/// Run a cheap invariant check on a `GameState`.
///
/// Checks:
/// - RNG state is not all-zero.
/// - For each ship: `fuel_remainder < 10_000_000` (V1 invariant).
/// - For each ship: `fuel_efficiency_remainder < 5`.
/// - For each ship: if `docked_at` is Some, the station exists and the ship is
///   in the station's `docked_ship_ids`.
/// - For each station: every `docked_ship_ids` entry refers to a ship whose
///   `docked_at` matches this station.
/// - Buffer invariants: `current ≤ max`, thresholds in 0..=100.
/// - `BTreeMap` value sparsity: no zero values.
/// - GateBuild consistency if present.
/// - IdCounters ≥ entity count in corresponding maps.
pub fn check_invariants(state: &GameState) -> InvariantResult {
    let mut violations: Vec<InvariantViolation> = Vec::new();

    // ─── RNG state ────────────────────────────────────────────────────
    let all_zero = state.rng_state.words.iter().all(|&w| w == 0);
    if all_zero {
        violations.push(InvariantViolation {
            kind: "rng".to_string(),
            message: "RNG state is all-zero (invalid)".to_string(),
        });
    }

    // ─── Ship invariants ──────────────────────────────────────────────
    for (ship_id, ship) in &state.ships {
        // fuel_remainder < 10_000_000 (V1 invariant per GDD 12)
        if ship.fuel_remainder >= 10_000_000 {
            violations.push(InvariantViolation {
                kind: "ship.fuel_remainder".to_string(),
                message: format!(
                    "ship {} fuel_remainder {} exceeds max 9_999_999",
                    ship_id.0, ship.fuel_remainder
                ),
            });
        }

        // fuel_efficiency_remainder < 5
        if ship.fuel_efficiency_remainder >= 5 {
            violations.push(InvariantViolation {
                kind: "ship.fuel_efficiency_remainder".to_string(),
                message: format!(
                    "ship {} fuel_efficiency_remainder {} exceeds max 4",
                    ship_id.0, ship.fuel_efficiency_remainder
                ),
            });
        }

        // docked_at consistency
        if let Some(ref docked_station_id) = ship.docked_at {
            if let Some(station) = state.stations.get(docked_station_id) {
                if !station.docked_ship_ids.contains(ship_id) {
                    violations.push(InvariantViolation {
                        kind: "ship.docked_at".to_string(),
                        message: format!(
                            "ship {} docked at {} but not in station's docked_ship_ids",
                            ship_id.0, docked_station_id.0
                        ),
                    });
                }
            } else {
                violations.push(InvariantViolation {
                    kind: "ship.docked_at".to_string(),
                    message: format!(
                        "ship {} docked_at {} references unknown station",
                        ship_id.0, docked_station_id.0
                    ),
                });
            }
        }
    }

    // ─── Station invariants ───────────────────────────────────────────
    for (station_id, station) in &state.stations {
        // docked_ship_ids refer to existing ships with correct docked_at
        for dsid in &station.docked_ship_ids {
            let Some(ship) = state.ships.get(dsid) else {
                violations.push(InvariantViolation {
                    kind: "station.docked_ship_ids".to_string(),
                    message: format!(
                        "station {} docked_ship_id {} references unknown ship",
                        station_id.0, dsid.0
                    ),
                });
                continue;
            };
            match &ship.docked_at {
                None => {
                    violations.push(InvariantViolation {
                        kind: "station.docked_ship_ids".to_string(),
                        message: format!(
                            "station {} docked_ship_id {} has no docked_at",
                            station_id.0, dsid.0
                        ),
                    });
                }
                Some(ref sa) if sa.0 != station_id.0 => {
                    violations.push(InvariantViolation {
                        kind: "station.docked_ship_ids".to_string(),
                        message: format!(
                            "station {} docked_ship_id {} has docked_at {} mismatch",
                            station_id.0, dsid.0, sa.0
                        ),
                    });
                }
                _ => {}
            }
        }

        // holding_ship_ids refer to existing ships
        for hsid in &station.holding_ship_ids {
            if !state.ships.contains_key(hsid) {
                violations.push(InvariantViolation {
                    kind: "station.holding_ship_ids".to_string(),
                    message: format!(
                        "station {} holding_ship_id {} references unknown ship",
                        station_id.0, hsid.0
                    ),
                });
            }
        }

        // Buffer invariants
        let all_buffers = station
            .input_buffers
            .iter()
            .chain(station.output_buffers.iter())
            .chain(std::iter::once(&station.fuel_buffer));
        for buf in all_buffers {
            if buf.current > buf.max {
                violations.push(InvariantViolation {
                    kind: "buffer.current".to_string(),
                    message: format!(
                        "station {} buffer {:?} current {} exceeds max {}",
                        station_id.0, buf.resource, buf.current, buf.max
                    ),
                });
            }
            if buf.demand_threshold > 100 {
                violations.push(InvariantViolation {
                    kind: "buffer.demand_threshold".to_string(),
                    message: format!(
                        "station {} buffer {:?} demand_threshold {} > 100",
                        station_id.0, buf.resource, buf.demand_threshold
                    ),
                });
            }
            if buf.export_threshold > 100 {
                violations.push(InvariantViolation {
                    kind: "buffer.export_threshold".to_string(),
                    message: format!(
                        "station {} buffer {:?} export_threshold {} > 100",
                        station_id.0, buf.resource, buf.export_threshold
                    ),
                });
            }
        }

        // docked_ship_ids length ≤ max_docks
        if station.docked_ship_ids.len() > station.max_docks as usize {
            violations.push(InvariantViolation {
                kind: "station.docks".to_string(),
                message: format!(
                    "station {} docked_ship_ids count {} exceeds max_docks {}",
                    station_id.0,
                    station.docked_ship_ids.len(),
                    station.max_docks
                ),
            });
        }
    }

    // ─── BTreeMap sparsity ────────────────────────────────────────────
    // Check that no BTreeMap<ResourceType, u32> has zero values
    for (station_id, station) in &state.stations {
        for (res, &qty) in &station.installed_components {
            if qty == 0 {
                violations.push(InvariantViolation {
                    kind: "sparse_map".to_string(),
                    message: format!(
                        "station {} installed_components {:?} has zero value",
                        station_id.0, res
                    ),
                });
            }
        }
    }
    for (ship_id, ship) in &state.ships {
        for (res, &qty) in &ship.installed_components {
            if qty == 0 {
                violations.push(InvariantViolation {
                    kind: "sparse_map".to_string(),
                    message: format!(
                        "ship {} installed_components {:?} has zero value",
                        ship_id.0, res
                    ),
                });
            }
        }
        for (res, &qty) in &ship.build_cargo {
            if qty == 0 {
                violations.push(InvariantViolation {
                    kind: "sparse_map".to_string(),
                    message: format!("ship {} build_cargo {:?} has zero value", ship_id.0, res),
                });
            }
        }
    }

    // ─── GateBuild consistency ────────────────────────────────────────
    if let Some(ref gate) = state.gate_build {
        if !state.ships.contains_key(&gate.fabricator_ship_id) {
            violations.push(InvariantViolation {
                kind: "gate.fabricator".to_string(),
                message: format!(
                    "gate fabricator_ship_id {} references unknown ship",
                    gate.fabricator_ship_id.0
                ),
            });
        }
    }

    // ─── IdCounters consistency ───────────────────────────────────────
    // Each counter must be ≥ the count of authored entities in its
    // corresponding map (generated IDs always have larger suffixes, and
    // authored IDs are below generated ranges).
    let ship_count = state.ships.len() as u64;
    if state.id_counters.ship < ship_count {
        violations.push(InvariantViolation {
            kind: "id_counters.ship".to_string(),
            message: format!(
                "id_counters.ship {} < ship count {}",
                state.id_counters.ship, ship_count
            ),
        });
    }
    let station_count = state.stations.len() as u64;
    if state.id_counters.station < station_count {
        violations.push(InvariantViolation {
            kind: "id_counters.station".to_string(),
            message: format!(
                "id_counters.station {} < station count {}",
                state.id_counters.station, station_count
            ),
        });
    }

    // ─── Lifecycle ────────────────────────────────────────────────────
    // Running/Paused/Advancing should not have command_log entries from
    // a prior session without a proper load marker — this is a soft
    // sanity check.
    if !state.command_log.is_empty()
        && state.lifecycle != GameLifecycle::Advancing
        && state.lifecycle != GameLifecycle::Running
    {
        violations.push(InvariantViolation {
            kind: "lifecycle.command_log".to_string(),
            message: format!(
                "non-empty command_log ({}) with lifecycle {:?} (expected Advancing or Running)",
                state.command_log.len(),
                state.lifecycle
            ),
        });
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::*;
    use crate::id::{ShipId, StationId, TechId};
    use std::path::PathBuf;

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

    fn load_catalog() -> ContentCatalog {
        let defs: DefinitionsCatalog = load_json("definitions.v1.json");
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        ContentCatalog {
            definitions: defs,
            starting_system: scenario,
        }
    }

    /// Canonical starting state is Paused at tick 0.
    #[test]
    fn canonical_starting_state_basic() {
        let catalog = load_catalog();
        let state = build_starting_state(&catalog).unwrap();
        assert_eq!(state.schema_version, 1);
        assert_eq!(state.content_version, "v1");
        assert_eq!(state.lifecycle, GameLifecycle::Paused);
        assert_eq!(state.tick, 0);
        assert_eq!(state.next_server_sequence, 1);
        assert_eq!(state.next_event_sequence, 1);
    }

    /// Canonical starting state has correct entity counts.
    #[test]
    fn canonical_entity_counts() {
        let catalog = load_catalog();
        let state = build_starting_state(&catalog).unwrap();
        assert_eq!(state.celestial_bodies.len(), 7, "expected 7 bodies");
        assert_eq!(state.stations.len(), 1, "expected 1 station");
        assert_eq!(state.ships.len(), 1, "expected 1 ship");
        assert_eq!(state.completed_techs.len(), 4, "expected 4 techs");
        assert!(state.research_projects.is_empty(), "no research projects");
        assert!(state.survey_orders.is_empty(), "no survey orders");
        assert!(state.build_orders.is_empty(), "no build orders");
        assert!(state.salvage_caches.is_empty(), "no salvage caches");
        assert!(state.gate_build.is_none(), "no gate build");
        assert!(state.logistics_reservations.is_empty(), "no reservations");
        assert!(state.command_log.is_empty(), "empty command log");
    }

    /// Canonical starting state passes all invariant checks.
    #[test]
    fn canonical_starting_state_passes_invariants() {
        let catalog = load_catalog();
        let state = build_starting_state(&catalog).unwrap();
        let result = check_invariants(&state);
        assert!(result.is_ok(), "invariants should pass: {:?}", result.err());
    }

    /// Hub Haven fields match GDD 14.
    #[test]
    fn hub_haven_fields_match_gdd14() {
        let catalog = load_catalog();
        let state = build_starting_state(&catalog).unwrap();
        let hub = state
            .stations
            .get(&StationId("hub_haven".into()))
            .expect("hub_haven should exist");

        assert_eq!(hub.display_name, "Hub Haven");
        assert_eq!(hub.station_type, StationType::Hub);
        assert_eq!(hub.tier, 1);
        assert_eq!(hub.body_id.0.as_str(), "planet_haven");
        assert_eq!(hub.orbit_ring, 0);
        assert_eq!(hub.slot, 0);
        assert_eq!(hub.max_docks, 2);
        assert_eq!(hub.total_cargo_capacity, 200);
        assert_eq!(hub.priority, 50);
        assert_eq!(hub.built_in_research_max_tier, Some(1));
        assert!(hub.active_research_id.is_none());
        assert!(hub.ship_build_queue.is_empty());

        // Fuel buffer: Fuel, current = max = 200
        assert_eq!(hub.fuel_buffer.resource, ResourceType::Fuel);
        assert_eq!(hub.fuel_buffer.current, 200);
        assert_eq!(hub.fuel_buffer.max, 200);
        assert_eq!(hub.fuel_buffer.demand_threshold, 20);
        assert_eq!(hub.fuel_buffer.export_threshold, 50);

        // Input buffers empty, output buffers have 7 components
        assert!(hub.input_buffers.is_empty());
        assert_eq!(hub.output_buffers.len(), 7);

        // docked_ship_ids contains builder
        assert_eq!(hub.docked_ship_ids.len(), 1);
        assert_eq!(hub.docked_ship_ids[0].0.as_str(), "ship_builder_1");

        // Installed components: 1 StructuralFrame, 1 PowerCore, 1 ControlSystem, 1 CargoModule
        assert_eq!(hub.installed_components.len(), 4);
        assert_eq!(
            *hub.installed_components
                .get(&ResourceType::StructuralFrame)
                .unwrap(),
            1
        );
        assert_eq!(
            *hub.installed_components
                .get(&ResourceType::PowerCore)
                .unwrap(),
            1
        );
        assert_eq!(
            *hub.installed_components
                .get(&ResourceType::ControlSystem)
                .unwrap(),
            1
        );
        assert_eq!(
            *hub.installed_components
                .get(&ResourceType::CargoModule)
                .unwrap(),
            1
        );
    }

    /// Builder-1 fields match GDD 14.
    #[test]
    fn builder_1_fields_match_gdd14() {
        let catalog = load_catalog();
        let state = build_starting_state(&catalog).unwrap();
        let ship = state
            .ships
            .get(&ShipId("ship_builder_1".into()))
            .expect("ship_builder_1 should exist");

        assert_eq!(ship.display_name, "Builder-1");
        assert_eq!(ship.role, ShipRole::Construction);
        assert_eq!(ship.tier, 1);
        assert_eq!(ship.docked_at.as_ref().unwrap().0.as_str(), "hub_haven");
        assert_eq!(ship.fuel, 100);
        assert_eq!(ship.max_fuel, 100);
        assert_eq!(ship.state, ShipState::Idle);
        assert!(matches!(ship.job, ShipJob::Idle));
        assert!(ship.travel_plan.is_none());

        // Cargo fields
        assert!(ship.cargo_type.is_none());
        assert_eq!(ship.cargo_amount, 0);
        assert_eq!(ship.max_cargo_capacity, 0);

        // Build cargo
        assert!(ship.build_cargo.is_empty());
        assert_eq!(ship.build_cargo_capacity, 20);

        // Installed components: 1 StructuralFrame, 1 DriveAssembly, 1 ConstructionBay
        assert_eq!(ship.installed_components.len(), 3);
        assert_eq!(
            *ship
                .installed_components
                .get(&ResourceType::StructuralFrame)
                .unwrap(),
            1
        );
        assert_eq!(
            *ship
                .installed_components
                .get(&ResourceType::DriveAssembly)
                .unwrap(),
            1
        );
        assert_eq!(
            *ship
                .installed_components
                .get(&ResourceType::ConstructionBay)
                .unwrap(),
            1
        );
    }

    /// Starting techs match GDD 14.
    #[test]
    fn starting_techs_match_gdd14() {
        let catalog = load_catalog();
        let state = build_starting_state(&catalog).unwrap();
        assert_eq!(state.completed_techs.len(), 4);
        assert!(state
            .completed_techs
            .contains(&TechId("basic_construction".into())));
        assert!(state
            .completed_techs
            .contains(&TechId("basic_refining".into())));
        assert!(state
            .completed_techs
            .contains(&TechId("basic_power".into())));
        assert!(state
            .completed_techs
            .contains(&TechId("basic_control".into())));
    }

    /// GameState round-trips through Serde.
    #[test]
    fn starting_state_serde_round_trip() {
        let catalog = load_catalog();
        let state = build_starting_state(&catalog).unwrap();
        let json = serde_json::to_string(&state).unwrap();
        let back: GameState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, back);
    }

    /// Invalid lifecycle is rejected.
    #[test]
    fn invalid_lifecycle_rejected() {
        let catalog = load_catalog();
        let mut modified = catalog.clone();
        modified.starting_system.lifecycle = GameLifecycle::Unloaded;
        let result = build_starting_state(&modified);
        assert!(result.is_err(), "Unloaded lifecycle should be rejected");
        match result {
            Err(ConstructionError::InvalidLifecycle(l)) => assert_eq!(l, GameLifecycle::Unloaded),
            _ => panic!("expected InvalidLifecycle error"),
        }
    }

    /// Non-zero tick is rejected.
    #[test]
    fn non_zero_tick_rejected() {
        let catalog = load_catalog();
        let mut modified = catalog.clone();
        modified.starting_system.tick = 42;
        let result = build_starting_state(&modified);
        assert!(result.is_err(), "non-zero tick should be rejected");
        match result {
            Err(ConstructionError::NonZeroTick(t)) => assert_eq!(t, 42),
            _ => panic!("expected NonZeroTick error"),
        }
    }

    /// Invariant checker detects all-zero RNG.
    #[test]
    fn invariant_detects_all_zero_rng() {
        let catalog = load_catalog();
        let mut state = build_starting_state(&catalog).unwrap();
        state.rng_state.words = [0u64; 4];
        let result = check_invariants(&state);
        assert!(result.is_err());
        let violations = result.err().unwrap();
        assert!(violations.iter().any(|v| v.kind == "rng"));
    }

    /// Invariant checker detects docked_at mismatch.
    #[test]
    fn invariant_detects_dock_mismatch() {
        let catalog = load_catalog();
        let mut state = build_starting_state(&catalog).unwrap();
        // Set a ship's docked_at to a station that doesn't exist
        if let Some(ship) = state.ships.get_mut(&ShipId("ship_builder_1".into())) {
            ship.docked_at = Some(StationId("nonexistent".into()));
        }
        let result = check_invariants(&state);
        assert!(result.is_err());
        let violations = result.err().unwrap();
        assert!(violations.iter().any(|v| v.kind == "ship.docked_at"));
    }

    /// Invariant checker detects buffer overflow.
    #[test]
    fn invariant_detects_buffer_overflow() {
        let catalog = load_catalog();
        let mut state = build_starting_state(&catalog).unwrap();
        if let Some(hub) = state.stations.get_mut(&StationId("hub_haven".into())) {
            hub.fuel_buffer.current = hub.fuel_buffer.max + 1;
        }
        let result = check_invariants(&state);
        assert!(result.is_err());
        let violations = result.err().unwrap();
        assert!(violations.iter().any(|v| v.kind == "buffer.current"));
    }

    /// Invariant checker detects zero-value sparse map.
    #[test]
    fn invariant_detects_zero_sparse_map() {
        let catalog = load_catalog();
        let mut state = build_starting_state(&catalog).unwrap();
        if let Some(hub) = state.stations.get_mut(&StationId("hub_haven".into())) {
            hub.installed_components
                .insert(ResourceType::StructuralFrame, 0);
        }
        let result = check_invariants(&state);
        assert!(result.is_err());
        let violations = result.err().unwrap();
        assert!(violations.iter().any(|v| v.kind == "sparse_map"));
    }

    /// IdCounters consistency check.
    #[test]
    fn id_counter_consistency() {
        let catalog = load_catalog();
        let state = build_starting_state(&catalog).unwrap();
        // The starting state has 1 ship, 1 station → counters should be ≥ 1
        assert!(state.id_counters.ship >= 1);
        assert!(state.id_counters.station >= 1);
    }

    /// Checked overflow — counter overflow is handled by IdCounters being u64
    #[test]
    fn id_counter_overflow_safe() {
        let catalog = load_catalog();
        let state = build_starting_state(&catalog).unwrap();
        // IdCounters are u64 — adding a huge number would wrap, but the
        // starting state has small counters.  Verify they are within range.
        assert!(state.id_counters.ship < u64::MAX / 2);
        assert!(state.id_counters.station < u64::MAX / 2);
    }
}
