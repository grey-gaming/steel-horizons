//! Content validation — structural rules.
//!
//! Every function in this module checks canonical authored JSON against the
//! GDD 13/GDD 14 structural invariants.  Precise errors carry a stable path
//! (e.g. `content.recipes[3].inputs`).  Table-driven invalid fixtures prove
//! each error type.
//!
//! ## Authoritative references
//!
//! - ADR-0005 §Content Validation Gate
//! - GDD 13 §Identifiers and Resources (generated-ID prefixes)
//! - GDD 13 §Content Definitions (DTO shapes)
//! - GDD 14 §Starting System Bodies (slot counts, parent rules)
//! - GDD 14 §Starting State (buffer capacities)
//! - GDD 14 §Automatic Buffer Defaults (threshold rules)
//! - GDD 14 §Canonical Station Definitions (stat ranges)

#![allow(missing_docs)]

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt;

use crate::content::*;
use crate::id::*;
use crate::types::*;

// ─── Error types ───────────────────────────────────────────────────────

/// A validation error with a stable path and a human-readable message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentValidationError {
    /// Dot-separated path to the failing element
    /// (e.g. `"definitions.recipes[3].inputs"`).
    pub path: String,
    /// Human-readable description of the violation.
    pub message: String,
}

impl fmt::Display for ContentValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "at `{}`: {}", self.path, self.message)
    }
}

/// Result type for content validation.
pub type ContentValidationResult<T = ()> = Result<T, Vec<ContentValidationError>>;

/// Collect a single error into the Vec accumulator.
macro_rules! err_at {
    ($path:expr, $($fmt:tt)*) => {
        ContentValidationError {
            path: $path.to_string(),
            message: format!($($fmt)*),
        }
    };
}

/// Append an error to a mutable accumulator.
macro_rules! push_err {
    ($errors:ident, $path:expr, $($fmt:tt)*) => {
        $errors.push(err_at!($path, $($fmt)*));
    };
}

// ─── Public API ────────────────────────────────────────────────────────

/// Validate both the definitions catalog and the starting scenario.
///
/// Returns all accumulated errors — callers should assert that the canonical
/// content produces zero errors.
pub fn validate_content(
    defs: &DefinitionsCatalog,
    scenario: &StartingScenario,
) -> Vec<ContentValidationError> {
    let mut errors = Vec::new();
    validate_definitions(defs, &mut errors);
    validate_starting_scenario(defs, scenario, &mut errors);
    errors
}

// ─── Definitions validation ────────────────────────────────────────────

fn validate_definitions(defs: &DefinitionsCatalog, errors: &mut Vec<ContentValidationError>) {
    validate_recipe_ids(defs, errors);
    validate_tech_ids(defs, errors);
    validate_ship_definitions(defs, errors);
    validate_station_definitions(defs, errors);
    validate_recipe_facilities(defs, errors);
    validate_recipe_inputs_outputs(defs, errors);
    validate_tech_prereqs_exist(defs, errors);
    validate_tech_cost_nonzero(defs, errors);
    validate_gate_definition(defs, errors);
    validate_authored_namespace(defs, errors);
    // P1-05 semantic validations
    validate_tech_dag(defs, errors);
    validate_recipe_required_techs_exist(defs, errors);
    validate_inverse_recipes(defs, errors);
    validate_component_costs(defs, errors);
    validate_build_hold_not_exceeds_capacity(defs, errors);
    validate_critical_resource_budget(defs, errors);
}

fn validate_recipe_ids(defs: &DefinitionsCatalog, errors: &mut Vec<ContentValidationError>) {
    let mut seen = BTreeSet::new();
    for (i, r) in defs.recipes.iter().enumerate() {
        let path = format!("definitions.recipes[{}].id", i);
        if !seen.insert(&r.id) {
            push_err!(errors, path, "duplicate recipe id: {}", r.id.0);
        }
        if r.id.0.is_empty() {
            push_err!(errors, path, "empty recipe id");
        }
    }
}

fn validate_tech_ids(defs: &DefinitionsCatalog, errors: &mut Vec<ContentValidationError>) {
    let mut seen = BTreeSet::new();
    for (i, t) in defs.technologies.iter().enumerate() {
        let path = format!("definitions.technologies[{}].id", i);
        if !seen.insert(&t.id) {
            push_err!(errors, path, "duplicate technology id: {}", t.id.0);
        }
        if t.id.0.is_empty() {
            push_err!(errors, path, "empty technology id");
        }
    }
}

fn validate_ship_definitions(defs: &DefinitionsCatalog, errors: &mut Vec<ContentValidationError>) {
    let tech_ids: BTreeSet<&TechId> = defs.technologies.iter().map(|t| &t.id).collect();
    let mut seen = BTreeSet::new();
    for (i, s) in defs.ships.iter().enumerate() {
        let path = format!("definitions.ships[{}]", i);
        if !seen.insert((s.role, s.tier)) {
            push_err!(
                errors,
                path,
                "duplicate ship role+tier: {:?} / {}",
                s.role,
                s.tier
            );
        }
        if s.name.is_empty() {
            push_err!(errors, format!("{}.name", path), "empty ship name");
        }
        if s.build_work == 0 {
            push_err!(
                errors,
                format!("{}.build_work", path),
                "ship {:?} T{} has zero build_work",
                s.role,
                s.tier
            );
        }
        if !tech_ids.contains(&s.required_tech) {
            push_err!(
                errors,
                format!("{}.required_tech", path),
                "ship {:?} T{} references unknown required tech '{}'",
                s.role,
                s.tier,
                s.required_tech.0
            );
        }
        for (&res, &qty) in &s.component_cost {
            if qty == 0 {
                push_err!(
                    errors,
                    format!("{}.component_cost.{}", path, res.variant_name()),
                    "ship {:?} T{} has zero-quantity component cost entry for {}",
                    s.role,
                    s.tier,
                    res.variant_name()
                );
            }
        }
    }
}

fn validate_station_definitions(
    defs: &DefinitionsCatalog,
    errors: &mut Vec<ContentValidationError>,
) {
    let tech_ids: BTreeSet<&TechId> = defs.technologies.iter().map(|t| &t.id).collect();
    let mut seen = BTreeSet::new();
    for (i, s) in defs.stations.iter().enumerate() {
        let path = format!("definitions.stations[{}]", i);
        if !seen.insert((s.station_type, s.tier)) {
            push_err!(
                errors,
                path,
                "duplicate station type+tier: {:?} / {}",
                s.station_type,
                s.tier
            );
        }
        if s.build_work == 0 {
            push_err!(
                errors,
                format!("{}.build_work", path),
                "station {:?} T{} has zero build_work",
                s.station_type,
                s.tier
            );
        }
        if !tech_ids.contains(&s.required_tech) {
            push_err!(
                errors,
                format!("{}.required_tech", path),
                "station {:?} T{} references unknown required tech '{}'",
                s.station_type,
                s.tier,
                s.required_tech.0
            );
        }
        for (&res, &qty) in &s.component_cost {
            if qty == 0 {
                push_err!(
                    errors,
                    format!("{}.component_cost.{}", path, res.variant_name()),
                    "station {:?} T{} has zero-quantity component cost entry for {}",
                    s.station_type,
                    s.tier,
                    res.variant_name()
                );
            }
        }
        // Validate station stats: production_slots should be zero for Hub, Mining, Research
        match s.station_type {
            StationType::Hub | StationType::Mining | StationType::Research => {
                if s.stats.production_slots != 0 {
                    push_err!(
                        errors,
                        format!("{}.stats.production_slots", path),
                        "expected 0 production_slots for {:?} station (tier {}), got {}",
                        s.station_type,
                        s.tier,
                        s.stats.production_slots
                    );
                }
            }
            StationType::Refinery | StationType::Construction => {
                // production_slots should match tier (1..4)
                if s.stats.production_slots != s.tier {
                    push_err!(
                        errors,
                        format!("{}.stats.production_slots", path),
                        "expected production_slots = tier ({}) for {:?} station, got {}",
                        s.tier,
                        s.station_type,
                        s.stats.production_slots
                    );
                }
            }
        }
    }
}

fn validate_recipe_facilities(defs: &DefinitionsCatalog, errors: &mut Vec<ContentValidationError>) {
    let known_stations: BTreeSet<(StationType, u8)> = defs
        .stations
        .iter()
        .map(|s| (s.station_type, s.tier))
        .collect();

    for (i, r) in defs.recipes.iter().enumerate() {
        if r.facilities.is_empty() {
            push_err!(
                errors,
                format!("definitions.recipes[{}].facilities", i),
                "recipe {} has zero facilities",
                r.id.0
            );
            continue;
        }
        for (j, f) in r.facilities.iter().enumerate() {
            let fpath = format!("definitions.recipes[{}].facilities[{}]", i, j);
            if f.cycle_ticks == 0 {
                push_err!(errors, fpath, "cycle_ticks is 0 for recipe {}", r.id.0);
            }
            // Check facility station type and tier exist in definitions
            if !known_stations.contains(&(f.station_type, f.minimum_tier)) {
                push_err!(
                    errors,
                    fpath,
                    "recipe {} references unknown station type+tier: {:?}/{}",
                    r.id.0,
                    f.station_type,
                    f.minimum_tier
                );
            }
        }
    }
}

fn validate_recipe_inputs_outputs(
    defs: &DefinitionsCatalog,
    errors: &mut Vec<ContentValidationError>,
) {
    for (i, r) in defs.recipes.iter().enumerate() {
        let rpath = format!("definitions.recipes[{}]", i);
        if r.inputs.is_empty() && r.outputs.is_empty() {
            push_err!(
                errors,
                format!("{}.inputs", rpath),
                "recipe {} has both zero inputs and zero outputs",
                r.id.0
            );
        }
        // Check for zero-quantity entries in inputs/outputs
        for (&res, &qty) in &r.inputs {
            if qty == 0 {
                push_err!(
                    errors,
                    format!("{}.inputs.{}", rpath, res.variant_name()),
                    "recipe {} has zero-quantity input {}",
                    r.id.0,
                    res.variant_name()
                );
            }
        }
        for (&res, &qty) in &r.outputs {
            if qty == 0 {
                push_err!(
                    errors,
                    format!("{}.outputs.{}", rpath, res.variant_name()),
                    "recipe {} has zero-quantity output {}",
                    r.id.0,
                    res.variant_name()
                );
            }
        }
    }
}

fn validate_tech_prereqs_exist(
    defs: &DefinitionsCatalog,
    errors: &mut Vec<ContentValidationError>,
) {
    let tech_ids: BTreeSet<&TechId> = defs.technologies.iter().map(|t| &t.id).collect();
    for (i, t) in defs.technologies.iter().enumerate() {
        for (j, prereq) in t.prerequisites.iter().enumerate() {
            if !tech_ids.contains(prereq) {
                push_err!(
                    errors,
                    format!("definitions.technologies[{}].prerequisites[{}]", i, j),
                    "technology {} references unknown prerequisite {}",
                    t.id.0,
                    prereq.0
                );
            }
        }
    }
}

fn validate_tech_cost_nonzero(defs: &DefinitionsCatalog, errors: &mut Vec<ContentValidationError>) {
    // Tier-0 techs should have zero costs/duration; tier-1+ should have non-zero costs
    for (i, t) in defs.technologies.iter().enumerate() {
        let path = format!("definitions.technologies[{}]", i);
        if t.tier == 0 {
            if !t.costs.is_empty() {
                push_err!(
                    errors,
                    format!("{}.costs", path),
                    "tier-0 technology {} has non-zero costs",
                    t.id.0
                );
            }
            if t.duration_ticks != 0 {
                push_err!(
                    errors,
                    format!("{}.duration_ticks", path),
                    "tier-0 technology {} has non-zero duration ({})",
                    t.id.0,
                    t.duration_ticks
                );
            }
        } else {
            if t.costs.is_empty() {
                push_err!(
                    errors,
                    format!("{}.costs", path),
                    "tier-{} technology {} has zero costs",
                    t.tier,
                    t.id.0
                );
            }
            if t.duration_ticks == 0 {
                push_err!(
                    errors,
                    format!("{}.duration_ticks", path),
                    "tier-{} technology {} has zero duration",
                    t.tier,
                    t.id.0
                );
            }
        }
    }
}

fn validate_gate_definition(defs: &DefinitionsCatalog, errors: &mut Vec<ContentValidationError>) {
    let gate = &defs.gate;

    // Build set of known tech IDs for reference validation
    let tech_ids: BTreeSet<&TechId> = defs.technologies.iter().map(|t| &t.id).collect();

    // Check required_techs references exist
    for (j, tech_id) in gate.required_techs.iter().enumerate() {
        if !tech_ids.contains(tech_id) {
            push_err!(
                errors,
                format!("definitions.gate.required_techs[{}]", j),
                "gate references unknown required tech '{}'",
                tech_id.0
            );
        }
    }

    // Check manifest entries are non-zero
    for (&res, &qty) in &gate.manifest {
        if qty == 0 {
            push_err!(
                errors,
                format!("definitions.gate.manifest.{}", res.variant_name()),
                "gate manifest has zero-quantity entry for {}",
                res.variant_name()
            );
        }
    }

    // Check logistics_priority is set
    if gate.logistics_priority == 0 {
        push_err!(
            errors,
            "definitions.gate.logistics_priority",
            "gate logistics_priority is zero"
        );
    }

    // Check transfer_berths is set
    if gate.transfer_berths == 0 {
        push_err!(
            errors,
            "definitions.gate.transfer_berths",
            "gate transfer_berths is zero"
        );
    }

    // Check minimum_fabricator_tier >= 1
    if gate.minimum_fabricator_tier == 0 {
        push_err!(
            errors,
            "definitions.gate.minimum_fabricator_tier",
            "gate minimum_fabricator_tier is zero"
        );
    }

    // Validate phases
    for (i, phase) in gate.phases.iter().enumerate() {
        if phase.work == 0 {
            push_err!(
                errors,
                format!("definitions.gate.phases[{}].work", i),
                "phase has zero work"
            );
        }

        // Check required_deliveries entries are non-zero
        for (&res, &qty) in &phase.required_deliveries {
            if qty == 0 {
                let del_path = format!(
                    "definitions.gate.phases[{}].required_deliveries.{}",
                    i,
                    res.variant_name()
                );
                push_err!(
                    errors,
                    del_path,
                    "phase {} has zero-quantity required delivery for {}",
                    i,
                    res.variant_name()
                );
            }
        }

        // Check completion_consumption entries are non-zero
        for (&res, &qty) in &phase.completion_consumption {
            if qty == 0 {
                let cons_path = format!(
                    "definitions.gate.phases[{}].completion_consumption.{}",
                    i,
                    res.variant_name()
                );
                push_err!(
                    errors,
                    cons_path,
                    "phase {} has zero-quantity completion consumption for {}",
                    i,
                    res.variant_name()
                );
            }
        }
    }
}

fn validate_authored_namespace(
    defs: &DefinitionsCatalog,
    errors: &mut Vec<ContentValidationError>,
) {
    // Authored IDs must not use reserved generated-ID prefixes
    let reserved_prefixes = [
        "ship_generated_",
        "station_generated_",
        "build_order_generated_",
        "reservation_generated_",
        "salvage_generated_",
        "survey_order_generated_",
    ];

    // Check recipe IDs
    for (i, r) in defs.recipes.iter().enumerate() {
        check_reserved_prefix(
            &r.id.0,
            &format!("definitions.recipes[{}].id", i),
            &reserved_prefixes,
            errors,
        );
    }
    // Check tech IDs
    for (i, t) in defs.technologies.iter().enumerate() {
        check_reserved_prefix(
            &t.id.0,
            &format!("definitions.technologies[{}].id", i),
            &reserved_prefixes,
            errors,
        );
    }
}

fn check_reserved_prefix(
    id_str: &str,
    path: &str,
    prefixes: &[&str],
    errors: &mut Vec<ContentValidationError>,
) {
    for prefix in prefixes {
        if id_str.starts_with(prefix) {
            push_err!(
                errors,
                path,
                "authored id '{}' uses reserved generated-ID prefix '{}'",
                id_str,
                prefix
            );
            return;
        }
    }
}

// ─── Starting scenario validation ──────────────────────────────────────

fn validate_starting_scenario(
    _defs: &DefinitionsCatalog,
    scenario: &StartingScenario,
    errors: &mut Vec<ContentValidationError>,
) {
    validate_body_ids(scenario, errors);
    validate_parent_references(scenario, errors);
    validate_slot_counts(scenario, errors);
    validate_station_slot_validity(scenario, errors);
    validate_hub_haven_buffers(scenario, errors);
    validate_hub_haven_capacity(scenario, errors);
    validate_starting_ship(scenario, errors);
    validate_starting_techs(scenario, errors);
    validate_station_ids_unique(scenario, errors);
    validate_ship_ids_unique(scenario, errors);
}

fn validate_body_ids(scenario: &StartingScenario, errors: &mut Vec<ContentValidationError>) {
    let mut seen = BTreeSet::new();
    for (id, body) in &scenario.celestial_bodies {
        let path = format!("starting_system.celestial_bodies[\"{}\"]", id.0);
        if !seen.insert(id) {
            push_err!(errors, path, "duplicate body id: {}", id.0);
        }
        // Check body id matches key
        if id.0 != body.id.0 {
            push_err!(
                errors,
                path,
                "body key '{}' does not match body.id '{}'",
                id.0,
                body.id.0
            );
        }
    }
}

fn validate_parent_references(
    scenario: &StartingScenario,
    errors: &mut Vec<ContentValidationError>,
) {
    let body_ids: BTreeSet<&BodyId> = scenario.celestial_bodies.keys().collect();
    for (id, body) in &scenario.celestial_bodies {
        let path = format!("starting_system.celestial_bodies[\"{}\"]", id.0);
        if let Some(ref parent) = body.parent_body_id {
            if !body_ids.contains(parent) {
                push_err!(
                    errors,
                    format!("{}.parent_body_id", path),
                    "body '{}' references unknown parent body '{}'",
                    id.0,
                    parent.0
                );
            }
        }
    }
}

fn validate_slot_counts(scenario: &StartingScenario, errors: &mut Vec<ContentValidationError>) {
    let mut total_slots: u32 = 0;
    for (id, body) in &scenario.celestial_bodies {
        let path = format!("starting_system.celestial_bodies[\"{}\"]", id.0);
        let count = body.orbit_ring_count as usize;
        if count != body.slot_counts.len() {
            push_err!(
                errors,
                format!("{}.slot_counts", path),
                "body '{}' has orbit_ring_count={} but slot_counts has {} entries",
                id.0,
                count,
                body.slot_counts.len()
            );
        }
        for &sc in &body.slot_counts {
            total_slots += sc as u32;
        }
    }
    if total_slots != 19 {
        push_err!(
            errors,
            "starting_system.celestial_bodies",
            "total station slots is {}, expected 19",
            total_slots
        );
    }
}

fn validate_station_slot_validity(
    scenario: &StartingScenario,
    errors: &mut Vec<ContentValidationError>,
) {
    for (sid, station) in &scenario.stations {
        let path = format!("starting_system.stations[\"{}\"]", sid.0);
        let body_entry = scenario.celestial_bodies.get(&station.body_id);
        match body_entry {
            Some(body) => {
                let ring = station.orbit_ring as usize;
                if ring >= body.orbit_ring_count as usize {
                    push_err!(
                        errors,
                        format!("{}.orbit_ring", path),
                        "station '{}' at body '{}' has orbit_ring={} but body has {} rings",
                        sid.0,
                        station.body_id.0,
                        ring,
                        body.orbit_ring_count
                    );
                }
                let slot = station.slot as usize;
                if ring < body.slot_counts.len() && slot >= body.slot_counts[ring] as usize {
                    push_err!(
                        errors,
                        format!("{}.slot", path),
                        "station '{}' at body '{}' ring {} has slot {} but ring has {} slots",
                        sid.0,
                        station.body_id.0,
                        ring,
                        slot,
                        body.slot_counts[ring]
                    );
                }
            }
            None => {
                push_err!(
                    errors,
                    format!("{}.body_id", path),
                    "station '{}' references unknown body '{}'",
                    sid.0,
                    station.body_id.0
                );
            }
        }
    }
}

fn validate_hub_haven_buffers(
    scenario: &StartingScenario,
    errors: &mut Vec<ContentValidationError>,
) {
    let hub = match scenario.stations.get(&StationId("hub_haven".into())) {
        Some(h) => h,
        None => {
            push_err!(
                errors,
                "starting_system.stations",
                "hub_haven not found in starting stations"
            );
            return;
        }
    };

    let path = "starting_system.stations[\"hub_haven\"]";
    if hub.station_type != StationType::Hub {
        push_err!(
            errors,
            format!("{}.station_type", path),
            "expected Hub, got {:?}",
            hub.station_type
        );
    }
    if hub.tier != 1 {
        push_err!(
            errors,
            format!("{}.tier", path),
            "expected tier 1, got {}",
            hub.tier
        );
    }

    // Check buffer thresholds are valid
    for (i, buf) in hub.input_buffers.iter().enumerate() {
        let bpath = format!("{}.input_buffers[{}]", path, i);
        let dt = u32::from(buf.demand_threshold);
        if dt > 100 {
            push_err!(
                errors,
                bpath,
                "demand_threshold {} exceeds max 100",
                buf.demand_threshold
            );
        }
        let et = u32::from(buf.export_threshold);
        if et > buf.max {
            push_err!(
                errors,
                bpath,
                "export_threshold {} exceeds max {}",
                buf.export_threshold,
                buf.max
            );
        }
        if buf.current > buf.max {
            push_err!(
                errors,
                bpath,
                "current {} exceeds max {}",
                buf.current,
                buf.max
            );
        }
    }
    for (i, buf) in hub.output_buffers.iter().enumerate() {
        let bpath = format!("{}.output_buffers[{}]", path, i);
        let dt = u32::from(buf.demand_threshold);
        if dt > 100 {
            push_err!(
                errors,
                bpath,
                "demand_threshold {} exceeds max 100",
                buf.demand_threshold
            );
        }
        let et = u32::from(buf.export_threshold);
        if et > buf.max {
            push_err!(
                errors,
                bpath,
                "export_threshold {} exceeds max {}",
                buf.export_threshold,
                buf.max
            );
        }
        if buf.current > buf.max {
            push_err!(
                errors,
                bpath,
                "current {} exceeds max {}",
                buf.current,
                buf.max
            );
        }
    }

    // Check fuel buffer thresholds
    let fpath = format!("{}.fuel_buffer", path);
    let fdt = u32::from(hub.fuel_buffer.demand_threshold);
    if fdt > 100 {
        push_err!(
            errors,
            fpath,
            "fuel demand_threshold {} exceeds max 100",
            hub.fuel_buffer.demand_threshold
        );
    }
    let fet = u32::from(hub.fuel_buffer.export_threshold);
    if fet > 100 {
        push_err!(
            errors,
            fpath,
            "fuel export_threshold {} exceeds max 100",
            hub.fuel_buffer.export_threshold
        );
    }
    if hub.fuel_buffer.current > hub.fuel_buffer.max {
        push_err!(
            errors,
            fpath,
            "fuel current {} exceeds max {}",
            hub.fuel_buffer.current,
            hub.fuel_buffer.max
        );
    }
}

fn validate_hub_haven_capacity(
    scenario: &StartingScenario,
    errors: &mut Vec<ContentValidationError>,
) {
    let hub = match scenario.stations.get(&StationId("hub_haven".into())) {
        Some(h) => h,
        None => return,
    };

    let path = "starting_system.stations[\"hub_haven\"]";
    // Sum of all output buffer max values must not exceed total_cargo_capacity
    let total_output_max: u32 = hub.output_buffers.iter().map(|b| b.max).sum();
    let total_input_max: u32 = hub.input_buffers.iter().map(|b| b.max).sum();
    let total_buffer_max = total_output_max + total_input_max;
    // Fuel is a separate compartment
    if total_buffer_max > hub.total_cargo_capacity {
        push_err!(
            errors,
            format!("{}.total_cargo_capacity", path),
            "buffer max sum {} exceeds total_cargo_capacity {}",
            total_buffer_max,
            hub.total_cargo_capacity
        );
    }
}

fn validate_starting_ship(scenario: &StartingScenario, errors: &mut Vec<ContentValidationError>) {
    let ship = match scenario.ships.get(&ShipId("ship_builder_1".into())) {
        Some(s) => s,
        None => {
            push_err!(
                errors,
                "starting_system.ships",
                "ship_builder_1 not found in starting ships"
            );
            return;
        }
    };

    let path = "starting_system.ships[\"ship_builder_1\"]";
    if ship.role != ShipRole::Construction {
        push_err!(
            errors,
            format!("{}.role", path),
            "expected Construction role, got {:?}",
            ship.role
        );
    }
    if ship.tier != 1 {
        push_err!(
            errors,
            format!("{}.tier", path),
            "expected tier 1, got {}",
            ship.tier
        );
    }
    if ship.fuel > ship.max_fuel {
        push_err!(
            errors,
            format!("{}.fuel", path),
            "fuel {} exceeds max_fuel {}",
            ship.fuel,
            ship.max_fuel
        );
    }
    if ship.fuel_remainder >= 100_000 {
        push_err!(
            errors,
            format!("{}.fuel_remainder", path),
            "fuel_remainder {} is suspiciously large",
            ship.fuel_remainder
        );
    }
}

fn validate_starting_techs(scenario: &StartingScenario, errors: &mut Vec<ContentValidationError>) {
    let expected: BTreeSet<TechId> = [
        TechId("basic_construction".into()),
        TechId("basic_refining".into()),
        TechId("basic_power".into()),
        TechId("basic_control".into()),
    ]
    .into();

    let path = "starting_system.completed_techs";
    for tech in &scenario.completed_techs {
        if !expected.contains(tech) {
            push_err!(
                errors,
                path,
                "unexpected completed tech '{}' in starting state",
                tech.0
            );
        }
    }
    // Check all expected techs are present
    for expected_tech in &expected {
        if !scenario.completed_techs.contains(expected_tech) {
            push_err!(
                errors,
                path,
                "missing expected starting tech '{}'",
                expected_tech.0
            );
        }
    }
}

fn validate_station_ids_unique(
    scenario: &StartingScenario,
    errors: &mut Vec<ContentValidationError>,
) {
    let mut seen = BTreeSet::new();
    for id in scenario.stations.keys() {
        let path = format!("starting_system.stations[\"{}\"]", id.0);
        if !seen.insert(id) {
            push_err!(errors, path, "duplicate station id");
        }
    }
}

fn validate_ship_ids_unique(scenario: &StartingScenario, errors: &mut Vec<ContentValidationError>) {
    let mut seen = BTreeSet::new();
    for id in scenario.ships.keys() {
        let path = format!("starting_system.ships[\"{}\"]", id.0);
        if !seen.insert(id) {
            push_err!(errors, path, "duplicate ship id");
        }
    }
}

// ─── Helper: variant name for ResourceType ─────────────────────────────

/// Get a stable string name for a ResourceType variant (lowercase, no prefix).
impl ResourceType {
    pub fn variant_name(self) -> &'static str {
        match self {
            Self::MetalOre => "metalOre",
            Self::CarbonSoil => "carbonSoil",
            Self::SiliconDust => "siliconDust",
            Self::VolcanicSulfur => "volcanicSulfur",
            Self::WaterIce => "waterIce",
            Self::FrozenGases => "frozenGases",
            Self::Helium3 => "helium3",
            Self::RareEarthMinerals => "rareEarthMinerals",
            Self::CrystalDeposits => "crystalDeposits",
            Self::Metals => "metals",
            Self::CarbonFiber => "carbonFiber",
            Self::SiliconWafers => "siliconWafers",
            Self::Chemicals => "chemicals",
            Self::Fuel => "fuel",
            Self::Alloys => "alloys",
            Self::Optics => "optics",
            Self::ReactorRods => "reactorRods",
            Self::StructuralFrame => "structuralFrame",
            Self::PowerCore => "powerCore",
            Self::ControlSystem => "controlSystem",
            Self::DriveAssembly => "driveAssembly",
            Self::CargoModule => "cargoModule",
            Self::ResearchLab => "researchLab",
            Self::ConstructionBay => "constructionBay",
            Self::GateNode => "gateNode",
        }
    }
}

// ─── Semantic validation (P1-05) ───────────────────────────────────────

/// Detect cycles in the technology prerequisite DAG.
///
/// Uses a simple topological-sort approach: compute in-degree for each tech
/// from its prerequisites, then walk.  Any tech remaining after the walk is
/// part of a cycle.
fn validate_tech_dag(defs: &DefinitionsCatalog, errors: &mut Vec<ContentValidationError>) {
    let tech_ids: BTreeSet<&TechId> = defs.technologies.iter().map(|t| &t.id).collect();

    // Kahn's algorithm for topological sort
    // Build adjacency: prereq -> t means t depends on prereq
    let mut in_deg: BTreeMap<&TechId, usize> = BTreeMap::new();
    let mut edges: BTreeMap<&TechId, Vec<&TechId>> = BTreeMap::new();

    for t in &defs.technologies {
        let prereq_count = t.prerequisites.len();
        *in_deg.entry(&t.id).or_insert(0) += prereq_count;
        for prereq in &t.prerequisites {
            // Edge prereq -> t
            edges.entry(prereq).or_default().push(&t.id);
        }
    }

    // Kahn's algorithm — collect initial zero-degree techs separately
    let mut queue: Vec<&&TechId> = in_deg
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(id, _)| id)
        .collect();

    let mut processed = 0usize;
    // Use indices instead of references to avoid borrow conflicts
    let mut deg_copy: BTreeMap<&TechId, usize> = in_deg.iter().map(|(k, &v)| (*k, v)).collect();
    while let Some(id) = queue.pop() {
        processed += 1;
        if let Some(succ) = edges.get(id) {
            for next in succ {
                let deg = deg_copy.get_mut(next).expect("tech must be in in_deg");
                *deg = deg.saturating_sub(1);
                if *deg == 0 {
                    queue.push(next);
                }
            }
        }
    }

    if processed != tech_ids.len() {
        let unprocessed: Vec<&&TechId> = in_deg
            .iter()
            .filter(|(_, &deg)| deg > 0)
            .map(|(id, _)| id)
            .collect();
        for id in &unprocessed {
            let path = format!("definitions.technologies.{}.prerequisites", id.0);
            push_err!(
                errors,
                path,
                "technology '{}' is part of a prerequisite cycle",
                id.0
            );
        }
    }
}

/// Validate that every recipe's `required_tech` (if set) references a known tech.
fn validate_recipe_required_techs_exist(
    defs: &DefinitionsCatalog,
    errors: &mut Vec<ContentValidationError>,
) {
    let tech_ids: BTreeSet<&TechId> = defs.technologies.iter().map(|t| &t.id).collect();
    for (i, r) in defs.recipes.iter().enumerate() {
        if let Some(ref req) = r.required_tech {
            if !tech_ids.contains(req) {
                push_err!(
                    errors,
                    format!("definitions.recipes[{}].required_tech", i),
                    "recipe '{}' references unknown required tech '{}'",
                    r.id.0,
                    req.0
                );
            }
        }
    }
}

/// Validate inverse-recipe equality.
///
/// Every disassembly recipe must be the exact inverse of its assembly
/// counterpart: the disassembly consumes the assembly's one output unit
/// and returns the assembly's complete input map.
fn validate_inverse_recipes(defs: &DefinitionsCatalog, errors: &mut Vec<ContentValidationError>) {
    let assembly_map: BTreeMap<String, &RecipeDefinition> = defs
        .recipes
        .iter()
        .filter(|r| r.id.0.starts_with("assemble_"))
        .map(|r| (r.id.0.clone(), r))
        .collect();

    for (i, r) in defs.recipes.iter().enumerate() {
        if !r.id.0.starts_with("disassemble_") {
            continue;
        }
        let assembly_id = format!("assemble_{}", &r.id.0[12..]);
        let assembly = match assembly_map.get(&assembly_id) {
            Some(a) => a,
            None => continue,
        };

        let rpath = format!("definitions.recipes[{}]", i);

        if r.outputs != assembly.inputs {
            push_err!(
                errors,
                format!("{}.outputs", rpath),
                "disassembly recipe '{}' outputs {:?} don't match assembly '{}' inputs {:?}",
                r.id.0,
                r.outputs,
                assembly_id,
                assembly.inputs
            );
        }

        if r.inputs != assembly.outputs {
            push_err!(
                errors,
                format!("{}.inputs", rpath),
                "disassembly recipe '{}' inputs {:?} don't match assembly '{}' outputs {:?}",
                r.id.0,
                r.inputs,
                assembly_id,
                assembly.outputs
            );
        }

        if r.facilities != assembly.facilities {
            push_err!(
                errors,
                format!("{}.facilities", rpath),
                "disassembly recipe '{}' facilities don't match assembly '{}'",
                r.id.0,
                assembly_id
            );
        }

        if r.required_tech != assembly.required_tech {
            push_err!(
                errors,
                format!("{}.required_tech", rpath),
                "disassembly recipe '{}' required_tech doesn't match assembly '{}'",
                r.id.0,
                assembly_id
            );
        }
    }
}

/// Validate that component costs in ship/station definitions reference
/// valid resources (non-zero quantity).
fn validate_component_costs(defs: &DefinitionsCatalog, errors: &mut Vec<ContentValidationError>) {
    for (i, s) in defs.ships.iter().enumerate() {
        let path = format!("definitions.ships[{}]", i);
        for (&res, &qty) in &s.component_cost {
            if qty == 0 {
                push_err!(
                    errors,
                    format!("{}.component_cost.{}", path, res.variant_name()),
                    "ship {:?} T{} has zero-quantity component cost for {}",
                    s.role,
                    s.tier,
                    res.variant_name()
                );
            }
        }
    }
    for (i, s) in defs.stations.iter().enumerate() {
        let path = format!("definitions.stations[{}]", i);
        for (&res, &qty) in &s.component_cost {
            if qty == 0 {
                push_err!(
                    errors,
                    format!("{}.component_cost.{}", path, res.variant_name()),
                    "station {:?} T{} has zero-quantity component cost for {}",
                    s.station_type,
                    s.tier,
                    res.variant_name()
                );
            }
        }
    }
}

/// Validate that `build_cargo_capacity` does not exceed `cargo_capacity` for
/// each ship definition.
///
/// Construction ships have zero cargo capacity by design — they exclusively use
/// their build-hold for construction materials.  Skip them.
fn validate_build_hold_not_exceeds_capacity(
    defs: &DefinitionsCatalog,
    errors: &mut Vec<ContentValidationError>,
) {
    for (i, s) in defs.ships.iter().enumerate() {
        if s.role == crate::types::ShipRole::Construction {
            continue;
        }
        let path = format!("definitions.ships[{}]", i);
        if s.stats.build_cargo_capacity > s.stats.cargo_capacity {
            push_err!(
                errors,
                format!("{}.stats.build_cargo_capacity", path),
                "ship {:?} T{} build_cargo_capacity {} exceeds cargo_capacity {}",
                s.role,
                s.tier,
                s.stats.build_cargo_capacity,
                s.stats.cargo_capacity
            );
        }
    }
}

/// Validate the critical-resource solvability budget per GDD 14 §Solvability Budget.
fn validate_critical_resource_budget(
    defs: &DefinitionsCatalog,
    errors: &mut Vec<ContentValidationError>,
) {
    let mut rare_tech: u64 = 0;
    let mut helium_tech: u64 = 0;
    let mut crystal_tech: u64 = 0;

    for t in &defs.technologies {
        for (&res, &qty) in &t.costs {
            let q = qty as u64;
            match res {
                ResourceType::RareEarthMinerals => rare_tech += q,
                ResourceType::Helium3 => helium_tech += q,
                ResourceType::CrystalDeposits => crystal_tech += q,
                _ => {}
            }
        }
    }

    let mut rare_gate: u64 = 0;
    let mut helium_gate: u64 = 0;
    let mut crystal_gate: u64 = 0;

    let gate = &defs.gate;
    for (&res, &qty) in &gate.manifest {
        let q = qty as u64;
        match res {
            ResourceType::RareEarthMinerals => rare_gate += q,
            ResourceType::Helium3 => helium_gate += q,
            ResourceType::CrystalDeposits => crystal_gate += q,
            _ => {}
        }
    }
    for phase in &gate.phases {
        for (&res, &qty) in &phase.required_deliveries {
            let q = qty as u64;
            match res {
                ResourceType::RareEarthMinerals => rare_gate += q,
                ResourceType::Helium3 => helium_gate += q,
                ResourceType::CrystalDeposits => crystal_gate += q,
                _ => {}
            }
        }
    }

    let total_rare = rare_tech + rare_gate;
    let total_helium = helium_tech + helium_gate;
    let total_crystal = crystal_tech + crystal_gate;

    if total_rare > 1600 {
        push_err!(
            errors,
            "definitions.critical_resource_budget.rareEarthMinerals",
            "total RareEarthMinerals budget {} exceeds authored finite amount 1600",
            total_rare
        );
    }
    if total_helium > 1000 {
        push_err!(
            errors,
            "definitions.critical_resource_budget.helium3",
            "total Helium3 budget {} exceeds authored finite amount 1000",
            total_helium
        );
    }
    if total_crystal > 1000 {
        push_err!(
            errors,
            "definitions.critical_resource_budget.crystalDeposits",
            "total CrystalDeposits budget {} exceeds authored finite amount 1000",
            total_crystal
        );
    }

    let tech_rare = rare_tech;
    let tech_helium = helium_tech;
    let tech_crystal = crystal_tech;

    if tech_rare + rare_gate > 725 {
        push_err!(
            errors,
            "definitions.critical_resource_budget.rareEarthMinerals",
            "tech+gate RareEarthMinerals budget {} exceeds validated maximum 725",
            tech_rare + rare_gate
        );
    }
    if tech_helium + helium_gate > 350 {
        push_err!(
            errors,
            "definitions.critical_resource_budget.helium3",
            "tech+gate Helium3 budget {} exceeds validated maximum 350",
            tech_helium + helium_gate
        );
    }
    if tech_crystal + crystal_gate > 400 {
        push_err!(
            errors,
            "definitions.critical_resource_budget.crystalDeposits",
            "tech+gate CrystalDeposits budget {} exceeds validated maximum 400",
            tech_crystal + crystal_gate
        );
    }
}

// ─── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
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

    /// Canonical content must pass validation with zero errors.
    #[test]
    fn canonical_content_validates() {
        let defs: DefinitionsCatalog = load_json("definitions.v1.json");
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let errors = validate_content(&defs, &scenario);
        assert!(
            errors.is_empty(),
            "canonical content should have zero validation errors, got {}:\n  {}",
            errors.len(),
            errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("\n  ")
        );
    }

    // ─── Table-driven invalid fixtures ─────────────────────────────────

    /// Fixture: duplicate recipe ID.
    #[test]
    fn invalid_duplicate_recipe_id() {
        let mut defs: DefinitionsCatalog = load_json("definitions.v1.json");
        defs.recipes.push(defs.recipes[0].clone());
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let errors = validate_content(&defs, &scenario);
        assert!(
            errors
                .iter()
                .any(|e| e.path.contains("definitions.recipes") && e.message.contains("duplicate")),
            "expected duplicate recipe id error, got: {:?}",
            errors
        );
    }

    /// Fixture: duplicate tech ID.
    #[test]
    fn invalid_duplicate_tech_id() {
        let mut defs: DefinitionsCatalog = load_json("definitions.v1.json");
        defs.technologies.push(defs.technologies[0].clone());
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let errors = validate_content(&defs, &scenario);
        assert!(
            errors
                .iter()
                .any(|e| e.path.contains("definitions.technologies")
                    && e.message.contains("duplicate")),
            "expected duplicate tech id error, got: {:?}",
            errors
        );
    }

    /// Fixture: duplicate ship role+tier.
    #[test]
    fn invalid_duplicate_ship() {
        let mut defs: DefinitionsCatalog = load_json("definitions.v1.json");
        defs.ships.push(defs.ships[0].clone());
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let errors = validate_content(&defs, &scenario);
        assert!(
            errors
                .iter()
                .any(|e| e.path.contains("definitions.ships") && e.message.contains("duplicate")),
            "expected duplicate ship error, got: {:?}",
            errors
        );
    }

    /// Fixture: duplicate station type+tier.
    #[test]
    fn invalid_duplicate_station() {
        let mut defs: DefinitionsCatalog = load_json("definitions.v1.json");
        defs.stations.push(defs.stations[0].clone());
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let errors = validate_content(&defs, &scenario);
        assert!(
            errors.iter().any(|e| e.path.contains("definitions.stations") && e.message.contains("duplicate")),
            "expected duplicate station error, got: {:?}",
            errors
        );
    }

    /// Fixture: empty recipe ID.
    #[test]
    fn invalid_empty_recipe_id() {
        let mut defs: DefinitionsCatalog = load_json("definitions.v1.json");
        let mut bad = defs.recipes[0].clone();
        bad.id = RecipeId(String::new());
        defs.recipes.push(bad);
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let errors = validate_content(&defs, &scenario);
        assert!(
            errors.iter().any(|e| e.message.contains("empty recipe id")),
            "expected empty recipe id error, got: {:?}",
            errors
        );
    }

    /// Fixture: empty tech ID.
    #[test]
    fn invalid_empty_tech_id() {
        let mut defs: DefinitionsCatalog = load_json("definitions.v1.json");
        let mut bad = defs.technologies[0].clone();
        bad.id = TechId(String::new());
        defs.technologies.push(bad);
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let errors = validate_content(&defs, &scenario);
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("empty technology id")),
            "expected empty tech id error, got: {:?}",
            errors
        );
    }

    /// Fixture: recipe with zero facilities.
    #[test]
    fn invalid_recipe_no_facilities() {
        let mut defs: DefinitionsCatalog = load_json("definitions.v1.json");
        let mut bad = defs.recipes[0].clone();
        bad.facilities = vec![];
        defs.recipes.push(bad);
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let errors = validate_content(&defs, &scenario);
        assert!(
            errors.iter().any(|e| e.message.contains("zero facilities")),
            "expected zero facilities error, got: {:?}",
            errors
        );
    }

    /// Fixture: recipe with zero cycle ticks.
    #[test]
    fn invalid_recipe_zero_cycle() {
        let mut defs: DefinitionsCatalog = load_json("definitions.v1.json");
        let mut bad = defs.recipes[0].clone();
        bad.facilities[0].cycle_ticks = 0;
        defs.recipes.push(bad);
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let errors = validate_content(&defs, &scenario);
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("cycle_ticks is 0")),
            "expected zero cycle_ticks error, got: {:?}",
            errors
        );
    }

    /// Fixture: recipe with both zero inputs and outputs.
    #[test]
    fn invalid_recipe_no_inputs_no_outputs() {
        let mut defs: DefinitionsCatalog = load_json("definitions.v1.json");
        let mut bad = defs.recipes[0].clone();
        bad.inputs = BTreeMap::new();
        bad.outputs = BTreeMap::new();
        defs.recipes.push(bad);
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let errors = validate_content(&defs, &scenario);
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("zero inputs and zero outputs")),
            "expected zero inputs/outputs error, got: {:?}",
            errors
        );
    }

    /// Fixture: zero-quantity input in recipe.
    #[test]
    fn invalid_recipe_zero_input_quantity() {
        let mut defs: DefinitionsCatalog = load_json("definitions.v1.json");
        let mut bad = defs.recipes[0].clone();
        bad.inputs.insert(ResourceType::MetalOre, 0);
        defs.recipes.push(bad);
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let errors = validate_content(&defs, &scenario);
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("zero-quantity input")),
            "expected zero-quantity input error, got: {:?}",
            errors
        );
    }

    /// Fixture: unknown prerequisite in tech.
    #[test]
    fn invalid_unknown_prerequisite() {
        let mut defs: DefinitionsCatalog = load_json("definitions.v1.json");
        let mut bad = defs.technologies[0].clone();
        bad.prerequisites.push(TechId("nonexistent_tech".into()));
        defs.technologies.push(bad);
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let errors = validate_content(&defs, &scenario);
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("unknown prerequisite")),
            "expected unknown prereq error, got: {:?}",
            errors
        );
    }

    /// Fixture: tier-0 tech with non-zero costs.
    #[test]
    fn invalid_tier0_tech_has_costs() {
        let mut defs: DefinitionsCatalog = load_json("definitions.v1.json");
        let mut bad = defs.technologies[0].clone(); // basic_construction is tier 0
        bad.costs.insert(ResourceType::Metals, 10);
        defs.technologies.push(bad);
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let errors = validate_content(&defs, &scenario);
        assert!(
            errors.iter().any(|e| e.message.contains("non-zero costs")),
            "expected tier-0 costs error, got: {:?}",
            errors
        );
    }

    /// Fixture: authored ID using reserved generated prefix.
    #[test]
    fn invalid_reserved_prefix() {
        let mut defs: DefinitionsCatalog = load_json("definitions.v1.json");
        let mut bad = defs.recipes[0].clone();
        bad.id = RecipeId("ship_generated_00000001".into());
        defs.recipes.push(bad);
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let errors = validate_content(&defs, &scenario);
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("reserved generated-ID prefix")),
            "expected reserved prefix error, got: {:?}",
            errors
        );
    }

    /// Fixture: station slot exceeds body ring capacity.
    #[test]
    fn invalid_station_slot_out_of_range() {
        let mut scenario: StartingScenario = load_json("starting_system.v1.json");
        // Find a station and change its slot to exceed the ring's capacity
        if let Some(body) = scenario
            .celestial_bodies
            .get(&BodyId("planet_haven".into()))
        {
            if let Some((_sid, station)) = scenario
                .stations
                .iter_mut()
                .find(|(_, s)| s.body_id == body.id)
            {
                station.slot = 10; // Haven ring 0 has 3 slots, 10 is out of range
                let defs: DefinitionsCatalog = load_json("definitions.v1.json");
                let errors = validate_content(&defs, &scenario);
                assert!(
                    errors.iter().any(|e| e.message.contains("has slot")),
                    "expected slot out of range error, got: {:?}",
                    errors
                );
            }
        }
    }

    /// Fixture: moon references unknown parent.
    #[test]
    fn invalid_missing_parent() {
        let mut scenario: StartingScenario = load_json("starting_system.v1.json");
        if let Some(moon) = scenario
            .celestial_bodies
            .get_mut(&BodyId("moon_rime".into()))
        {
            moon.parent_body_id = Some(BodyId("nonexistent_body".into()));
            let defs: DefinitionsCatalog = load_json("definitions.v1.json");
            let errors = validate_content(&defs, &scenario);
            assert!(
                errors
                    .iter()
                    .any(|e| e.message.contains("unknown parent body")),
                "expected unknown parent body error, got: {:?}",
                errors
            );
        }
    }

    /// Fixture: orbit_ring_count doesn't match slot_counts length.
    #[test]
    fn invalid_slot_count_mismatch() {
        let mut scenario: StartingScenario = load_json("starting_system.v1.json");
        if let Some(body) = scenario
            .celestial_bodies
            .get_mut(&BodyId("planet_haven".into()))
        {
            body.slot_counts.push(1); // Now 4 entries but orbit_ring_count=3
            let defs: DefinitionsCatalog = load_json("definitions.v1.json");
            let errors = validate_content(&defs, &scenario);
            assert!(
                errors
                    .iter()
                    .any(|e| e.message.contains("orbit_ring_count")),
                "expected orbit_ring_count mismatch error, got: {:?}",
                errors
            );
        }
    }

    /// Fixture: station body_id references unknown body.
    #[test]
    fn invalid_station_unknown_body() {
        let mut scenario: StartingScenario = load_json("starting_system.v1.json");
        if let Some((_, station)) = scenario.stations.iter_mut().next() {
            station.body_id = BodyId("nonexistent".into());
            let defs: DefinitionsCatalog = load_json("definitions.v1.json");
            let errors = validate_content(&defs, &scenario);
            assert!(
                errors.iter().any(|e| e.message.contains("unknown body")),
                "expected unknown body error, got: {:?}",
                errors
            );
        }
    }

    /// Fixture: station orbit_ring exceeds body's ring count.
    #[test]
    fn invalid_station_ring_out_of_range() {
        let mut scenario: StartingScenario = load_json("starting_system.v1.json");
        if let Some((_, station)) = scenario.stations.iter_mut().next() {
            station.orbit_ring = 99;
            let defs: DefinitionsCatalog = load_json("definitions.v1.json");
            let errors = validate_content(&defs, &scenario);
            assert!(
                errors
                    .iter()
                    .any(|e| e.message.contains("orbit_ring") && e.message.contains("rings")),
                "expected orbit_ring out of range error, got: {:?}",
                errors
            );
        }
    }

    /// Fixture: Hub buffer current exceeds max.
    #[test]
    fn invalid_buffer_current_exceeds_max() {
        let mut scenario: StartingScenario = load_json("starting_system.v1.json");
        let hub_id = StationId("hub_haven".into());
        if let Some(hub) = scenario.stations.get_mut(&hub_id) {
            if let Some(buf) = hub.output_buffers.get_mut(0) {
                buf.current = buf.max + 1;
            }
            let defs: DefinitionsCatalog = load_json("definitions.v1.json");
            let errors = validate_content(&defs, &scenario);
            assert!(
                errors.iter().any(|e| e.message.contains("exceeds max")),
                "expected buffer current>max error, got: {:?}",
                errors
            );
        }
    }

    /// Fixture: Hub total buffer max exceeds cargo capacity.
    #[test]
    fn invalid_hub_capacity_exceeded() {
        let mut scenario: StartingScenario = load_json("starting_system.v1.json");
        let hub_id = StationId("hub_haven".into());
        if let Some(hub) = scenario.stations.get_mut(&hub_id) {
            hub.total_cargo_capacity = 5; // Buffer max sum is 6+4+1=11, so 5 triggers error
            let defs: DefinitionsCatalog = load_json("definitions.v1.json");
            let errors = validate_content(&defs, &scenario);
            assert!(
                errors
                    .iter()
                    .any(|e| e.message.contains("exceeds total_cargo_capacity")),
                "expected capacity exceeded error, got: {:?}",
                errors
            );
        }
    }

    /// Fixture: station with wrong production_slots (non-zero for Hub).
    #[test]
    fn invalid_hub_production_slots() {
        let mut defs: DefinitionsCatalog = load_json("definitions.v1.json");
        let mut bad = defs.stations[0].clone(); // Hub T1
        bad.stats.production_slots = 1;
        defs.stations.push(bad);
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let errors = validate_content(&defs, &scenario);
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("production_slots")),
            "expected production_slots error, got: {:?}",
            errors
        );
    }

    /// Fixture: station production_slots doesn't match tier for Refinery.
    #[test]
    fn invalid_refinery_production_slots() {
        let mut defs: DefinitionsCatalog = load_json("definitions.v1.json");
        // Find refinery T1 (index 8+ in stations after 4 hubs)
        let refinery_idx = defs
            .stations
            .iter()
            .position(|s| s.station_type == StationType::Refinery && s.tier == 1)
            .unwrap();
        let mut bad = defs.stations[refinery_idx].clone();
        bad.stats.production_slots = 0;
        defs.stations.push(bad);
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let errors = validate_content(&defs, &scenario);
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("production_slots")),
            "expected production_slots error, got: {:?}",
            errors
        );
    }

    /// Fixture: missing starting tech.
    #[test]
    fn invalid_missing_starting_tech() {
        let mut scenario: StartingScenario = load_json("starting_system.v1.json");
        scenario
            .completed_techs
            .remove(&TechId("basic_construction".into()));
        let defs: DefinitionsCatalog = load_json("definitions.v1.json");
        let errors = validate_content(&defs, &scenario);
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("missing expected starting tech")),
            "expected missing starting tech error, got: {:?}",
            errors
        );
    }

    /// Fixture: ship fuel exceeds max.
    #[test]
    fn invalid_ship_fuel_exceeds_max() {
        let mut scenario: StartingScenario = load_json("starting_system.v1.json");
        let ship_id = ShipId("ship_builder_1".into());
        if let Some(ship) = scenario.ships.get_mut(&ship_id) {
            ship.fuel = ship.max_fuel + 1;
            let defs: DefinitionsCatalog = load_json("definitions.v1.json");
            let errors = validate_content(&defs, &scenario);
            assert!(
                errors
                    .iter()
                    .any(|e| e.message.contains("fuel") && e.message.contains("exceeds max_fuel")),
                "expected fuel exceeds max error, got: {:?}",
                errors
            );
        }
    }

    // ─── New validation fixtures (P1-04 review) ─────────────────────

    /// Fixture: ship with zero build_work.
    #[test]
    fn invalid_ship_zero_build_work() {
        let mut defs: DefinitionsCatalog = load_json("definitions.v1.json");
        let mut bad = defs.ships[0].clone();
        bad.build_work = 0;
        defs.ships.push(bad);
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let errors = validate_content(&defs, &scenario);
        assert!(
            errors.iter().any(|e| e.message.contains("build_work")),
            "expected zero build_work error, got: {:?}",
            errors
        );
    }

    /// Fixture: station with zero build_work.
    #[test]
    fn invalid_station_zero_build_work() {
        let mut defs: DefinitionsCatalog = load_json("definitions.v1.json");
        let mut bad = defs.stations[0].clone();
        bad.build_work = 0;
        defs.stations.push(bad);
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let errors = validate_content(&defs, &scenario);
        assert!(
            errors.iter().any(|e| e.message.contains("build_work")),
            "expected zero build_work error, got: {:?}",
            errors
        );
    }

    /// Fixture: ship references unknown required tech.
    #[test]
    fn invalid_ship_unknown_tech() {
        let mut defs: DefinitionsCatalog = load_json("definitions.v1.json");
        let mut bad = defs.ships[0].clone();
        bad.required_tech = TechId("nonexistent_tech".into());
        defs.ships.push(bad);
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let errors = validate_content(&defs, &scenario);
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("unknown required tech")),
            "expected unknown required tech error, got: {:?}",
            errors
        );
    }

    /// Fixture: station references unknown required tech.
    #[test]
    fn invalid_station_unknown_tech() {
        let mut defs: DefinitionsCatalog = load_json("definitions.v1.json");
        let mut bad = defs.stations[0].clone();
        bad.required_tech = TechId("nonexistent_tech".into());
        defs.stations.push(bad);
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let errors = validate_content(&defs, &scenario);
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("unknown required tech")),
            "expected unknown required tech error, got: {:?}",
            errors
        );
    }

    /// Fixture: gate references unknown required tech.
    #[test]
    fn invalid_gate_unknown_tech() {
        let mut defs: DefinitionsCatalog = load_json("definitions.v1.json");
        defs.gate
            .required_techs
            .push(TechId("nonexistent_tech".into()));
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let errors = validate_content(&defs, &scenario);
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("unknown required tech")),
            "expected unknown required tech error, got: {:?}",
            errors
        );
    }

    /// Fixture: gate logistics_priority is zero.
    #[test]
    fn invalid_gate_logistics_priority_zero() {
        let mut defs: DefinitionsCatalog = load_json("definitions.v1.json");
        defs.gate.logistics_priority = 0;
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let errors = validate_content(&defs, &scenario);
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("logistics_priority")),
            "expected logistics_priority zero error, got: {:?}",
            errors
        );
    }

    /// Fixture: gate transfer_berths is zero.
    #[test]
    fn invalid_gate_transfer_berths_zero() {
        let mut defs: DefinitionsCatalog = load_json("definitions.v1.json");
        defs.gate.transfer_berths = 0;
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let errors = validate_content(&defs, &scenario);
        assert!(
            errors.iter().any(|e| e.message.contains("transfer_berths")),
            "expected transfer_berths zero error, got: {:?}",
            errors
        );
    }

    /// Fixture: gate minimum_fabricator_tier is zero.
    #[test]
    fn invalid_gate_min_fab_tier_zero() {
        let mut defs: DefinitionsCatalog = load_json("definitions.v1.json");
        defs.gate.minimum_fabricator_tier = 0;
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let errors = validate_content(&defs, &scenario);
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("minimum_fabricator_tier")),
            "expected minimum_fabricator_tier zero error, got: {:?}",
            errors
        );
    }

    // ─── P1-05 semantic validation fixture tests ────────────────────────

    /// Fixture: technology prerequisite cycle.
    #[test]
    fn invalid_tech_prerequisite_cycle() {
        let mut defs: DefinitionsCatalog = load_json("definitions.v1.json");
        // Create a simple cycle: tech A -> tech B -> tech A
        let tech_a_id = TechId("test_tech_a".into());
        let tech_b_id = TechId("test_tech_b".into());
        defs.technologies.push(TechDefinition {
            id: tech_a_id.clone(),
            tier: 0,
            prerequisites: vec![tech_b_id.clone()],
            costs: Default::default(),
            duration_ticks: 0,
            mechanic_unlocks: vec![],
        });
        defs.technologies.push(TechDefinition {
            id: tech_b_id.clone(),
            tier: 0,
            prerequisites: vec![tech_a_id.clone()],
            costs: Default::default(),
            duration_ticks: 0,
            mechanic_unlocks: vec![],
        });
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let errors = validate_content(&defs, &scenario);
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("prerequisite cycle")),
            "expected prerequisite cycle error, got: {:?}",
            errors
        );
    }

    /// Fixture: recipe references unknown required tech.
    #[test]
    fn invalid_recipe_unknown_required_tech() {
        let mut defs: DefinitionsCatalog = load_json("definitions.v1.json");
        defs.recipes[0].required_tech = Some(TechId("nonexistent_tech".into()));
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let errors = validate_content(&defs, &scenario);
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("unknown required tech")),
            "expected unknown required tech error, got: {:?}",
            errors
        );
    }

    /// Fixture: inverse recipe mismatch (disassembly outputs != assembly inputs).
    #[test]
    fn invalid_inverse_recipe_mismatch() {
        let mut defs: DefinitionsCatalog = load_json("definitions.v1.json");
        // Find a disassembly recipe and modify its outputs to break the inverse
        let dis_idx = defs
            .recipes
            .iter()
            .position(|r| r.id.0.starts_with("disassemble_"));
        if let Some(idx) = dis_idx {
            // Clone the outputs first to avoid borrow conflict
            let first_output = defs.recipes[idx]
                .outputs
                .iter()
                .next()
                .map(|(&k, &v)| (k, v));
            if let Some((k, v)) = first_output {
                defs.recipes[idx].outputs.insert(k, v.saturating_add(1));
            }
        }
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let errors = validate_content(&defs, &scenario);
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("disassembly") && e.message.contains("don't match")),
            "expected inverse recipe mismatch error, got: {:?}",
            errors
        );
    }

    /// Fixture: ship with zero-quantity component cost.
    #[test]
    fn invalid_zero_quantity_component_cost() {
        let mut defs: DefinitionsCatalog = load_json("definitions.v1.json");
        let first_cost = defs.ships[0]
            .component_cost
            .iter()
            .next()
            .map(|(&k, &v)| (k, v));
        if let Some((res, _)) = first_cost {
            defs.ships[0].component_cost.insert(res, 0);
        }
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let errors = validate_content(&defs, &scenario);
        assert!(
            errors.iter().any(|e| e.message.contains("zero-quantity")),
            "expected zero-quantity component cost error, got: {:?}",
            errors
        );
    }

    /// Fixture: non-Construction ship with build_cargo_capacity > cargo_capacity.
    #[test]
    fn invalid_build_hold_exceeds_cargo() {
        let mut defs: DefinitionsCatalog = load_json("definitions.v1.json");
        // Find a non-Construction ship and set build_hold > cargo
        for ship in &mut defs.ships {
            if ship.role != crate::types::ShipRole::Construction {
                ship.stats.build_cargo_capacity = ship.stats.cargo_capacity + 100;
                break;
            }
        }
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let errors = validate_content(&defs, &scenario);
        assert!(
            errors.iter().any(
                |e| e.message.contains("build_cargo_capacity") && e.message.contains("exceeds")
            ),
            "expected build_hold exceeds cargo error, got: {:?}",
            errors
        );
    }

    /// Fixture: critical resource budget exceeds authored finite amount.
    #[test]
    fn invalid_critical_resource_budget() {
        let mut defs: DefinitionsCatalog = load_json("definitions.v1.json");
        // Artificially inflate a tech cost for a critical resource
        for tech in &mut defs.technologies {
            if tech.costs.contains_key(&ResourceType::RareEarthMinerals) {
                let cost = tech.costs.get(&ResourceType::RareEarthMinerals).unwrap();
                tech.costs
                    .insert(ResourceType::RareEarthMinerals, cost + 10_000);
                break;
            }
        }
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let errors = validate_content(&defs, &scenario);
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("RareEarthMinerals") && e.message.contains("budget")),
            "expected critical resource budget error, got: {:?}",
            errors
        );
    }
}
