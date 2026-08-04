|---
|status: accepted
|owner: Tech Lead
|date: 2026-08-04
|---

# ADR-0010: Mining Boundary Behavior

## Context

Phase 1 requires deterministic mining extraction at stations and belt-density drift for renewable deposits. GDD 12 §Mining establishes the standard rate accumulator for finite deposits and the denominator-specific rational accumulator for belt mining. GDD 5 §Mining Station defines tier-dependent targets and retune mechanics. GDD 14 §The Veil defines belt drift every 1,000 ticks with PRNG-based density updates. GDD 13 defines `MiningTarget`, `ResourceDeposit`, and `Station.mining_targets`.

Three unresolved specification questions remain:

1. **Full-output handling.** When a mining station's output buffer reaches its maximum (`current >= max`), can extraction continue? Does the accumulator advance or stall? What happens to produced units that cannot fit in the buffer?

2. **Finite-deposit exhaustion.** When a finite deposit has fewer units remaining than the accumulator would produce (e.g., 1 unit remaining but the accumulator crosses 1,000 and would produce 3 units), how is extraction capped? What happens to the accumulator remainder?

3. **Extraction/belt-drift order at tick multiples of 1,000.** Belt drift updates current density every 1,000 ticks (GDD 14). Mining extraction uses current density for belt targets. The relative order of drift and extraction on tick 1,000, 2,000, etc. affects deterministic output.

## Decision

### 1. Full-output handling: extraction stalls when the buffer is full

When a mining target's output buffer is at capacity (`current >= max`), extraction for that target **produces no units** and the **accumulator is not advanced** for that tick. The mining station's phase-4 processing skips the accumulator addition entirely for that target.

This means:

- The remainder stays unchanged during blocked ticks.
- When buffer space opens (`current < max`), extraction resumes on the next tick with the existing remainder.
- The remainder does not accumulate during blockage, so no "catch-up surge" occurs when space opens.
- Each target's output buffer is evaluated independently: one target may be blocked while another target on the same station continues extraction.
- Full-output blockage does not affect retune progress (`retune_ticks_remaining` continues to decrement normally during retune even if extraction is blocked).

**Rationale.** Stalling the accumulator is simpler than allowing unbounded remainder growth during blockage. The deposit retains its unextracted material during blockage (correct for both finite and belt deposits). No overflow-to-salvage path is needed because the buffer is the sole extraction destination and no material is lost. This differs from recipe OutputBlocked (which holds a completed batch) because mining has no batch boundary—it produces one unit at a time from a continuous stream.

**Edge cases.**

- If a target becomes blocked during retune, the retune timer continues. When the retune completes and the target switches to the new resource, the old target's remainder is discarded (see GDD 5: retune "never deletes the prior target's buffer or inventory" — the buffer persists, but the mining target's accumulator is reset).
- After save/load, the buffer state and remainder are serialized. A loaded save with full buffers resumes correctly without extraction on the first tick.

### 2. Finite-deposit exhaustion: capped extraction with remainder reset

When a finite (non-renewable) deposit has fewer units remaining than the accumulator would produce, extraction proceeds as follows:

1. Compute `produced = remainder / 1000` (from the standard rate accumulator).
2. Compute `actual_extraction = min(produced, deposit.current)`.
3. Decrement `deposit.current` by `actual_extraction`.
4. Transfer `actual_extraction` units to the output buffer (capped by available buffer space; see §1 above).
5. If `produced > deposit.current` (the deposit was exhausted this tick):
   - The accumulator remainder is **reset to zero** (`rate_remainder.value = 0`). The excess production potential is lost because the deposit contained insufficient material.
6. On subsequent ticks, if `deposit.current == 0`:
   - Extraction is skipped entirely for that target (no accumulator advance, no production). The mining target remains configured but produces nothing.

**Rationale.** Resetting the remainder when the deposit runs out avoids a permanently non-productive accumulator that would grow unboundedly. The lost excess is bounded: at most `(increment - 1)` milli-units of lost remainder per exhaustion event, because the accumulator crosses 1,000 at most once per tick and the excess beyond the deposit's remainder is at most `999 + increment`. This is an intentional design choice: the player must monitor finite deposits and relocate mining when a deposit is near exhaustion. The 999-unit worst-case remainder loss represents <1 unit of material.

**Edge cases.**

- If a finite deposit is exhausted on the same tick that output is full: extraction produces nothing (blocked by §1), and the deposit is not decremented. The accumulator is not advanced. The deposit survives until a tick where buffer space exists. This is correct—material cannot be extracted into a full buffer.
- If multiple targets extract from the same deposit: this is not possible because each target maps to a distinct `ResourceType` and a deposit holds at most one resource per type per body. The `max_targets` limit per station tier prevents duplicate-target issues.

### 3. Belt drift occurs before extraction on tick multiples of 1,000

On ticks where `tick > 0` and `tick % 1000 == 0`, the belt drift phase executes **before** mining extraction within phase 4. The per-tick order within phase 4 is:

1. **Belt density drift.** For every belt (`renewable: true`) deposit in `celestial_bodies`, update current density in `ResourceType` order (the enum order defined in GDD 13 §ResourceType). For each belt deposit:

   ```
   range = baseline / 10                        // integer division, floor
   delta_raw = next_rng_u64() % (2 * range + 1) // [0, 2*range]
   delta = delta_raw - range                    // [-range, +range]
   new_density = deposit.current + delta
   deposit.current = clamp(new_density, baseline * 70 / 100, baseline * 130 / 100)
   ```

   This matches GDD 14's formula with explicit clamping and ResourceType ordering.

2. **Mining extraction.** For each station's mining targets (in ascending station ID, then target creation order), perform extraction using the updated deposit densities for belt targets and the current deposit amount for finite targets. Extraction follows §1 (full-output handling) and §2 (exhaustion) using the current deposit state.

**Rationale.** Drift-before-extraction means extraction on tick 1000 uses the new density. This is the natural interpretation: the drift event represents a real-time change in belt composition, and mining responds to the new conditions in the same tick. ResourceType ordering provides deterministic drift-event sequencing. Extraction order (station ID then target creation order) matches the simulation's general entity-ordering principle and prevents ordering-dependent state divergence.

**Belt extraction accumulator.** For belt (renewable) targets, the denominator-specific accumulator from GDD 12 is used:

```
remainder += base_quantity * deposit.current       // u64 addition
produced = remainder / (cycle_ticks * baseline)    // u64 division
remainder = remainder % (cycle_ticks * baseline)   // u64 remainder
```

Where:
- `base_quantity` = `StationStats.extraction_per_target_per_10_ticks` (1/2/3/5 for T1–T4)
- `cycle_ticks` = 10 (the authored per-target cycle length)
- `baseline` = the deposit's `baseline` field (e.g., 2000 for MetalOre in The Veil)
- `deposit.current` = the current density after drift (fluctuates 70–130% of baseline)

This is equivalent to "output multiplied by `current / baseline`" as stated in GDD 14, because:

```
output_per_10_ticks = base_quantity * deposit.current / baseline
                    = extraction_per_target_per_10_ticks * current_density / baseline_density
```

At baseline density (100%), the target produces exactly `base_quantity` units per 10 ticks. At higher densities it produces faster; at lower densities, slower. The denominator `cycle_ticks * baseline` ensures the accumulator's unit is milli-units of `baseline_density` rather than 1000, which is necessary because `current_density / baseline_density` is a rational that need not divide 1000 evenly.

**Belt deposit invariants.** Belt `deposit.current` is never decremented by extraction. It represents density, not a consumable quantity. The `deposit.current` field is re-purposed: for finite deposits it tracks remaining material; for belt deposits it tracks current density (starting at `baseline`). The `renewable` flag distinguishes the two behaviors.

## Consequences

### Positive

- Full-output stalling is simple: no catch-up surge, no overflow salvage, no accumulator unbounded growth.
- Exhaustion with remainder reset is bounded and deterministic: at most 999 + `increment` milli-units lost per exhaustion event.
- Drift-before-extraction on multiples of 1,000 is deterministic and uses the natural ResourceType ordering.
- Belt extraction accumulator formula is explicit and matches both GDD 12 and GDD 14 descriptions.

### Negative

- Full-output stalling means a player who fills a mining buffer loses extraction progress until the buffer drains. This is a player-planning concern (configure appropriate export thresholds and Courier logistics), not a simulation invariant.
- Remainder reset on exhaustion loses <1 unit of potential material per exhaustion event. Over the game's lifespan this is negligible (at most a few dozen events across all finite deposits).
- Belt drift before extraction means the PRNG is called during phase 4, which is after command application and ship movement. The PRNG state at that point is deterministic given the initial RNG words and the tick number, but care is needed for replay equivalence: the PRNG call count per tick must be stable.

### Mitigations

- The <1-unit loss per exhaustion is bounded and consistent across save/load and replay because the accumulator state is serialized.
- PRNG call count per tick is fixed: exactly one call per belt deposit per drift tick (7 deposits at tick 1000, no drift between). The drift ticks are at deterministic intervals (1000, 2000, ...), and replay equivalence holds because the same RNG words produce the same drift sequence.
- Buffer blockage is self-correcting: the export threshold and logistics system naturally drain full buffers. A mining station with a full output buffer will have no supply advertised for that resource, and Cargo Ships will not attempt to pick up from it.

## Related ADRs

- ADR-0006 — Canonical Content/State Hashing (serialized `MiningTarget.rate_remainder` and `ResourceDeposit.current` are included in replay-mode hash)
- ADR-0007 — Save Envelope Format, Content Hash Placement, and Migration Fixtures (serialized mining state in save files)
