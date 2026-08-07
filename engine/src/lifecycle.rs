//! Lifecycle state machine and runtime status types.
//!
//! Defines the `GameLifecycle` transition table, `LoadingStatus` for the
//! Loading lifecycle, `ServerStatus` for the runtime status publication,
//! and helpers for validating transitions.
//!
//! ## Authoritative references
//!
//! - ADR-0004 §Game Lifecycle State Machine
//! - ADR-0004 §States and Commands
//! - ADR-0004 §Atomicity
//! - TDD 00 §Simulation Actor
//! - TDD 02 §Queries, §Errors

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::*;
use crate::types::*;

// ─── Loading status ───────────────────────────────────────────────────

/// The operation being performed during Loading.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum LoadingOperation {
    /// Constructing a new game from a scenario.
    NewGame,
    /// Loading a saved game from disk.
    LoadAutosave,
    /// Initial autoload at server startup.
    StartupAutoload,
}

/// The stage within a loading operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum LoadingStage {
    /// Validating content definitions.
    ValidatingContent,
    /// Constructing starting scenario state.
    ConstructingScenario,
    /// Reading save file from disk.
    ReadingSave,
    /// Verifying save envelope integrity.
    VerifyingEnvelope,
    /// Running schema migrations.
    Migrating,
    /// Validating deserialized state invariants.
    ValidatingState,
    /// Publishing the new state and transitioning lifecycle.
    Publishing,
}

/// Progress reported during the Loading lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct LoadingStatus {
    /// The loading operation being performed.
    pub operation: LoadingOperation,
    /// The current stage within the loading operation.
    pub stage: LoadingStage,
}

// ─── Server status ───────────────────────────────────────────────────

/// Runtime status published by the actor for query endpoints.
#[derive(Debug, Clone, PartialEq, Eq, JsonSchema)]
pub struct ServerStatus {
    /// Protocol version string.
    pub protocol_version: String,
    /// Server readiness indicator.
    pub server: String,
    /// Current game lifecycle state.
    pub game_state: GameLifecycle,
    /// Current simulation tick, if a game is loaded.
    pub tick: Option<u64>,
    /// The latest event sequence number.
    pub latest_event_sequence: u64,
    /// Schema version of the loaded game state.
    pub schema_version: Option<u32>,
    /// Content version string of the loaded game.
    pub content_version: Option<String>,
    /// Loading progress during the Loading lifecycle.
    pub loading: Option<LoadingStatus>,
}

impl ServerStatus {
    /// Build a status for the Unloaded lifecycle (no game state).
    pub fn unloaded(latest_event_sequence: u64) -> Self {
        ServerStatus {
            protocol_version: "v1".to_string(),
            server: "ready".to_string(),
            game_state: GameLifecycle::Unloaded,
            tick: None,
            latest_event_sequence,
            schema_version: None,
            content_version: None,
            loading: None,
        }
    }

    /// Build a status for the Loading lifecycle.
    pub fn loading(
        latest_event_sequence: u64,
        operation: LoadingOperation,
        stage: LoadingStage,
    ) -> Self {
        ServerStatus {
            protocol_version: "v1".to_string(),
            server: "ready".to_string(),
            game_state: GameLifecycle::Loading,
            tick: None,
            latest_event_sequence,
            schema_version: None,
            content_version: None,
            loading: Some(LoadingStatus { operation, stage }),
        }
    }

    /// Build a status from a stable game snapshot.
    pub fn from_snapshot(snapshot: &GameSnapshot) -> Self {
        ServerStatus {
            protocol_version: "v1".to_string(),
            server: "ready".to_string(),
            game_state: snapshot.state.lifecycle,
            tick: Some(snapshot.state.tick),
            latest_event_sequence: snapshot.latest_event_sequence,
            schema_version: Some(snapshot.state.schema_version),
            content_version: Some(snapshot.state.content_version.clone()),
            loading: None,
        }
    }
}

// ─── Lifecycle transition validation ─────────────────────────────────

/// A typed error for invalid lifecycle transitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecycleError {
    /// The requested transition is not allowed from the current lifecycle.
    InvalidTransition {
        /// The current lifecycle before the transition.
        current: GameLifecycle,
        /// The lifecycle that was requested but is not allowed.
        requested: GameLifecycle,
        /// A human-readable explanation of why the transition is invalid.
        message: String,
    },
    /// The command is not valid in the current lifecycle.
    InvalidCommand {
        /// The lifecycle in which the command was attempted.
        lifecycle: GameLifecycle,
        /// The type or name of the command that was rejected.
        command_type: String,
    },
    /// Loading failed and rollback completed.
    LoadFailed(String),
}

impl std::fmt::Display for LifecycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LifecycleError::InvalidTransition {
                current,
                requested,
                message,
            } => write!(
                f,
                "invalid lifecycle transition from {:?} to {:?}: {}",
                current, requested, message
            ),
            LifecycleError::InvalidCommand {
                lifecycle,
                command_type,
            } => write!(
                f,
                "command '{}' is not valid in lifecycle {:?}",
                command_type, lifecycle
            ),
            LifecycleError::LoadFailed(msg) => write!(f, "load failed: {}", msg),
        }
    }
}

/// Result type for lifecycle operations.
pub type LifecycleResult<T = ()> = Result<T, LifecycleError>;

/// Validate a lifecycle transition according to ADR-0004's state machine.
///
/// Returns `Err(LifecycleError::InvalidTransition)` if the transition is not
/// allowed.
pub fn validate_transition(current: GameLifecycle, next: GameLifecycle) -> LifecycleResult {
    use GameLifecycle::*;

    let allowed = match (current, next) {
        // Unloaded --NewGame/LoadAutosave--> Loading
        (Unloaded, Loading) => true,
        // Loading --success--> Paused or Won; --failure--> Unloaded
        (Loading, Paused) | (Loading, Won) => true,
        (Loading, Unloaded) => true,

        // Paused <--Pause/Resume--> Running
        (Paused, Running) => true,
        (Running, Paused) => true,

        // Paused --AdvanceTicks(N)--> Advancing
        (Paused, Advancing) => true,

        // Advancing --complete/error--> Paused
        (Advancing, Paused) => true,

        // Paused/Running --GateComplete--> Won
        (Paused, Won) | (Running, Won) => true,

        // Paused/Running/Won --NewGame/LoadAutosave--> Loading
        (Paused, Loading) | (Running, Loading) | (Won, Loading) => true,

        // Everything else is invalid.
        _ => false,
    };

    if allowed {
        Ok(())
    } else {
        Err(LifecycleError::InvalidTransition {
            current,
            requested: next,
            message: format!("no transition from {:?} to {:?} exists", current, next),
        })
    }
}

/// Validate that a command type is allowed in the given lifecycle.
///
/// Returns `Err(LifecycleError::InvalidCommand)` if the command is not
/// valid.
pub fn validate_command_for_lifecycle(
    lifecycle: GameLifecycle,
    command_type: &str,
) -> LifecycleResult {
    use GameLifecycle::*;

    let allowed = match lifecycle {
        Unloaded => {
            // Only NewGame and LoadAutosave are allowed.
            matches!(command_type, "NewGame" | "LoadAutosave")
        }
        Loading => {
            // No commands allowed during loading.
            false
        }
        Paused => {
            // All gameplay/configuration commands, Resume, AdvanceTicks,
            // SaveNow, NewGame, LoadAutosave.
            true
        }
        Running => {
            // All gameplay/configuration commands, Pause, SaveNow,
            // NewGame, LoadAutosave.  AdvanceTicks is unavailable.
            command_type != "AdvanceTicks"
        }
        Advancing => {
            // No commands allowed during advancement.
            false
        }
        Won => {
            // Only queries, SaveNow, NewGame, LoadAutosave.
            matches!(command_type, "SaveNow" | "NewGame" | "LoadAutosave")
        }
    };

    if allowed {
        Ok(())
    } else {
        Err(LifecycleError::InvalidCommand {
            lifecycle,
            command_type: command_type.to_string(),
        })
    }
}

/// The allowed-state set for error responses, as a list of lifecycle names.
pub fn allowed_states(lifecycle: GameLifecycle) -> Vec<String> {
    use GameLifecycle::*;
    match lifecycle {
        Unloaded => vec!["Unloaded".to_string()],
        Loading => vec!["Loading".to_string()],
        Paused => vec!["Paused".to_string()],
        Running => vec!["Running".to_string()],
        Advancing => vec!["Advancing".to_string()],
        Won => vec!["Won".to_string()],
    }
}

// ─── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    // ─── Transition table tests ───────────────────────────────────────

    /// Every allowed transition in ADR-0004's state machine is valid.
    #[test]
    fn allowed_transitions_are_valid() {
        // Unloaded -> Loading
        assert!(validate_transition(GameLifecycle::Unloaded, GameLifecycle::Loading).is_ok());
        // Loading -> Paused (success)
        assert!(validate_transition(GameLifecycle::Loading, GameLifecycle::Paused).is_ok());
        // Loading -> Won (loaded a Won save)
        assert!(validate_transition(GameLifecycle::Loading, GameLifecycle::Won).is_ok());
        // Loading -> Unloaded (failure)
        assert!(validate_transition(GameLifecycle::Loading, GameLifecycle::Unloaded).is_ok());
        // Paused -> Running
        assert!(validate_transition(GameLifecycle::Paused, GameLifecycle::Running).is_ok());
        // Running -> Paused
        assert!(validate_transition(GameLifecycle::Running, GameLifecycle::Paused).is_ok());
        // Paused -> Advancing
        assert!(validate_transition(GameLifecycle::Paused, GameLifecycle::Advancing).is_ok());
        // Advancing -> Paused
        assert!(validate_transition(GameLifecycle::Advancing, GameLifecycle::Paused).is_ok());
        // Paused -> Won
        assert!(validate_transition(GameLifecycle::Paused, GameLifecycle::Won).is_ok());
        // Running -> Won
        assert!(validate_transition(GameLifecycle::Running, GameLifecycle::Won).is_ok());
        // Paused -> Loading
        assert!(validate_transition(GameLifecycle::Paused, GameLifecycle::Loading).is_ok());
        // Running -> Loading
        assert!(validate_transition(GameLifecycle::Running, GameLifecycle::Loading).is_ok());
        // Won -> Loading
        assert!(validate_transition(GameLifecycle::Won, GameLifecycle::Loading).is_ok());
        // Loading -> Running is not a valid transition (ADR-0004 §Save Normalization)
        assert!(validate_transition(GameLifecycle::Loading, GameLifecycle::Running).is_err());
    }

    /// Every disallowed transition is rejected.
    #[test]
    fn disallowed_transitions_are_rejected() {
        use GameLifecycle::*;

        // Unloaded can only go to Loading.
        assert!(validate_transition(Unloaded, Paused).is_err());
        assert!(validate_transition(Unloaded, Running).is_err());
        assert!(validate_transition(Unloaded, Advancing).is_err());
        assert!(validate_transition(Unloaded, Won).is_err());
        assert!(validate_transition(Unloaded, Unloaded).is_err());

        // Loading cannot stay Loading or jump to Advancing.
        assert!(validate_transition(Loading, Advancing).is_err());

        // Paused cannot stay Paused.
        assert!(validate_transition(Paused, Paused).is_err());
        assert!(validate_transition(Paused, Advancing).is_ok()); // allowed

        // Running cannot go to Advancing.
        assert!(validate_transition(Running, Advancing).is_err());

        // Advancing can only go to Paused.
        assert!(validate_transition(Advancing, Loading).is_err());
        assert!(validate_transition(Advancing, Running).is_err());
        assert!(validate_transition(Advancing, Won).is_err());

        // Won cannot go to Running or Advancing.
        assert!(validate_transition(Won, Running).is_err());
        assert!(validate_transition(Won, Advancing).is_err());
    }

    /// Error message contains the current and requested lifecycle.
    #[test]
    fn invalid_transition_error_message() {
        let err = validate_transition(GameLifecycle::Unloaded, GameLifecycle::Running).unwrap_err();
        match err {
            LifecycleError::InvalidTransition {
                current, requested, ..
            } => {
                assert_eq!(current, GameLifecycle::Unloaded);
                assert_eq!(requested, GameLifecycle::Running);
            }
            _ => panic!("expected InvalidTransition"),
        }
    }

    // ─── Command lifecycle validation tests ───────────────────────────

    /// In Unloaded, only NewGame and LoadAutosave are valid.
    #[test]
    fn unloaded_allows_only_newgame_and_loadautosave() {
        assert!(validate_command_for_lifecycle(GameLifecycle::Unloaded, "NewGame").is_ok());
        assert!(validate_command_for_lifecycle(GameLifecycle::Unloaded, "LoadAutosave").is_ok());
        assert!(validate_command_for_lifecycle(GameLifecycle::Unloaded, "Pause").is_err());
        assert!(validate_command_for_lifecycle(GameLifecycle::Unloaded, "QueueBuildShip").is_err());
    }

    /// In Loading, no commands are allowed.
    #[test]
    fn loading_allows_no_commands() {
        assert!(validate_command_for_lifecycle(GameLifecycle::Loading, "NewGame").is_err());
        assert!(validate_command_for_lifecycle(GameLifecycle::Loading, "SaveNow").is_err());
    }

    /// In Paused, all commands are allowed.
    #[test]
    fn paused_allows_all_commands() {
        assert!(validate_command_for_lifecycle(GameLifecycle::Paused, "Resume").is_ok());
        assert!(validate_command_for_lifecycle(GameLifecycle::Paused, "AdvanceTicks").is_ok());
        assert!(validate_command_for_lifecycle(GameLifecycle::Paused, "QueueBuildShip").is_ok());
        assert!(validate_command_for_lifecycle(GameLifecycle::Paused, "SaveNow").is_ok());
    }

    /// In Running, AdvanceTicks is disallowed.
    #[test]
    fn running_disallows_advance_ticks() {
        assert!(validate_command_for_lifecycle(GameLifecycle::Running, "AdvanceTicks").is_err());
        assert!(validate_command_for_lifecycle(GameLifecycle::Running, "Pause").is_ok());
        assert!(validate_command_for_lifecycle(GameLifecycle::Running, "QueueBuildShip").is_ok());
    }

    /// In Advancing, no commands are allowed.
    #[test]
    fn advancing_allows_no_commands() {
        assert!(validate_command_for_lifecycle(GameLifecycle::Advancing, "Pause").is_err());
        assert!(
            validate_command_for_lifecycle(GameLifecycle::Advancing, "QueueBuildShip").is_err()
        );
    }

    /// In Won, only SaveNow, NewGame, and LoadAutosave are valid.
    #[test]
    fn won_allows_save_newgame_loadautosave() {
        assert!(validate_command_for_lifecycle(GameLifecycle::Won, "SaveNow").is_ok());
        assert!(validate_command_for_lifecycle(GameLifecycle::Won, "NewGame").is_ok());
        assert!(validate_command_for_lifecycle(GameLifecycle::Won, "LoadAutosave").is_ok());
        assert!(validate_command_for_lifecycle(GameLifecycle::Won, "Pause").is_err());
        assert!(validate_command_for_lifecycle(GameLifecycle::Won, "QueueBuildShip").is_err());
    }

    // ─── ServerStatus tests ───────────────────────────────────────────

    /// Unloaded status has no game state fields.
    #[test]
    fn server_status_unloaded() {
        let status = ServerStatus::unloaded(0);
        assert_eq!(status.game_state, GameLifecycle::Unloaded);
        assert!(status.tick.is_none());
        assert!(status.schema_version.is_none());
        assert!(status.content_version.is_none());
        assert!(status.loading.is_none());
    }

    /// Loading status reports the operation and stage.
    #[test]
    fn server_status_loading() {
        let status = ServerStatus::loading(
            0,
            LoadingOperation::NewGame,
            LoadingStage::ConstructingScenario,
        );
        assert_eq!(status.game_state, GameLifecycle::Loading);
        assert!(status.tick.is_none());
        assert!(status.loading.is_some());
        let loading = status.loading.as_ref().unwrap();
        assert_eq!(loading.operation, LoadingOperation::NewGame);
        assert_eq!(loading.stage, LoadingStage::ConstructingScenario);
    }

    /// Snapshot-derived status reflects the game state.
    #[test]
    fn server_status_from_snapshot() {
        use std::collections::BTreeMap;
        let state = GameState {
            schema_version: 1,
            content_version: "v1".to_string(),
            lifecycle: GameLifecycle::Paused,
            tick: 42,
            next_server_sequence: 1,
            next_event_sequence: 100,
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
        let snapshot = GameSnapshot {
            protocol_version: "v1".to_string(),
            latest_event_sequence: 99,
            state,
        };
        let status = ServerStatus::from_snapshot(&snapshot);
        assert_eq!(status.game_state, GameLifecycle::Paused);
        assert_eq!(status.tick, Some(42));
        assert_eq!(status.schema_version, Some(1));
        assert_eq!(status.content_version, Some("v1".to_string()));
        assert!(status.loading.is_none());
    }
}
