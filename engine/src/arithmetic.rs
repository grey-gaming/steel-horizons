//! Simulation arithmetic types and checked overflow errors.
//!
//! This module implements the integer-only, checked-arithmetic kernel for
//! the Steel Horizons simulation engine.  It provides:
//!
//! - Standard rate accumulator (`MilliRemainder`) — quotient/remainder at
//!   scale 1,000, used by mining.
//! - Denominator-specific rational accumulator (`consume_tick`) — used by
//!   research consumption where durations need not divide 1,000.
//! - Fuel accumulator (`consume_fuel`) — actual-distance-based consumption
//!   at the authored 10,000,000 denominator, with Life Support discount.
//! - Checked arithmetic error types for overflow and rate validation.
//!
//! ## Authoritative references
//!
//! - GDD 12 §Integer Numeric Representation
//! - ADR-0002 §Numeric Rules
//! - TDD 01 §Arithmetic Types

use serde::{Deserialize, Serialize};
use std::fmt;

// ─── Error types ──────────────────────────────────────────────────────

/// Checked arithmetic overflow or invariant violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithmeticError {
    /// Multiplication or addition overflowed the type's capacity.
    Overflow,
    /// Division by zero (zero denominator, zero cycle ticks, etc.).
    DivisionByZero,
    /// Integer underflow (subtraction result below zero).
    Underflow,
}

impl fmt::Display for ArithmeticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArithmeticError::Overflow => write!(f, "arithmetic overflow"),
            ArithmeticError::DivisionByZero => write!(f, "division by zero"),
            ArithmeticError::Underflow => write!(f, "arithmetic underflow"),
        }
    }
}

impl std::error::Error for ArithmeticError {}

/// Rate construction error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateError {
    /// The rate does not divide 1,000 exactly; use a rational accumulator.
    UseRationalAccumulator,
    /// Overflow during rate computation.
    Overflow,
}

impl fmt::Display for RateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RateError::UseRationalAccumulator => {
                write!(f, "rate does not divide 1000 exactly")
            }
            RateError::Overflow => write!(f, "rate overflow"),
        }
    }
}

impl std::error::Error for RateError {}

// ─── Milli-Remainder (standard rate accumulator, scale 1,000) ─────────

/// Standard milli-rate accumulator with invariant `value < 1000`.
///
/// Used by mining and other rates whose cycle length divides 1,000.
/// Each tick adds a per-tick increment; the accumulator returns whole
/// units produced and keeps the fractional remainder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MilliRemainder(u32);

impl MilliRemainder {
    /// Create a new remainder from an initial value.
    ///
    /// Returns `Err(ArithmeticError::Overflow)` if `value >= 1000`.
    pub fn new(value: u32) -> Result<Self, ArithmeticError> {
        if value >= 1000 {
            return Err(ArithmeticError::Overflow);
        }
        Ok(MilliRemainder(value))
    }

    /// Returns the current remainder value (always < 1000).
    pub fn value(&self) -> u32 {
        self.0
    }

    /// Add `increment` to the remainder and return whole units produced.
    ///
    /// The formula (GDD 12 §Standard Rate Accumulator):
    ///
    /// ```text
    /// remainder += increment
    /// produced = remainder / 1000
    /// remainder = remainder % 1000
    /// ```
    pub fn add_increment(&mut self, increment: u32) -> Result<u32, ArithmeticError> {
        let total = self
            .0
            .checked_add(increment)
            .ok_or(ArithmeticError::Overflow)?;
        let whole = total / 1000;
        self.0 = total % 1000;
        Ok(whole)
    }

    /// Compute the per-tick increment for `quantity` units per `cycle_ticks`.
    ///
    /// The formula (GDD 12 §Standard Rate Accumulator):
    ///
    /// ```text
    /// increment = quantity * 1000 / cycle_ticks
    /// ```
    ///
    /// Returns `Err(RateError::UseRationalAccumulator)` if the division is
    /// not exact (i.e., `quantity * 1000 % cycle_ticks != 0`).
    pub fn increment_for(quantity: u32, cycle_ticks: u32) -> Result<u32, RateError> {
        if cycle_ticks == 0 {
            return Err(RateError::UseRationalAccumulator);
        }
        let scaled = u32::checked_mul(quantity, 1000).ok_or(RateError::Overflow)?;
        if scaled % cycle_ticks != 0 {
            return Err(RateError::UseRationalAccumulator);
        }
        Ok(scaled / cycle_ticks)
    }
}

// ─── Rational research consumption ────────────────────────────────────

/// Consume `total_required` resources over `total_ticks` using a
/// denominator-specific rational accumulator.
///
/// Each tick adds `total_required` to `remainder`, then extracts
/// `consumed = remainder / total_ticks` and keeps the remainder.
/// At completion, exactly `total_required` units have been consumed.
///
/// The formula (GDD 12 §Denominator-Specific Rational Accumulator):
///
/// ```text
/// remainder += total_required
/// consumed_this_tick = remainder / total_ticks
/// remainder = remainder % total_ticks
/// ```
pub fn consume_tick(
    remainder: &mut u64,
    total_required: u64,
    total_ticks: u64,
) -> Result<u64, ArithmeticError> {
    if total_ticks == 0 {
        return Err(ArithmeticError::DivisionByZero);
    }
    let total = remainder
        .checked_add(total_required)
        .ok_or(ArithmeticError::Overflow)?;
    let consumed = total / total_ticks;
    *remainder = total % total_ticks;
    Ok(consumed)
}

// ─── Fuel accumulator ─────────────────────────────────────────────────

/// Consume Fuel based on actual distance moved.
///
/// The formula (GDD 12 §Fuel Accumulator):
///
/// ```text
/// mass_units = base_mass + payload_amount
/// raw_charge = actual_distance_moved_milli * mass_units
/// if LifeSupport is complete and segment.life_support_eligible:
///     discounted = raw_charge * 4 + fuel_efficiency_remainder
///     charge = discounted / 5
///     fuel_efficiency_remainder = discounted % 5
/// else:
///     charge = raw_charge
/// fuel_remainder += charge
/// fuel_consumed = fuel_remainder / 10_000_000
/// fuel_remainder = fuel_remainder % 10_000_000
/// ```
///
/// `actual_distance_milli` is capped by the remaining route segment, so
/// the final arrival tick never charges for distance not traveled.
pub fn consume_fuel(
    remainder: &mut u64,
    efficiency_remainder: &mut u8,
    actual_distance_milli: u64,
    base_mass: u64,
    payload_amount: u64,
    life_support_discount: bool,
) -> Result<u64, ArithmeticError> {
    let mass = base_mass
        .checked_add(payload_amount)
        .ok_or(ArithmeticError::Overflow)?;
    let raw = actual_distance_milli
        .checked_mul(mass)
        .ok_or(ArithmeticError::Overflow)?;

    let charge = if life_support_discount {
        // discounted = raw * 4 + efficiency_remainder
        let d4 = raw.checked_mul(4).ok_or(ArithmeticError::Overflow)?;
        let discounted = d4
            .checked_add(u64::from(*efficiency_remainder))
            .ok_or(ArithmeticError::Overflow)?;
        *efficiency_remainder = (discounted % 5) as u8;
        discounted / 5
    } else {
        raw
    };

    let total = remainder
        .checked_add(charge)
        .ok_or(ArithmeticError::Overflow)?;
    let consumed = total / 10_000_000;
    *remainder = total % 10_000_000;
    Ok(consumed)
}

// ─── Scale newtypes ───────────────────────────────────────────────────

/// Milli-distance wrapper — represents a distance in thousandths of a unit.
///
/// Provides checked arithmetic operations to prevent overflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MilliDistance(u64);

impl MilliDistance {
    /// Create a new milli-distance from a raw value.
    pub fn new(value: u64) -> Self {
        MilliDistance(value)
    }

    /// Return the raw milli-distance value.
    pub fn value(&self) -> u64 {
        self.0
    }

    /// Checked addition with another milli-distance.
    pub fn checked_add(&self, other: MilliDistance) -> Result<MilliDistance, ArithmeticError> {
        let sum = self
            .0
            .checked_add(other.0)
            .ok_or(ArithmeticError::Overflow)?;
        Ok(MilliDistance(sum))
    }

    /// Checked subtraction of another milli-distance.
    pub fn checked_sub(&self, other: MilliDistance) -> Result<MilliDistance, ArithmeticError> {
        let diff = self
            .0
            .checked_sub(other.0)
            .ok_or(ArithmeticError::Underflow)?;
        Ok(MilliDistance(diff))
    }

    /// Checked multiplication by a `u64` factor.
    pub fn checked_mul_u64(&self, factor: u64) -> Result<MilliDistance, ArithmeticError> {
        let prod = self
            .0
            .checked_mul(factor)
            .ok_or(ArithmeticError::Overflow)?;
        Ok(MilliDistance(prod))
    }
}

/// Milli-speed wrapper — speed in milli-units per tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MilliSpeed(u32);

impl MilliSpeed {
    /// Create a new milli-speed from a raw value.
    pub fn new(value: u32) -> Self {
        MilliSpeed(value)
    }

    /// Return the raw milli-speed value.
    pub fn value(&self) -> u32 {
        self.0
    }

    /// Compute effective speed with a rational multiplier `num / den`.
    ///
    /// Returns `speed * num / den` using checked u32/u64 arithmetic.
    pub fn checked_mul_ratio(&self, num: u32, den: u32) -> Result<MilliSpeed, ArithmeticError> {
        if den == 0 {
            return Err(ArithmeticError::DivisionByZero);
        }
        let product = u64::from(self.0)
            .checked_mul(u64::from(num))
            .ok_or(ArithmeticError::Overflow)?;
        let result = product / u64::from(den);
        if result > u64::from(u32::MAX) {
            return Err(ArithmeticError::Overflow);
        }
        Ok(MilliSpeed(result as u32))
    }

    /// Apply payload speed reduction factor.
    ///
    /// Formula (GDD 12 §Deterministic Travel):
    ///
    /// ```text
    /// payload_num = payload_capacity * 10 - payload_amount * 3
    /// payload_den = payload_capacity * 10
    /// effective_speed = base_speed * payload_num / payload_den
    /// ```
    pub fn checked_with_payload(
        &self,
        payload_amount: u32,
        payload_capacity: u32,
    ) -> Result<MilliSpeed, ArithmeticError> {
        if payload_capacity == 0 {
            // Zero-capacity ships (Research Ships) use multiplier 1/1
            return Ok(*self);
        }
        let payload_num = {
            let mul10 = payload_capacity
                .checked_mul(10)
                .ok_or(ArithmeticError::Overflow)?;
            let mul3 = payload_amount
                .checked_mul(3)
                .ok_or(ArithmeticError::Overflow)?;
            mul10.checked_sub(mul3).ok_or(ArithmeticError::Overflow)?
        };
        let payload_den = payload_capacity
            .checked_mul(10)
            .ok_or(ArithmeticError::Overflow)?;
        self.checked_mul_ratio(payload_num, payload_den)
    }
}

// ─── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ─── MilliRemainder ───────────────────────────────────────────────

    #[test]
    fn milli_remainder_new_valid() {
        let r = MilliRemainder::new(0).unwrap();
        assert_eq!(r.value(), 0);
        let r = MilliRemainder::new(999).unwrap();
        assert_eq!(r.value(), 999);
    }

    #[test]
    fn milli_remainder_new_invalid() {
        let r = MilliRemainder::new(1000);
        assert_eq!(r, Err(ArithmeticError::Overflow));
        let r = MilliRemainder::new(5000);
        assert_eq!(r, Err(ArithmeticError::Overflow));
    }

    #[test]
    fn milli_remainder_exact_rate() {
        // 1 unit per 10 ticks: increment = 1 * 1000 / 10 = 100
        let inc = MilliRemainder::increment_for(1, 10).unwrap();
        assert_eq!(inc, 100);

        let mut r = MilliRemainder::new(0).unwrap();
        for _tick in 1..=10 {
            let produced = r.add_increment(inc).unwrap();
            if _tick == 10 {
                assert_eq!(produced, 1, "exact unit at tick 10");
            } else {
                assert_eq!(produced, 0, "no unit before tick 10");
            }
        }
        assert_eq!(r.value(), 0, "remainder consumed after 10 ticks");
    }

    #[test]
    fn milli_remainder_accumulation() {
        // 3 units per 10 ticks: increment = 300
        let inc = MilliRemainder::increment_for(3, 10).unwrap();
        assert_eq!(inc, 300);

        let mut r = MilliRemainder::new(0).unwrap();
        let mut total = 0;
        for _tick in 1..=10 {
            total += r.add_increment(inc).unwrap();
        }
        assert_eq!(total, 3, "3 units over 10 ticks");
    }

    #[test]
    fn milli_remainder_non_exact_rate() {
        // 1 unit per 3 ticks — does not divide 1000 evenly
        let inc = MilliRemainder::increment_for(1, 3);
        assert_eq!(inc, Err(RateError::UseRationalAccumulator));
    }

    #[test]
    fn milli_remainder_overflow() {
        // 999 + 1 = 1000 → produces 1 whole unit, remainder 0 (not overflow)
        let mut r = MilliRemainder::new(999).unwrap();
        let produced = r.add_increment(1).unwrap();
        assert_eq!(produced, 1);
        assert_eq!(r.value(), 0);

        // Starting from 1, add u32::MAX: 1 + 4294967295 = 4294967296 overflows u32
        let mut r2 = MilliRemainder::new(1).unwrap();
        let err = r2.add_increment(u32::MAX);
        assert_eq!(err, Err(ArithmeticError::Overflow));
    }

    #[test]
    fn milli_remainder_zero_cycle() {
        let inc = MilliRemainder::increment_for(1, 0);
        assert_eq!(inc, Err(RateError::UseRationalAccumulator));
    }

    #[test]
    fn milli_remainder_overflow_increment() {
        // quantity=4294968 * 1000 > u32::MAX (4294967295)
        let inc = MilliRemainder::increment_for(4294968, 1);
        assert_eq!(inc, Err(RateError::Overflow));
    }

    // ─── Consume tick (rational research) ────────────────────────────

    #[test]
    fn consume_tick_exact_project() {
        // 100 units over 50 ticks: each tick consumes 2
        let mut rem = 0;
        for tick in 0..50 {
            let consumed = consume_tick(&mut rem, 100, 50).unwrap();
            assert_eq!(consumed, 2, "tick {tick}: consume 2");
        }
        assert_eq!(rem, 0, "remainder zero at completion");
    }

    #[test]
    fn consume_tick_remainder_bounds() {
        // 10 units over 7 ticks: not exact
        let mut rem = 0;
        let mut total_consumed = 0;
        for _ in 0..7 {
            total_consumed += consume_tick(&mut rem, 10, 7).unwrap();
        }
        assert_eq!(total_consumed, 10, "exactly 10 consumed");
        assert_eq!(rem, 0, "remainder zero after 7 ticks");
    }

    #[test]
    fn consume_tick_large_remainder() {
        let mut rem = 100;
        let consumed = consume_tick(&mut rem, 100, 10).unwrap();
        assert_eq!(consumed, 20, "100 + 100 = 200 / 10 = 20");
        assert_eq!(rem, 0, "remainder zero");
    }

    #[test]
    fn consume_tick_zero_total_ticks() {
        let mut rem = 0;
        let err = consume_tick(&mut rem, 10, 0);
        assert_eq!(err, Err(ArithmeticError::DivisionByZero));
    }

    #[test]
    fn consume_tick_overflow() {
        let mut rem = u64::MAX;
        let err = consume_tick(&mut rem, 100, 10);
        assert_eq!(err, Err(ArithmeticError::Overflow));
    }

    // ─── Fuel accumulator ────────────────────────────────────────────

    #[test]
    fn consume_fuel_no_discount() {
        // Move 1,000,000 milli-units, mass 100, raw = 100M, fuel_consumed = 10
        let mut rem = 0;
        let mut eff_rem = 0;
        let consumed = consume_fuel(&mut rem, &mut eff_rem, 1_000_000, 100, 0, false).unwrap();
        assert_eq!(consumed, 10);
        assert_eq!(rem, 0);
        assert_eq!(eff_rem, 0);
    }

    #[test]
    fn consume_fuel_with_discount() {
        // Same movement with Life Support discount: 4/5 factor
        let mut rem = 0;
        let mut eff_rem = 0;
        let consumed = consume_fuel(&mut rem, &mut eff_rem, 1_000_000, 100, 0, true).unwrap();
        // raw = 100M, discounted = 80M, charge = 80M / 5 = 16M? Wait:
        // raw = 1_000_000 * 100 = 100_000_000
        // discounted = 100_000_000 * 4 = 400_000_000
        // charge = 400_000_000 / 5 = 80_000_000
        // consumed = (0 + 80_000_000) / 10_000_000 = 8
        assert_eq!(consumed, 8);
        assert_eq!(rem, 0);
    }

    #[test]
    fn consume_fuel_partial_tick() {
        // Small movement: no fuel consumed yet, remainder accumulates
        let mut rem = 0;
        let mut eff_rem = 0;
        let consumed = consume_fuel(&mut rem, &mut eff_rem, 1000, 100, 0, false).unwrap();
        assert_eq!(consumed, 0);
        assert_eq!(rem, 100_000); // 1000 * 100 = 100_000
    }

    #[test]
    fn consume_fuel_with_payload() {
        // Mass = 100 base + 50 payload = 150
        let mut rem = 0;
        let mut eff_rem = 0;
        let consumed = consume_fuel(&mut rem, &mut eff_rem, 100_000, 100, 50, false).unwrap();
        // raw = 100_000 * 150 = 15_000_000
        // consumed = 15_000_000 / 10_000_000 = 1
        // remainder = 15_000_000 % 10_000_000 = 5_000_000
        assert_eq!(consumed, 1);
        assert_eq!(rem, 5_000_000);
    }

    #[test]
    fn consume_fuel_efficiency_accumulation() {
        // Test that efficiency_remainder accumulates across ticks
        let mut rem = 0;
        let mut eff_rem = 0;
        // raw = 1 * 1 = 1
        // discounted = 4 * 1 + 0 = 4
        // charge = 4 / 5 = 0, remainder = 4
        let consumed = consume_fuel(&mut rem, &mut eff_rem, 1, 1, 0, true).unwrap();
        assert_eq!(consumed, 0);
        assert_eq!(eff_rem, 4);

        // Second tick: raw = 1 * 1 = 1
        // discounted = 4 + 4 = 8
        // charge = 8 / 5 = 1, remainder = 3
        let consumed = consume_fuel(&mut rem, &mut eff_rem, 1, 1, 0, true).unwrap();
        assert_eq!(consumed, 0); // fuel_remainder 0 + 1 = 1, consumed = 0
        assert_eq!(eff_rem, 3);
    }

    #[test]
    fn consume_fuel_overflow_mass() {
        let mut rem = 0;
        let mut eff_rem = 0;
        let err = consume_fuel(&mut rem, &mut eff_rem, 1, u64::MAX, 1, false);
        assert_eq!(err, Err(ArithmeticError::Overflow));
    }

    #[test]
    fn consume_fuel_overflow_distance_mul() {
        let mut rem = 0;
        let mut eff_rem = 0;
        let err = consume_fuel(&mut rem, &mut eff_rem, u64::MAX, 2, 0, false);
        assert_eq!(err, Err(ArithmeticError::Overflow));
    }

    // ─── Scale newtypes ──────────────────────────────────────────────

    #[test]
    fn milli_distance_checked_add() {
        let a = MilliDistance::new(100);
        let b = MilliDistance::new(200);
        let sum = a.checked_add(b).unwrap();
        assert_eq!(sum.value(), 300);
    }

    #[test]
    fn milli_distance_checked_add_overflow() {
        let a = MilliDistance::new(u64::MAX);
        let b = MilliDistance::new(1);
        let err = a.checked_add(b);
        assert_eq!(err, Err(ArithmeticError::Overflow));
    }

    #[test]
    fn milli_distance_checked_sub() {
        let a = MilliDistance::new(300);
        let b = MilliDistance::new(100);
        let diff = a.checked_sub(b).unwrap();
        assert_eq!(diff.value(), 200);
    }

    #[test]
    fn milli_distance_checked_sub_underflow() {
        let a = MilliDistance::new(100);
        let b = MilliDistance::new(200);
        let err = a.checked_sub(b);
        assert_eq!(err, Err(ArithmeticError::Underflow));
    }

    #[test]
    fn milli_speed_checked_mul_ratio() {
        let s = MilliSpeed::new(1000);
        let r = s.checked_mul_ratio(3, 2).unwrap(); // 1000 * 3 / 2 = 1500
        assert_eq!(r.value(), 1500);
    }

    #[test]
    fn milli_speed_checked_mul_ratio_zero_den() {
        let s = MilliSpeed::new(100);
        let err = s.checked_mul_ratio(1, 0);
        assert_eq!(err, Err(ArithmeticError::DivisionByZero));
    }

    #[test]
    fn milli_speed_with_payload() {
        // base speed 1000, capacity 100, payload 30
        // payload_num = 100*10 - 30*3 = 1000 - 90 = 910
        // payload_den = 100*10 = 1000
        // effective = 1000 * 910 / 1000 = 910
        let s = MilliSpeed::new(1000);
        let eff = s.checked_with_payload(30, 100).unwrap();
        assert_eq!(eff.value(), 910);
    }

    #[test]
    fn milli_speed_with_payload_no_capacity() {
        // Zero-capacity ship (Research Ship): multiplier 1/1
        let s = MilliSpeed::new(500);
        let eff = s.checked_with_payload(0, 0).unwrap();
        assert_eq!(eff.value(), 500);
    }

    // ─── Property-style tests ────────────────────────────────────────

    /// After `cycle_ticks` ticks of exact rate, total == quantity and remainder == 0.
    fn verify_exact_cycle(quantity: u32, cycle_ticks: u32) {
        let inc = MilliRemainder::increment_for(quantity, cycle_ticks).unwrap();
        let mut r = MilliRemainder::new(0).unwrap();
        let mut total = 0u32;
        for _ in 0..cycle_ticks {
            total += r.add_increment(inc).unwrap();
        }
        assert_eq!(total, quantity);
        assert_eq!(r.value(), 0);
    }

    #[test]
    fn exact_cycles() {
        verify_exact_cycle(1, 1);
        verify_exact_cycle(1, 2);
        verify_exact_cycle(1, 4);
        verify_exact_cycle(1, 5);
        verify_exact_cycle(1, 8);
        verify_exact_cycle(1, 10);
        verify_exact_cycle(1, 20);
        verify_exact_cycle(1, 25);
        verify_exact_cycle(1, 40);
        verify_exact_cycle(1, 50);
        verify_exact_cycle(1, 100);
        verify_exact_cycle(1, 125);
        verify_exact_cycle(1, 200);
        verify_exact_cycle(1, 250);
        verify_exact_cycle(1, 500);
        verify_exact_cycle(1, 1000);
        verify_exact_cycle(3, 10);
        verify_exact_cycle(5, 8);
        verify_exact_cycle(7, 10);
        verify_exact_cycle(12, 15);
        verify_exact_cycle(25, 40);
        verify_exact_cycle(99, 100);
        verify_exact_cycle(100, 100);
    }

    /// `consume_tick` always consumes exactly `total_required` over `total_ticks`.
    fn verify_exact_consumption(total_required: u64, total_ticks: u64) {
        let mut rem = 0;
        let mut total_consumed = 0;
        for _ in 0..total_ticks {
            total_consumed += consume_tick(&mut rem, total_required, total_ticks).unwrap();
        }
        assert_eq!(total_consumed, total_required);
        assert_eq!(rem, 0);
    }

    #[test]
    fn exact_consumptions() {
        verify_exact_consumption(10, 1);
        verify_exact_consumption(10, 3);
        verify_exact_consumption(10, 7);
        verify_exact_consumption(100, 50);
        verify_exact_consumption(500, 100);
        verify_exact_consumption(1000, 100);
        verify_exact_consumption(1000, 137);
        verify_exact_consumption(1000, 500);
        verify_exact_consumption(10000, 333);
    }

    /// Fuel conservation: accumulating fuel charges eventually produces
    /// exact fuel consumption at the 10_000_000 denominator.
    fn verify_fuel_cycle(mass: u64, distance_per_tick: u64, ticks: u64, discount: bool) {
        let mut rem = 0;
        let mut eff_rem = 0;
        let mut total_consumed = 0;
        for _ in 0..ticks {
            total_consumed +=
                consume_fuel(&mut rem, &mut eff_rem, distance_per_tick, mass, 0, discount).unwrap();
        }
        let expected = (mass * distance_per_tick * ticks) / 10_000_000;
        if discount {
            let discounted = (mass * distance_per_tick * ticks * 4) / 5;
            assert_eq!(total_consumed, discounted / 10_000_000);
        } else {
            assert_eq!(total_consumed, expected);
        }
    }

    #[test]
    fn fuel_conservation() {
        verify_fuel_cycle(100, 1_000_000, 10, false);
        verify_fuel_cycle(100, 1_000_000, 10, true);
        verify_fuel_cycle(200, 500_000, 20, false);
        verify_fuel_cycle(50, 10_000_000, 5, false);
    }
}
