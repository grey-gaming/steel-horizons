//! Content hash computation — ADR-0006 canonical content digest.
//!
//! Computes the versioned SHA-256 content hash over the validated
//! `ContentCatalog` using the canonical JSON v1 writer.
//!
//! ## Authoritative references
//!
//! - ADR-0006 §Canonical content hash input
//! - ADR-0006 §Domain separation prefixes
//! - ADR-0007 §Content hash in save envelope

#![allow(missing_docs)]

use sha2::{Digest, Sha256};

use crate::canonical::to_canonical_bytes;
use crate::content::ContentCatalog;

/// The V1 content-domain prefix bytes for SHA-256 domain separation.
const CONTENT_DOMAIN_PREFIX: &[u8] = b"steel-horizons/content/sha256-canonical-json-v1";

/// Errors produced by content hashing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentHashError {
    /// Canonical JSON serialization failed.
    Canonical(String),
}

impl std::fmt::Display for ContentHashError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentHashError::Canonical(msg) => write!(f, "canonical serialization error: {}", msg),
        }
    }
}

/// Result type for content hash operations.
pub type ContentHashResult<T = [u8; 32]> = Result<T, ContentHashError>;

/// Compute the V1 content hash for a validated `ContentCatalog`.
///
/// The hash is:
///
/// ```text
/// SHA-256(content-domain-prefix || 0x00 || canonical(CanonicalContentHashInput))
/// ```
///
/// where `CanonicalContentHashInput` is the two validated roots wrapped in a
/// single object per ADR-0006 §Canonical content hash input.
pub fn compute_content_hash(catalog: &ContentCatalog) -> ContentHashResult {
    // Build the canonical input object: { definitions: ..., starting_system: ... }
    let defs_value = serde_json::to_value(&catalog.definitions)
        .map_err(|e| ContentHashError::Canonical(e.to_string()))?;
    let scenario_value = serde_json::to_value(&catalog.starting_system)
        .map_err(|e| ContentHashError::Canonical(e.to_string()))?;

    let input_value = serde_json::json!({
        "definitions": defs_value,
        "starting_system": scenario_value,
    });

    // Canonical JSON bytes
    let canonical_bytes =
        to_canonical_bytes(&input_value).map_err(|e| ContentHashError::Canonical(e.to_string()))?;

    // Domain-prefixed hash
    let mut hasher = Sha256::new();
    hasher.update(CONTENT_DOMAIN_PREFIX);
    hasher.update([0x00]);
    hasher.update(&canonical_bytes);
    Ok(hasher.finalize().into())
}

/// Format a 32-byte hash as 64 lowercase hex characters.
pub fn format_hash(hash: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for byte in hash.iter() {
        s.push_str(&format!("{:02x}", byte));
    }
    s
}

// ─── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::*;
    use crate::id::RecipeId;
    use std::path::PathBuf;

    fn load_json<T: serde::de::DeserializeOwned>(path: &str) -> T {
        let content_dir = PathBuf::from(
            std::env::var("CARGO_MANIFEST_DIR")
                .map(|p| PathBuf::from(p).parent().unwrap().join("content"))
                .unwrap_or_else(|_| PathBuf::from("content")),
        );
        let full_path = content_dir.join(path);
        let data = std::fs::read_to_string(&full_path)
            .unwrap_or_else(|e| panic!("Cannot read {}: {}", full_path.display(), e));
        serde_json::from_str(&data)
            .unwrap_or_else(|e| panic!("Cannot parse {}: {}", full_path.display(), e))
    }

    /// The canonical content hash is stable and matches the committed golden.
    #[test]
    fn canonical_content_hash() {
        let defs: DefinitionsCatalog = load_json("definitions.v1.json");
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let catalog = ContentCatalog {
            definitions: defs,
            starting_system: scenario,
        };
        let hash = compute_content_hash(&catalog).unwrap();
        let hash_str = format_hash(&hash);

        // Check against golden file
        let golden_path = PathBuf::from(
            std::env::var("CARGO_MANIFEST_DIR")
                .map(|p| PathBuf::from(p).parent().unwrap().join("tests/goldens"))
                .unwrap_or_else(|_| PathBuf::from("tests/goldens")),
        )
        .join("content_hash.txt");
        let golden = std::fs::read_to_string(&golden_path)
            .unwrap_or_else(|e| panic!("Cannot read golden file {}: {}", golden_path.display(), e));
        let golden = golden.trim();
        assert_eq!(
            hash_str,
            golden,
            "content hash does not match golden at {}",
            golden_path.display()
        );
    }

    /// Content hash is deterministic (same input → same hash).
    #[test]
    fn content_hash_deterministic() {
        let defs: DefinitionsCatalog = load_json("definitions.v1.json");
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let catalog = ContentCatalog {
            definitions: defs,
            starting_system: scenario,
        };
        let h1 = compute_content_hash(&catalog).unwrap();
        let h2 = compute_content_hash(&catalog).unwrap();
        assert_eq!(h1, h2, "content hash must be deterministic");
    }

    /// Content hash changes when definitions change.
    #[test]
    fn content_hash_sensitive_to_definitions() {
        let defs: DefinitionsCatalog = load_json("definitions.v1.json");
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let catalog = ContentCatalog {
            definitions: defs,
            starting_system: scenario,
        };
        let original = compute_content_hash(&catalog).unwrap();

        // Modify a recipe and re-hash
        let mut modified = catalog.clone();
        modified.definitions.recipes[0].id = RecipeId("modified".into());
        let new_hash = compute_content_hash(&modified).unwrap();
        assert_ne!(
            original, new_hash,
            "hash must change when definitions change"
        );
    }

    /// Content hash changes when starting scenario changes.
    #[test]
    fn content_hash_sensitive_to_scenario() {
        let defs: DefinitionsCatalog = load_json("definitions.v1.json");
        let scenario: StartingScenario = load_json("starting_system.v1.json");
        let catalog = ContentCatalog {
            definitions: defs,
            starting_system: scenario,
        };
        let original = compute_content_hash(&catalog).unwrap();

        // Modify starting scenario and re-hash
        let mut modified = catalog.clone();
        modified.starting_system.tick = 42;
        let new_hash = compute_content_hash(&modified).unwrap();
        assert_ne!(
            original, new_hash,
            "hash must change when starting scenario changes"
        );
    }
}
