#![allow(missing_docs)]

//! Command types and sequencing DTOs.
//!
//! Every enum and struct in this module is a Serde-only data-transfer object
//! matching ADR-0003 and GDD 13.  No runtime command processing or sequencing
//! logic lives here — that belongs in the command and actor modules (P1-12
//! onward).
//!
//! ## Authoritative references
//!
//! - ADR-0003 §Command Envelope and Ordering
//! - ADR-0003 §V1 Commands
//! - ADR-0003 §Response
//! - GDD 13 §Lifecycle, RNG, Commands, and Root State

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::id::*;
use crate::types::*;

// ─── Buffer configuration ─────────────────────────────────────────────

/// Buffer configuration for a station's input, output, or fuel buffer.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum BufferConfiguration {
    Input {
        resource: ResourceType,
        max: u32,
        demand_threshold: u8,
    },
    Output {
        resource: ResourceType,
        max: u32,
        export_threshold: u8,
    },
    Fuel {
        demand_threshold: u8,
        export_threshold: u8,
    },
}

// ─── Actor control commands ───────────────────────────────────────────

/// Actor-level lifecycle commands (not replayable in the game timeline).
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum ActorControl {
    NewGame { scenario_id: ScenarioId },
    LoadAutosave,
    SaveNow,
    Pause,
    Resume,
    AdvanceTicks { count: u16 },
}

// ─── Replayable game commands ─────────────────────────────────────────

/// Replayable game commands that enter the command log.
///
/// This is the ADR-0003 `Command` union excluding `NewGame`, `LoadAutosave`,
/// `SaveNow`, `Pause`, `Resume`, and `AdvanceTicks`.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum ReplayableGameCommand {
    // ── Construction ──
    QueueBuildShip {
        hub_id: StationId,
        role: ShipRole,
        tier: u8,
    },
    QueueBuildStation {
        source_hub_id: StationId,
        body_id: BodyId,
        orbit_ring: u8,
        slot: u8,
        station_type: StationType,
        tier: u8,
    },
    QueueUpgrade {
        source_hub_id: StationId,
        station_id: StationId,
        target_tier: u8,
    },
    CancelBuildOrder {
        order_id: BuildOrderId,
    },
    QueueDemolishStation {
        station_id: StationId,
        recovery_hub_id: StationId,
    },
    ScrapShip {
        ship_id: ShipId,
    },
    BeginGateAssembly {
        fabricator_ship_id: ShipId,
    },

    // ── Configuration ──
    SetStationPriority {
        station_id: StationId,
        priority: u8,
    },
    ConfigureBuffer {
        station_id: StationId,
        configuration: BufferConfiguration,
    },
    SetProductionRecipe {
        station_id: StationId,
        slot_index: u8,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        recipe_id: Option<RecipeId>,
    },
    SetMiningTarget {
        station_id: StationId,
        slot_index: u8,
        resource: ResourceType,
    },

    // ── Research and survey ──
    QueueResearch {
        facility_id: StationId,
        tech_id: TechId,
    },
    PauseResearch {
        tech_id: TechId,
        release_unused: bool,
    },
    QueueSurvey {
        body_id: BodyId,
        target_depth: u8,
        priority: u8,
    },
    CancelSurveyOrder {
        order_id: SurveyOrderId,
    },
}

// ─── Full command union ───────────────────────────────────────────────

impl From<&Command> for ReplayableGameCommand {
    fn from(cmd: &Command) -> Self {
        match cmd {
            Command::QueueBuildShip { hub_id, role, tier } => {
                ReplayableGameCommand::QueueBuildShip {
                    hub_id: hub_id.clone(),
                    role: *role,
                    tier: *tier,
                }
            }
            Command::QueueBuildStation {
                source_hub_id,
                body_id,
                orbit_ring,
                slot,
                station_type,
                tier,
            } => ReplayableGameCommand::QueueBuildStation {
                source_hub_id: source_hub_id.clone(),
                body_id: body_id.clone(),
                orbit_ring: *orbit_ring,
                slot: *slot,
                station_type: *station_type,
                tier: *tier,
            },
            Command::QueueUpgrade {
                source_hub_id,
                station_id,
                target_tier,
            } => ReplayableGameCommand::QueueUpgrade {
                source_hub_id: source_hub_id.clone(),
                station_id: station_id.clone(),
                target_tier: *target_tier,
            },
            Command::CancelBuildOrder { order_id } => ReplayableGameCommand::CancelBuildOrder {
                order_id: order_id.clone(),
            },
            Command::QueueDemolishStation {
                station_id,
                recovery_hub_id,
            } => ReplayableGameCommand::QueueDemolishStation {
                station_id: station_id.clone(),
                recovery_hub_id: recovery_hub_id.clone(),
            },
            Command::ScrapShip { ship_id } => ReplayableGameCommand::ScrapShip {
                ship_id: ship_id.clone(),
            },
            Command::BeginGateAssembly { fabricator_ship_id } => {
                ReplayableGameCommand::BeginGateAssembly {
                    fabricator_ship_id: fabricator_ship_id.clone(),
                }
            }
            Command::SetStationPriority {
                station_id,
                priority,
            } => ReplayableGameCommand::SetStationPriority {
                station_id: station_id.clone(),
                priority: *priority,
            },
            Command::ConfigureBuffer {
                station_id,
                configuration,
            } => ReplayableGameCommand::ConfigureBuffer {
                station_id: station_id.clone(),
                configuration: configuration.clone(),
            },
            Command::SetProductionRecipe {
                station_id,
                slot_index,
                recipe_id,
            } => ReplayableGameCommand::SetProductionRecipe {
                station_id: station_id.clone(),
                slot_index: *slot_index,
                recipe_id: recipe_id.clone(),
            },
            Command::SetMiningTarget {
                station_id,
                slot_index,
                resource,
            } => ReplayableGameCommand::SetMiningTarget {
                station_id: station_id.clone(),
                slot_index: *slot_index,
                resource: *resource,
            },
            Command::QueueResearch {
                facility_id,
                tech_id,
            } => ReplayableGameCommand::QueueResearch {
                facility_id: facility_id.clone(),
                tech_id: tech_id.clone(),
            },
            Command::PauseResearch {
                tech_id,
                release_unused,
            } => ReplayableGameCommand::PauseResearch {
                tech_id: tech_id.clone(),
                release_unused: *release_unused,
            },
            Command::QueueSurvey {
                body_id,
                target_depth,
                priority,
            } => ReplayableGameCommand::QueueSurvey {
                body_id: body_id.clone(),
                target_depth: *target_depth,
                priority: *priority,
            },
            Command::CancelSurveyOrder { order_id } => ReplayableGameCommand::CancelSurveyOrder {
                order_id: order_id.clone(),
            },
            // Non-replayable commands should never reach this conversion.
            // An `unreachable!()` ensures a new control variant is caught
            // immediately rather than silently producing a misleading record.
            _ => unreachable!("non-replayable command reached ReplayableGameCommand::from"),
        }
    }
}

/// The complete command union (actor controls + replayable game commands).
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum Command {
    // Actor controls
    NewGame {
        scenario_id: ScenarioId,
    },
    LoadAutosave,
    SaveNow,
    Pause,
    Resume,
    AdvanceTicks {
        count: u16,
    },
    // Replayable game commands
    QueueBuildShip {
        hub_id: StationId,
        role: ShipRole,
        tier: u8,
    },
    QueueBuildStation {
        source_hub_id: StationId,
        body_id: BodyId,
        orbit_ring: u8,
        slot: u8,
        station_type: StationType,
        tier: u8,
    },
    QueueUpgrade {
        source_hub_id: StationId,
        station_id: StationId,
        target_tier: u8,
    },
    CancelBuildOrder {
        order_id: BuildOrderId,
    },
    QueueDemolishStation {
        station_id: StationId,
        recovery_hub_id: StationId,
    },
    ScrapShip {
        ship_id: ShipId,
    },
    BeginGateAssembly {
        fabricator_ship_id: ShipId,
    },
    SetStationPriority {
        station_id: StationId,
        priority: u8,
    },
    ConfigureBuffer {
        station_id: StationId,
        configuration: BufferConfiguration,
    },
    SetProductionRecipe {
        station_id: StationId,
        slot_index: u8,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        recipe_id: Option<RecipeId>,
    },
    SetMiningTarget {
        station_id: StationId,
        slot_index: u8,
        resource: ResourceType,
    },
    QueueResearch {
        facility_id: StationId,
        tech_id: TechId,
    },
    PauseResearch {
        tech_id: TechId,
        release_unused: bool,
    },
    QueueSurvey {
        body_id: BodyId,
        target_depth: u8,
        priority: u8,
    },
    CancelSurveyOrder {
        order_id: SurveyOrderId,
    },
}

// ─── Wire envelope ────────────────────────────────────────────────────

/// The strict command envelope for all API commands.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct CommandEnvelope {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_tick: Option<u64>,
    pub command: Command,
}

/// Deterministic structural fingerprint of a `CommandEnvelope`.
///
/// ADR-0006 canonical JSON v1 writer + domain-separated SHA-256, truncated to
/// a u64 for the idempotency receipt.  Unlike `DefaultHasher` (randomly seeded
/// per process), this value is stable across runs, so idempotency receipts
/// survive save/load replay exactly.
///
/// Returns a typed error on serialization failure; it can only fail for an
/// envelope that contains a non-canonical value (never for a valid envelope).
pub fn envelope_canonical_hash(envelope: &CommandEnvelope) -> Result<u64, String> {
    use sha2::{Digest, Sha256};
    const DOMAIN: &[u8] = b"steel-horizons/envelope/sha256-canonical-json-v1";

    let value = serde_json::to_value(envelope).map_err(|e| format!("envelope to_value: {e}"))?;
    let bytes = crate::canonical::to_canonical_bytes(&value)
        .map_err(|e| format!("envelope canonical bytes: {e}"))?;

    let mut hasher = Sha256::new();
    hasher.update(DOMAIN);
    hasher.update([0x00]);
    hasher.update(&bytes);
    let digest = hasher.finalize();

    // First 8 bytes as little-endian u64 (deterministic; the truncation is
    // fine for idempotency keys, see ADR-0003 §Idempotency).
    Ok(u64::from_le_bytes(
        digest[0..8]
            .try_into()
            .map_err(|_| "digest too short".to_string())?,
    ))
}

/// Acknowledgement returned after accepting a command envelope.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct CommandAcknowledgement {
    pub protocol_version: String,
    pub id: String,
    pub accepted: bool,
    pub status: CommandStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_tick: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resulting_tick: Option<u64>,
    pub server_sequence: u64,
    pub game_state: GameLifecycle,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<CommandResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<CommandRejection>,
}

/// The status of a command after processing.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum CommandStatus {
    Accepted,
    Applied,
    Rejected,
    Failed,
}

/// A command that has been assigned a server sequence number.
///
/// Sequenced commands are queued for tick-level application.  The
/// server sequence ensures deterministic ordering within a tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SequencedCommand {
    /// The server-assigned sequence number.
    pub server_sequence: u64,
    /// The command envelope.
    pub envelope: CommandEnvelope,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn round_trip<T>(val: &T)
    where
        T: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug + PartialEq,
    {
        let json = serde_json::to_string(val).unwrap();
        let back: T = serde_json::from_str(&json).unwrap();
        assert_eq!(*val, back);
    }

    #[test]
    fn buffer_configuration_input_round_trip() {
        round_trip(&BufferConfiguration::Input {
            resource: ResourceType::Metals,
            max: 1000,
            demand_threshold: 50,
        });
    }

    #[test]
    fn buffer_configuration_output_round_trip() {
        round_trip(&BufferConfiguration::Output {
            resource: ResourceType::Metals,
            max: 1000,
            export_threshold: 80,
        });
    }

    #[test]
    fn buffer_configuration_fuel_round_trip() {
        round_trip(&BufferConfiguration::Fuel {
            demand_threshold: 30,
            export_threshold: 70,
        });
    }

    #[test]
    fn actor_control_new_game_round_trip() {
        round_trip(&ActorControl::NewGame {
            scenario_id: ScenarioId("default".into()),
        });
    }

    #[test]
    fn actor_control_variants_round_trip() {
        round_trip(&ActorControl::LoadAutosave);
        round_trip(&ActorControl::SaveNow);
        round_trip(&ActorControl::Pause);
        round_trip(&ActorControl::Resume);
        round_trip(&ActorControl::AdvanceTicks { count: 10 });
    }

    #[test]
    fn replayable_queue_build_ship_round_trip() {
        round_trip(&ReplayableGameCommand::QueueBuildShip {
            hub_id: StationId("hub_haven".into()),
            role: ShipRole::Construction,
            tier: 1,
        });
    }

    #[test]
    fn replayable_queue_build_station_round_trip() {
        round_trip(&ReplayableGameCommand::QueueBuildStation {
            source_hub_id: StationId("hub_haven".into()),
            body_id: BodyId("planet_haven".into()),
            orbit_ring: 0,
            slot: 1,
            station_type: StationType::Mining,
            tier: 1,
        });
    }

    #[test]
    fn replayable_queue_upgrade_round_trip() {
        round_trip(&ReplayableGameCommand::QueueUpgrade {
            source_hub_id: StationId("hub_haven".into()),
            station_id: StationId("hub_haven".into()),
            target_tier: 2,
        });
    }

    #[test]
    fn replayable_cancel_build_order_round_trip() {
        round_trip(&ReplayableGameCommand::CancelBuildOrder {
            order_id: BuildOrderId("bo_1".into()),
        });
    }

    #[test]
    fn replayable_queue_demolish_round_trip() {
        round_trip(&ReplayableGameCommand::QueueDemolishStation {
            station_id: StationId("mine_1".into()),
            recovery_hub_id: StationId("hub_haven".into()),
        });
    }

    #[test]
    fn replayable_scrap_ship_round_trip() {
        round_trip(&ReplayableGameCommand::ScrapShip {
            ship_id: ShipId("ship_transport_1".into()),
        });
    }

    #[test]
    fn replayable_begin_gate_assembly_round_trip() {
        round_trip(&ReplayableGameCommand::BeginGateAssembly {
            fabricator_ship_id: ShipId("ship_builder_1".into()),
        });
    }

    #[test]
    fn replayable_set_station_priority_round_trip() {
        round_trip(&ReplayableGameCommand::SetStationPriority {
            station_id: StationId("hub_haven".into()),
            priority: 128,
        });
    }

    #[test]
    fn replayable_configure_buffer_round_trip() {
        round_trip(&ReplayableGameCommand::ConfigureBuffer {
            station_id: StationId("hub_haven".into()),
            configuration: BufferConfiguration::Input {
                resource: ResourceType::Metals,
                max: 1000,
                demand_threshold: 50,
            },
        });
    }

    #[test]
    fn replayable_set_production_recipe_round_trip() {
        round_trip(&ReplayableGameCommand::SetProductionRecipe {
            station_id: StationId("hub_haven".into()),
            slot_index: 0,
            recipe_id: Some(RecipeId("smelt_iron".into())),
        });
    }

    #[test]
    fn replayable_set_production_recipe_clear_round_trip() {
        round_trip(&ReplayableGameCommand::SetProductionRecipe {
            station_id: StationId("hub_haven".into()),
            slot_index: 1,
            recipe_id: None,
        });
    }

    #[test]
    fn replayable_set_mining_target_round_trip() {
        round_trip(&ReplayableGameCommand::SetMiningTarget {
            station_id: StationId("mine_1".into()),
            slot_index: 0,
            resource: ResourceType::MetalOre,
        });
    }

    #[test]
    fn replayable_queue_research_round_trip() {
        round_trip(&ReplayableGameCommand::QueueResearch {
            facility_id: StationId("hub_haven".into()),
            tech_id: TechId("refined_metals".into()),
        });
    }

    #[test]
    fn replayable_pause_research_round_trip() {
        round_trip(&ReplayableGameCommand::PauseResearch {
            tech_id: TechId("refined_metals".into()),
            release_unused: true,
        });
    }

    #[test]
    fn replayable_queue_survey_round_trip() {
        round_trip(&ReplayableGameCommand::QueueSurvey {
            body_id: BodyId("planet_haven".into()),
            target_depth: 3,
            priority: 64,
        });
    }

    #[test]
    fn replayable_cancel_survey_order_round_trip() {
        round_trip(&ReplayableGameCommand::CancelSurveyOrder {
            order_id: SurveyOrderId("so_1".into()),
        });
    }

    #[test]
    fn command_envelope_round_trip() {
        round_trip(&CommandEnvelope {
            id: "cmd_001".to_string(),
            expected_tick: Some(42),
            command: Command::QueueBuildShip {
                hub_id: StationId("hub_haven".into()),
                role: ShipRole::Construction,
                tier: 1,
            },
        });
    }

    #[test]
    fn command_envelope_no_expected_tick_round_trip() {
        round_trip(&CommandEnvelope {
            id: "cmd_002".to_string(),
            expected_tick: None,
            command: Command::Pause,
        });
    }

    #[test]
    fn command_acknowledgement_accepted_round_trip() {
        round_trip(&CommandAcknowledgement {
            protocol_version: "1.0".to_string(),
            id: "cmd_001".to_string(),
            accepted: true,
            status: CommandStatus::Accepted,
            effective_tick: None,
            resulting_tick: None,
            server_sequence: 1,
            game_state: GameLifecycle::Running,
            result: None,
            error: None,
        });
    }

    #[test]
    fn command_acknowledgement_applied_round_trip() {
        round_trip(&CommandAcknowledgement {
            protocol_version: "1.0".to_string(),
            id: "cmd_001".to_string(),
            accepted: true,
            status: CommandStatus::Applied,
            effective_tick: Some(100),
            resulting_tick: Some(101),
            server_sequence: 2,
            game_state: GameLifecycle::Running,
            result: Some(CommandResult::None),
            error: None,
        });
    }

    #[test]
    fn command_acknowledgement_rejected_round_trip() {
        round_trip(&CommandAcknowledgement {
            protocol_version: "1.0".to_string(),
            id: "cmd_001".to_string(),
            accepted: false,
            status: CommandStatus::Rejected,
            effective_tick: None,
            resulting_tick: None,
            server_sequence: 1,
            game_state: GameLifecycle::Running,
            result: None,
            error: Some(CommandRejection {
                code: "invalid_state".to_string(),
                details: BTreeMap::from([(
                    "field".to_string(),
                    ErrorDetail::String("ticks".into()),
                )]),
                message: "Game is paused".to_string(),
            }),
        });
    }

    #[test]
    fn command_status_variants_round_trip() {
        round_trip(&CommandStatus::Accepted);
        round_trip(&CommandStatus::Applied);
        round_trip(&CommandStatus::Rejected);
        round_trip(&CommandStatus::Failed);
    }
}
