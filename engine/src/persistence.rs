//! Persistence: save envelope, FIFO worker, platform atomic replacement,
//! schema migration framework, and load validation.
//!
//! ## Authoritative references
//!
//! - ADR-0007 — Save Envelope Format, Content Hash Placement, and Migration Fixtures
//! - ADR-0008 — Accepted-Command Persistence
//! - ADR-0006 — Canonical Content/State Hashing
//!
//! ## Save envelope v1
//!
//! Every V1 save is a JSON object with exactly these fields:
//!
//! ```json
//! { "format_version": 1, "hash_scheme": "sha256-canonical-json-v1",
//!   "schema_version": 1, "content_version": "v1",
//!   "content_hash": "...", "state_hash": "...",
//!   "timestamp": "2026-08-04T12:00:00Z",
//!   "game_state": { "...": "..." } }
//! ```
//!
//! See ADR-0007 for the full contract.

#![allow(missing_docs)]

use std::fmt;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::state::GameState;
use crate::state_hash::{compute_state_hash, format_state_hash};

// ─── Save envelope ────────────────────────────────────────────────────

/// The V1 save envelope structure (ADR-0007 §1).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveEnvelope {
    pub format_version: u32,
    pub hash_scheme: String,
    pub schema_version: u32,
    pub content_version: String,
    pub content_hash: String,
    pub state_hash: String,
    pub timestamp: String,
    pub game_state: GameState,
}

impl SaveEnvelope {
    /// Build a V1 save envelope from a `GameState` snapshot.
    ///
    /// `content_version` and `content_hash` come from the running catalog.
    /// `state_hash` is computed from the normalized state.  The timestamp is
    /// informational and excluded from deterministic hashes.
    pub fn new(
        game_state: GameState,
        content_version: &str,
        content_hash: &str,
    ) -> Result<Self, SaveError> {
        // ── 1. Normalize lifecycle ─────────────────────────────────────
        let mut normalized = game_state.clone();
        match normalized.lifecycle {
            crate::types::GameLifecycle::Running | crate::types::GameLifecycle::Advancing => {
                normalized.lifecycle = crate::types::GameLifecycle::Paused;
            }
            crate::types::GameLifecycle::Won => { /* stays Won */ }
            _ => { /* Paused or Loading/Unloaded cannot be saved — caller validates */ }
        }

        // ── 2. Compute state hash over normalized state ────────────────
        let state_hash_bytes = compute_state_hash(&normalized)
            .map_err(|e| SaveError::Hash(format!("state hash failed: {}", e)))?;
        let state_hash_str = format_state_hash(&state_hash_bytes);

        // ── 3. Timestamp ───────────────────────────────────────────────
        let timestamp = format_timestamp();

        Ok(SaveEnvelope {
            format_version: 1,
            hash_scheme: "sha256-canonical-json-v1".to_string(),
            schema_version: normalized.schema_version,
            content_version: content_version.to_string(),
            content_hash: content_hash.to_string(),
            state_hash: state_hash_str,
            timestamp,
            game_state: normalized,
        })
    }
}

// ─── Save errors ───────────────────────────────────────────────────────

/// Typed errors produced by save/load operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveError {
    /// I/O error.
    Io(String),
    /// JSON serialization or deserialization failed.
    Json(String),
    /// Hash computation failed.
    Hash(String),
    /// Invalid envelope format.
    Format(String),
    /// Content version or hash mismatch.
    ContentMismatch(String),
    /// State hash integrity check failed.
    StateHashMismatch(String),
    /// Schema version unsupported or migration chain incomplete.
    SchemaVersion(String),
    /// Command log validation failed.
    CommandLog(String),
    /// Invariant check failed after load.
    Invariant(String),
    /// Lifecycle not savable.
    Lifecycle(String),
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveError::Io(msg) => write!(f, "I/O error: {}", msg),
            SaveError::Json(msg) => write!(f, "JSON error: {}", msg),
            SaveError::Hash(msg) => write!(f, "hash error: {}", msg),
            SaveError::Format(msg) => write!(f, "format error: {}", msg),
            SaveError::ContentMismatch(msg) => {
                write!(f, "content mismatch: {}", msg)
            }
            SaveError::StateHashMismatch(msg) => {
                write!(f, "state hash mismatch: {}", msg)
            }
            SaveError::SchemaVersion(msg) => {
                write!(f, "schema version error: {}", msg)
            }
            SaveError::CommandLog(msg) => {
                write!(f, "command log error: {}", msg)
            }
            SaveError::Invariant(msg) => write!(f, "invariant error: {}", msg),
            SaveError::Lifecycle(msg) => write!(f, "lifecycle error: {}", msg),
        }
    }
}

// ─── Schema migration framework ────────────────────────────────────────

/// Typed error for the schema-migration framework (ADR-0007 §6).
#[derive(Debug)]
pub enum MigrationError {
    /// A migration step registered is not adjacent (to != from + 1).
    NotAdjacent { from: u32, to: u32 },
    /// Two migrations share a starting version.
    DuplicateFrom(u32),
    /// No migrations are expected at schema v1.
    UnexpectedMigration(u32),
    /// The chain's next step does not start at the expected version.
    ChainMismatch { expected: u32, found: u32 },
    /// A chain step is not adjacent.
    NotAdjacentChain { from: u32, to: u32 },
    /// The chain ends before reaching the current schema version.
    ChainEnds { ends: u32, current: u32 },
    /// A saved save is newer than the current schema version.
    FutureVersion { saved: u32, current: u32 },
    /// The migration chain stopped before the target version.
    StoppedAt { stopped: u32, target: u32 },
}

impl std::fmt::Display for MigrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrationError::NotAdjacent { from, to } => write!(
                f,
                "migration {} -> {} is not adjacent (to != from + 1)",
                from, to
            ),
            MigrationError::DuplicateFrom(from) => {
                write!(f, "duplicate migration from version {}", from)
            }
            MigrationError::UnexpectedMigration(v) => {
                write!(f, "no migrations expected at schema v{}", v)
            }
            MigrationError::ChainMismatch { expected, found } => write!(
                f,
                "expected migration from {} but found from {}",
                expected, found
            ),
            MigrationError::NotAdjacentChain { from, to } => {
                write!(f, "migration {} -> {} is not adjacent", from, to)
            }
            MigrationError::ChainEnds { ends, current } => write!(
                f,
                "migration chain ends at {} but current version is {}",
                ends, current
            ),
            MigrationError::FutureVersion { saved, current } => {
                write!(f, "saved schema version {} > current {}", saved, current)
            }
            MigrationError::StoppedAt { stopped, target } => write!(
                f,
                "migration stopped at {} but target is {}",
                stopped, target
            ),
        }
    }
}

impl std::error::Error for MigrationError {}

/// A single schema migration step (ADR-0007 §6).
#[allow(clippy::wrong_self_convention)]
pub trait Migration: Send + Sync {
    fn from_version(&self) -> u32;
    fn to_version(&self) -> u32;
    fn migrate(&self, state: Value) -> Result<Value, MigrationError>;
}

/// Registry that validates a contiguous migration chain.
///
/// NOTE (deferred, L5): this registry only *validates* the migration chain —
/// it does not apply migrations to real save data.  Actual save migration is
/// deferred to a later increment; do not mistake this validation-only registry
/// for a live migrator.
pub struct MigrationRegistry {
    current_schema_version: u32,
    migrations: Vec<Box<dyn Migration>>,
}

impl MigrationRegistry {
    /// Create a new registry for the current schema version.
    pub fn new(current_schema_version: u32) -> Self {
        MigrationRegistry {
            current_schema_version,
            migrations: Vec::new(),
        }
    }

    /// Register a migration.  Validates adjacency and no duplicates.
    pub fn register(&mut self, migration: Box<dyn Migration>) -> Result<(), MigrationError> {
        let from = migration.from_version();
        let to = migration.to_version();
        if to != from + 1 {
            return Err(MigrationError::NotAdjacent { from, to });
        }
        // Check for duplicate from_version
        for m in &self.migrations {
            if m.from_version() == from {
                return Err(MigrationError::DuplicateFrom(from));
            }
        }
        self.migrations.push(migration);
        Ok(())
    }

    /// Validate that every version from 1 to `current_schema_version` has
    /// exactly one migration step.
    pub fn validate_chain(&self) -> Result<(), MigrationError> {
        if self.current_schema_version == 1 {
            // Schema v1 needs no migrations — chain is trivially valid.
            if !self.migrations.is_empty() {
                return Err(MigrationError::UnexpectedMigration(
                    self.current_schema_version,
                ));
            }
            return Ok(());
        }
        // Build a map of from -> to
        let mut map: Vec<(u32, u32)> = Vec::new();
        for m in &self.migrations {
            map.push((m.from_version(), m.to_version()));
        }
        map.sort_by_key(|&(from, _to)| from);

        let mut expected = 1u32;
        for (from, to) in &map {
            if *from != expected {
                return Err(MigrationError::ChainMismatch {
                    expected,
                    found: *from,
                });
            }
            if *to != *from + 1 {
                return Err(MigrationError::NotAdjacentChain {
                    from: *from,
                    to: *to,
                });
            }
            expected = *to;
        }
        if expected != self.current_schema_version {
            return Err(MigrationError::ChainEnds {
                ends: expected,
                current: self.current_schema_version,
            });
        }
        Ok(())
    }

    /// Run the migration chain on a raw JSON value to reach
    /// `current_schema_version`.  Returns the migrated value.
    pub fn migrate(&self, mut state: Value) -> Result<Value, MigrationError> {
        // Read initial schema_version from the value
        let initial_version = state
            .as_object()
            .and_then(|o| o.get("schema_version"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        if initial_version == self.current_schema_version {
            return Ok(state);
        }
        if initial_version > self.current_schema_version {
            return Err(MigrationError::FutureVersion {
                saved: initial_version,
                current: self.current_schema_version,
            });
        }

        // Apply migrations in order
        let mut current_version = initial_version;
        for m in &self.migrations {
            if m.from_version() == current_version {
                state = m.migrate(state)?;
                current_version = m.to_version();
            }
        }

        if current_version != self.current_schema_version {
            return Err(MigrationError::StoppedAt {
                stopped: current_version,
                target: self.current_schema_version,
            });
        }
        Ok(state)
    }
}

// ─── Persistence worker ────────────────────────────────────────────────

/// Result of a write operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteResult {
    Success,
    Failure(SaveError),
}

/// Atomically write a save envelope to a target path.
///
/// 1. Create a uniquely named temporary file in the same directory.
/// 2. Serialize and write all bytes.
/// 3. Sync the file.
/// 4. Atomically replace the target.
/// 5. Sync the containing directory (if supported).
///    On failure, remove only the known temp file; prior save remains intact.
///
/// The actor's SaveNow handler calls this function directly (synchronous) to
/// avoid the async acknowledgment problem (ADR-0008 §4 — the receipt must
/// become Applied only after the atomic write succeeds).
pub(crate) fn sync_atomic_write(target: &Path, envelope: &SaveEnvelope) -> WriteResult {
    let parent = target.parent().unwrap_or(Path::new("."));
    let temp_name = format!(
        ".steel-horizons-save-tmp-{}.tmp",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    let temp_path = parent.join(&temp_name);

    // Serialize envelope to JSON bytes
    let json_bytes = match serde_json::to_vec_pretty(envelope) {
        Ok(b) => b,
        Err(e) => return WriteResult::Failure(SaveError::Json(e.to_string())),
    };

    // Write to temp file with exclusive create (std::fs — no async needed)
    let mut file = match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp_path)
    {
        Ok(f) => f,
        Err(e) => {
            let _ = std::fs::remove_file(&temp_path);
            return WriteResult::Failure(SaveError::Io(e.to_string()));
        }
    };

    // Write all bytes and sync
    use std::io::Write;
    if let Err(e) = file.write_all(&json_bytes).and_then(|_| file.sync_all()) {
        let _ = std::fs::remove_file(&temp_path);
        return WriteResult::Failure(SaveError::Io(e.to_string()));
    }

    // Atomic rename
    if let Err(e) = std::fs::rename(&temp_path, target) {
        let _ = std::fs::remove_file(&temp_path);
        return WriteResult::Failure(SaveError::Io(e.to_string()));
    }

    // Sync parent directory (best-effort on macOS)
    if let Some(parent_path) = target.parent() {
        if let Ok(dir) = std::fs::File::open(parent_path) {
            let _ = dir.sync_all();
        }
    }

    WriteResult::Success
}

/// Read and validate a save envelope from a file path.
pub fn read_save_envelope(path: &Path) -> Result<SaveEnvelope, SaveError> {
    let data = std::fs::read(path).map_err(|e| SaveError::Io(format!("read error: {}", e)))?;
    let envelope: SaveEnvelope = serde_json::from_slice(&data)
        .map_err(|e| SaveError::Json(format!("deserialize error: {}", e)))?;

    // Validate envelope format version
    if envelope.format_version != 1 {
        return Err(SaveError::Format(format!(
            "unsupported format version {}",
            envelope.format_version
        )));
    }

    // Validate hash scheme
    if envelope.hash_scheme != "sha256-canonical-json-v1" {
        return Err(SaveError::Format(format!(
            "unsupported hash scheme '{}'",
            envelope.hash_scheme
        )));
    }

    // Validate hash string format (64 lowercase hex chars)
    if envelope.state_hash.len() != 64 || envelope.content_hash.len() != 64 {
        return Err(SaveError::Format(
            "hash strings must be 64 lowercase hex characters".to_string(),
        ));
    }

    Ok(envelope)
}

/// Validate the loaded state against the save envelope and content.
///
/// Checks:
/// - `content_version` matches envelope and running catalog
/// - `content_hash` matches computed catalog hash
/// - `state_hash` matches re-computed hash over deserialized state
/// - `schema_version` matches envelope (before any migration)
/// - Command log invariants (ADR-0008 §5)
pub fn validate_loaded_state(
    envelope: &SaveEnvelope,
    state: &GameState,
    content_version: &str,
    content_hash: &str,
) -> Result<(), SaveError> {
    // Content version must match envelope
    if state.content_version != envelope.content_version {
        return Err(SaveError::ContentMismatch(format!(
            "state content_version '{}' != envelope content_version '{}'",
            state.content_version, envelope.content_version
        )));
    }
    if envelope.content_version != content_version {
        return Err(SaveError::ContentMismatch(format!(
            "envelope content_version '{}' != catalog content_version '{}'",
            envelope.content_version, content_version
        )));
    }
    if envelope.content_hash != content_hash {
        return Err(SaveError::ContentMismatch(format!(
            "envelope content_hash '{}' != computed content_hash '{}'",
            envelope.content_hash, content_hash
        )));
    }

    // Schema version must match envelope
    if state.schema_version != envelope.schema_version {
        return Err(SaveError::SchemaVersion(format!(
            "state schema_version {} != envelope schema_version {}",
            state.schema_version, envelope.schema_version
        )));
    }

    // Verify state hash
    let computed_hash_bytes = compute_state_hash(state)
        .map_err(|e| SaveError::Hash(format!("state hash failed: {}", e)))?;
    let computed_hash_str = format_state_hash(&computed_hash_bytes);
    if computed_hash_str != envelope.state_hash {
        return Err(SaveError::StateHashMismatch(
            "state hash does not match envelope".to_string(),
        ));
    }

    // Validate command log invariants (ADR-0008 §5)
    validate_command_log(state)?;

    Ok(())
}

/// Validate command log invariants per ADR-0008 §5.
pub fn validate_command_log(state: &GameState) -> Result<(), SaveError> {
    let log = &state.command_log;
    let mut prev_sequence: Option<u64> = None;
    let mut prev_implied_tick: Option<u64> = None;
    let mut seen_ids = std::collections::BTreeSet::new();

    for record in log {
        // Every record must contain a replayable game command
        // (already guaranteed by type system — `command: ReplayableGameCommand`)

        // Command IDs are non-empty
        if record.id.is_empty() {
            return Err(SaveError::CommandLog("empty command id".to_string()));
        }

        // Unique IDs within the log
        if !seen_ids.insert(record.id.clone()) {
            return Err(SaveError::CommandLog(format!(
                "duplicate command id '{}'",
                record.id
            )));
        }

        // server_sequence values are unique and strictly increasing
        if let Some(prev) = prev_sequence {
            if record.server_sequence <= prev {
                return Err(SaveError::CommandLog(format!(
                    "server_sequence {} <= previous {}",
                    record.server_sequence, prev
                )));
            }
        }
        prev_sequence = Some(record.server_sequence);

        // Implied acceptance ticks are nondecreasing in server-sequence order
        // (ADR-0008 §5).  ScheduledTick -> effective_tick - 1, PausedImmediate -> effective_tick.
        let implied_tick = match record.application_boundary {
            crate::types::CommandApplicationBoundary::ScheduledTick => {
                record.effective_tick.saturating_sub(1)
            }
            crate::types::CommandApplicationBoundary::PausedImmediate => record.effective_tick,
        };
        if let Some(prev_implied) = prev_implied_tick {
            if implied_tick < prev_implied {
                return Err(SaveError::CommandLog(format!(
                    "implied acceptance tick {} < previous {}",
                    implied_tick, prev_implied
                )));
            }
        }
        prev_implied_tick = Some(implied_tick);

        // next_server_sequence > every record sequence (checked after load)

        // Implied acceptance tick invariants
        match record.application_boundary {
            crate::types::CommandApplicationBoundary::ScheduledTick => {
                if record.effective_tick == 0 {
                    return Err(SaveError::CommandLog(
                        "ScheduledTick effective_tick must be > 0".to_string(),
                    ));
                }
            }
            crate::types::CommandApplicationBoundary::PausedImmediate => {
                // effective_tick can be any tick (including 0 for tick-zero commands)
            }
        }

        // Outcome-specific invariants
        match record.outcome {
            crate::types::CommandOutcome::Accepted => {
                // Accepted records are ScheduledTick with effective_tick > tick
                if record.effective_tick <= state.tick {
                    return Err(SaveError::CommandLog(format!(
                        "Accepted record effective_tick {} <= tick {}",
                        record.effective_tick, state.tick
                    )));
                }
                if record.result.is_some() {
                    return Err(SaveError::CommandLog(
                        "Accepted record must have null result".to_string(),
                    ));
                }
                if record.rejection.is_some() {
                    return Err(SaveError::CommandLog(
                        "Accepted record must have null rejection".to_string(),
                    ));
                }
            }
            crate::types::CommandOutcome::Applied => {
                if record.effective_tick > state.tick {
                    return Err(SaveError::CommandLog(format!(
                        "Applied record effective_tick {} > tick {}",
                        record.effective_tick, state.tick
                    )));
                }
                if record.result.is_none() {
                    return Err(SaveError::CommandLog(
                        "Applied record must have non-null result".to_string(),
                    ));
                }
                if record.rejection.is_some() {
                    return Err(SaveError::CommandLog(
                        "Applied record must have null rejection".to_string(),
                    ));
                }
            }
            crate::types::CommandOutcome::Rejected => {
                // ADR-0008 §5 permits TickTransactionFailed records at tick + 1
                if record.effective_tick > state.tick + 1 {
                    return Err(SaveError::CommandLog(format!(
                        "Rejected record effective_tick {} > tick {} + 1",
                        record.effective_tick, state.tick
                    )));
                }
                if record.result.is_some() {
                    return Err(SaveError::CommandLog(
                        "Rejected record must have null result".to_string(),
                    ));
                }
                if record.rejection.is_none() {
                    return Err(SaveError::CommandLog(
                        "Rejected record must have non-null rejection".to_string(),
                    ));
                }
            }
        }
    }

    // Verify next_server_sequence > every record sequence
    if let Some(max_seq) = log.iter().map(|r| r.server_sequence).max() {
        if state.next_server_sequence <= max_seq {
            return Err(SaveError::CommandLog(format!(
                "next_server_sequence {} <= max record sequence {}",
                state.next_server_sequence, max_seq
            )));
        }
    }

    Ok(())
}

/// Convert a `ReplayableGameCommand` to the full `Command` enum.
///
/// This is the inverse of `Command -> ReplayableGameCommand` via the
/// `From<&Command>` impl.  Since no `From<ReplayableGameCommand> for Command`
/// exists in the command module, we implement it here for use during
/// command-log replay.
fn replayable_to_command(cmd: &crate::command::ReplayableGameCommand) -> crate::command::Command {
    use crate::command::Command;
    use crate::command::ReplayableGameCommand as RGC;

    match cmd {
        RGC::QueueBuildShip { hub_id, role, tier } => Command::QueueBuildShip {
            hub_id: hub_id.clone(),
            role: *role,
            tier: *tier,
        },
        RGC::QueueBuildStation {
            source_hub_id,
            body_id,
            orbit_ring,
            slot,
            station_type,
            tier,
        } => Command::QueueBuildStation {
            source_hub_id: source_hub_id.clone(),
            body_id: body_id.clone(),
            orbit_ring: *orbit_ring,
            slot: *slot,
            station_type: *station_type,
            tier: *tier,
        },
        RGC::QueueUpgrade {
            source_hub_id,
            station_id,
            target_tier,
        } => Command::QueueUpgrade {
            source_hub_id: source_hub_id.clone(),
            station_id: station_id.clone(),
            target_tier: *target_tier,
        },
        RGC::CancelBuildOrder { order_id } => Command::CancelBuildOrder {
            order_id: order_id.clone(),
        },
        RGC::QueueDemolishStation {
            station_id,
            recovery_hub_id,
        } => Command::QueueDemolishStation {
            station_id: station_id.clone(),
            recovery_hub_id: recovery_hub_id.clone(),
        },
        RGC::ScrapShip { ship_id } => Command::ScrapShip {
            ship_id: ship_id.clone(),
        },
        RGC::BeginGateAssembly { fabricator_ship_id } => Command::BeginGateAssembly {
            fabricator_ship_id: fabricator_ship_id.clone(),
        },
        RGC::SetStationPriority {
            station_id,
            priority,
        } => Command::SetStationPriority {
            station_id: station_id.clone(),
            priority: *priority,
        },
        RGC::ConfigureBuffer {
            station_id,
            configuration,
        } => Command::ConfigureBuffer {
            station_id: station_id.clone(),
            configuration: configuration.clone(),
        },
        RGC::SetProductionRecipe {
            station_id,
            slot_index,
            recipe_id,
        } => Command::SetProductionRecipe {
            station_id: station_id.clone(),
            slot_index: *slot_index,
            recipe_id: recipe_id.clone(),
        },
        RGC::SetMiningTarget {
            station_id,
            slot_index,
            resource,
        } => Command::SetMiningTarget {
            station_id: station_id.clone(),
            slot_index: *slot_index,
            resource: *resource,
        },
        RGC::QueueResearch {
            facility_id,
            tech_id,
        } => Command::QueueResearch {
            facility_id: facility_id.clone(),
            tech_id: tech_id.clone(),
        },
        RGC::PauseResearch {
            tech_id,
            release_unused,
        } => Command::PauseResearch {
            tech_id: tech_id.clone(),
            release_unused: *release_unused,
        },
        RGC::QueueSurvey {
            body_id,
            target_depth,
            priority,
        } => Command::QueueSurvey {
            body_id: body_id.clone(),
            target_depth: *target_depth,
            priority: *priority,
        },
        RGC::CancelSurveyOrder { order_id } => Command::CancelSurveyOrder {
            order_id: order_id.clone(),
        },
    }
}

/// Rebuild pending command schedule from command_log Accepted records.
///
/// Returns a map from effective_tick to list of `SequencedCommand` entries,
/// sorted by server_sequence within each tick group.
pub fn rebuild_pending_from_log(
    state: &GameState,
) -> std::collections::BTreeMap<u64, Vec<crate::command::SequencedCommand>> {
    use crate::command::{CommandEnvelope, SequencedCommand};

    let mut pending: std::collections::BTreeMap<u64, Vec<SequencedCommand>> =
        std::collections::BTreeMap::new();

    for record in &state.command_log {
        if record.outcome != crate::types::CommandOutcome::Accepted {
            continue;
        }
        // Only rebuild pending from ScheduledTick records (ADR-0008 §5)
        if record.application_boundary != crate::types::CommandApplicationBoundary::ScheduledTick {
            continue;
        }

        // Build a CommandEnvelope from the record
        let envelope = CommandEnvelope {
            id: record.id.clone(),
            expected_tick: record.expected_tick,
            command: replayable_to_command(&record.command),
        };

        pending
            .entry(record.effective_tick)
            .or_default()
            .push(SequencedCommand {
                server_sequence: record.server_sequence,
                envelope,
            });
    }

    // Sort each group by server_sequence
    for commands in pending.values_mut() {
        commands.sort_by_key(|cmd| cmd.server_sequence);
    }

    pending
}

/// Seed session receipts from a loaded command log.
pub fn seed_receipts_from_log(
    state: &GameState,
) -> Result<
    std::collections::BTreeMap<String, crate::actor::SessionReceipt>,
    crate::command::FingerprintError,
> {
    use crate::actor::SessionReceipt;
    use crate::command::CommandStatus;

    let mut receipts: std::collections::BTreeMap<String, SessionReceipt> =
        std::collections::BTreeMap::new();

    for record in &state.command_log {
        let status = match record.outcome {
            crate::types::CommandOutcome::Accepted => CommandStatus::Accepted,
            crate::types::CommandOutcome::Applied => CommandStatus::Applied,
            crate::types::CommandOutcome::Rejected => CommandStatus::Rejected,
        };

        // Compute the canonical envelope hash for idempotency (ADR-0006
        // canonical v1 writer — deterministic, unlike DefaultHasher).
        let envelope = crate::command::CommandEnvelope {
            id: record.id.clone(),
            expected_tick: record.expected_tick,
            command: replayable_to_command(&record.command),
        };
        let env_hash = crate::command::envelope_canonical_hash(&envelope)?;

        receipts.insert(
            record.id.clone(),
            SessionReceipt {
                id: record.id.clone(),
                server_sequence: record.server_sequence,
                status,
                effective_tick: Some(record.effective_tick),
                resulting_tick: None,
                result: record.result.clone(),
                error: record.rejection.clone(),
                envelope_hash: env_hash,
            },
        );
    }

    Ok(receipts)
}

// ─── Timestamp formatting ──────────────────────────────────────────────

/// Format current UTC time as RFC 3339 string.
fn format_timestamp() -> String {
    let now = SystemTime::now();
    let since_epoch = now.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = since_epoch.as_secs();
    let nanos = since_epoch.subsec_nanos();

    // Simple UTC formatting without external dependency
    // Days since 1970-01-01
    const SECS_PER_DAY: u64 = 86400;
    let days = secs / SECS_PER_DAY;
    let time_secs = secs % SECS_PER_DAY;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    // Compute year/month/day from days since epoch using a simple algorithm
    let mut y = 1970i64;
    let mut d = days as i64;
    loop {
        let days_in_year = if is_leap_year(y) { 366 } else { 365 };
        if d < days_in_year {
            break;
        }
        d -= days_in_year;
        y += 1;
    }
    let year = y;
    let month_days = month_day_table(year);
    let mut month = 1u32;
    for &md in &month_days {
        if d < md {
            break;
        }
        d -= md;
        month += 1;
    }
    let day = d as u32 + 1;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:09}Z",
        year, month, day, hours, minutes, seconds, nanos
    )
}

fn is_leap_year(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn month_day_table(year: i64) -> [i64; 12] {
    if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    }
}

// ─── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::*;
    use crate::content::*;
    use crate::content_hash::{compute_content_hash, format_hash};
    use crate::id::{BodyId, StationId};
    use crate::state::CommandRecord;
    use crate::state_construct::build_starting_state;
    use crate::state_hash::compute_state_hash;
    use crate::types::*;
    use std::path::PathBuf;

    fn load_catalog() -> ContentCatalog {
        let content_dir = std::env::var("CARGO_MANIFEST_DIR")
            .map(|p| PathBuf::from(p).parent().unwrap().join("content"))
            .unwrap_or_else(|_| PathBuf::from("content"));
        let defs: DefinitionsCatalog = serde_json::from_str(
            &std::fs::read_to_string(content_dir.join("definitions.v1.json")).unwrap(),
        )
        .unwrap();
        let scenario: StartingScenario = serde_json::from_str(
            &std::fs::read_to_string(content_dir.join("starting_system.v1.json")).unwrap(),
        )
        .unwrap();
        ContentCatalog {
            definitions: defs,
            starting_system: scenario,
        }
    }

    fn content_version_and_hash() -> (String, String) {
        let catalog = load_catalog();
        let hash = compute_content_hash(&catalog).unwrap();
        let hash_str = format_hash(&hash);
        ("v1".to_string(), hash_str)
    }

    /// Build a canonical tick-zero GameState for testing.
    fn tick_zero_state() -> GameState {
        let catalog = load_catalog();
        build_starting_state(&catalog).unwrap()
    }

    /// ── SaveEnvelope round-trip ────────────────────────────────────────

    #[test]
    fn save_envelope_canonical_tick_zero() {
        let state = tick_zero_state();
        let (cv, ch) = content_version_and_hash();
        let envelope = SaveEnvelope::new(state, &cv, &ch).unwrap();

        assert_eq!(envelope.format_version, 1);
        assert_eq!(envelope.hash_scheme, "sha256-canonical-json-v1");
        assert_eq!(envelope.schema_version, 1);
        assert_eq!(envelope.content_version, "v1");
        assert_eq!(envelope.content_hash.len(), 64);
        assert_eq!(envelope.state_hash.len(), 64);
        assert!(!envelope.timestamp.is_empty());

        // Lifecycle should be Paused (Running/Advancing normalized to Paused)
        assert_eq!(
            envelope.game_state.lifecycle,
            GameLifecycle::Paused,
            "lifecycle must be normalized to Paused"
        );
    }

    /// ── Save/load round-trip ───────────────────────────────────────────

    #[test]
    fn save_load_round_trip() {
        let state = tick_zero_state();
        let (cv, ch) = content_version_and_hash();
        let envelope = SaveEnvelope::new(state.clone(), &cv, &ch).unwrap();

        // Serialize to JSON and deserialize
        let json = serde_json::to_vec_pretty(&envelope).unwrap();
        let loaded: SaveEnvelope = serde_json::from_slice(&json).unwrap();

        // Verify state hash matches
        let computed_hash = compute_state_hash(&loaded.game_state).unwrap();
        let computed_str = format_state_hash(&computed_hash);
        assert_eq!(computed_str, loaded.state_hash);

        // Validate loaded state
        let (cv2, ch2) = content_version_and_hash();
        validate_loaded_state(&loaded, &loaded.game_state, &cv2, &ch2).unwrap();

        // Round-trip: original state should match loaded state
        assert_eq!(state, loaded.game_state);
    }

    /// ── Content hash mismatch ──────────────────────────────────────────

    #[test]
    fn content_hash_mismatch_rejected() {
        let state = tick_zero_state();
        let (cv, ch) = content_version_and_hash();
        // Create envelope with the real content hash
        let envelope = SaveEnvelope::new(state.clone(), &cv, &ch).unwrap();

        // Call validate_loaded_state with a different content hash
        let result = validate_loaded_state(&envelope, &envelope.game_state, &cv, "0000");
        assert!(result.is_err());
        assert!(matches!(result, Err(SaveError::ContentMismatch(_))));
    }

    /// ── State hash mismatch ────────────────────────────────────────────

    #[test]
    fn state_hash_mismatch_rejected() {
        let state = tick_zero_state();
        let (cv, ch) = content_version_and_hash();
        let mut envelope = SaveEnvelope::new(state, &cv, &ch).unwrap();
        envelope.state_hash = "0000".to_string();

        let (cv2, ch2) = content_version_and_hash();
        let result = validate_loaded_state(&envelope, &envelope.game_state, &cv2, &ch2);
        assert!(result.is_err());
        assert!(matches!(result, Err(SaveError::StateHashMismatch(_))));
    }

    /// ── Lifecycle normalization ────────────────────────────────────────

    #[test]
    fn save_normalizes_running_to_paused() {
        let mut state = tick_zero_state();
        state.lifecycle = GameLifecycle::Running;
        let (cv, ch) = content_version_and_hash();
        let envelope = SaveEnvelope::new(state, &cv, &ch).unwrap();
        assert_eq!(envelope.game_state.lifecycle, GameLifecycle::Paused);
    }

    #[test]
    fn save_normalizes_advancing_to_paused() {
        let mut state = tick_zero_state();
        state.lifecycle = GameLifecycle::Advancing;
        let (cv, ch) = content_version_and_hash();
        let envelope = SaveEnvelope::new(state, &cv, &ch).unwrap();
        assert_eq!(envelope.game_state.lifecycle, GameLifecycle::Paused);
    }

    #[test]
    fn save_preserves_won() {
        let mut state = tick_zero_state();
        state.lifecycle = GameLifecycle::Won;
        let (cv, ch) = content_version_and_hash();
        let envelope = SaveEnvelope::new(state, &cv, &ch).unwrap();
        assert_eq!(envelope.game_state.lifecycle, GameLifecycle::Won);
    }

    /// ── Schema migration framework ─────────────────────────────────────

    #[test]
    fn migration_registry_empty_at_v1() {
        let reg = MigrationRegistry::new(1);
        assert!(reg.validate_chain().is_ok());
    }

    #[test]
    fn migration_registry_rejects_non_adjacent() {
        let mut reg = MigrationRegistry::new(2);
        let result = reg.register(Box::new(TestMigration { from: 1, to: 3 }));
        assert!(result.is_err());
    }

    #[test]
    fn migration_registry_rejects_duplicate_from() {
        let mut reg = MigrationRegistry::new(3);
        reg.register(Box::new(TestMigration { from: 1, to: 2 }))
            .unwrap();
        let result = reg.register(Box::new(TestMigration { from: 1, to: 2 }));
        assert!(result.is_err());
    }

    struct TestMigration {
        from: u32,
        to: u32,
    }

    impl Migration for TestMigration {
        fn from_version(&self) -> u32 {
            self.from
        }
        fn to_version(&self) -> u32 {
            self.to
        }
        fn migrate(&self, state: Value) -> Result<Value, MigrationError> {
            Ok(state)
        }
    }

    /// ── Command log validation ─────────────────────────────────────────

    #[test]
    fn command_log_empty_valid() {
        let state = tick_zero_state();
        assert!(validate_command_log(&state).is_ok());
    }

    #[test]
    fn command_log_duplicate_id_rejected() {
        let mut state = tick_zero_state();
        state.command_log.push(CommandRecord {
            id: "cmd_001".to_string(),
            expected_tick: None,
            effective_tick: 1,
            server_sequence: 1,
            application_boundary: CommandApplicationBoundary::ScheduledTick,
            command: ReplayableGameCommand::QueueBuildShip {
                hub_id: StationId("hub_haven".into()),
                role: ShipRole::Construction,
                tier: 1,
            },
            outcome: CommandOutcome::Accepted,
            result: None,
            rejection: None,
        });
        state.command_log.push(CommandRecord {
            id: "cmd_001".to_string(),
            expected_tick: None,
            effective_tick: 1,
            server_sequence: 2,
            application_boundary: CommandApplicationBoundary::ScheduledTick,
            command: ReplayableGameCommand::QueueBuildShip {
                hub_id: StationId("hub_haven".into()),
                role: ShipRole::Construction,
                tier: 1,
            },
            outcome: CommandOutcome::Accepted,
            result: None,
            rejection: None,
        });
        assert!(validate_command_log(&state).is_err());
    }

    #[test]
    fn command_log_out_of_order_sequence_rejected() {
        let mut state = tick_zero_state();
        state.command_log.push(CommandRecord {
            id: "cmd_001".to_string(),
            expected_tick: None,
            effective_tick: 1,
            server_sequence: 2,
            application_boundary: CommandApplicationBoundary::ScheduledTick,
            command: ReplayableGameCommand::QueueBuildShip {
                hub_id: StationId("hub_haven".into()),
                role: ShipRole::Construction,
                tier: 1,
            },
            outcome: CommandOutcome::Accepted,
            result: None,
            rejection: None,
        });
        state.command_log.push(CommandRecord {
            id: "cmd_002".to_string(),
            expected_tick: None,
            effective_tick: 1,
            server_sequence: 1,
            application_boundary: CommandApplicationBoundary::ScheduledTick,
            command: ReplayableGameCommand::QueueBuildShip {
                hub_id: StationId("hub_haven".into()),
                role: ShipRole::Construction,
                tier: 1,
            },
            outcome: CommandOutcome::Accepted,
            result: None,
            rejection: None,
        });
        assert!(validate_command_log(&state).is_err());
    }

    /// ── Rebuild pending from log ───────────────────────────────────────

    #[test]
    fn rebuild_pending_from_log_basic() {
        let mut state = tick_zero_state();
        state.command_log.push(CommandRecord {
            id: "cmd_001".to_string(),
            expected_tick: None,
            effective_tick: 5,
            server_sequence: 1,
            application_boundary: CommandApplicationBoundary::ScheduledTick,
            command: ReplayableGameCommand::QueueBuildShip {
                hub_id: StationId("hub_haven".into()),
                role: ShipRole::Construction,
                tier: 1,
            },
            outcome: CommandOutcome::Accepted,
            result: None,
            rejection: None,
        });
        state.command_log.push(CommandRecord {
            id: "cmd_002".to_string(),
            expected_tick: None,
            effective_tick: 5,
            server_sequence: 2,
            application_boundary: CommandApplicationBoundary::ScheduledTick,
            command: ReplayableGameCommand::QueueBuildStation {
                source_hub_id: StationId("hub_haven".into()),
                body_id: BodyId("planet_haven".into()),
                orbit_ring: 0,
                slot: 1,
                station_type: StationType::Mining,
                tier: 1,
            },
            outcome: CommandOutcome::Accepted,
            result: None,
            rejection: None,
        });

        let pending = rebuild_pending_from_log(&state);
        assert!(pending.contains_key(&5));
        assert_eq!(pending.get(&5).unwrap().len(), 2);
        // Sequences should be sorted
        assert_eq!(pending.get(&5).unwrap()[0].server_sequence, 1);
        assert_eq!(pending.get(&5).unwrap()[1].server_sequence, 2);
    }

    /// ── Atomic write test ──────────────────────────────────────────────

    #[test]
    fn atomic_write_round_trip() {
        // Create a temp directory for testing
        let tmp_dir = std::env::temp_dir().join("steel-horizons-test-save");
        let _ = std::fs::create_dir_all(&tmp_dir);
        let target = tmp_dir.join("autosave.json");

        let state = tick_zero_state();
        let (cv, ch) = content_version_and_hash();
        let envelope = SaveEnvelope::new(state, &cv, &ch).unwrap();

        // Write atomically (sync variant)
        let result = sync_atomic_write(&target, &envelope);
        assert_eq!(result, WriteResult::Success);

        // Read back and validate
        let loaded = read_save_envelope(&target).unwrap();
        assert_eq!(loaded.format_version, 1);
        assert_eq!(loaded.state_hash, envelope.state_hash);
        assert_eq!(loaded.content_hash, envelope.content_hash);
        assert_eq!(loaded.game_state.lifecycle, GameLifecycle::Paused);

        // Clean up
        let _ = std::fs::remove_file(&target);
        let _ = std::fs::remove_dir(&tmp_dir);
    }

    /// ── Timestamp formatting ───────────────────────────────────────────

    #[test]
    fn timestamp_format_is_rfc3339() {
        let ts = format_timestamp();
        // Basic format check: YYYY-MM-DDTHH:MM:SS.uuuuuuuuuZ
        assert!(ts.len() > 25, "timestamp should be at least 26 chars");
        assert!(ts.ends_with('Z'), "timestamp must end with Z");
        // Contains a 'T' separator
        assert!(ts.contains('T'), "timestamp must contain T separator");
    }

    /// ── Envelope JSON shape ────────────────────────────────────────────

    #[test]
    fn envelope_json_has_correct_fields() {
        let state = tick_zero_state();
        let (cv, ch) = content_version_and_hash();
        let envelope = SaveEnvelope::new(state, &cv, &ch).unwrap();
        let json = serde_json::to_value(&envelope).unwrap();
        let obj = json.as_object().unwrap();

        // All required fields present
        assert!(obj.contains_key("format_version"));
        assert!(obj.contains_key("hash_scheme"));
        assert!(obj.contains_key("schema_version"));
        assert!(obj.contains_key("content_version"));
        assert!(obj.contains_key("content_hash"));
        assert!(obj.contains_key("state_hash"));
        assert!(obj.contains_key("timestamp"));
        assert!(obj.contains_key("game_state"));
        assert_eq!(obj.len(), 8, "V1 envelope must have exactly 8 fields");
    }
}
