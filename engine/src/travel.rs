//! Deterministic travel geometry — route construction and speed calculation.
//!
//! Provides `TravelPlan::between()` for constructing two-segment travel plans
//! (radial burn + lane arc) between any two system positions, and helpers for
//! computing effective speed with lane multipliers and payload factors.
//!
//! ## Authoritative references
//!
//! - GDD 12 §Deterministic Travel
//! - ADR-0002 §Numeric Rules
//! - TDD 01 §Travel Geometry

use crate::arithmetic::{ArithmeticError, MilliSpeed};
use crate::state::{SystemPosition, TravelPlan, TravelSegment, TravelSegmentKind};
use crate::types::{ArcDirection, DestinationRef, LaneId};

// ─── Constants ────────────────────────────────────────────────────────

/// Full-turn constant: π × 2000, truncated to integer.
/// Any two angles on the same lane will differ by less than this.
pub const TAU_MILLI: i32 = 6283;

// ─── Lane multiplier lookup ───────────────────────────────────────────

/// Return the speed multiplier (num, den) for a given lane.
///
/// Inner: 3/2 — tight orbital arcs at high speed.
/// Habitable: 1/1 — nominal orbital speed.
/// Outer: 7/10 — reduced speed for sparse traffic.
/// Fringe: 1/2 — slow outermost lane.
pub fn lane_speed_multiplier(lane: LaneId) -> (u32, u32) {
    match lane {
        LaneId::Inner => (3, 2),
        LaneId::Habitable => (1, 1),
        LaneId::Outer => (7, 10),
        LaneId::Fringe => (1, 2),
    }
}

/// Determine whether a segment is life-support eligible based on its lane.
///
/// Radial burn: eligible if either endpoint lane is Outer or Fringe.
/// Lane arc: eligible if the destination lane is Outer or Fringe.
fn lane_is_life_support_eligible(lane: LaneId) -> bool {
    matches!(lane, LaneId::Outer | LaneId::Fringe)
}

// ─── Angular difference ───────────────────────────────────────────────

/// Compute the shortest angular difference between two angles in milliradians.
///
/// Returns a value in `(-TAU_MILLI/2, TAU_MILLI/2]` range, positive for
/// clockwise (direction of increasing angle) and negative for counter-clockwise.
/// The caller uses the sign to pick `ArcDirection`.
///
/// Formula (GDD 12 §Deterministic Travel):
///
/// ```text
/// diff = destination_angle - source_angle
/// if diff > TAU_MILLI / 2:  diff -= TAU_MILLI
/// if diff < -TAU_MILLI / 2: diff += TAU_MILLI
/// ```
pub fn angular_diff(source_angle: i32, destination_angle: i32) -> i32 {
    let half_tau = TAU_MILLI / 2;
    let mut diff = destination_angle - source_angle;
    if diff > half_tau {
        diff -= TAU_MILLI;
    } else if diff < -half_tau {
        diff += TAU_MILLI;
    }
    diff
}

/// Convert an angular difference (signed milliradians) to an `ArcDirection`.
///
/// Zero diff is assigned `ArcDirection::CounterClockwise` arbitrarily.
pub fn diff_to_direction(diff: i32) -> ArcDirection {
    if diff >= 0 {
        ArcDirection::Clockwise
    } else {
        ArcDirection::CounterClockwise
    }
}

/// Compute the absolute angular difference for distance calculation.
pub fn abs_angular_diff(source_angle: i32, destination_angle: i32) -> u64 {
    angular_diff(source_angle, destination_angle).unsigned_abs() as u64
}

// ─── TravelPlan construction ──────────────────────────────────────────

impl TravelPlan {
    /// Construct a travel plan between two system positions.
    ///
    /// The plan has at most two segments:
    ///
    /// 1. **Radial burn** — if `source.radius_units != destination.radius_units`,
    ///    moves from source radius to destination radius at half speed.
    /// 2. **Lane arc** — if source angle differs from destination angle (at
    ///    destination radius), moves along the destination lane to the target angle.
    ///
    /// If both radii are equal the radial segment is omitted.  If both radii
    /// and angles are equal the returned plan has an empty segment list (zero-length
    /// route — the caller should handle instant arrival).
    pub fn between(
        source: &SystemPosition,
        destination: &SystemPosition,
        dest_ref: DestinationRef,
    ) -> Self {
        let mut segments: Vec<TravelSegment> = Vec::with_capacity(2);

        // ── Radial burn ──
        let radius_diff =
            (source.radius_units as i64 - destination.radius_units as i64).unsigned_abs();
        let radial_distance = radius_diff * 1000;

        // Life support eligibility: radial burn eligible if either endpoint lane is Outer/Fringe
        let radial_ls = lane_is_life_support_eligible(source.lane_id)
            || lane_is_life_support_eligible(destination.lane_id);

        if radial_distance > 0 {
            segments.push(TravelSegment {
                kind: TravelSegmentKind::RadialBurn,
                lane_id: source.lane_id,
                total_distance_milli: radial_distance,
                remaining_distance_milli: radial_distance,
                speed_multiplier_num: 1,
                speed_multiplier_den: 2,
                life_support_eligible: radial_ls,
                arc_direction: None,
            });
        }

        // ── Lane arc ──
        let angular_distance = abs_angular_diff(source.angle_milli, destination.angle_milli);
        let arc_distance = angular_distance * destination.radius_units as u64;
        let diff = angular_diff(source.angle_milli, destination.angle_milli);

        let arc_ls = lane_is_life_support_eligible(destination.lane_id);

        if arc_distance > 0 {
            segments.push(TravelSegment {
                kind: TravelSegmentKind::LaneArc,
                lane_id: destination.lane_id,
                total_distance_milli: arc_distance,
                remaining_distance_milli: arc_distance,
                speed_multiplier_num: lane_speed_multiplier(destination.lane_id).0,
                speed_multiplier_den: lane_speed_multiplier(destination.lane_id).1,
                life_support_eligible: arc_ls,
                arc_direction: Some(diff_to_direction(diff)),
            });
        }

        TravelPlan {
            origin: source.clone(),
            destination: dest_ref,
            segments,
            active_segment: 0,
        }
    }
}

// ─── Speed calculation ────────────────────────────────────────────────

/// Compute the effective speed for a ship traversing a given travel segment.
///
/// Combines the ship's base speed, the segment's lane multiplier, and the
/// payload speed-reduction factor into a single checked result.
///
/// Formula (GDD 12 §Deterministic Travel):
///
/// ```text
/// effective_speed = base_speed * segment.num * payload.num
///                   / (segment.den * payload.den)
/// ```
///
/// where payload uses `checked_with_payload` which internally computes:
///
/// ```text
/// payload_num = capacity * 10 - amount * 3
/// payload_den = capacity * 10
/// ```
///
/// For zero-capacity ships (Research Ships) the payload factor is 1/1.
pub fn effective_speed(
    base_speed: MilliSpeed,
    segment: &TravelSegment,
    payload_amount: u32,
    payload_capacity: u32,
) -> Result<MilliSpeed, ArithmeticError> {
    let lane_adjusted =
        base_speed.checked_mul_ratio(segment.speed_multiplier_num, segment.speed_multiplier_den)?;
    lane_adjusted.checked_with_payload(payload_amount, payload_capacity)
}

// ─── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::BodyId;

    // ─── Angular difference ────────────────────────────────────────────

    #[test]
    fn angular_diff_same_angle() {
        let diff = angular_diff(0, 0);
        assert_eq!(diff, 0);
    }

    #[test]
    fn angular_diff_short_arc_clockwise() {
        // source=100, dest=200 — diff=100 clockwise
        let diff = angular_diff(100, 200);
        assert_eq!(diff, 100);
        assert_eq!(diff_to_direction(diff), ArcDirection::Clockwise);
    }

    #[test]
    fn angular_diff_short_arc_counter() {
        // source=200, dest=100 — diff=-100 counter-clockwise
        let diff = angular_diff(200, 100);
        assert_eq!(diff, -100);
        assert_eq!(diff_to_direction(diff), ArcDirection::CounterClockwise);
    }

    #[test]
    fn angular_diff_wrap_clockwise() {
        // source=6280, dest=10 — diff across 0 boundary
        // diff = 10 - 6280 = -6270
        // -6270 < -3141 → diff += 6283 = 13
        let diff = angular_diff(6280, 10);
        assert_eq!(diff, 13);
        assert_eq!(diff_to_direction(diff), ArcDirection::Clockwise);
    }

    #[test]
    fn angular_diff_wrap_counter() {
        // source=10, dest=6280 — diff = 6270 > 3141 → diff -= 6283 = -13
        let diff = angular_diff(10, 6280);
        assert_eq!(diff, -13);
        assert_eq!(diff_to_direction(diff), ArcDirection::CounterClockwise);
    }

    #[test]
    fn angular_diff_half_tau_boundary() {
        // source=0, dest=3142 (just over 3141) — diff = 3142 > 3141 → diff -= 6283 = -3141
        // This tests that diff > half_tau wraps
        let diff = angular_diff(0, 3142);
        assert_eq!(diff, -3141);
    }

    #[test]
    fn abs_angular_diff_same_angle() {
        assert_eq!(abs_angular_diff(0, 0), 0);
    }

    #[test]
    fn abs_angular_diff_positive() {
        assert_eq!(abs_angular_diff(100, 200), 100);
    }

    #[test]
    fn abs_angular_diff_negative() {
        assert_eq!(abs_angular_diff(200, 100), 100);
    }

    #[test]
    fn abs_angular_diff_wrap() {
        assert_eq!(abs_angular_diff(6280, 10), 13);
    }

    // ─── Lane multiplier ──────────────────────────────────────────────

    #[test]
    fn lane_multiplier_inner() {
        let (n, d) = lane_speed_multiplier(LaneId::Inner);
        assert_eq!((n, d), (3, 2));
    }

    #[test]
    fn lane_multiplier_habitable() {
        let (n, d) = lane_speed_multiplier(LaneId::Habitable);
        assert_eq!((n, d), (1, 1));
    }

    #[test]
    fn lane_multiplier_outer() {
        let (n, d) = lane_speed_multiplier(LaneId::Outer);
        assert_eq!((n, d), (7, 10));
    }

    #[test]
    fn lane_multiplier_fringe() {
        let (n, d) = lane_speed_multiplier(LaneId::Fringe);
        assert_eq!((n, d), (1, 2));
    }

    // ─── TravelPlan::between ───────────────────────────────────────────

    fn pos(lane: LaneId, radius: u32, angle: i32) -> SystemPosition {
        SystemPosition {
            lane_id: lane,
            radius_units: radius,
            angle_milli: angle,
        }
    }

    fn dest_body(body_id: &str) -> DestinationRef {
        DestinationRef::Body {
            body_id: BodyId(body_id.to_string()),
        }
    }

    #[test]
    fn radial_burn_only() {
        // Same angle, different radius — only radial segment
        let src = pos(LaneId::Inner, 600, 0);
        let dst = pos(LaneId::Habitable, 1200, 0);
        let plan = TravelPlan::between(&src, &dst, dest_body("planet_haven"));

        assert_eq!(plan.segments.len(), 1);
        let seg = &plan.segments[0];
        assert_eq!(seg.kind, TravelSegmentKind::RadialBurn);
        assert_eq!(seg.total_distance_milli, 600_000); // |1200-600| * 1000
        assert_eq!(seg.speed_multiplier_num, 1);
        assert_eq!(seg.speed_multiplier_den, 2);
        assert_eq!(seg.life_support_eligible, false);
        assert!(seg.arc_direction.is_none());
    }

    #[test]
    fn lane_arc_only() {
        // Same radius, different angle — only arc segment
        let src = pos(LaneId::Habitable, 1200, 0);
        let dst = pos(LaneId::Habitable, 1200, 1570);
        let plan = TravelPlan::between(&src, &dst, dest_body("planet_haven"));

        assert_eq!(plan.segments.len(), 1);
        let seg = &plan.segments[0];
        assert_eq!(seg.kind, TravelSegmentKind::LaneArc);
        assert_eq!(seg.total_distance_milli, 1570 * 1200); // angular diff * radius
        assert_eq!(seg.speed_multiplier_num, 1);
        assert_eq!(seg.speed_multiplier_den, 1);
        assert_eq!(seg.life_support_eligible, false);
        assert!(seg.arc_direction.is_some());
    }

    #[test]
    fn full_two_segment_route() {
        // Different radius and angle — both segments
        let src = pos(LaneId::Inner, 600, 0);
        let dst = pos(LaneId::Habitable, 1200, 1570);
        let plan = TravelPlan::between(&src, &dst, dest_body("planet_haven"));

        assert_eq!(plan.segments.len(), 2);

        // Segment 0: radial burn
        let rad = &plan.segments[0];
        assert_eq!(rad.kind, TravelSegmentKind::RadialBurn);
        assert_eq!(rad.total_distance_milli, 600_000);

        // Segment 1: lane arc
        let arc = &plan.segments[1];
        assert_eq!(arc.kind, TravelSegmentKind::LaneArc);
        assert_eq!(arc.lane_id, LaneId::Habitable);
        assert_eq!(arc.total_distance_milli, 1570 * 1200);
        assert_eq!(arc.speed_multiplier_num, 1);
        assert_eq!(arc.speed_multiplier_den, 1);
        assert_eq!(arc.life_support_eligible, false);
    }

    #[test]
    fn zero_distance_route() {
        // Same position — no segments (instant arrival)
        let src = pos(LaneId::Habitable, 1200, 0);
        let plan = TravelPlan::between(&src, &src, dest_body("planet_haven"));

        assert!(plan.segments.is_empty());
        assert_eq!(plan.active_segment, 0);
    }

    #[test]
    fn radial_burn_life_support_outer() {
        // Radial burn from Outer to Habitable — eligible because source is Outer
        let src = pos(LaneId::Outer, 1800, 0);
        let dst = pos(LaneId::Habitable, 1200, 1570);
        let plan = TravelPlan::between(&src, &dst, dest_body("planet_haven"));

        assert_eq!(plan.segments.len(), 2);
        let rad = &plan.segments[0];
        assert_eq!(rad.kind, TravelSegmentKind::RadialBurn);
        assert!(rad.life_support_eligible);
    }

    #[test]
    fn radial_burn_life_support_destination_fringe() {
        // Radial burn from Inner to Fringe — eligible because destination is Fringe
        let src = pos(LaneId::Inner, 600, 0);
        let dst = pos(LaneId::Fringe, 2400, 0);
        let plan = TravelPlan::between(&src, &dst, dest_body("planet_haven"));

        let rad = &plan.segments[0];
        assert!(rad.life_support_eligible);
    }

    #[test]
    fn lane_arc_life_support_outer() {
        // Lane arc on Outer — eligible
        let src = pos(LaneId::Outer, 1800, 0);
        let dst = pos(LaneId::Outer, 1800, 1570);
        let plan = TravelPlan::between(&src, &dst, dest_body("planet_haven"));

        let arc = &plan.segments[0];
        assert_eq!(arc.kind, TravelSegmentKind::LaneArc);
        assert!(arc.life_support_eligible);
    }

    #[test]
    fn lane_arc_life_support_fringe() {
        // Lane arc on Fringe — eligible
        let src = pos(LaneId::Fringe, 2400, 0);
        let dst = pos(LaneId::Fringe, 2400, 1570);
        let plan = TravelPlan::between(&src, &dst, dest_body("planet_haven"));

        let arc = &plan.segments[0];
        assert!(arc.life_support_eligible);
    }

    #[test]
    fn lane_arc_no_life_support_inner() {
        // Lane arc on Inner — not eligible
        let src = pos(LaneId::Inner, 600, 0);
        let dst = pos(LaneId::Inner, 600, 1570);
        let plan = TravelPlan::between(&src, &dst, dest_body("planet_haven"));

        let arc = &plan.segments[0];
        assert!(!arc.life_support_eligible);
    }

    // ─── Effective speed ──────────────────────────────────────────────

    #[test]
    fn effective_speed_basic() {
        // Base speed 1000, Inner multiplier 3/2, no payload → 1500
        let seg = TravelSegment {
            kind: TravelSegmentKind::LaneArc,
            lane_id: LaneId::Inner,
            total_distance_milli: 1000,
            remaining_distance_milli: 1000,
            speed_multiplier_num: 3,
            speed_multiplier_den: 2,
            life_support_eligible: false,
            arc_direction: Some(ArcDirection::Clockwise),
        };
        let speed = effective_speed(MilliSpeed::new(1000), &seg, 0, 100).unwrap();
        assert_eq!(speed.value(), 1500);
    }

    #[test]
    fn effective_speed_with_payload() {
        // Base speed 1000, Habitable 1/1, payload 30 of 100 capacity
        // payload_num = 100*10 - 30*3 = 1000 - 90 = 910
        // payload_den = 100*10 = 1000
        // effective = 1000 * 910 / 1000 = 910
        let seg = TravelSegment {
            kind: TravelSegmentKind::LaneArc,
            lane_id: LaneId::Habitable,
            total_distance_milli: 1000,
            remaining_distance_milli: 1000,
            speed_multiplier_num: 1,
            speed_multiplier_den: 1,
            life_support_eligible: false,
            arc_direction: Some(ArcDirection::Clockwise),
        };
        let speed = effective_speed(MilliSpeed::new(1000), &seg, 30, 100).unwrap();
        assert_eq!(speed.value(), 910);
    }

    #[test]
    fn effective_speed_research_ship() {
        // Research Ship: zero capacity → multiplier 1/1
        let seg = TravelSegment {
            kind: TravelSegmentKind::LaneArc,
            lane_id: LaneId::Outer,
            total_distance_milli: 1000,
            remaining_distance_milli: 1000,
            speed_multiplier_num: 7,
            speed_multiplier_den: 10,
            life_support_eligible: true,
            arc_direction: Some(ArcDirection::Clockwise),
        };
        let speed = effective_speed(MilliSpeed::new(1000), &seg, 0, 0).unwrap();
        assert_eq!(speed.value(), 700); // 1000 * 7 / 10 * 1/1
    }

    #[test]
    fn effective_speed_fringe_with_payload() {
        // Base speed 1000, Fringe 1/2, payload 50 of 100
        // payload_num = 100*10 - 50*3 = 1000 - 150 = 850
        // payload_den = 1000
        // effective = 1000 * 1/2 * 850/1000 = 425
        let seg = TravelSegment {
            kind: TravelSegmentKind::LaneArc,
            lane_id: LaneId::Fringe,
            total_distance_milli: 1000,
            remaining_distance_milli: 1000,
            speed_multiplier_num: 1,
            speed_multiplier_den: 2,
            life_support_eligible: true,
            arc_direction: Some(ArcDirection::Clockwise),
        };
        let speed = effective_speed(MilliSpeed::new(1000), &seg, 50, 100).unwrap();
        assert_eq!(speed.value(), 425);
    }

    #[test]
    fn effective_speed_overflow() {
        // Overflow when multiplier * base_speed exceeds u32
        let seg = TravelSegment {
            kind: TravelSegmentKind::LaneArc,
            lane_id: LaneId::Inner,
            total_distance_milli: 1000,
            remaining_distance_milli: 1000,
            speed_multiplier_num: 3,
            speed_multiplier_den: 2,
            life_support_eligible: false,
            arc_direction: Some(ArcDirection::Clockwise),
        };
        // MilliSpeed::new(u32::MAX) with multiplier 3/2 overflows
        let err = effective_speed(MilliSpeed::new(u32::MAX), &seg, 0, 100);
        assert_eq!(err, Err(ArithmeticError::Overflow));
    }

    // ─── TravelPlan serde (round-trip) ────────────────────────────────

    #[test]
    fn travel_plan_serde_round_trip() {
        let src = pos(LaneId::Inner, 600, 0);
        let dst = pos(LaneId::Habitable, 1200, 1570);
        let plan = TravelPlan::between(&src, &dst, dest_body("planet_haven"));

        let json = serde_json::to_string(&plan).unwrap();
        let back: TravelPlan = serde_json::from_str(&json).unwrap();
        assert_eq!(plan, back);
    }

    // ─── Deterministic arc direction consistency ──────────────────────

    #[test]
    fn arc_direction_clockwise_consistency() {
        // source=0, dest=1570 → clockwise
        let src = pos(LaneId::Habitable, 1200, 0);
        let dst = pos(LaneId::Habitable, 1200, 1570);
        let plan = TravelPlan::between(&src, &dst, dest_body("planet_haven"));

        let arc = &plan.segments[0];
        assert_eq!(arc.arc_direction, Some(ArcDirection::Clockwise));
    }

    #[test]
    fn arc_direction_counter_clockwise_consistency() {
        // source=1570, dest=0 → counter-clockwise
        let src = pos(LaneId::Habitable, 1200, 1570);
        let dst = pos(LaneId::Habitable, 1200, 0);
        let plan = TravelPlan::between(&src, &dst, dest_body("planet_haven"));

        let arc = &plan.segments[0];
        assert_eq!(arc.arc_direction, Some(ArcDirection::CounterClockwise));
    }

    // ─── Edge cases ───────────────────────────────────────────────────

    #[test]
    fn angular_diff_at_zero() {
        // source=6283, dest=0 — diff = 0-6283 = -6283
        // -6283 < -3141 → diff += 6283 = 0
        let diff = angular_diff(6283, 0);
        assert_eq!(diff, 0);
    }

    #[test]
    fn radius_1000_scale() {
        // TravelPlan::between multiplies radius diff by 1000
        let src = pos(LaneId::Inner, 1, 0);
        let dst = pos(LaneId::Habitable, 2, 0);
        let plan = TravelPlan::between(&src, &dst, dest_body("planet_haven"));

        let seg = &plan.segments[0];
        assert_eq!(seg.total_distance_milli, 1000);
    }
}
