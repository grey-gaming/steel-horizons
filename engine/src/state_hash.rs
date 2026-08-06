//! State hash computation — ADR-0006 canonical state digest.
//!
//! Computes the versioned SHA-256 state hash over a `GameState` using the
//! canonical JSON v1 writer.  This is the same mechanism as the content hash
//! but operates on runtime simulation state.
//!
//! ## Authoritative references
//!
//! - ADR-0006 §Canonical content hash input
//! - ADR-0006 §Domain separation prefixes

#![allow(missing_docs)]

use sha2::{Digest, Sha256};

use crate::canonical::to_canonical_bytes;
use crate::state::GameState;

/// The V1 state-domain prefix bytes for SHA-256 domain separation.
const STATE_DOMAIN_PREFIX: &[u8] = b"steel-horizons/state/sha256-canonical-json-v1";

/// Errors produced by state hashing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateHashError {
    /// Canonical JSON serialization failed.
    Canonical(String),
}

impl std::fmt::Display for StateHashError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateHashError::Canonical(msg) => {
                write!(f, "canonical serialization error: {}", msg)
            }
        }
    }
}

/// Result type for state hash operations.
pub type StateHashResult<T = [u8; 32]> = Result<T, StateHashError>;

/// Compute the V1 state hash for a `GameState`.
///
/// The hash is:
///
/// ```text
/// SHA-256(state-domain-prefix || 0x00 || canonical(GameState))
/// ```
pub fn compute_state_hash(state: &GameState) -> StateHashResult {
    // Serialize to serde_json::Value
    let value =
        serde_json::to_value(state).map_err(|e| StateHashError::Canonical(e.to_string()))?;

    // Canonical JSON bytes
    let canonical_bytes =
        to_canonical_bytes(&value).map_err(|e| StateHashError::Canonical(e.to_string()))?;

    // Domain-prefixed hash
    let mut hasher = Sha256::new();
    hasher.update(STATE_DOMAIN_PREFIX);
    hasher.update([0x00]);
    hasher.update(&canonical_bytes);
    Ok(hasher.finalize().into())
}

/// Format a 32-byte hash as 64 lowercase hex characters.
pub fn format_state_hash(hash: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for byte in hash.iter() {
        s.push_str(&format!("{:02x}", byte));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::*;
    use crate::state_construct::build_starting_state;
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

    /// The canonical starting state hash is stable and matches the committed golden.
    #[test]
    fn canonical_starting_state_hash() {
        let catalog = load_catalog();
        let state = build_starting_state(&catalog).unwrap();
        let hash = compute_state_hash(&state).unwrap();
        let hash_str = format_state_hash(&hash);

        // Check against golden file
        let golden_path = std::env::var("CARGO_MANIFEST_DIR")
            .map(|p| PathBuf::from(p).parent().unwrap().join("tests/goldens"))
            .unwrap_or_else(|_| PathBuf::from("tests/goldens"))
            .join("state_hash.txt");
        let golden = std::fs::read_to_string(&golden_path)
            .unwrap_or_else(|e| panic!("Cannot read golden file {}: {}", golden_path.display(), e));
        let golden = golden.trim();
        assert_eq!(
            hash_str,
            golden,
            "state hash does not match golden at {}",
            golden_path.display()
        );
    }

    /// State hash is deterministic (same input → same hash).
    #[test]
    fn state_hash_deterministic() {
        let catalog = load_catalog();
        let state = build_starting_state(&catalog).unwrap();
        let h1 = compute_state_hash(&state).unwrap();
        let h2 = compute_state_hash(&state).unwrap();
        assert_eq!(h1, h2, "state hash must be deterministic");
    }

    /// State hash changes when state changes.
    #[test]
    fn state_hash_sensitive_to_state() {
        let catalog = load_catalog();
        let state = build_starting_state(&catalog).unwrap();
        let original = compute_state_hash(&state).unwrap();

        // Modify tick and re-hash
        let mut modified = state.clone();
        modified.tick = 42;
        let new_hash = compute_state_hash(&modified).unwrap();
        assert_ne!(original, new_hash, "hash must change when state changes");
    }

    /// State hash is stable across GameState re-serialization.
    #[test]
    fn state_hash_insertion_order_independence() {
        let catalog = load_catalog();
        let state = build_starting_state(&catalog).unwrap();

        // Serialize and deserialize — BTreeMap keys stay sorted
        let json = serde_json::to_string(&state).unwrap();
        let state2: GameState = serde_json::from_str(&json).unwrap();
        let h1 = compute_state_hash(&state).unwrap();
        let h2 = compute_state_hash(&state2).unwrap();
        assert_eq!(h1, h2, "hash must survive Serde round trip");
    }
}
