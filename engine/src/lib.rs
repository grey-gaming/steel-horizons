// Steel Horizons — deterministic simulation engine
// Copyright (c) 2026 Steel Horizons contributors
// UNLICENSED — private development; no public grant

#![deny(unsafe_code)]
#![deny(missing_docs)]
#![deny(elided_lifetimes_in_paths)]
#![forbid(noop_method_call)]

//! Deterministic simulation engine for Steel Horizons.
//!
//! This crate implements the integer-only, checked-arithmetic simulation
//! core, authenticated loopback HTTP/WebSocket API, canonical JSON content,
//! and replay-equivalent persistence.

/// Simulation arithmetic types and checked overflow errors.
pub mod arithmetic;

/// Project-owned deterministic PRNG.
pub mod prng;

/// Identity types for entities, resources, and commands.
pub mod id;

/// Root simulation state and canonical tick-zero constructor.
pub mod state;

/// Tick transaction skeleton with phase hooks.
pub mod tick;

/// Command types and sequencing.
pub mod command;

/// Serialized content definitions and loading.
pub mod content;

/// Convenience re-export of the engine version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
