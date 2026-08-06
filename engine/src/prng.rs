//! Project-owned deterministic PRNG (xoshiro256**).
//!
//! Implements the xoshiro256** algorithm as defined in GDD 12 §Save, Load, and Replay
//! and ADR-0002 §Randomness.  The four `u64` state words are serialized in `RNGState`.
//! This is the sole exception to checked gameplay arithmetic: wrapping-modulo-2^64 `u64`
//! operations are intentional and golden-tested.
//!
//! ## Authoritative references
//!
//! - GDD 12 §Save, Load, and Replay (xoshiro256** transition)
//! - ADR-0002 §Randomness, §Stable Iteration and Arithmetic

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ─── xoshiro256** ────────────────────────────────────────────────────

/// Project-owned deterministic PRNG (xoshiro256**).
///
/// Four `u64` state words are serialized in `RNGState`.  The all-zero state
/// is invalid and produces `None` from `Prng::new`.
///
/// ## Transition formula (GDD 12)
///
/// ```text
/// result = rotl(s1 * 5, 7) * 9
/// t = s1 << 17
/// s2 ^= s0; s3 ^= s1; s1 ^= s2; s0 ^= s3
/// s2 ^= t; s3 = rotl(s3, 45)
/// return result
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct Prng {
    /// The four internal state words.
    s: [u64; 4],
}

impl Prng {
    /// Create a new PRNG from four state words.
    ///
    /// Returns `None` if all words are zero (invalid all-zero state).
    pub fn new(words: [u64; 4]) -> Option<Self> {
        if words == [0; 4] {
            return None;
        }
        Some(Prng { s: words })
    }

    /// Return the internal state words (for serialization into `RNGState`).
    pub fn words(&self) -> [u64; 4] {
        self.s
    }

    /// Rotate left (wrapping u64).
    fn rotl(x: u64, k: u32) -> u64 {
        x.rotate_left(k)
    }

    /// Advance the PRNG and return the next `u64` output.
    ///
    /// This function intentionally uses wrapping arithmetic — it is the sole
    /// exception to the project's checked-overflow rule.
    pub fn next_u64(&mut self) -> u64 {
        let s0 = self.s[0];
        let s1 = self.s[1];
        let s2 = self.s[2];
        let s3 = self.s[3];

        let result = Self::rotl(s1.wrapping_mul(5), 7).wrapping_mul(9);

        // s2 ^= s0;  s3 ^= s1;  s1 ^= s2;  s0 ^= s3
        let ns2 = s2 ^ s0;
        let ns3 = s3 ^ s1;
        let ns1 = s1 ^ ns2;
        let ns0 = s0 ^ ns3;

        let t = s1 << 17;
        self.s[0] = ns0;
        self.s[1] = ns1;
        self.s[2] = ns2 ^ t;
        self.s[3] = Self::rotl(ns3, 45);

        result
    }

    /// Generate a `u64` in `[min, max]` (inclusive).
    ///
    /// Uses rejection-free modulo-biased sampling.  For unbiased uniform
    /// selection, callers should use this only when `max - min` is small
    /// relative to `u64::MAX`.
    pub fn next_u64_range(&mut self, min: u64, max: u64) -> u64 {
        debug_assert!(min <= max);
        let range = max.wrapping_sub(min).wrapping_add(1);
        let value = self.next_u64();
        min.wrapping_add(value.wrapping_rem(range))
    }

    /// Derive a new independent PRNG from the current state.
    ///
    /// Consumes two `u64` outputs to seed the child, leaving `self` advanced.
    pub fn split(&mut self) -> Self {
        Prng {
            s: [
                self.next_u64(),
                self.next_u64(),
                self.next_u64(),
                self.next_u64(),
            ],
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Reference golden vectors from the authoritative xoshiro256** implementation.
    ///
    /// Seeds: [1, 2, 3, 4]
    /// Expected first 10 outputs (computed against the Rust reference implementation
    /// matching the actual crate code).  Seed: [1, 2, 3, 4].
    const EXPECTED_GOLDEN: [u64; 10] = [
        11520,
        0,
        1509978240,
        1215971899390074240,
        1216172134540287360,
        607988272756665600,
        16172922978634559625,
        8476171486693032832,
        10595114339597558777,
        2904607092377533576,
    ];

    /// The PRNG with all-zero state is invalid.
    #[test]
    fn all_zero_state_invalid() {
        assert!(Prng::new([0; 4]).is_none());
    }

    /// Golden vectors match the reference xoshiro256** implementation.
    #[test]
    fn golden_vectors() {
        let mut prng = Prng::new([1, 2, 3, 4]).unwrap();
        for i in 0..10 {
            let output = prng.next_u64();
            assert_eq!(
                output, EXPECTED_GOLDEN[i],
                "golden vector mismatch at index {i}"
            );
        }
    }

    /// Split creates a new independent PRNG and advances the parent.
    #[test]
    fn split_is_independent() {
        let mut parent = Prng::new([1, 2, 3, 4]).unwrap();
        let mut child = parent.split();

        // Parent advanced by 4 (the split consumed 4 outputs)
        let parent_output = parent.next_u64();
        let child_output = child.next_u64();
        // They should differ — independent streams
        assert_ne!(parent_output, child_output);
    }

    /// Range generation stays within bounds.
    #[test]
    fn range_bounds() {
        let mut prng = Prng::new([42, 0, 0, 1]).unwrap();
        for _ in 0..100 {
            let v = prng.next_u64_range(10, 20);
            assert!(v >= 10 && v <= 20, "value {v} outside [10, 20]");
        }
    }

    /// Range with min == max returns that single value.
    #[test]
    fn range_single_value() {
        let mut prng = Prng::new([99, 88, 77, 66]).unwrap();
        for _ in 0..10 {
            assert_eq!(prng.next_u64_range(5, 5), 5);
        }
    }

    /// PRNG serialization round-trips through JSON.
    #[test]
    fn prng_serde_round_trip() {
        let prng = Prng::new([1, 2, 3, 4]).unwrap();
        let json = serde_json::to_string(&prng).unwrap();
        let back: Prng = serde_json::from_str(&json).unwrap();
        assert_eq!(prng, back);
    }

    /// Deterministic: same seed produces same sequence.
    #[test]
    fn deterministic_sequence() {
        let mut a = Prng::new([10, 20, 30, 40]).unwrap();
        let mut b = Prng::new([10, 20, 30, 40]).unwrap();
        for _ in 0..20 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    /// Different seeds produce different sequences (very likely).
    #[test]
    fn different_seeds_diverge() {
        let mut a = Prng::new([1, 2, 3, 4]).unwrap();
        let mut b = Prng::new([4, 3, 2, 1]).unwrap();
        // First output should differ
        assert_ne!(a.next_u64(), b.next_u64());
    }
}
