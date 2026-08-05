---
status: accepted
owner: Tech Lead
date: 2026-08-04
---

# ADR-0010: Mining Boundary Behavior

## Context

Finite deposits are shared by every Mining Station on their body. Renewable belt
deposits instead store a changing density and are never consumed. Mining targets
also have independently blocked output buffers and a ten-tick retune state.

These mechanics run in phase 4 under the tick-transaction rule: phases read the
committed tick-N snapshot and publish only explicit facts/reducers before the
atomic tick-N+1 commit. Therefore same-phase drift and shared-deposit contention
cannot rely on one station observing another station's staged writes.

This ADR defines the serialized target identity, retune timing, buffer boundaries,
finite-deposit allocation, drift tick boundary, checked arithmetic, and exact
within-phase facts.

## Decision

### 1. Mining slots and retune state are explicit

Each serialized target carries its stable station-local slot:

```text
struct MiningTarget {
  slot_index: u8
  resource: ResourceType
  rate_remainder: RationalRemainder
  retune_ticks_remaining: u16
}
```

`mining_targets` is serialized in ascending `slot_index`. Slot indices are unique,
must be below the station tier's `max_targets`, and need not be contiguous. A
station may not configure the same `ResourceType` in two slots; this also gives
each target one unambiguous resource-keyed output buffer. Across different
stations, any number of targets may legally share the same body deposit.

For every serialized target, `rate_remainder.denominator` is nonzero and
`rate_remainder.value < denominator`; the denominator matches the selected
deposit formula below. `retune_ticks_remaining <= 10`, and a positive retune
count requires remainder value zero. The matching output buffer and visible body
deposit must exist. A depleted finite deposit requires remainder value zero on
every target configured for it. Load validation rejects, rather than normalizes,
violations of these invariants.

`SetMiningTarget` immediately stores the requested resource, resets the remainder,
and sets `retune_ticks_remaining` to the authored value 10. The requested deposit
must exist and be visible at the body's current survey depth. The remainder
denominator is set to:

- `1_000` for a finite target; or
- checked `cycle_ticks * deposit.baseline` for a renewable target.

Changing a slot that is already retuning replaces the requested resource and
restarts the full ten ticks. The prior resource's output buffer and inventory are
not deleted. Setting a slot to the same resource is still an explicit retune and
restarts and resets progress; idempotent command IDs prevent accidental duplicate
application.

If several valid `SetMiningTarget` commands address the same slot at one effective
tick, command-phase processing applies them in `server_sequence`; the last request
is the committed target. Phase 4 receives one coalesced `RetuneStarted` fact for
that slot, counts the application tick once, and performs no extraction. Earlier
commands are still recorded as applied—their configuration was deterministically
superseded by a later command in the same transaction.

Retune duration counts simulation ticks, not paused planning transactions:

- A Paused command commits `retune_ticks_remaining = 10`; each of the next ten
  simulation ticks skips extraction and decrements it once. A tick that observes
  `1` writes `0` but still does not extract. Extraction first resumes on a later
  tick that begins with `0`.
- A Running command applied at the start of a simulation tick emits an explicit
  `RetuneStarted(station_id, slot_index)` fact. Phase 4 skips extraction for that
  slot and an explicit retune reducer changes the command-staged value from 10 to
  9. That application tick is the first of the ten inactive ticks. This named
  reducer prevents conflicting implicit writes between command application and
  phase 4.

The remainder is reset when retuning starts, so no production credit transfers
between resources. Retune countdown runs before output-capacity checks, so a full
new-resource buffer does not pause the ten-tick retune.

### 2. Common output-capacity rule

For a target that is not retuning, compute:

```text
available_space = output_buffer.max - output_buffer.current
```

with checked subtraction after validating `current <= max`.

If `available_space == 0`, extraction for that target is fully stalled:

- no accumulator increment is added;
- its fractional remainder is unchanged;
- a finite deposit is not decremented; and
- a belt still drifts globally when due, but this target produces nothing.

When space later opens, the target resumes from its saved fractional remainder.
There is no catch-up for blocked ticks.

If a noncanonical future content definition can produce more whole units in one
tick than the remaining partial space, only units that fit are materialized. The
unstoreable whole-unit throughput is discarded for that tick; it never decrements
a finite deposit and is not retained as whole-unit accumulator credit. The
fractional remainder below the denominator is retained unless the finite deposit
is exhausted. This preserves bounded accumulator invariants without deleting
material.

A full output buffer normally advertises supply above its configured export floor,
which lets logistics drain it. It does not advertise “no supply”; whether it can
drain depends on thresholds, reservations, and available Cargo Ships.

### 3. Accumulator step

Every active target computes a checked prospective step before any material
mutation:

```text
total = prior_remainder.value + increment
potential_units = total / prior_remainder.denominator
fractional_remainder = total % prior_remainder.denominator
```

The denominator must be nonzero and match the target/deposit invariant in §1.
All addition, multiplication, conversion, and buffer/deposit updates use checked
integer arithmetic and return typed errors. No production path wraps or panics.

For finite deposits:

```text
increment = extraction_per_target_per_10_ticks * 1_000 / 10
denominator = 1_000
```

The authored numerator divides exactly; content validation rejects a future value
that does not. For a renewable belt:

```text
increment = extraction_per_target_per_10_ticks * effective_density
denominator = 10 * baseline
```

This is exactly the authored output multiplied by `current_density / baseline`.

### 4. Finite-deposit extraction and shared contention

Multiple legal targets may draw from one finite `(BodyId, ResourceType)` deposit.
Phase 4 creates extraction intents from the committed snapshot, then allocates
them through a named per-deposit reducer.

For every finite deposit, initialize:

```text
remaining_budget = committed_deposit.current
```

Eligible intents are processed by ascending Station ID, then ascending serialized
`slot_index`. For each intent:

1. If it is retuning, depleted, or has zero output space, apply the corresponding
   skip rule and do not advance its accumulator.
2. Compute `potential_units` and `fractional_remainder` from §3.
3. Compute
   `actual = min(potential_units, available_space, remaining_budget)`.
4. Add exactly `actual` to the target's output buffer.
5. Subtract exactly `actual` from `remaining_budget`.
6. Provisionally store `fractional_remainder` while the deposit budget remains
   nonzero.

After all intents, stage one aggregate deposit update equal to the final
`remaining_budget`. If that value is zero, reset the remainder value of every
target configured for this deposit to zero, including targets processed before
the exhausting intent. Fractional progress against a nonexistent finite deposit
cannot survive or transfer to another resource. The sum of all target outputs is
therefore exactly the deposit decrement and can never exceed the committed amount.
Later intents see the phase-local budget through the explicit reducer, not through
staged `GameState`.

Targets reached after the budget becomes zero skip with no accumulator advance.
A target remains configured after depletion and can be retuned by command.

An intent exhausts the deposit only when `actual` equals its positive budget on
entry. This covers exact potential and overshoot only when output space can accept
the remaining material; partial output space cannot falsely exhaust it. On final
exhaustion all target fractions for the deposit reset as above. Each discarded
fractional value is strictly less than its denominator; unavailable whole-unit
throughput was never extracted material.

### 5. Belt drift uses the resulting tick and an immutable fact

For a transaction beginning at committed tick `N`, compute checked:

```text
resulting_tick = N + 1
```

Drift occurs when `resulting_tick > 0` and `resulting_tick % 1_000 == 0`. Thus the
transaction that commits state tick 1,000 performs the first drift, and extraction
committed at tick 1,000 uses that new density.

Renewable deposits are visited in ascending Body ID, then `ResourceType` enum
order. Exactly one project-owned PRNG value is consumed per renewable deposit on
every drift tick, including a deposit whose integer range is zero.

For each deposit, use wide checked arithmetic:

```text
range = u64(baseline) / 10
span = checked_add(checked_mul(2, range), 1)
delta_raw = next_rng_u64() % span
delta = i128(delta_raw) - i128(range)
candidate = i128(committed_current) + delta
minimum = checked_mul(u64(baseline), 70) / 100
maximum = checked_mul(u64(baseline), 130) / 100
drifted_density = clamp(candidate, minimum, maximum)
```

Content validation requires `baseline > 0`, and the checked result must fit the
serialized `u32` field.

Phase 4 publishes an immutable
`DriftedDensityFact<(BodyId, ResourceType), u32>` and stages the corresponding
deposit updates through a named reducer. Belt extraction reads the fact when
present and otherwise reads committed density. It never reads an implicit staged
write.

### 6. Renewable extraction

Renewable intents are processed in ascending Station ID and slot index after the
drift facts are built. Each target applies §§2–3 with the effective density from
§5. Its actual output is:

```text
actual = min(potential_units, available_space)
```

The target stores the fractional remainder. Excess whole-unit throughput above
available space is discarded for the tick. `ResourceDeposit.current` is never
decremented by extraction because it is density, not inventory.

Retuning or output blockage affects only the target. Drift still consumes its
fixed PRNG call and updates the renewable deposit even when no station can extract.

### 7. Phase-4 total order and atomicity

The phase executes these substeps:

1. Compute `resulting_tick` and, when due, all checked drift facts in Body
   ID/ResourceType order.
2. Apply retune facts/countdowns in Station ID/slot order.
3. Build active extraction intents from the committed snapshot plus named retune
   and drift facts.
4. Allocate finite intents with per-deposit budgets in Station ID/slot order.
5. Compute renewable target outputs in Station ID/slot order.
6. Stage aggregate deposit, buffer, remainder, countdown, RNG, and event changes.

Any overflow, invalid denominator, invalid command-fact reduction, capacity
violation, or invariant failure aborts the complete tick transaction. No partial
drift, deposit debit, output credit, remainder update, or PRNG advance commits.

Drift events, if externally emitted, use the same Body ID/ResourceType order.
Mining-output deltas use Station ID/slot order. Event ordering therefore does not
depend on map insertion or thread scheduling.

## Required executable proofs

P1-17/P1-26 must cover at least:

- Slot identity and order survive Serde, canonical hashing, save/load, and replay.
- Duplicate slots/resources and slot indices beyond tier capacity are rejected.
- New and repeated retunes suppress exactly ten simulation ticks, reset the
  remainder, and preserve old buffers/inventory.
- Full buffers do not advance the accumulator or decrement deposits.
- Partial space with multi-unit potential never removes an unstored finite unit.
- Exact exhaustion and overshoot both conserve `deposit + outputs`, reset every
  target fraction for the exhausted deposit, and never underflow; insufficient
  output space cannot falsely exhaust it.
- Two or more stations sharing one finite deposit allocate in Station ID/slot
  order and cannot overdraw it.
- Tick 999→1,000 drifts before extraction; tick 1,000→1,001 does not drift.
- A save at tick 999 produces the same tick-1,000 density, PRNG state, output, and
  events as an uninterrupted/replayed run.
- Body insertion order cannot change PRNG call order or hashes.
- Full/retuning belt targets still receive global drift but no output.
- Quotients greater than one, zero range, checked-product boundaries, and every
  typed rollback path are exercised.

## Consequences

- No buffer boundary or shared-deposit race can lose or overdraw finite material;
  renewable output remains the intentional authored source.
- Stable slot identity removes the prior un-serialized “target creation order.”
- Retune timing is identical in real-time and explicit batch advancement.
- Drift-before-extraction is defined against the committed resulting tick and is
  compatible with immutable transaction phases.
- Partial output space may waste that tick's available throughput, but never
  deposit inventory; this is the intentional no-catch-up policy.

## Related ADRs

- ADR-0002 — Deterministic Tick Simulation
- ADR-0006 — Canonical Content/State Hashing
- ADR-0007 — Save Envelope
