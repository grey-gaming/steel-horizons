//! Simulation actor — sole mutable `GameState` owner.
//!
//! The actor owns a mailbox for commands, scheduler ticks, persistence
//! operations, and shutdown.  It publishes an immutable
//! `Arc<GameSnapshot>` after each committed state change, plus a
//! `ServerStatus` that is always available regardless of game lifecycle.
//!
//! ## Authoritative references
//!
//! - ADR-0004 §Game Lifecycle State Machine
//! - TDD 00 §Simulation Actor, §Actor Transactions
//! - TDD 02 §Queries

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::{mpsc, oneshot, watch};

use crate::command::CommandAcknowledgement;
use crate::command::CommandEnvelope;
use crate::command::CommandStatus;
use crate::command::ReplayableGameCommand;
use crate::content::ContentCatalog;
use crate::lifecycle::*;
use crate::persistence::{SaveEnvelope, WriteResult};
use crate::state::*;
use crate::tick::*;
use crate::types::*;

// ─── Actor messages ───────────────────────────────────────────────────

/// Messages accepted by the simulation actor mailbox.
#[derive(Debug)]
pub enum ActorMessage {
    /// Submit a command envelope for processing.
    SubmitCommand {
        /// The command envelope to process.
        envelope: CommandEnvelope,
        /// One-shot channel for sending the acknowledgement back.
        response_tx: oneshot::Sender<CommandAcknowledgement>,
    },
    /// Execute one scheduler tick (advance simulation by one tick while Running).
    SchedulerTick,
    /// Advance ticks while Paused (batch advancement).
    AdvanceTicks {
        /// Number of ticks to advance in batch.
        count: u16,
        /// One-shot channel for sending the batch result back.
        response_tx: oneshot::Sender<AdvanceTicksResult>,
    },
    /// Get a clone of the current immutable snapshot.
    GetSnapshot {
        /// One-shot channel for sending the snapshot back.
        response_tx: oneshot::Sender<Option<Arc<GameSnapshot>>>,
    },
    /// Get a clone of the current server status.
    GetStatus {
        /// One-shot channel for sending the status back.
        response_tx: oneshot::Sender<Arc<ServerStatus>>,
    },
    /// NewGame or LoadAutosave lifecycle command.
    LoadGame {
        /// The loading operation to perform.
        operation: LoadingOperation,
        /// One-shot channel for sending the load result back.
        response_tx: oneshot::Sender<LoadGameResult>,
    },
    /// Shutdown the actor.
    Shutdown,
}

/// Result of an AdvanceTicks batch operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvanceTicksResult {
    /// The tick after advancement.
    pub resulting_tick: u64,
    /// Number of ticks actually advanced.
    pub ticks_advanced: u64,
    /// Any error that occurred during advancement.
    pub error: Option<SimulationError>,
}

/// Result of a load game operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadGameResult {
    /// The resulting lifecycle after loading.
    pub lifecycle: GameLifecycle,
    /// The tick of the loaded game (0 for NewGame).
    pub tick: u64,
    /// Error message if loading failed.
    pub error: Option<String>,
}

// ─── Simulation actor ─────────────────────────────────────────────────

/// The sole mutable `GameState` owner.
///
/// The actor processes messages sequentially from its mailbox.  No other
/// code holds `&mut GameState`.  After each committed change, it publishes
/// an immutable snapshot and an updated status.
///
/// Fields `pending_commands`, `session_receipts`, `event_store`, and
/// `mailbox_tx` are intentionally unused until later increments (P1-12,
/// P1-31).
#[expect(dead_code)]
pub struct SimulationActor {
    /// Current runtime lifecycle (independent of state presence).
    lifecycle: GameLifecycle,
    /// The mutable simulation state.  `None` in Unloaded and Loading.
    state: Option<GameState>,
    /// Loading progress during the Loading lifecycle.
    loading: Option<LoadingStatus>,
    /// The loaded content catalog.
    content: Arc<ContentCatalog>,
    /// Pending commands scheduled at future ticks.
    pending_commands: BTreeMap<u64, Vec<SequencedCommand>>,
    /// Session receipt ledger for idempotency.
    session_receipts: BTreeMap<String, SessionReceipt>,
    /// Monotonically increasing server sequence counter.
    next_session_sequence: u64,
    /// Monotonically increasing event sequence counter.
    next_session_event_sequence: u64,
    /// Event store (placeholder — full implementation in P1-31).
    event_store: Vec<StoredEvent>,
    /// Publisher for immutable game snapshots.
    snapshot_tx: watch::Sender<Option<Arc<GameSnapshot>>>,
    /// Publisher for server status.
    status_tx: watch::Sender<Arc<ServerStatus>>,
    /// Receiver for the actor mailbox.
    mailbox_rx: mpsc::UnboundedReceiver<ActorMessage>,
    /// The sender handle for the mailbox (given to the caller).
    mailbox_tx: Option<mpsc::UnboundedSender<ActorMessage>>,
    /// Whether shutdown has been requested.
    shutdown_requested: bool,
    /// Autosave file path (None if persistence disabled).
    autosave_path: Option<PathBuf>,
}

pub use crate::command::SequencedCommand;

/// A session receipt for idempotency tracking.
///
/// The `envelope_hash` field stores a structural fingerprint of the full
/// incoming `CommandEnvelope` (including the `command` body and
/// `expected_tick`).  When a later command arrives with the same id, the
/// implementation compares the fresh hash against the stored hash to
/// distinguish full-envelope idempotency (same id + same envelope →
/// stored receipt) from idempotency conflict (same id + different
/// envelope → `IdempotencyConflict`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionReceipt {
    /// The client-provided command id.
    pub id: String,
    /// The server-assigned sequence number.
    pub server_sequence: u64,
    /// The command status (Accepted, Rejected, etc.).
    pub status: CommandStatus,
    /// The tick at which the command was effective.
    pub effective_tick: Option<u64>,
    /// The tick after the command was processed.
    pub resulting_tick: Option<u64>,
    /// The result of the command, if accepted.
    pub result: Option<CommandResult>,
    /// The rejection details, if rejected.
    pub error: Option<CommandRejection>,
    /// Structural fingerprint of the full envelope for idempotency.
    pub envelope_hash: u64,
}

/// A stored event (placeholder — full event store in P1-31).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredEvent {
    /// The event sequence number.
    pub sequence: u64,
    /// The tick at which the event was emitted.
    pub tick: Option<u64>,
    /// The event kind string.
    pub kind: String,
}

impl SimulationActor {
    /// Create a new simulation actor with the given content catalog.
    ///
    /// Starts in Unloaded lifecycle with no game state.
    #[allow(clippy::type_complexity)]
    pub fn new(
        content: ContentCatalog,
    ) -> (
        Self,
        mpsc::UnboundedSender<ActorMessage>,
        watch::Receiver<Option<Arc<GameSnapshot>>>,
        watch::Receiver<Arc<ServerStatus>>,
    ) {
        let (mailbox_tx, mailbox_rx) = mpsc::unbounded_channel();
        let (snapshot_tx, snapshot_rx) = watch::channel::<Option<Arc<GameSnapshot>>>(None);
        let (status_tx, status_rx) =
            watch::channel::<Arc<ServerStatus>>(Arc::new(ServerStatus::unloaded(0)));

        let actor = SimulationActor {
            lifecycle: GameLifecycle::Unloaded,
            state: None,
            loading: None,
            content: Arc::new(content),
            pending_commands: BTreeMap::new(),
            session_receipts: BTreeMap::new(),
            next_session_sequence: 1,
            next_session_event_sequence: 1,
            event_store: Vec::new(),
            snapshot_tx,
            status_tx: status_tx.clone(),
            mailbox_rx,
            mailbox_tx: Some(mailbox_tx.clone()),
            shutdown_requested: false,
            autosave_path: None,
        };

        (actor, mailbox_tx, snapshot_rx, status_rx)
    }

    /// Process a single message from the mailbox.
    ///
    /// Returns `true` if the actor should continue processing, `false` if
    /// shutdown was requested.
    fn process_message(&mut self, msg: ActorMessage) -> bool {
        match msg {
            ActorMessage::SubmitCommand {
                envelope,
                response_tx,
            } => {
                let result = self.handle_command(envelope);
                let _ = response_tx.send(result);
            }
            ActorMessage::SchedulerTick => {
                self.handle_scheduler_tick();
            }
            ActorMessage::AdvanceTicks { count, response_tx } => {
                let result = self.handle_advance_ticks(count);
                let _ = response_tx.send(result);
            }
            ActorMessage::GetSnapshot { response_tx } => {
                let snapshot = self.snapshot_tx.borrow().clone();
                let _ = response_tx.send(snapshot);
            }
            ActorMessage::GetStatus { response_tx } => {
                let status = self.status_tx.borrow().clone();
                let _ = response_tx.send(status);
            }
            ActorMessage::LoadGame {
                operation,
                response_tx,
            } => {
                let result = self.handle_load_game(operation);
                let _ = response_tx.send(result);
            }
            ActorMessage::Shutdown => {
                self.shutdown_requested = true;
                return false;
            }
        }
        true
    }

    /// Enable persistence with the given autosave path.
    ///
    /// Stores the autosave path for SaveNow and LoadAutosave operations.
    /// SaveNow performs synchronous atomic writes directly (no worker needed).
    /// LoadAutosave reads the save file directly via the persistence module.
    pub fn enable_persistence(&mut self, autosave_path: PathBuf) {
        self.autosave_path = Some(autosave_path);
    }

    /// Run the actor event loop, processing messages until shutdown.
    pub async fn run(&mut self) {
        while !self.shutdown_requested {
            tokio::select! {
                msg = self.mailbox_rx.recv() => {
                    match msg {
                        Some(msg) => {
                            self.process_message(msg);
                        }
                        None => {
                            // All senders dropped — shut down.
                            break;
                        }
                    }
                }
            }
        }
    }

    // ─── Handle commands ───────────────────────────────────────────────

    /// Handle a submitted command envelope.
    ///
    /// Implements full command sequencing and idempotency:
    /// - Validates lifecycle for command type
    /// - Validates expected_tick (if set, must match current tick)
    /// - Checks idempotency: same ID + same full envelope → return stored receipt
    /// - Same ID + different envelope → IdempotencyConflict rejection
    /// - Classifies as replayable or control
    /// - In Running: queues replayable commands for next tick, applies controls immediately
    /// - In Paused: applies all commands immediately
    /// - Records session receipt and CommandRecord in command_log
    fn handle_command(&mut self, envelope: CommandEnvelope) -> CommandAcknowledgement {
        // ── 1. Extract command type for lifecycle validation ────────────
        let command_type = match &envelope.command {
            crate::command::Command::NewGame { .. } => "NewGame",
            crate::command::Command::LoadAutosave => "LoadAutosave",
            crate::command::Command::SaveNow => "SaveNow",
            crate::command::Command::Pause => "Pause",
            crate::command::Command::Resume => "Resume",
            crate::command::Command::AdvanceTicks { .. } => "AdvanceTicks",
            _ => "Gameplay",
        };

        // ── 2. Validate lifecycle for this command type ────────────────
        if let Err(e) = validate_command_for_lifecycle(self.lifecycle, command_type) {
            let msg = format!("{}", e);
            return CommandAcknowledgement {
                protocol_version: "v1".to_string(),
                id: envelope.id,
                accepted: false,
                status: CommandStatus::Rejected,
                effective_tick: None,
                resulting_tick: None,
                server_sequence: self.next_session_sequence,
                game_state: self.lifecycle,
                result: None,
                error: Some(CommandRejection {
                    code: "InvalidLifecycle".to_string(),
                    message: msg,
                    details: BTreeMap::new(),
                }),
            };
        }

        // ── 3. Idempotency check ──────────────────────────────────────
        if let Some(existing) = self.session_receipts.get(&envelope.id) {
            // Compute a structural hash of the incoming envelope.
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            envelope.hash(&mut hasher);
            let incoming_hash = hasher.finish();

            if incoming_hash == existing.envelope_hash {
                // Idempotency hit — return the stored receipt.
                return CommandAcknowledgement {
                    protocol_version: "v1".to_string(),
                    id: envelope.id.clone(),
                    accepted: existing.status != CommandStatus::Rejected,
                    status: existing.status,
                    effective_tick: existing.effective_tick,
                    resulting_tick: existing.resulting_tick,
                    server_sequence: existing.server_sequence,
                    game_state: self.lifecycle,
                    result: existing.result.clone(),
                    error: existing.error.clone(),
                };
            } else {
                // Same ID but different envelope — conflict.
                return CommandAcknowledgement {
                    protocol_version: "v1".to_string(),
                    id: envelope.id,
                    accepted: false,
                    status: CommandStatus::Rejected,
                    effective_tick: None,
                    resulting_tick: None,
                    server_sequence: self.next_session_sequence,
                    game_state: self.lifecycle,
                    result: None,
                    error: Some(CommandRejection {
                        code: "IdempotencyConflict".to_string(),
                        message: "command id already used with a different envelope".to_string(),
                        details: BTreeMap::new(),
                    }),
                };
            }
        }

        // ── 4. Validate expected_tick ─────────────────────────────────
        // If no GameState exists, expected_tick cannot be validated.
        if envelope.expected_tick.is_some() && self.state.is_none() {
            return CommandAcknowledgement {
                protocol_version: "v1".to_string(),
                id: envelope.id,
                accepted: false,
                status: CommandStatus::Rejected,
                effective_tick: None,
                resulting_tick: None,
                server_sequence: self.next_session_sequence,
                game_state: self.lifecycle,
                result: None,
                error: Some(CommandRejection {
                    code: "ExpectedTickUnavailable".to_string(),
                    message: "cannot validate expected_tick: no game state loaded".to_string(),
                    details: BTreeMap::new(),
                }),
            };
        }
        let current_tick = self.state.as_ref().map_or(0, |s| s.tick);
        if let Some(expected) = envelope.expected_tick {
            if expected != current_tick {
                return CommandAcknowledgement {
                    protocol_version: "v1".to_string(),
                    id: envelope.id,
                    accepted: false,
                    status: CommandStatus::Rejected,
                    effective_tick: None,
                    resulting_tick: None,
                    server_sequence: self.next_session_sequence,
                    game_state: self.lifecycle,
                    result: None,
                    error: Some(CommandRejection {
                        code: "ExpectedTickMismatch".to_string(),
                        message: format!(
                            "expected tick {} does not match current tick {}",
                            expected, current_tick
                        ),
                        details: BTreeMap::new(),
                    }),
                };
            }
        }

        // ── 5. Classify as replayable vs control ──────────────────────
        let is_control = matches!(
            &envelope.command,
            crate::command::Command::Pause
                | crate::command::Command::Resume
                | crate::command::Command::NewGame { .. }
                | crate::command::Command::LoadAutosave
                | crate::command::Command::SaveNow
                | crate::command::Command::AdvanceTicks { .. }
        );

        // ── 6. Assign server sequence ─────────────────────────────────
        let seq = self.next_session_sequence;
        self.next_session_sequence += 1;

        // ── 7. Determine effective behavior based on lifecycle ─────────
        let effective_tick: Option<u64>;
        let mut resulting_tick: Option<u64>;
        let mut status: CommandStatus;
        let result: Option<CommandResult>;
        let mut error: Option<CommandRejection>;

        // Control commands are allowed from any lifecycle (they've already
        // passed step 2's per-command validation).  Gameplay commands are
        // restricted to Running and Paused.
        if is_control {
            // Control commands apply immediately.
            effective_tick = Some(current_tick);
            status = CommandStatus::Applied;
            resulting_tick = None;
            result = None;
            error = None;
        } else {
            match self.lifecycle {
                GameLifecycle::Running => {
                    // Replayable commands queue for the next tick.
                    let next_tick = current_tick + 1;
                    self.pending_commands
                        .entry(next_tick)
                        .or_default()
                        .push(SequencedCommand {
                            server_sequence: seq,
                            envelope: envelope.clone(),
                        });
                    effective_tick = Some(next_tick);
                    resulting_tick = None;
                    status = CommandStatus::Accepted;
                    result = None;
                    error = None;
                }
                GameLifecycle::Paused => {
                    // All commands apply immediately when paused.
                    effective_tick = Some(current_tick);
                    status = CommandStatus::Applied;
                    resulting_tick = None;
                    result = None;
                    error = None;
                }
                _ => {
                    // Other lifecycle states reject gameplay commands.
                    effective_tick = None;
                    resulting_tick = None;
                    status = CommandStatus::Rejected;
                    result = None;
                    error = Some(CommandRejection {
                        code: "InvalidLifecycle".to_string(),
                        message: format!(
                            "cannot process commands in {:?} lifecycle",
                            self.lifecycle
                        ),
                        details: BTreeMap::new(),
                    });
                }
            }
        }

        // ── 8. Apply immediate commands (control or paused) ───────────
        let is_immediate = is_control || self.lifecycle == GameLifecycle::Paused;

        if is_immediate && status != CommandStatus::Rejected {
            match &envelope.command {
                crate::command::Command::NewGame { .. } => {
                    self.lifecycle = GameLifecycle::Loading;
                    self.loading = Some(LoadingStatus {
                        operation: LoadingOperation::NewGame,
                        stage: LoadingStage::ValidatingContent,
                    });
                    self.update_status();
                }
                crate::command::Command::SaveNow => {
                    // SaveNow requires a loaded game state and persistence enabled.
                    let save_result = if let (Some(state), Some(ref autosave_path)) =
                        (self.state.as_ref(), &self.autosave_path)
                    {
                        // Compute content hash from the catalog.
                        let content_hash = match crate::content_hash::compute_content_hash(
                            &self.content,
                        )
                        .map(|h| crate::content_hash::format_hash(&h))
                        {
                            Ok(h) => h,
                            Err(e) => {
                                error = Some(CommandRejection {
                                    code: "SaveFailed".to_string(),
                                    message: format!("content hash failed: {}", e),
                                    details: BTreeMap::new(),
                                });
                                status = CommandStatus::Rejected;
                                // Use a dummy hash so the error path is taken below
                                "0000000000000000000000000000000000000000000000000000000000000000"
                                    .to_string()
                            }
                        };
                        let content_version = &self.content.starting_system.content_version;

                        match SaveEnvelope::new((*state).clone(), content_version, &content_hash) {
                            Ok(envelope) => {
                                // Write synchronously — the actor must acknowledge
                                // only after the write completes (ADR-0008 §4).
                                match crate::persistence::sync_atomic_write(
                                    autosave_path,
                                    &envelope,
                                ) {
                                    WriteResult::Success => None,
                                    WriteResult::Failure(e) => Some(e.to_string()),
                                }
                            }
                            Err(e) => Some(e.to_string()),
                        }
                    } else {
                        if self.state.is_none() {
                            Some("no game state loaded".to_string())
                        } else {
                            Some("persistence not enabled".to_string())
                        }
                    };

                    if let Some(ref msg) = save_result {
                        error = Some(CommandRejection {
                            code: "SaveFailed".to_string(),
                            message: msg.clone(),
                            details: BTreeMap::new(),
                        });
                        status = CommandStatus::Rejected;
                    }
                }
                crate::command::Command::LoadAutosave => {
                    self.lifecycle = GameLifecycle::Loading;
                    self.loading = Some(LoadingStatus {
                        operation: LoadingOperation::LoadAutosave,
                        stage: LoadingStage::ReadingSave,
                    });
                    self.update_status();
                }
                crate::command::Command::Pause => {
                    if self.lifecycle == GameLifecycle::Running {
                        self.lifecycle = GameLifecycle::Paused;
                        if let Some(ref mut state) = self.state {
                            state.lifecycle = GameLifecycle::Paused;
                        }
                        self.update_status();
                    }
                }
                crate::command::Command::Resume => {
                    if self.lifecycle == GameLifecycle::Paused {
                        self.lifecycle = GameLifecycle::Running;
                        if let Some(ref mut state) = self.state {
                            state.lifecycle = GameLifecycle::Running;
                        }
                        self.update_status();
                    }
                }
                crate::command::Command::AdvanceTicks { count } => {
                    let _ = count;
                    self.lifecycle = GameLifecycle::Advancing;
                    self.update_status();
                }
                _ => {
                    // Gameplay commands while Paused — apply to state immediately.
                    if let Some(ref mut state) = self.state {
                        // Build a ReplayableGameCommand from the envelope.
                        let game_command = ReplayableGameCommand::from(&envelope.command);
                        state.command_log.push(CommandRecord {
                            id: envelope.id.clone(),
                            expected_tick: envelope.expected_tick,
                            effective_tick: current_tick,
                            server_sequence: seq,
                            application_boundary: CommandApplicationBoundary::PausedImmediate,
                            command: game_command,
                            outcome: CommandOutcome::Applied,
                            result: None,
                            rejection: None,
                        });
                    }
                }
            }

            resulting_tick = Some(self.state.as_ref().map_or(current_tick, |s| s.tick));
        }

        // ── 9. Record session receipt ─────────────────────────────────
        // Compute envelope hash for idempotency before consuming
        // `envelope` (it is moved below for `id`).
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        envelope.hash(&mut hasher);
        let env_hash = hasher.finish();
        let receipt = SessionReceipt {
            id: envelope.id.clone(),
            server_sequence: seq,
            status,
            effective_tick,
            resulting_tick,
            result: result.clone(),
            error: error.clone(),
            envelope_hash: env_hash,
        };
        self.session_receipts.insert(envelope.id.clone(), receipt);

        // ── 10. Build and return acknowledgement ──────────────────────
        CommandAcknowledgement {
            protocol_version: "v1".to_string(),
            id: envelope.id,
            accepted: status != CommandStatus::Rejected,
            status,
            effective_tick,
            resulting_tick,
            server_sequence: seq,
            game_state: self.lifecycle,
            result,
            error,
        }
    }

    // ─── Handle scheduler tick ─────────────────────────────────────────

    /// Handle one scheduler tick while Running.
    ///
    /// Advances the simulation by exactly one tick using the ordinary tick
    /// function.  Publishes the updated snapshot.
    fn handle_scheduler_tick(&mut self) {
        if self.lifecycle != GameLifecycle::Running {
            return;
        }
        if let Some(ref mut state) = self.state {
            let current_tick = state.tick;
            let next_tick = current_tick + 1;
            let pending = self.pending_commands.remove(&next_tick).unwrap_or_default();
            match advance_one_tick(state, pending) {
                Ok(committed) => {
                    *state = committed.state;
                    self.publish_snapshot();
                }
                Err(_e) => {
                    // On error, transition to Paused and emit error.
                    self.lifecycle = GameLifecycle::Paused;
                    state.lifecycle = GameLifecycle::Paused;
                    self.update_status();
                }
            }
        }
    }

    // ─── Handle batch advancement ──────────────────────────────────────

    /// Handle AdvanceTicks batch advancement while Paused.
    ///
    /// Transitions to Advancing, executes N ordinary ticks, then returns to
    /// Paused.  On error, stops at the last successfully committed tick and
    /// returns to Paused.
    fn handle_advance_ticks(&mut self, count: u16) -> AdvanceTicksResult {
        let start_tick = self.state.as_ref().map_or(0, |s| s.tick);

        // Validate lifecycle — must be Paused.
        if self.lifecycle != GameLifecycle::Paused {
            return AdvanceTicksResult {
                resulting_tick: start_tick,
                ticks_advanced: 0,
                error: Some(SimulationError::Phase(
                    "AdvanceTicks requires Paused lifecycle".to_string(),
                )),
            };
        }

        // Transition to Advancing.
        self.lifecycle = GameLifecycle::Advancing;
        if let Some(ref mut state) = self.state {
            state.lifecycle = GameLifecycle::Advancing;
        }
        self.update_status();

        // Execute N ticks.
        let mut ticks_advanced = 0u64;
        let max_ticks = count as u64;
        let mut error: Option<SimulationError> = None;

        for _ in 0..max_ticks {
            if let Some(ref mut state) = self.state {
                let current_tick = state.tick;
                let next_tick = current_tick + 1;
                let pending = self.pending_commands.remove(&next_tick).unwrap_or_default();
                match advance_one_tick(state, pending) {
                    Ok(committed) => {
                        *state = committed.state;
                        ticks_advanced += 1;
                    }
                    Err(e) => {
                        error = Some(e);
                        break;
                    }
                }
            } else {
                error = Some(SimulationError::Phase(
                    "No game state to advance".to_string(),
                ));
                break;
            }
        }

        // Publish the final snapshot after batch advancement
        // (publish_snapshot already calls update_status internally).
        self.publish_snapshot();

        // Return to Paused.
        self.lifecycle = GameLifecycle::Paused;
        if let Some(ref mut state) = self.state {
            state.lifecycle = GameLifecycle::Paused;
        }

        let resulting_tick = self.state.as_ref().map_or(start_tick, |s| s.tick);

        AdvanceTicksResult {
            resulting_tick,
            ticks_advanced,
            error,
        }
    }

    // ─── Handle load game ──────────────────────────────────────────────

    /// Handle NewGame or LoadAutosave lifecycle commands.
    ///
    /// This is a placeholder that constructs a new game state or returns
    /// an error.  Real persistence integration lands in P1-13.
    fn handle_load_game(&mut self, operation: LoadingOperation) -> LoadGameResult {
        let prior_lifecycle = self.lifecycle;
        let prior_state = self.state.take();

        // Transition to Loading.
        self.lifecycle = GameLifecycle::Loading;
        self.loading = Some(LoadingStatus {
            operation,
            stage: LoadingStage::ValidatingContent,
        });
        self.update_status();

        // Placeholder: attempt to construct a new game state.
        match operation {
            LoadingOperation::NewGame | LoadingOperation::StartupAutoload => {
                let result = crate::state_construct::build_starting_state(&self.content);
                match result {
                    Ok(game_state) => {
                        self.state = Some(game_state);
                        self.lifecycle = GameLifecycle::Paused;
                        self.loading = None;
                        self.publish_snapshot();
                        LoadGameResult {
                            lifecycle: GameLifecycle::Paused,
                            tick: 0,
                            error: None,
                        }
                    }
                    Err(e) => {
                        // Loading failure — restore prior state if available.
                        self.state = prior_state;
                        self.lifecycle = prior_lifecycle;
                        self.loading = None;
                        self.update_status();
                        LoadGameResult {
                            lifecycle: prior_lifecycle,
                            tick: 0,
                            error: Some(format!("{}", e)),
                        }
                    }
                }
            }
            LoadingOperation::LoadAutosave => {
                // ── Load autosave via persistence worker ────────────────
                let load_result = if let Some(ref autosave_path) = self.autosave_path {
                    // Use the persistence module's read_save_envelope directly.
                    let envelope = crate::persistence::read_save_envelope(autosave_path);
                    match envelope {
                        Ok(envelope) => {
                            // Validate content compatibility
                            let computed_hash =
                                crate::content_hash::compute_content_hash(&self.content)
                                    .map(|h| crate::content_hash::format_hash(&h))
                                    .unwrap_or_else(|_| "unknown".to_string());
                            let content_version = &self.content.starting_system.content_version;

                            // Validate envelope against content and compute
                            // state hash verification.
                            let validation = crate::persistence::validate_loaded_state(
                                &envelope,
                                &envelope.game_state,
                                content_version,
                                &computed_hash,
                            );

                            match validation {
                                Ok(()) => {
                                    // Load succeeded — replace state.
                                    let state = envelope.game_state;

                                    // Rebuild pending commands and session
                                    // receipts from the command_log.
                                    self.pending_commands =
                                        crate::persistence::rebuild_pending_from_log(&state);
                                    self.session_receipts =
                                        crate::persistence::seed_receipts_from_log(&state);

                                    // Restore next_server_sequence from state
                                    // ADR-0008 §2: max of runtime counter and loaded lower bound
                                    self.next_session_sequence = std::cmp::max(
                                        self.next_session_sequence,
                                        state.next_server_sequence,
                                    );

                                    self.state = Some(state);
                                    self.lifecycle = GameLifecycle::Paused;
                                    self.loading = None;
                                    self.publish_snapshot();
                                    LoadGameResult {
                                        lifecycle: GameLifecycle::Paused,
                                        tick: self.state.as_ref().map_or(0, |s| s.tick),
                                        error: None,
                                    }
                                }
                                Err(e) => {
                                    // Validation failed — restore prior state.
                                    self.state = prior_state;
                                    self.lifecycle = prior_lifecycle;
                                    self.loading = None;
                                    self.update_status();
                                    LoadGameResult {
                                        lifecycle: prior_lifecycle,
                                        tick: 0,
                                        error: Some(format!("{}", e)),
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            // Read failed — restore prior state.
                            self.state = prior_state;
                            self.lifecycle = prior_lifecycle;
                            self.loading = None;
                            self.update_status();
                            LoadGameResult {
                                lifecycle: prior_lifecycle,
                                tick: 0,
                                error: Some(format!("{}", e)),
                            }
                        }
                    }
                } else {
                    // No persistence path configured.
                    self.state = prior_state;
                    self.lifecycle = prior_lifecycle;
                    self.loading = None;
                    self.update_status();
                    LoadGameResult {
                        lifecycle: prior_lifecycle,
                        tick: 0,
                        error: Some("persistence not enabled".to_string()),
                    }
                };
                // Return the result from the closure
                load_result
            }
        }
    }

    // ─── Snapshot and status publication ───────────────────────────────

    /// Publish an immutable snapshot of the current game state.
    fn publish_snapshot(&mut self) {
        if let Some(ref state) = self.state {
            let snapshot = GameSnapshot {
                protocol_version: "v1".to_string(),
                latest_event_sequence: self.next_session_event_sequence.saturating_sub(1),
                state: state.clone(),
            };
            let _ = self.snapshot_tx.send(Some(Arc::new(snapshot)));
            self.update_status();
        }
    }

    /// Update the published server status.
    fn update_status(&mut self) {
        let status = if let Some(ref state) = self.state {
            ServerStatus::from_snapshot(&GameSnapshot {
                protocol_version: "v1".to_string(),
                latest_event_sequence: self.next_session_event_sequence.saturating_sub(1),
                state: state.clone(),
            })
        } else if let Some(ref loading) = self.loading {
            ServerStatus::loading(
                self.next_session_event_sequence.saturating_sub(1),
                loading.operation,
                loading.stage,
            )
        } else {
            ServerStatus::unloaded(self.next_session_event_sequence.saturating_sub(1))
        };
        let _ = self.status_tx.send(Arc::new(status));
    }
}

// ─── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::Command;
    use crate::command::CommandStatus;
    use crate::id::ScenarioId;
    use crate::id::StationId;
    use std::collections::BTreeSet;

    /// Helper: create a content catalog for testing.
    fn test_content() -> ContentCatalog {
        // Load from canonical content files via ScenarioHarness.
        crate::scenario::ScenarioHarness::new()
            .expect("canonical content catalog must load")
            .catalog()
            .clone()
    }

    /// Helper: create an actor with canonical content.
    fn test_actor() -> (SimulationActor, mpsc::UnboundedSender<ActorMessage>) {
        let content = test_content();
        let (actor, tx, _, _) = SimulationActor::new(content);
        (actor, tx)
    }

    // ─── Construction tests ───────────────────────────────────────────

    /// A new actor starts in Unloaded lifecycle with no state.
    #[test]
    fn actor_starts_unloaded() {
        let (actor, _) = test_actor();
        assert_eq!(actor.lifecycle, GameLifecycle::Unloaded);
        assert!(actor.state.is_none());
        assert!(actor.loading.is_none());
    }

    // ─── Lifecycle transition tests ───────────────────────────────────

    /// NewGame command transitions to Loading.
    #[test]
    fn new_game_transitions_to_loading() {
        let (mut actor, _) = test_actor();
        let result = actor.handle_command(CommandEnvelope {
            id: "cmd_001".to_string(),
            expected_tick: None,
            command: Command::NewGame {
                scenario_id: ScenarioId("starting_system".into()),
            },
        });
        assert_eq!(result.status, CommandStatus::Applied);
        assert_eq!(actor.lifecycle, GameLifecycle::Loading);
        assert!(actor.loading.is_some());
    }

    /// Pause command from Running transitions to Paused.
    #[test]
    fn pause_from_running() {
        let (mut actor, _) = test_actor();
        actor.lifecycle = GameLifecycle::Running;
        actor.state = Some(paused_state_at_tick0());
        let result = actor.handle_command(CommandEnvelope {
            id: "cmd_002".to_string(),
            expected_tick: None,
            command: Command::Pause,
        });
        assert_eq!(result.status, CommandStatus::Applied);
        assert_eq!(actor.lifecycle, GameLifecycle::Paused);
    }

    /// Resume from Paused transitions to Running.
    #[test]
    fn resume_from_paused() {
        let (mut actor, _) = test_actor();
        actor.lifecycle = GameLifecycle::Paused;
        actor.state = Some(paused_state_at_tick0());
        let result = actor.handle_command(CommandEnvelope {
            id: "cmd_003".to_string(),
            expected_tick: None,
            command: Command::Resume,
        });
        assert_eq!(result.status, CommandStatus::Applied);
        assert_eq!(actor.lifecycle, GameLifecycle::Running);
    }

    /// A gameplay command while Unloaded is rejected.
    #[test]
    fn gameplay_command_rejected_in_unloaded() {
        let (mut actor, _) = test_actor();
        let result = actor.handle_command(CommandEnvelope {
            id: "cmd_004".to_string(),
            expected_tick: None,
            command: Command::QueueBuildShip {
                hub_id: StationId("hub_haven".into()),
                role: ShipRole::Construction,
                tier: 1,
            },
        });
        assert_eq!(result.status, CommandStatus::Rejected);
        assert!(result.error.is_some());
        assert_eq!(result.error.as_ref().unwrap().code, "InvalidLifecycle");
    }

    // ─── Scheduler tick tests ────────────────────────────────────────

    /// Scheduler tick while Running advances tick by 1.
    #[test]
    fn scheduler_tick_advances_when_running() {
        let (mut actor, _) = test_actor();
        actor.lifecycle = GameLifecycle::Running;
        actor.state = Some(paused_state_at_tick0());
        actor.handle_scheduler_tick();
        assert_eq!(actor.state.as_ref().unwrap().tick, 1);
    }

    /// Scheduler tick while Paused does nothing.
    #[test]
    fn scheduler_tick_noop_when_paused() {
        let (mut actor, _) = test_actor();
        actor.lifecycle = GameLifecycle::Paused;
        actor.state = Some(paused_state_at_tick0());
        actor.handle_scheduler_tick();
        assert_eq!(actor.state.as_ref().unwrap().tick, 0);
    }

    // ─── Batch advancement tests ──────────────────────────────────────

    /// AdvanceTicks while Paused advances the tick count.
    #[test]
    fn advance_ticks_while_paused() {
        let (mut actor, _) = test_actor();
        actor.lifecycle = GameLifecycle::Paused;
        actor.state = Some(paused_state_at_tick0());
        let result = actor.handle_advance_ticks(10);
        assert!(result.error.is_none());
        assert_eq!(result.ticks_advanced, 10);
        assert_eq!(result.resulting_tick, 10);
        assert_eq!(actor.lifecycle, GameLifecycle::Paused);
    }

    /// AdvanceTicks while Running is rejected.
    #[test]
    fn advance_ticks_rejected_when_running() {
        let (mut actor, _) = test_actor();
        actor.lifecycle = GameLifecycle::Running;
        actor.state = Some(paused_state_at_tick0());
        let result = actor.handle_advance_ticks(10);
        assert!(result.error.is_some());
        assert_eq!(result.ticks_advanced, 0);
    }

    /// Batch advancement produces the same final state as tick-by-tick.
    #[test]
    fn batch_equivalence_via_actor() {
        // Path A: advance one tick at a time via scheduler ticks.
        let (mut actor_a, _) = test_actor();
        actor_a.lifecycle = GameLifecycle::Running;
        actor_a.state = Some(paused_state_at_tick0());
        for _ in 0..10 {
            actor_a.handle_scheduler_tick();
        }
        let hash_a = crate::state_hash::compute_state_hash(actor_a.state.as_ref().unwrap())
            .map(|h| crate::state_hash::format_state_hash(&h))
            .unwrap_or("<err>".to_string());

        // Path B: advance in one batch via AdvanceTicks.
        let (mut actor_b, _) = test_actor();
        actor_b.lifecycle = GameLifecycle::Paused;
        actor_b.state = Some(paused_state_at_tick0());
        let result = actor_b.handle_advance_ticks(10);
        assert!(result.error.is_none());
        let hash_b = crate::state_hash::compute_state_hash(actor_b.state.as_ref().unwrap())
            .map(|h| crate::state_hash::format_state_hash(&h))
            .unwrap_or("<err>".to_string());

        assert_eq!(
            hash_a, hash_b,
            "tick-by-tick and batch state hashes must match at tick 10"
        );
        assert_eq!(actor_a.state.as_ref().unwrap().tick, 10);
        assert_eq!(actor_b.state.as_ref().unwrap().tick, 10);
    }

    // ─── Load game tests ─────────────────────────────────────────────

    /// NewGame from Unloaded constructs a paused game at tick 0.
    #[test]
    fn new_game_from_unloaded() {
        let (mut actor, _) = test_actor();
        let result = actor.handle_load_game(LoadingOperation::NewGame);
        assert!(result.error.is_none());
        assert_eq!(result.lifecycle, GameLifecycle::Paused);
        assert_eq!(result.tick, 0);
        assert_eq!(actor.lifecycle, GameLifecycle::Paused);
        assert!(actor.state.is_some());
    }

    /// Failed load from Unloaded returns to Unloaded.
    #[test]
    fn failed_load_returns_to_unloaded() {
        let (mut actor, _) = test_actor();
        // Set an invalid content catalog that will fail construction
        // (empty definitions with no scenarios).
        let bad_content = ContentCatalog {
            definitions: crate::content::DefinitionsCatalog {
                content_version: "v1".to_string(),
                defaults: crate::content::AuthoredDefaults {
                    new_station_priority: 1,
                    general_buffer_preferred_maximum: 100,
                    input_demand_threshold: 4,
                    output_export_threshold: 9,
                    fuel_demand_threshold: 4,
                    fuel_export_threshold: 9,
                    mining_retune_ticks: 600,
                    upgrade_work_per_tier: 1000,
                    demolition_work: 500,
                    survey_depth_work: [10, 20, 30],
                    hub_shipyard_work_per_tick: 10,
                },
                recipes: Vec::new(),
                technologies: Vec::new(),
                ships: Vec::new(),
                stations: Vec::new(),
                gate: crate::content::GateDefinition {
                    site_position: crate::state::SystemPosition {
                        lane_id: crate::types::LaneId::Inner,
                        radius_units: 1000,
                        angle_milli: 0,
                    },
                    manifest: BTreeMap::new(),
                    required_techs: Vec::new(),
                    required_fabricator_role: crate::types::ShipRole::Construction,
                    minimum_fabricator_tier: 1,
                    logistics_priority: 1,
                    transfer_berths: 1,
                    phases: Vec::new(),
                },
            },
            starting_system: crate::content::StartingScenario {
                id: ScenarioId("test".into()),
                content_version: "v1".to_string(),
                lifecycle: GameLifecycle::Advancing,
                tick: 1,
                next_server_sequence: 1,
                next_event_sequence: 1,
                id_counters: crate::state::IdCounters {
                    ship: 0,
                    station: 0,
                    build_order: 0,
                    reservation: 0,
                    salvage: 0,
                    survey_order: 0,
                },
                rng_state: crate::state::RNGState {
                    words: [1, 2, 3, 4],
                },
                celestial_bodies: BTreeMap::new(),
                stations: BTreeMap::new(),
                ships: BTreeMap::new(),
                completed_techs: BTreeSet::new(),
            },
        };
        actor.content = Arc::new(bad_content);
        let result = actor.handle_load_game(LoadingOperation::NewGame);
        assert!(result.error.is_some());
        assert_eq!(actor.lifecycle, GameLifecycle::Unloaded);
    }

    /// Failed load from a stable game restores prior state.
    #[test]
    fn failed_load_restores_prior_state() {
        let (mut actor, _) = test_actor();
        // Start with a valid game.
        actor.state = Some(paused_state_at_tick0());
        actor.lifecycle = GameLifecycle::Paused;
        let prior_tick = actor.state.as_ref().unwrap().tick;

        // Attempt a load with bad content.
        let bad_content = ContentCatalog {
            definitions: crate::content::DefinitionsCatalog {
                content_version: "v1".to_string(),
                defaults: crate::content::AuthoredDefaults {
                    new_station_priority: 1,
                    general_buffer_preferred_maximum: 100,
                    input_demand_threshold: 4,
                    output_export_threshold: 9,
                    fuel_demand_threshold: 4,
                    fuel_export_threshold: 9,
                    mining_retune_ticks: 600,
                    upgrade_work_per_tier: 1000,
                    demolition_work: 500,
                    survey_depth_work: [10, 20, 30],
                    hub_shipyard_work_per_tick: 10,
                },
                recipes: Vec::new(),
                technologies: Vec::new(),
                ships: Vec::new(),
                stations: Vec::new(),
                gate: crate::content::GateDefinition {
                    site_position: crate::state::SystemPosition {
                        lane_id: crate::types::LaneId::Inner,
                        radius_units: 1000,
                        angle_milli: 0,
                    },
                    manifest: BTreeMap::new(),
                    required_techs: Vec::new(),
                    required_fabricator_role: crate::types::ShipRole::Construction,
                    minimum_fabricator_tier: 1,
                    logistics_priority: 1,
                    transfer_berths: 1,
                    phases: Vec::new(),
                },
            },
            starting_system: crate::content::StartingScenario {
                id: ScenarioId("test".into()),
                content_version: "v1".to_string(),
                lifecycle: GameLifecycle::Advancing,
                tick: 1,
                next_server_sequence: 1,
                next_event_sequence: 1,
                id_counters: crate::state::IdCounters {
                    ship: 0,
                    station: 0,
                    build_order: 0,
                    reservation: 0,
                    salvage: 0,
                    survey_order: 0,
                },
                rng_state: crate::state::RNGState {
                    words: [1, 2, 3, 4],
                },
                celestial_bodies: BTreeMap::new(),
                stations: BTreeMap::new(),
                ships: BTreeMap::new(),
                completed_techs: BTreeSet::new(),
            },
        };
        actor.content = Arc::new(bad_content);
        let result = actor.handle_load_game(LoadingOperation::NewGame);
        assert!(result.error.is_some());
        assert_eq!(actor.lifecycle, GameLifecycle::Paused);
        assert!(actor.state.is_some());
        assert_eq!(actor.state.as_ref().unwrap().tick, prior_tick);
    }

    // ─── Snapshot publication tests ───────────────────────────────────

    /// Publishing a snapshot creates an immutable Arc<GameSnapshot>.
    #[test]
    fn publish_snapshot_creates_arc() {
        let content = test_content();
        let (mut actor, _, snapshot_rx, _) = SimulationActor::new(content);
        actor.state = Some(paused_state_at_tick0());
        actor.publish_snapshot();
        // Read from the receiver (keep it alive so send succeeds).
        let snapshot = snapshot_rx.borrow().clone();
        assert!(snapshot.is_some());
        let snapshot = snapshot.unwrap();
        assert_eq!(snapshot.state.tick, 0);
        assert_eq!(snapshot.state.lifecycle, GameLifecycle::Paused);
    }

    /// Status is always available even without game state.
    #[test]
    fn status_available_without_state() {
        let content = test_content();
        let (mut actor, _, _, status_rx) = SimulationActor::new(content);
        assert_eq!(actor.lifecycle, GameLifecycle::Unloaded);
        actor.update_status();
        let status = status_rx.borrow().clone();
        assert_eq!(status.game_state, GameLifecycle::Unloaded);
    }

    // ─── Concurrent exclusion tests ──────────────────────────────────

    /// Sequential AdvanceTicks calls (first completes before second starts).
    #[test]
    fn sequential_advance_ticks_allowed() {
        let (mut actor, _) = test_actor();
        actor.lifecycle = GameLifecycle::Paused;
        actor.state = Some(paused_state_at_tick0());
        let result = actor.handle_advance_ticks(5);
        assert!(result.error.is_none());

        // Attempt another while still in Paused (now returned) — should work.
        let result2 = actor.handle_advance_ticks(5);
        assert!(result2.error.is_none());
        assert_eq!(result2.resulting_tick, 10);
    }

    // ─── Realtime batch equivalence ──────────────────────────────────

    /// The realtime_batch_equivalence requirement: advancing N ticks via
    /// scheduler ticks (simulating real-time) and via a single AdvanceTicks
    /// batch produces identical state hashes and event traces.
    #[test]
    fn realtime_batch_equivalence() {
        // Path A: real-time simulation via scheduler ticks.
        let (mut actor_rt, _) = test_actor();
        actor_rt.lifecycle = GameLifecycle::Running;
        actor_rt.state = Some(paused_state_at_tick0());
        for _ in 0..10 {
            actor_rt.handle_scheduler_tick();
        }
        assert_eq!(actor_rt.state.as_ref().unwrap().tick, 10);
        let hash_rt = crate::state_hash::compute_state_hash(actor_rt.state.as_ref().unwrap())
            .map(|h| crate::state_hash::format_state_hash(&h))
            .unwrap_or("<err>".to_string());

        // Path B: batch advancement via AdvanceTicks.
        let (mut actor_batch, _) = test_actor();
        actor_batch.lifecycle = GameLifecycle::Paused;
        actor_batch.state = Some(paused_state_at_tick0());
        let result = actor_batch.handle_advance_ticks(10);
        assert!(result.error.is_none());
        assert_eq!(actor_batch.state.as_ref().unwrap().tick, 10);
        let hash_batch = crate::state_hash::compute_state_hash(actor_batch.state.as_ref().unwrap())
            .map(|h| crate::state_hash::format_state_hash(&h))
            .unwrap_or("<err>".to_string());

        assert_eq!(
            hash_rt, hash_batch,
            "realtime (scheduler ticks) and batch (AdvanceTicks) must produce identical hashes"
        );
    }

    // ─── Command sequencing and idempotency tests ────────────────────

    /// A replayable command while Running is Accepted and queued for
    /// the next tick.
    #[test]
    fn running_queues_replayable_command() {
        let (mut actor, _) = test_actor();
        actor.lifecycle = GameLifecycle::Running;
        actor.state = Some(paused_state_at_tick0());
        let result = actor.handle_command(CommandEnvelope {
            id: "cmd_g1".to_string(),
            expected_tick: None,
            command: Command::SetStationPriority {
                station_id: StationId("hub_haven".into()),
                priority: 5,
            },
        });
        assert_eq!(result.status, CommandStatus::Accepted);
        assert_eq!(result.effective_tick, Some(1));
        assert!(actor.pending_commands.contains_key(&1));
    }

    /// A control command while Running is Applied immediately.
    #[test]
    fn running_applies_control_immediately() {
        let (mut actor, _) = test_actor();
        actor.lifecycle = GameLifecycle::Running;
        actor.state = Some(paused_state_at_tick0());
        let result = actor.handle_command(CommandEnvelope {
            id: "cmd_g2".to_string(),
            expected_tick: None,
            command: Command::Pause,
        });
        assert_eq!(result.status, CommandStatus::Applied);
        assert_eq!(actor.lifecycle, GameLifecycle::Paused);
    }

    /// A replayable command while Paused is Applied immediately and
    /// recorded in the command_log.
    #[test]
    fn paused_applies_command_immediately() {
        let (mut actor, _) = test_actor();
        actor.lifecycle = GameLifecycle::Paused;
        actor.state = Some(paused_state_at_tick0());
        let result = actor.handle_command(CommandEnvelope {
            id: "cmd_g3".to_string(),
            expected_tick: None,
            command: Command::SetStationPriority {
                station_id: StationId("hub_haven".into()),
                priority: 5,
            },
        });
        assert_eq!(result.status, CommandStatus::Applied);
        // Command should be recorded in command_log
        let state = actor.state.as_ref().unwrap();
        assert!(!state.command_log.is_empty());
        assert_eq!(state.command_log[0].id, "cmd_g3");
    }

    /// Same ID + same full envelope returns the stored receipt
    /// (idempotency replay).
    #[test]
    fn same_id_same_envelope_idempotent() {
        let (mut actor, _) = test_actor();
        actor.lifecycle = GameLifecycle::Paused;
        actor.state = Some(paused_state_at_tick0());
        let envelope = CommandEnvelope {
            id: "cmd_idem".to_string(),
            expected_tick: None,
            command: Command::SetStationPriority {
                station_id: StationId("hub_haven".into()),
                priority: 3,
            },
        };
        let first = actor.handle_command(envelope.clone());
        assert_eq!(first.status, CommandStatus::Applied);

        // Same envelope again — must return stored receipt
        let second = actor.handle_command(envelope);
        assert_eq!(second.status, CommandStatus::Applied);
        assert_eq!(second.server_sequence, first.server_sequence);
        assert_eq!(second.effective_tick, first.effective_tick);
    }

    /// Same ID with different envelope (changed command or expected_tick)
    /// is rejected as IdempotencyConflict.
    #[test]
    fn same_id_different_envelope_conflict() {
        let (mut actor, _) = test_actor();
        actor.lifecycle = GameLifecycle::Paused;
        actor.state = Some(paused_state_at_tick0());
        let envelope1 = CommandEnvelope {
            id: "cmd_conflict".to_string(),
            expected_tick: None,
            command: Command::SetStationPriority {
                station_id: StationId("hub_haven".into()),
                priority: 3,
            },
        };
        let first = actor.handle_command(envelope1);
        assert_eq!(first.status, CommandStatus::Applied);

        // Same ID but different expected_tick — conflict
        let envelope2 = CommandEnvelope {
            id: "cmd_conflict".to_string(),
            expected_tick: Some(1),
            command: Command::SetStationPriority {
                station_id: StationId("hub_haven".into()),
                priority: 3,
            },
        };
        let second = actor.handle_command(envelope2);
        assert_eq!(second.status, CommandStatus::Rejected);
        assert_eq!(second.error.as_ref().unwrap().code, "IdempotencyConflict");
    }

    /// Same ID with same expected_tick but different command body is also
    /// rejected as IdempotencyConflict.
    #[test]
    fn same_id_different_command_body_conflict() {
        let (mut actor, _) = test_actor();
        actor.lifecycle = GameLifecycle::Paused;
        actor.state = Some(paused_state_at_tick0());
        let envelope1 = CommandEnvelope {
            id: "cmd_body_conflict".to_string(),
            expected_tick: None,
            command: Command::SetStationPriority {
                station_id: StationId("hub_haven".into()),
                priority: 3,
            },
        };
        let first = actor.handle_command(envelope1);
        assert_eq!(first.status, CommandStatus::Applied);

        // Same ID, same expected_tick, but different command (priority
        // changed from 3 to 9) — must be rejected.
        let envelope2 = CommandEnvelope {
            id: "cmd_body_conflict".to_string(),
            expected_tick: None,
            command: Command::SetStationPriority {
                station_id: StationId("hub_haven".into()),
                priority: 9,
            },
        };
        let second = actor.handle_command(envelope2);
        assert_eq!(second.status, CommandStatus::Rejected);
        assert_eq!(second.error.as_ref().unwrap().code, "IdempotencyConflict");
    }

    /// expected_tick mismatch rejects the command.
    #[test]
    fn expected_tick_mismatch_rejected() {
        let (mut actor, _) = test_actor();
        actor.lifecycle = GameLifecycle::Running;
        actor.state = Some(paused_state_at_tick0());
        // Current tick is 0, expect tick 5 — should be rejected
        let result = actor.handle_command(CommandEnvelope {
            id: "cmd_etick".to_string(),
            expected_tick: Some(5),
            command: Command::SetStationPriority {
                station_id: StationId("hub_haven".into()),
                priority: 3,
            },
        });
        assert_eq!(result.status, CommandStatus::Rejected);
        assert_eq!(result.error.as_ref().unwrap().code, "ExpectedTickMismatch");
    }

    /// Commands queued during Running are applied at the correct tick.
    #[test]
    fn queued_command_applied_at_next_tick() {
        let (mut actor, _) = test_actor();
        actor.lifecycle = GameLifecycle::Running;
        actor.state = Some(paused_state_at_tick0());

        // Queue a command for tick 1
        let result = actor.handle_command(CommandEnvelope {
            id: "cmd_queue1".to_string(),
            expected_tick: None,
            command: Command::SetStationPriority {
                station_id: StationId("hub_haven".into()),
                priority: 5,
            },
        });
        assert_eq!(result.status, CommandStatus::Accepted);
        assert_eq!(result.effective_tick, Some(1));

        // Advance one tick — the command should be consumed
        actor.handle_scheduler_tick();
        assert_eq!(actor.state.as_ref().unwrap().tick, 1);
        // pending_commands for tick 1 should now be empty
        assert!(!actor.pending_commands.contains_key(&1));
    }

    /// Queued commands are sorted by server_sequence within a tick.
    #[test]
    fn queued_commands_sorted_by_sequence() {
        let (mut actor, _) = test_actor();
        actor.lifecycle = GameLifecycle::Running;
        actor.state = Some(paused_state_at_tick0());

        // Submit two commands for the same tick (tick 1)
        let r1 = actor.handle_command(CommandEnvelope {
            id: "cmd_seq1".to_string(),
            expected_tick: None,
            command: Command::SetStationPriority {
                station_id: StationId("alpha".into()),
                priority: 1,
            },
        });
        assert_eq!(r1.status, CommandStatus::Accepted);

        let r2 = actor.handle_command(CommandEnvelope {
            id: "cmd_seq2".to_string(),
            expected_tick: None,
            command: Command::SetStationPriority {
                station_id: StationId("beta".into()),
                priority: 2,
            },
        });
        assert_eq!(r2.status, CommandStatus::Accepted);

        // The two sequenced commands should have different server_sequences
        let seq1 = r1.server_sequence;
        let seq2 = r2.server_sequence;
        assert!(seq2 > seq1);

        // After tick advance, they're sorted and consumed
        actor.handle_scheduler_tick();
        assert!(!actor.pending_commands.contains_key(&1));
    }

    /// Session receipts persist after NewGame (state replacement).
    /// After NewGame the actor is in Loading; gameplay commands are
    /// rejected by lifecycle validation, but the receipt itself is
    /// still present for future idempotency once the lifecycle allows
    /// commands again.
    #[test]
    fn session_receipts_survive_new_game() {
        let (mut actor, _) = test_actor();
        actor.lifecycle = GameLifecycle::Paused;
        actor.state = Some(paused_state_at_tick0());

        // Submit a command and get a receipt
        let result = actor.handle_command(CommandEnvelope {
            id: "cmd_rec1".to_string(),
            expected_tick: None,
            command: Command::SetStationPriority {
                station_id: StationId("hub_haven".into()),
                priority: 3,
            },
        });
        assert_eq!(result.status, CommandStatus::Applied);

        // NewGame replaces state but receipts survive
        let ng_result = actor.handle_command(CommandEnvelope {
            id: "cmd_rec_ng".to_string(),
            expected_tick: None,
            command: Command::NewGame {
                scenario_id: ScenarioId("starting_system".into()),
            },
        });
        assert_eq!(ng_result.status, CommandStatus::Applied);

        // The receipt for cmd_rec1 should still be present in the map
        assert!(actor.session_receipts.contains_key("cmd_rec1"));

        // Replay cmd_rec1 while in Loading — lifecycle rejects gameplay
        // commands, but the receipt remains for use after Loading completes.
        let replay = actor.handle_command(CommandEnvelope {
            id: "cmd_rec1".to_string(),
            expected_tick: None,
            command: Command::SetStationPriority {
                station_id: StationId("hub_haven".into()),
                priority: 3,
            },
        });
        // Rejected because Loading disallows all commands
        assert_eq!(replay.status, CommandStatus::Rejected);
        // But the receipt is still in the map
        assert!(actor.session_receipts.contains_key("cmd_rec1"));
    }

    /// A control command (AdvanceTicks) from Loading is rejected because
    /// the lifecycle validation (step 2) rejects all commands during Loading
    /// per ADR-0004.  Control commands are applied only after lifecycle
    /// validation passes.
    #[test]
    fn advance_ticks_rejected_during_loading() {
        let (mut actor, _) = test_actor();
        actor.lifecycle = GameLifecycle::Loading;
        actor.loading = Some(LoadingStatus {
            operation: LoadingOperation::NewGame,
            stage: LoadingStage::ValidatingContent,
        });
        let result = actor.handle_command(CommandEnvelope {
            id: "cmd_loading".to_string(),
            expected_tick: None,
            command: Command::AdvanceTicks { count: 10 },
        });
        // Loading rejects all commands per lifecycle validation
        assert_eq!(result.status, CommandStatus::Rejected);
        assert_eq!(result.error.as_ref().unwrap().code, "InvalidLifecycle");
    }

    /// A gameplay command with expected_tick=Some(0) at tick 0 passes.
    #[test]
    fn expected_tick_zero_at_tick_zero_passes() {
        let (mut actor, _) = test_actor();
        actor.lifecycle = GameLifecycle::Paused;
        actor.state = Some(paused_state_at_tick0());
        let result = actor.handle_command(CommandEnvelope {
            id: "cmd_et0".to_string(),
            expected_tick: Some(0),
            command: Command::SetStationPriority {
                station_id: StationId("hub_haven".into()),
                priority: 3,
            },
        });
        assert_eq!(result.status, CommandStatus::Applied);
    }

    // ─── Helpers ─────────────────────────────────────────────────────

    fn paused_state_at_tick0() -> GameState {
        GameState {
            schema_version: 1,
            content_version: "v1".to_string(),
            lifecycle: GameLifecycle::Paused,
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
        }
    }
}
