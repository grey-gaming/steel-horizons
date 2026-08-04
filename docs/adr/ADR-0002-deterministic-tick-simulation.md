---
status: accepted
owner: Tech Lead
date: 2026-08-04
---

# ADR-0002: Deterministic Tick Simulation with Integer Arithmetic

## Context

The engine must produce identical state for the same authored content and recorded command log across macOS/Windows, ARM/x86, save/load, batch execution, and agent replay. Production, movement, research, and fuel include fractional rates, while concurrent API consumers can submit commands near tick boundaries.

## Decision

We will use a single-owner simulation actor, discrete one-second ticks, integer-only state, explicit rational accumulators, a project-owned deterministic PRNG, and a recorded command order.

### Numeric Rules

- Positions and speeds use thousandths (`milli-*`) with checked integer intermediates.
- Standard rates use quotient/remainder accumulation at scale 1,000:

  ```text
  remainder += increment
  whole = remainder / 1000
  remainder %= 1000
  ```

- Research uses denominator-specific integer remainders because arbitrary project durations need not divide 1,000.
- Fuel uses the authored 10,000,000 milli-distance denominator and actual distance moved, not nominal speed.
- Percentages are integers 0–100.
- API state and deltas expose these integers; display conversion occurs in clients.

There is no generic helper that labels `rate * 1000 / denominator` as whole units. Every accumulator returns the quotient only after adding its per-tick increment to persistent remainder.

### Tick Transaction

The simulation actor is the only mutable owner of `GameState`:

1. Drain commands scheduled for the next tick in `server_sequence` order.
2. Read committed tick-N state.
3. Execute the fixed phase list from GDD 12 into `PendingChanges`.
4. Commit once as tick N+1.
5. Emit events from the committed transaction.

No API handler mutates `GameState` directly. Paused commands are serialized through the same actor and recorded at the current tick with a unique sequence.

### Execution Modes

- Graphical play: one tick per real second.
- Paused batch advancement: execute exactly N ordinary ticks without wall-clock waits, then return to Paused.
- Short scheduler catch-up: at most 10 ticks before yielding.

Execution speed never enters simulation state and cannot affect outcomes.

### Randomness

The engine owns an implementation of xoshiro256** and serializes all four `u64` state words. Golden vectors lock the transition algorithm. Random calls occur only in named, ordered phases; belt drift iterates Body IDs and ResourceType values in stable order.

### Stable Iteration and Arithmetic

- Serialized maps use `BTreeMap` or explicitly sorted keys.
- Equal logistics candidates use the complete deterministic tie-break list in GDD 12.
- Intermediate multiplication uses checked `u64`/`u128` operations.
- Overflow returns a typed simulation error and never wraps or panics.

## Consequences

### Positive

- Save/load and batch/real-time equivalence are exact.
- Agent play-tests can replay a command log rather than approximate wall-clock timing.
- Cross-platform golden states are meaningful.
- Concurrency at the API boundary cannot mutate partial tick state.

### Negative

- Remainders are part of serialized state and require migration support.
- Content rates must declare their denominators/cycle lengths explicitly.
- The single-owner actor requires snapshots or channels for readers rather than shared mutation.

### Mitigations

- Newtypes prevent mixing scales.
- Property tests cover quotient/remainder conservation, overflow boundaries, and exact completion.
- Scenario tests compare state hashes across real-time-equivalent, batch, save/load, and replay paths.
- The canonical formulas and phase order live in GDD 12; implementation comments link to those sections.

## Related ADRs

- ADR-0001 — Rust Simulation Engine with HTTP/WS API
- ADR-0003 — Command/Query API with WebSocket Streaming
- ADR-0004 — Game Lifecycle State Machine
- ADR-0005 — Test Architecture
